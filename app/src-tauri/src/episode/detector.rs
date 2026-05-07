use crate::capture::Capture;
use super::{Episode, EpisodeDecision};
use super::scoring::{routing_score, AFFINITY_THRESHOLD, GAP_THRESHOLD_SECS};

pub struct EpisodeDetector {
    pub active: Vec<Episode>,
}

impl EpisodeDetector {
    pub fn new() -> Self {
        Self { active: Vec::new() }
    }

    pub fn process(&mut self, capture: &Capture) -> EpisodeDecision {
        let now = capture.ts as u64;

        // expire stale episodes
        self.active
            .retain(|ep| now.saturating_sub(ep.last_ts as u64) < GAP_THRESHOLD_SECS);

        // find best-matching active episode
        let best = self
            .active
            .iter()
            .enumerate()
            .map(|(i, ep)| (i, routing_score(capture, ep)))
            .filter(|(_, score)| *score > AFFINITY_THRESHOLD)
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        match best {
            Some((idx, _)) => {
                self.active[idx].absorb(capture);
                EpisodeDecision::Continuing(self.active[idx].clone())
            }
            None => {
                let ep = Episode::new(capture);
                self.active.push(ep.clone());
                EpisodeDecision::NewEpisode(ep)
            }
        }
    }
}
