mod detector;
mod scoring;

use std::collections::HashSet;
use crate::capture::Capture;
use crate::tokenizer::tokenize;

pub use detector::EpisodeDetector;

#[derive(Debug, Clone)]
pub struct Episode {
    pub id:             String,
    pub sources:        HashSet<String>,
    pub doc_identities: HashSet<String>,
    pub domain_tokens:  HashSet<String>,
    pub start_ts:       i64,
    pub end_ts:         Option<i64>,
    pub last_ts:        i64,
    pub suppressed:     bool,
}

impl Episode {
    pub fn new(capture: &Capture) -> Self {
        Self {
            id: ulid::Ulid::new().to_string(),
            sources: HashSet::from([capture.source.clone()]),
            doc_identities: HashSet::from([doc_identity(capture)]),
            domain_tokens: tokenize(capture),
            start_ts: capture.ts,
            end_ts: None,
            last_ts: capture.ts,
            suppressed: false,
        }
    }

    pub fn absorb(&mut self, capture: &Capture) {
        self.sources.insert(capture.source.clone());
        self.doc_identities.insert(doc_identity(capture));
        self.domain_tokens.extend(tokenize(capture));
        self.last_ts = capture.ts;
        self.end_ts = Some(capture.ts);
    }
}

#[derive(Debug)]
pub enum EpisodeDecision {
    Continuing(Episode),
    NewEpisode(Episode),
}

pub fn doc_identity(capture: &Capture) -> String {
    capture.url.clone().unwrap_or_else(|| capture.title.clone())
}
