//! Merge (weld) operations.

pub mod at_center;
pub mod by_distance;

pub use at_center::merge_at_center;
pub use by_distance::merge_by_distance;

/// Parameters for merge-by-distance.
#[derive(Debug, Clone)]
pub struct MergeParams {
    /// Maximum distance between vertices to be merged.
    pub threshold: f32,
}

impl Default for MergeParams {
    fn default() -> Self {
        Self { threshold: 0.001 }
    }
}
