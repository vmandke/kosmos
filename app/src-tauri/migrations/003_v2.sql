ALTER TABLE episodes ADD COLUMN suppressed INTEGER NOT NULL DEFAULT 0;
ALTER TABLE episodes ADD COLUMN summary    TEXT;
ALTER TABLE chunks   ADD COLUMN suppressed INTEGER NOT NULL DEFAULT 0;
ALTER TABLE chunks   ADD COLUMN content_hash TEXT;

CREATE INDEX IF NOT EXISTS idx_chunks_hash        ON chunks(content_hash);
CREATE INDEX IF NOT EXISTS idx_chunks_episode     ON chunks(episode_id);
CREATE INDEX IF NOT EXISTS idx_episodes_end_ts    ON episodes(end_ts DESC);
CREATE INDEX IF NOT EXISTS idx_chunks_suppressed  ON chunks(suppressed);
