use crate::capture::Capture;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::UnixListener;

const SOCKET_PATH: &str = "/tmp/kosmos.sock";

pub async fn run_server(app: AppHandle) {
    let _ = std::fs::remove_file(SOCKET_PATH);

    let listener = match UnixListener::bind(SOCKET_PATH) {
        Ok(l)  => { println!("Listening on {SOCKET_PATH}"); l }
        Err(e) => { eprintln!("Socket bind failed: {e}"); return; }
    };

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let app = app.clone();
                tokio::spawn(async move {
                    let mut lines = BufReader::new(stream).lines();
                    while let Ok(Some(line)) = lines.next_line().await {
                        if let Ok(capture) = serde_json::from_str::<Capture>(&line) {
                            let _ = app.emit("capture", capture);
                        }
                    }
                });
            }
            Err(e) => eprintln!("Accept error: {e}"),
        }
    }
}
