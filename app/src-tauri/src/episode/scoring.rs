use crate::capture::Capture;
use crate::tokenizer::tokenize;
use super::Episode;

pub const GAP_THRESHOLD_SECS: u64 = 300;
pub const AFFINITY_THRESHOLD: f32 = 0.25;

pub fn routing_score(capture: &Capture, episode: &Episode) -> f32 {
    let gap = (capture.ts as u64).saturating_sub(episode.last_ts as u64);
    0.5 * temporal_score(gap, GAP_THRESHOLD_SECS)
        + 0.4 * token_overlap_score(capture, episode)
        + 0.1 * source_continuity_score(capture, episode)
}

fn temporal_score(gap: u64, threshold: u64) -> f32 {
    1.0 - (gap as f32 / threshold as f32).min(1.0)
}

fn token_overlap_score(capture: &Capture, episode: &Episode) -> f32 {
    let tokens = tokenize(capture);
    if tokens.is_empty() {
        return 0.0;
    }
    let overlap = tokens.intersection(&episode.domain_tokens).count();
    overlap as f32 / tokens.len() as f32
}

fn source_continuity_score(capture: &Capture, episode: &Episode) -> f32 {
    if episode.sources.contains(&capture.source) { 1.0 } else { 0.3 }
}
