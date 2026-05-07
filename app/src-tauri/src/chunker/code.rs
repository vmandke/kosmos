use super::{Chunk, ChunkConfig, Chunker};

pub struct CodeChunker {
    config: ChunkConfig,
}

impl CodeChunker {
    pub fn new(config: ChunkConfig) -> Self {
        Self { config }
    }
}

impl Chunker for CodeChunker {
    fn name(&self) -> &'static str {
        "code"
    }

    fn chunk(&self, content: &str) -> Vec<Chunk> {
        let mut chunks = Vec::new();
        let mut current: Vec<&str> = Vec::new();
        let min = self.config.min_chars;
        let max_lines = self.config.max_lines;

        let flush = |buf: &mut Vec<&str>, out: &mut Vec<Chunk>| {
            if !buf.is_empty() {
                let joined = buf.join("\n");
                if joined.len() >= min {
                    out.push(Chunk::new(joined));
                }
                buf.clear();
            }
        };

        for line in content.lines() {
            if line.trim().is_empty() {
                flush(&mut current, &mut chunks);
            } else {
                current.push(line);
                if current.len() >= max_lines {
                    flush(&mut current, &mut chunks);
                }
            }
        }
        flush(&mut current, &mut chunks);
        chunks
    }
}
