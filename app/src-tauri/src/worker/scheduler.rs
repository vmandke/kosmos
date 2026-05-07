use std::sync::Arc;
use tokio::sync::mpsc::Receiver;
use crate::db::Db;
use super::{Job, Worker};

pub struct Scheduler {
    workers: Vec<Arc<dyn Worker>>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self { workers: Vec::new() }
    }

    pub fn register(mut self, worker: Arc<dyn Worker>) -> Self {
        self.workers.push(worker);
        self
    }

    pub async fn run(self, mut rx: Receiver<Job>, db: Db) {
        while let Some(job) = rx.recv().await {
            let worker = self
                .workers
                .iter()
                .find(|w| w.handles().contains(&job.job_type.as_str()))
                .cloned();

            match worker {
                Some(w) => {
                    let db = db.clone();
                    tokio::spawn(async move {
                        if let Err(e) = w.process(job, db).await {
                            eprintln!("[worker:{}] error: {e}", w.name());
                        }
                    });
                }
                None => eprintln!("[scheduler] no worker for job type: {}", job.job_type),
            }
        }
    }
}
