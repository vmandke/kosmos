use anyhow::Context;
use rusqlite::Connection;
use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::chunker::Chunk;
use crate::episode::Episode;

/// Shared DB handle. All functions are synchronous — never hold this lock across an await.
pub type Db = Arc<Mutex<Connection>>;

const MIGRATIONS: &[(&str, &str)] = &[
    ("001_initial",  include_str!("../migrations/001_initial.sql")),
    ("002_add_fts",  include_str!("../migrations/002_add_fts.sql")),
    ("003_v2",       include_str!("../migrations/003_v2.sql")),
];

pub fn open(path: &Path) -> anyhow::Result<Db> {
    let conn = Connection::open(path)
        .with_context(|| format!("opening database at {path:?}"))?;

    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
    run_migrations(&conn)?;

    Ok(Arc::new(Mutex::new(conn)))
}

fn run_migrations(conn: &Connection) -> anyhow::Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS _migrations (
            name TEXT PRIMARY KEY,
            applied_at INTEGER NOT NULL
        );",
    )?;

    for (name, sql) in MIGRATIONS {
        let applied: bool = conn
            .query_row(
                "SELECT 1 FROM _migrations WHERE name = ?1",
                [name],
                |_| Ok(true),
            )
            .unwrap_or(false);

        if !applied {
            conn.execute_batch(sql)
                .with_context(|| format!("applying migration {name}"))?;
            conn.execute(
                "INSERT INTO _migrations (name, applied_at) VALUES (?1, unixepoch())",
                [name],
            )?;
            println!("[db] applied migration: {name}");
        }
    }

    Ok(())
}

pub fn upsert_episode(db: &Db, episode: &Episode) -> anyhow::Result<()> {
    let conn = db.lock().unwrap();
    conn.execute(
        "INSERT INTO episodes (id, source, sources, doc_identity, doc_identities, start_ts, end_ts, suppressed)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(id) DO UPDATE SET
           sources        = excluded.sources,
           doc_identities = excluded.doc_identities,
           end_ts         = excluded.end_ts",
        rusqlite::params![
            episode.id,
            episode.sources.iter().next().cloned().unwrap_or_default(),
            serde_json::to_string(&episode.sources.iter().collect::<Vec<_>>())?,
            episode.doc_identities.iter().next().cloned().unwrap_or_default(),
            serde_json::to_string(&episode.doc_identities.iter().collect::<Vec<_>>())?,
            episode.start_ts,
            episode.end_ts,
            episode.suppressed as i32,
        ],
    )?;
    Ok(())
}

/// Returns true if chunk was newly inserted, false if it was a duplicate (occurrence recorded only).
pub fn insert_chunk(
    db: &Db,
    episode_id: &str,
    chunk: &Chunk,
    source: &str,
    doc_identity: &str,
    ts: i64,
) -> anyhow::Result<bool> {
    let conn = db.lock().unwrap();

    let existing_id: Option<String> = conn
        .query_row(
            "SELECT id FROM chunks WHERE content_hash = ?1",
            [&chunk.fingerprint],
            |row| row.get(0),
        )
        .ok();

    match existing_id {
        Some(id) => {
            conn.execute(
                "INSERT OR IGNORE INTO chunk_occurrences (chunk_id, episode_id, ts) VALUES (?1, ?2, ?3)",
                rusqlite::params![id, episode_id, ts],
            )?;
            Ok(false)
        }
        None => {
            let id = ulid::Ulid::new().to_string();
            conn.execute(
                "INSERT INTO chunks (id, episode_id, content, content_hash, source, doc_identity, ts, suppressed)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0)",
                rusqlite::params![
                    id, episode_id, chunk.content, chunk.fingerprint, source, doc_identity, ts
                ],
            )?;
            conn.execute(
                "INSERT INTO chunk_occurrences (chunk_id, episode_id, ts) VALUES (?1, ?2, ?3)",
                rusqlite::params![id, episode_id, ts],
            )?;
            Ok(true)
        }
    }
}

pub fn delete_episode(db: &Db, episode_id: &str) -> anyhow::Result<()> {
    let conn = db.lock().unwrap();
    conn.execute(
        "DELETE FROM chunk_occurrences WHERE episode_id = ?1",
        [episode_id],
    )?;
    conn.execute("DELETE FROM chunks WHERE episode_id = ?1", [episode_id])?;
    conn.execute("DELETE FROM episodes WHERE id = ?1", [episode_id])?;
    Ok(())
}

pub fn suppress_episode(db: &Db, episode_id: &str) -> anyhow::Result<()> {
    let conn = db.lock().unwrap();
    conn.execute(
        "UPDATE episodes SET suppressed = 1 WHERE id = ?1",
        [episode_id],
    )?;
    conn.execute(
        "UPDATE chunks SET suppressed = 1 WHERE episode_id = ?1",
        [episode_id],
    )?;
    Ok(())
}
