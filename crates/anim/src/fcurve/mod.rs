//! F-Curve: the fundamental animation curve type.

mod evaluate;
mod sampling;

pub use evaluate::evaluate_fcurve;
pub use sampling::sample_fcurve_range;

use crate::keyframe::Keyframe;
use crate::modifiers::FcurveModifier;
use crate::{DataPath, Extrapolation};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

/// An F-Curve maps time (frames) to a single scalar value for a specific property component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FCurve {
    /// The property this curve animates.
    pub data_path: DataPath,
    /// Sorted list of keyframes (sorted by time).
    pub keyframes: Vec<Keyframe>,
    /// How to extrapolate before the first keyframe.
    pub extrapolation_before: Extrapolation,
    /// How to extrapolate after the last keyframe.
    pub extrapolation_after: Extrapolation,
    /// Stack of modifiers applied after base evaluation.
    pub modifiers: SmallVec<[FcurveModifier; 2]>,
    /// Whether this F-Curve is muted (skipped during evaluation).
    pub muted: bool,
    /// Whether auto-handle smoothing is enabled.
    pub auto_smooth: bool,
}

impl FCurve {
    /// Create a new empty F-Curve for the given data path.
    pub fn new(path: impl Into<String>, index: u32) -> Self {
        Self {
            data_path: DataPath::new(path, index),
            keyframes: Vec::new(),
            extrapolation_before: Extrapolation::Constant,
            extrapolation_after: Extrapolation::Constant,
            modifiers: SmallVec::new(),
            muted: false,
            auto_smooth: true,
        }
    }

    /// Insert a keyframe, maintaining sorted order by time.
    /// If a keyframe already exists at the same time (within epsilon), it is replaced.
    pub fn insert_keyframe(&mut self, kf: Keyframe) {
        let pos = self
            .keyframes
            .binary_search_by(|k| k.time.partial_cmp(&kf.time).unwrap_or(std::cmp::Ordering::Equal));
        match pos {
            Ok(idx) => self.keyframes[idx] = kf,
            Err(idx) => self.keyframes.insert(idx, kf),
        }
    }

    /// Remove the keyframe at the given index.
    pub fn remove_keyframe(&mut self, index: usize) -> Option<Keyframe> {
        if index < self.keyframes.len() {
            Some(self.keyframes.remove(index))
        } else {
            None
        }
    }

    /// Number of keyframes.
    #[inline]
    pub fn keyframe_count(&self) -> usize {
        self.keyframes.len()
    }

    /// Time range spanned by keyframes, or `None` if empty.
    pub fn time_range(&self) -> Option<(f32, f32)> {
        if self.keyframes.is_empty() {
            None
        } else {
            Some((
                self.keyframes.first().unwrap().time,
                self.keyframes.last().unwrap().time,
            ))
        }
    }
}
