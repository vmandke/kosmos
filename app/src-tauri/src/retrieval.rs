use serde::Serialize;
use tauri::State;

use crate::db::Db;

#[derive(Serialize)]
pub struct ChunkResult {
    pub chunk_id:      String,
    pub episode_id:    String,
    pub content:       String,
    pub source:        String,
    pub doc_identity:  Option<String>,
    pub ts:            i64,
    pub episode_start: i64,
    pub episode_end:   Option<i64>,
    pub ep_sources:    Option<String>,
}

#[derive(Serialize)]
pub struct EpisodeSummary {
    pub id:             String,
    pub sources:        Option<String>,
    pub doc_identities: Option<String>,
    pub start_ts:       i64,
    pub end_ts:         Option<i64>,
    pub chunk_count:    i64,
    pub summary:        Option<String>,
}

#[derive(Serialize)]
pub struct ChunkItem {
    pub id:           String,
    pub content:      String,
    pub source:       String,
    pub doc_identity: Option<String>,
    pub ts:           i64,
    pub occurrences:  i64,
}

#[derive(Serialize)]
pub struct OccurrenceItem {
    pub episode_id: String,
    pub ts:         i64,
}

#[tauri::command]
pub fn search_chunks(query: String, db: State<Db>) -> Result<Vec<ChunkResult>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT c.id, c.episode_id, c.content, c.source, c.doc_identity, c.ts,
                    e.start_ts, e.end_ts, e.sources
             FROM chunks_fts
             JOIN chunks   c ON chunks_fts.rowid = c.rowid
             JOIN episodes e ON c.episode_id = e.id
             WHERE chunks_fts MATCH ?1
               AND c.suppressed = 0
               AND e.suppressed = 0
             ORDER BY rank + (1.0 / (1.0 + (unixepoch('now') - CAST(c.ts AS REAL)) / 86400.0)) DESC
             LIMIT 20",
        )
        .map_err(|e| e.to_string())?;

    let results = stmt
        .query_map([&query], |row| {
            Ok(ChunkResult {
                chunk_id:      row.get(0)?,
                episode_id:    row.get(1)?,
                content:       row.get(2)?,
                source:        row.get(3)?,
                doc_identity:  row.get(4)?,
                ts:            row.get(5)?,
                episode_start: row.get(6)?,
                episode_end:   row.get(7)?,
                ep_sources:    row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(results)
}

#[tauri::command]
pub fn get_recent_episodes(
    limit: Option<i64>,
    db: State<Db>,
) -> Result<Vec<EpisodeSummary>, String> {
    let limit = limit.unwrap_or(20);
    let conn = db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT e.id, e.sources, e.doc_identities, e.start_ts, e.end_ts,
                    COUNT(c.id) AS chunk_count, e.summary
             FROM episodes e
             LEFT JOIN chunks c ON c.episode_id = e.id AND c.suppressed = 0
             WHERE e.suppressed = 0
             GROUP BY e.id
             ORDER BY COALESCE(e.end_ts, e.start_ts) DESC
             LIMIT ?1",
        )
        .map_err(|e| e.to_string())?;

    let results = stmt
        .query_map([limit], |row| {
            Ok(EpisodeSummary {
                id:             row.get(0)?,
                sources:        row.get(1)?,
                doc_identities: row.get(2)?,
                start_ts:       row.get(3)?,
                end_ts:         row.get(4)?,
                chunk_count:    row.get(5)?,
                summary:        row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(results)
}

#[tauri::command]
pub fn delete_episode_cmd(episode_id: String, db: State<Db>) -> Result<(), String> {
    crate::db::delete_episode(&db, &episode_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn suppress_episode_cmd(episode_id: String, db: State<Db>) -> Result<(), String> {
    crate::db::suppress_episode(&db, &episode_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_episode_chunks(episode_id: String, db: State<Db>) -> Result<Vec<ChunkItem>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT c.id, c.content, c.source, c.doc_identity, c.ts,
                    COUNT(co.chunk_id) AS occurrences
             FROM chunks c
             LEFT JOIN chunk_occurrences co ON co.chunk_id = c.id
             WHERE c.episode_id = ?1 AND c.suppressed = 0
             GROUP BY c.id
             ORDER BY c.ts ASC",
        )
        .map_err(|e| e.to_string())?;

    let results = stmt
        .query_map([&episode_id], |row| {
            Ok(ChunkItem {
                id:           row.get(0)?,
                content:      row.get(1)?,
                source:       row.get(2)?,
                doc_identity: row.get(3)?,
                ts:           row.get(4)?,
                occurrences:  row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(results)
}

#[tauri::command]
pub fn get_chunk_occurrences(chunk_id: String, db: State<Db>) -> Result<Vec<OccurrenceItem>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT episode_id, ts FROM chunk_occurrences WHERE chunk_id = ?1 ORDER BY ts ASC",
        )
        .map_err(|e| e.to_string())?;

    let results = stmt
        .query_map([&chunk_id], |row| {
            Ok(OccurrenceItem {
                episode_id: row.get(0)?,
                ts:         row.get(1)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(results)
}
