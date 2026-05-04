CREATE TABLE episodes (
    id             TEXT    PRIMARY KEY,
    source         TEXT    NOT NULL,
    sources        TEXT,              -- JSON array ["chrome","vscode"]
    doc_identity   TEXT,
    doc_identities TEXT,              -- JSON array of URLs/paths
    start_ts       INTEGER NOT NULL,
    end_ts         INTEGER
);

CREATE TABLE chunks (
    id           TEXT    PRIMARY KEY,
    episode_id   TEXT    NOT NULL REFERENCES episodes(id),
    content      TEXT    NOT NULL,
    source       TEXT    NOT NULL,
    doc_identity TEXT,
    ts           INTEGER NOT NULL
);

CREATE TABLE chunk_occurrences (
    chunk_id   TEXT    NOT NULL REFERENCES chunks(id),
    episode_id TEXT    NOT NULL REFERENCES episodes(id),
    ts         INTEGER NOT NULL
);
