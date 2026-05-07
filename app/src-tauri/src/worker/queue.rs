use tokio::sync::mpsc::{channel, Receiver, Sender};

#[derive(Debug, Clone)]
pub struct Job {
    pub job_type: String,
    pub payload:  serde_json::Value,
}

#[derive(Clone)]
pub struct WorkerQueue {
    tx: Sender<Job>,
}

impl WorkerQueue {
    pub fn new(buffer: usize) -> (Self, Receiver<Job>) {
        let (tx, rx) = channel(buffer);
        (Self { tx }, rx)
    }

    pub async fn enqueue(&self, job: Job) {
        if let Err(e) = self.tx.send(job).await {
            eprintln!("[worker_queue] send failed: {e}");
        }
    }
}
