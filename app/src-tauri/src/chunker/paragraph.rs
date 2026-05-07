use super::{Chunk, ChunkConfig, Chunker};

pub struct ParagraphChunker {
    config: ChunkConfig,
}

impl ParagraphChunker {
    pub fn new(config: ChunkConfig) -> Self {
        Self { config }
    }

    fn split_oversized(&self, s: String) -> Vec<String> {
        if s.len() <= self.config.max_chars {
            return vec![s];
        }
        let mut result = Vec::new();
        let mut start = 0;
        while start < s.len() {
            let end = (start + self.config.max_chars).min(s.len());
            // snap end back to a char boundary
            let end = (start..=end)
                .rev()
                .find(|&i| s.is_char_boundary(i))
                .unwrap_or(end);
            let slice = &s[start..end];
            // prefer sentence boundary
            let cut = slice
                .rfind(". ")
                .or_else(|| slice.rfind("? "))
                .or_else(|| slice.rfind("! "))
                .map(|i| start + i + 2)
                .unwrap_or(end);
            // ensure cut is on a char boundary
            let cut = (start..=cut)
                .rev()
                .find(|&i| s.is_char_boundary(i))
                .unwrap_or(end);
            let chunk = s[start..cut].trim().to_string();
            if !chunk.is_empty() {
                result.push(chunk);
            }
            start = cut;
            // skip leading whitespace
            while start < s.len() && s.as_bytes().get(start).map_or(false, |b| b.is_ascii_whitespace()) {
                start += 1;
            }
        }
        result
    }
}

impl Chunker for ParagraphChunker {
    fn name(&self) -> &'static str {
        "paragraph"
    }

    fn chunk(&self, content: &str) -> Vec<Chunk> {
        content
            .split("\n\n")
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .flat_map(|s| self.split_oversized(s))
            .filter(|s| s.len() >= self.config.min_chars)
            .map(Chunk::new)
            .collect()
    }
}
