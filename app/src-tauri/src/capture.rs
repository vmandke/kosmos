use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capture {
    pub ts:      String,
    pub app:     String,
    pub title:   String,
    pub content: String,
    pub chars:   usize,
}

pub fn spawn() {
    let path = bin_path();
    match std::process::Command::new(&path).spawn() {
        Ok(_)  => println!("Spawned capture: {path:?}"),
        Err(e) => eprintln!("Could not spawn capture ({path:?}): {e}"),
    }
}

fn bin_path() -> PathBuf {
    // Production: sidecar sits next to the main binary inside Contents/MacOS/
    if let Ok(exe) = std::env::current_exe() {
        let p = exe.parent().unwrap_or(std::path::Path::new(".")).join("kosmos-capture");
        if p.exists() { return p; }
    }
    // Dev: compiled by `make build-capture` into capture/
    PathBuf::from("../capture/kosmos-capture")
}
