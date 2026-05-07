mod queue;
mod scheduler;
mod summarizer;

use async_trait::async_trait;
use crate::db::Db;

pub use queue::{Job, WorkerQueue};
pub use scheduler::Scheduler;
pub use summarizer::SummarizerWorker;

/// Implement this trait to add a new background worker.
/// Register it in main.rs with `Scheduler::new().register(Arc::new(MyWorker))`.
#[async_trait]
pub trait Worker: Send + Sync {
    fn name(&self) -> &'static str;
    fn handles(&self) -> &[&'static str];
    async fn process(&self, job: Job, db: Db) -> anyhow::Result<()>;
}
