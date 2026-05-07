#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod capture;
mod chunker;
mod db;
mod episode;
mod retrieval;
mod socket;
mod tokenizer;
mod worker;

use episode::EpisodeDetector;
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex;
use worker::{Scheduler, SummarizerWorker, WorkerQueue};

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .expect("could not resolve app data dir");
            std::fs::create_dir_all(&data_dir)?;
            let db_path = data_dir.join("kosmos.db");

            let db = db::open(&db_path).expect("failed to open kosmos.db");

            let detector = Arc::new(Mutex::new(EpisodeDetector::new()));

            let (queue, rx) = WorkerQueue::new(1024);
            let scheduler = Scheduler::new().register(Arc::new(SummarizerWorker));
            tauri::async_runtime::spawn(scheduler.run(rx, db.clone()));

            capture::spawn();
            tauri::async_runtime::spawn(socket::run_server(
                app.handle().clone(),
                db.clone(),
                detector,
                queue,
            ));

            app.manage(db);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            retrieval::search_chunks,
            retrieval::get_recent_episodes,
            retrieval::delete_episode_cmd,
            retrieval::suppress_episode_cmd,
            retrieval::get_episode_chunks,
            retrieval::get_chunk_occurrences,
        ])
        .run(tauri::generate_context!())
        .expect("error running tauri app");
}
