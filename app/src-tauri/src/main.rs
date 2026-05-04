#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod capture;
mod socket;

use tauri_plugin_sql::{Migration, MigrationKind};

fn main() {
    let migrations = vec![Migration {
        version: 1,
        description: "initial schema",
        sql: include_str!("../migrations/001_initial.sql"),
        kind: MigrationKind::Up,
    }];

    tauri::Builder::default()
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:kosmos.db", migrations)
                .build(),
        )
        .setup(|app| {
            capture::spawn();
            tauri::async_runtime::spawn(socket::run_server(app.handle().clone()));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error running tauri app");
}
