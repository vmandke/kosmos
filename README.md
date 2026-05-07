# Kosmos

A local-first memory system for macOS. Reads text from your apps via the Accessibility API and makes it searchable. No screenshots. No cloud. Stays on your machine.

Built as a reaction to tools that solve the right problem but make the wrong privacy tradeoff.

---

## Architecture

### Swift capture daemon — `capture/`

Reads text from each app's UI tree via the macOS Accessibility API. No pixels, no OCR.

- Per-app extraction strategies: each app has its own AX traversal logic
- Triggers on focus change, window change, and title change (tab/file switch)
- 5s fallback poll for Electron apps that don't fire AX events reliably
- Sliding-window dedup before anything hits the socket
- TCC revocation detection: exits cleanly if Accessibility permission is removed

### Rust/Tauri backend — `app/src-tauri/`

Receives captures over a Unix domain socket. Owns all storage and retrieval.

- SQLite with WAL mode and FTS5 full-text search (porter stemmer)
- SHA-256 content fingerprinting: same content seen twice writes one chunk row and one occurrence row, not two chunks
- Per-source tokenizer: URL host/path/query for browsers, file path segments for editors, window title fallback for everything else
- Per-source chunker: paragraph splits for browsers, blank-line splits for editors, fixed-size fallback
- Episode builder: clusters captures dynamically using temporal + token-overlap + source-continuity affinity scoring. Multiple episodes can be active at once.
- Background worker queue: pluggable workers over a tokio channel. Summarizer V0 concatenates first few chunks; V1 will call a model.

### Frontend — `app/index.html`

- **Live**: streaming feed of captures as they arrive
- **Episodes**: filterable list of sessions, expandable to chunks, each chunk shows occurrence history
- **Search**: FTS5 full-text search with keyword highlighting and recency decay ranking

---

## Supported apps

| App | Capture strategy |
|-----|-----------------|
| Chrome | URL + page text |
| VS Code | File path + editor content |
| Sublime Text | File path + editor content |
| PyCharm | File path + editor content |
| Discord | Channel + message text |

---

## Status

Still Early. Captures flow from the AX daemon through Rust into SQLite, episodes cluster up over a session, and search works. The background worker stub is wired in but the summarizer is V0 (no model call yet).

What's still missing:
- Embeddings + semantic search (local ONNX, was planning all-MiniLM-L6-v2)
- Episode summaries via LLM (worker is there, model call isn't)
- Broader app support: Safari, Slack, Terminal, Notes

This might change as I read more and get more familiar with the space.

---

## Running

```sh
make dev       # build Swift capture + start Tauri in dev mode
make build     # production build
make capture   # run the capture daemon standalone (foreground)
make log       # stream os.log from the daemon (open a second terminal)
```

---

## Bugs

- **Episodes is still In-progress**: And it needs thorough testing.
- **Espisodes Dont list individual chunks on clicking**
- **Chunking is not clean**: And it needs thorough testing.
- **Search is very basic**
- **Episode formations needs to be more sophisticated**
