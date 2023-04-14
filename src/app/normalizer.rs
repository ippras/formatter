use egui::util::cache::{ComputerMut, FrameCache};
use std::collections::BTreeMap;

/// Normalized
pub(super) type Normalized = FrameCache<BTreeMap<u64, f64>, Normalizer>;

/// Normalizer
#[derive(Default)]
pub(super) struct Normalizer;

impl ComputerMut<(&BTreeMap<u64, u64>, bool), BTreeMap<u64, f64>> for Normalizer {
    fn compute(&mut self, (peaks, percent): (&BTreeMap<u64, u64>, bool)) -> BTreeMap<u64, f64> {
        let max = peaks.values().max().copied().unwrap_or_default();
        peaks
            .iter()
            .map(|(&mass, &intensity)| {
                let mut intensity = intensity as f64 / max as f64;
                if percent {
                    intensity *= 100.0;
                }
                (mass, intensity)
            })
            .collect()
    }
}
