use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::UnixListener;
use tokio::sync::Mutex;

use crate::capture::RawCapture;
use crate::chunker::{chunker_for, ChunkConfig};
use crate::db::{self, Db};
use crate::episode::{doc_identity, EpisodeDecision, EpisodeDetector};
use crate::worker::{Job, WorkerQueue};

const SOCKET_PATH: &str = "/tmp/kosmos.sock";

pub async fn run_server(
    app: AppHandle,
    db: Db,
    detector: Arc<Mutex<EpisodeDetector>>,
    queue: WorkerQueue,
) {
    let _ = std::fs::remove_file(SOCKET_PATH);

    let listener = match UnixListener::bind(SOCKET_PATH) {
        Ok(l) => {
            println!("Listening on {SOCKET_PATH}");
            l
        }
        Err(e) => {
            eprintln!("Socket bind failed: {e}");
            return;
        }
    };

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let app = app.clone();
                let db = db.clone();
                let detector = detector.clone();
                let queue = queue.clone();

                tokio::spawn(async move {
                    let mut lines = BufReader::new(stream).lines();
                    while let Ok(Some(line)) = lines.next_line().await {
                        if let Ok(raw) = serde_json::from_str::<RawCapture>(&line) {
                            let capture = raw.into();

                            // 1. emit to live feed (unchanged behaviour)
                            let _ = app.emit("capture", &capture);

                            // 2. episode routing
                            let decision = {
                                let mut det = detector.lock().await;
                                det.process(&capture)
                            };

                            let episode = match &decision {
                                EpisodeDecision::NewEpisode(ep)
                                | EpisodeDecision::Continuing(ep) => ep,
                            };

                            // 3. persist episode
                            if let Err(e) = db::upsert_episode(&db, episode) {
                                eprintln!("[socket] upsert_episode: {e}");
                                continue;
                            }

                            // 4. chunk and persist
                            let chunker = chunker_for(&capture.source, ChunkConfig::default());
                            let chunks = chunker.chunk(&capture.content);
                            let doc = doc_identity(&capture);

                            let mut new_chunks = 0usize;
                            for chunk in &chunks {
                                match db::insert_chunk(
                                    &db,
                                    &episode.id,
                                    chunk,
                                    &capture.source,
                                    &doc,
                                    capture.ts,
                                ) {
                                    Ok(true) => new_chunks += 1,
                                    Ok(false) => {} // duplicate
                                    Err(e) => eprintln!("[socket] insert_chunk: {e}"),
                                }
                            }

                            // 5. enqueue background work if new content was stored
                            if new_chunks > 0 {
                                queue
                                    .enqueue(Job {
                                        job_type: "episode_summary".to_string(),
                                        payload: serde_json::json!({
                                            "episode_id": episode.id
                                        }),
                                    })
                                    .await;
                            }
                        }
                    }
                });
            }
            Err(e) => eprintln!("Accept error: {e}"),
        }
    }
}
