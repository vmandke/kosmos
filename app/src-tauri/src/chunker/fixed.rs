use super::{Chunk, ChunkConfig, Chunker};

pub struct FixedChunker {
    config: ChunkConfig,
}

impl FixedChunker {
    pub fn new(config: ChunkConfig) -> Self {
        Self { config }
    }
}

impl Chunker for FixedChunker {
    fn name(&self) -> &'static str {
        "fixed"
    }

    fn chunk(&self, content: &str) -> Vec<Chunk> {
        let chars: Vec<char> = content.chars().collect();
        chars
            .chunks(self.config.max_chars)
            .map(|c| c.iter().collect::<String>())
            .filter(|s| s.len() >= self.config.min_chars)
            .map(Chunk::new)
            .collect()
    }
}
