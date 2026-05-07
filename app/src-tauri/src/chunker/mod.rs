mod code;
mod fixed;
pub mod normalize;
mod paragraph;

use sha2::{Digest, Sha256};
use std::sync::Arc;

pub use code::CodeChunker;
pub use fixed::FixedChunker;
pub use paragraph::ParagraphChunker;

/// A chunk carries original content (stored verbatim in DB) and a normalized
/// fingerprint (SHA-256 of whitespace-normalized lowercase text) used for dedup.
#[derive(Debug, Clone)]
pub struct Chunk {
    pub content:     String,
    pub fingerprint: String,
}

impl Chunk {
    pub fn new(content: String) -> Self {
        let normalized = normalize::normalize(&content);
        let fingerprint = hex::encode(Sha256::digest(normalized.as_bytes()));
        Self { content, fingerprint }
    }
}

pub trait Chunker: Send + Sync {
    fn chunk(&self, content: &str) -> Vec<Chunk>;
    #[allow(dead_code)]
    fn name(&self) -> &'static str;
}

#[derive(Clone)]
pub struct ChunkConfig {
    pub min_chars: usize,
    pub max_chars: usize,
    pub max_lines: usize,
}

impl Default for ChunkConfig {
    fn default() -> Self {
        Self { min_chars: 50, max_chars: 800, max_lines: 25 }
    }
}

pub fn chunker_for(source: &str, config: ChunkConfig) -> Arc<dyn Chunker> {
    match source {
        "chrome" | "safari" => Arc::new(ParagraphChunker::new(config)),
        "vscode" | "sublime" | "pycharm" => Arc::new(CodeChunker::new(config)),
        _ => Arc::new(FixedChunker::new(config)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_scroll_stability_paragraphs() {
        let chunker = ParagraphChunker::new(ChunkConfig::default());
        let view_1 = "First long paragraph about Raft consensus protocol and how it works in distributed systems across many nodes.\n\nSecond paragraph about Paxos and its variants in modern distributed systems with high availability.\n\nThird paragraph about PBFT and Byzantine fault tolerance protocols used in blockchain.";
        let view_2 = "Second paragraph about Paxos and its variants in modern distributed systems with high availability.\n\nThird paragraph about PBFT and Byzantine fault tolerance protocols used in blockchain.\n\nFourth paragraph about Tendermint and modern consensus algorithms in proof of stake.";

        let fp1: HashSet<_> = chunker.chunk(view_1).into_iter().map(|c| c.fingerprint).collect();
        let fp2: HashSet<_> = chunker.chunk(view_2).into_iter().map(|c| c.fingerprint).collect();

        let overlap = fp1.intersection(&fp2).count();
        assert!(overlap >= 2, "shared paragraphs must produce identical fingerprints, overlap={overlap}");
    }

    #[test]
    fn test_normalization_handles_whitespace() {
        let chunker = ParagraphChunker::new(ChunkConfig::default());
        let v1 = "Hello world this is a test paragraph that is long enough to keep around in chunks.";
        let v2 = "Hello   world   this is  a test paragraph that is long  enough to keep  around in chunks.";

        let c1 = chunker.chunk(v1);
        let c2 = chunker.chunk(v2);
        assert!(!c1.is_empty() && !c2.is_empty());
        assert_eq!(
            c1[0].fingerprint, c2[0].fingerprint,
            "whitespace differences must not change fingerprint"
        );
    }

    #[test]
    fn test_min_size_filter() {
        let chunker = ParagraphChunker::new(ChunkConfig::default());
        let tiny = "ok\n\nyes\n\nno";
        assert!(chunker.chunk(tiny).is_empty());
    }

    #[test]
    fn test_max_size_split() {
        let chunker = ParagraphChunker::new(ChunkConfig::default());
        let long_para: String = (0..50).map(|i| format!("Sentence number {i} in this long paragraph. ")).collect();
        let chunks = chunker.chunk(&long_para);
        assert!(chunks.len() > 1, "oversized paragraph must be split");
        // allow small overshoot at sentence boundary
        assert!(chunks.iter().all(|c| c.content.len() <= 820));
    }

    #[test]
    fn test_code_chunker_blank_line_split() {
        let chunker = CodeChunker::new(ChunkConfig::default());
        let code = "fn one() {\n    println!(\"hello world here this is a long function body\");\n    let x = 42;\n    let y = x + 1;\n}\n\nfn two() {\n    println!(\"another function body here\");\n    let y = 100;\n    let z = y * 2;\n}";
        let chunks = chunker.chunk(code);
        assert_eq!(chunks.len(), 2, "blank line must split into 2 chunks");
    }

    #[test]
    fn test_fingerprint_is_stable() {
        let c1 = Chunk::new("Hello world, this is some content.".to_string());
        let c2 = Chunk::new("Hello world, this is some content.".to_string());
        assert_eq!(c1.fingerprint, c2.fingerprint);
    }
}
