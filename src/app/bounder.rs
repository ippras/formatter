use crate::app::Bounds;
use egui::util::cache::{ComputerMut, FrameCache};
use std::collections::BTreeMap;

/// Bounded
pub(super) type Bounded = FrameCache<BTreeMap<u64, u64>, Bounder>;

/// Bounder
#[derive(Default)]
pub(super) struct Bounder;

impl ComputerMut<(&BTreeMap<u64, u64>, &Bounds), BTreeMap<u64, u64>> for Bounder {
    fn compute(&mut self, (peaks, bounds): (&BTreeMap<u64, u64>, &Bounds)) -> BTreeMap<u64, u64> {
        peaks
            .iter()
            .filter_map(|(mass, &intensity)| {
                bounds.x().contains(mass).then_some((*mass, intensity))
            }) 
            .collect()
    }
}
