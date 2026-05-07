use async_trait::async_trait;
use crate::db::Db;
use super::{Job, Worker};

pub struct SummarizerWorker;

#[async_trait]
impl Worker for SummarizerWorker {
    fn name(&self) -> &'static str {
        "summarizer"
    }

    fn handles(&self) -> &[&'static str] {
        &["episode_summary"]
    }

    async fn process(&self, job: Job, db: Db) -> anyhow::Result<()> {
        let episode_id = job
            .payload
            .get("episode_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing episode_id in job payload"))?
            .to_string();

        tokio::task::spawn_blocking(move || {
            let conn = db.lock().unwrap();
            let summary: String = conn
                .query_row(
                    "SELECT GROUP_CONCAT(SUBSTR(content, 1, 200), ' | ')
                     FROM (SELECT content FROM chunks WHERE episode_id = ?1 ORDER BY ts LIMIT 5)",
                    [&episode_id],
                    |row| row.get(0),
                )
                .unwrap_or_default();

            conn.execute(
                "UPDATE episodes SET summary = ?1 WHERE id = ?2",
                rusqlite::params![summary, episode_id],
            )?;
            Ok::<_, rusqlite::Error>(())
        })
        .await??;

        Ok(())
    }
}
