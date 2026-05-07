use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Wire format from Swift daemon — matches JSON sent over the socket.
#[derive(Debug, Deserialize)]
pub struct RawCapture {
    pub ts:      String,
    pub app:     String,
    pub title:   String,
    pub content: String,
    pub chars:   usize,
    pub url:     Option<String>,
}

/// Internal representation used by all processing modules.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Capture {
    pub ts:           i64,
    pub source:       String,
    pub title:        String,
    pub content:      String,
    pub chars:        usize,
    pub url:          Option<String>,
    pub doc_identity: String,
}

impl From<RawCapture> for Capture {
    fn from(r: RawCapture) -> Self {
        let doc_identity = r.url.clone().unwrap_or_else(|| r.title.clone());
        let ts = chrono::DateTime::parse_from_rfc3339(&r.ts)
            .map(|dt| dt.timestamp())
            .unwrap_or_else(|_| {
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0)
            });
        Self {
            ts,
            source: r.app,
            title: r.title,
            content: r.content,
            chars: r.chars,
            url: r.url,
            doc_identity,
        }
    }
}

pub fn spawn() {
    let path = bin_path();
    match std::process::Command::new(&path).spawn() {
        Ok(_)  => println!("Spawned capture: {path:?}"),
        Err(e) => eprintln!("Could not spawn capture ({path:?}): {e}"),
    }
}

fn bin_path() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        let p = exe.parent().unwrap_or(std::path::Path::new(".")).join("kosmos-capture");
        if p.exists() { return p; }
    }
    PathBuf::from("../capture/kosmos-capture")
}
