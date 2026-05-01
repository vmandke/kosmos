# Kosmos

A local-first memory system for macOS. Captures what you read and work on across your apps — browser, editor, chat — using the Accessibility API, and makes it searchable. No screenshots. No cloud. Everything stays on your machine.

Built as a reaction to tools that solve the right problem but make the wrong privacy tradeoff.

---

## Architecture

### Swift Capture Daemon — `capture/`

Reads text from each app's UI tree via the macOS Accessibility API. No pixels, no OCR.

- Per-app traversal strategies — each app has its own AX extraction logic
- Triggers on focus change, window change, and title change (tab / file switch)
- 5s fallback poll for Electron apps that don't fire AX events reliably
- Sliding-window dedup — drops unchanged content before it hits the socket
- TCC revocation detection — exits cleanly if Accessibility permission is silently removed

### Rust / Tauri Backend — `app/`

Receives captures over a Unix domain socket and owns all storage and retrieval.

- Spawns the Swift daemon as a sidecar on launch
- `[planned]` SQLite with WAL mode and SHA-256 dedup on insert
- `[planned]` FTS5 full-text search
- `[planned]` Chunking (~512 tokens) + embeddings via ONNX (all-MiniLM-L6-v2)
- `[planned]` Hybrid BM25 / cosine ranking

### Frontend — `app/src/`

- Live capture feed
- `[planned]` History view and search UI

---

## Status

Early. The capture pipeline works end to end — Swift reads text, Rust receives it, the frontend displays it live. Storage and search are next. This might change anytime as I read more and get more familiar with the space.

**Up next**

- SQLite persistence in Rust — store every capture, deduplicate on SHA-256
- Full-text search — FTS5 over captured content
- Broader app support — Safari, Slack, Terminal, Notes
- Noise filtering — strip nav chrome, button labels, and aria text before storing
- Semantic search — local embeddings with hybrid keyword + vector ranking

---

## See also

- [Rewind](https://www.rewind.ai) — screen recording + cloud search, the main prior art
- [Screenpipe](https://github.com/mediar-ai/screenpipe) — open-source screen capture pipeline
- [Mem0](https://github.com/mem0ai/mem0) — memory layer for AI agents
- [Apple Accessibility API](https://developer.apple.com/documentation/accessibility)
