#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod capture;
mod socket;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            capture::spawn();
            tauri::async_runtime::spawn(socket::run_server(app.handle().clone()));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error running tauri app");
}
