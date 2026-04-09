//! # forge3d-anim
//!
//! Core animation data types and evaluation for Forge3D.
//!
//! Provides F-Curves, keyframes, actions, interpolation methods, and
//! animation modifiers (noise, cycles, envelope, etc.).

pub mod action;
pub mod fcurve;
pub mod interpolation;
pub mod keyframe;
pub mod modifiers;

use thiserror::Error;

/// Errors that can occur during animation evaluation.
#[derive(Debug, Error)]
pub enum AnimError {
    #[error("keyframe index {index} out of range (count: {count})")]
    KeyframeOutOfRange { index: usize, count: usize },

    #[error("empty F-Curve has no value to evaluate")]
    EmptyFCurve,

    #[error("action group `{0}` not found")]
    GroupNotFound(String),

    #[error("invalid time range: start ({start}) >= end ({end})")]
    InvalidTimeRange { start: f32, end: f32 },

    #[error("modifier evaluation failed: {0}")]
    ModifierError(String),
}

/// Result alias for animation operations.
pub type AnimResult<T> = Result<T, AnimError>;

/// A time value in frames (can be fractional for sub-frame precision).
pub type FrameTime = f32;

/// Identifies which property component an F-Curve drives (e.g., X=0, Y=1, Z=2, W=3).
pub type ArrayIndex = u32;

/// Extrapolation mode for F-Curves outside their keyframe range.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Extrapolation {
    /// Hold the boundary keyframe value.
    Constant,
    /// Linearly extrapolate from the boundary tangent.
    Linear,
    /// The value wraps around, producing a repeating pattern.
    /// Actual cycle behaviour is handled by the Cycles modifier.
    MakesCyclic,
}

impl Default for Extrapolation {
    fn default() -> Self {
        Self::Constant
    }
}

/// Identifies the data-path an animation channel targets.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct DataPath {
    /// RNA-style property path, e.g. `"location"`, `"pose.bones[\"Arm\"].rotation_quaternion"`.
    pub path: String,
    /// Component index within the property (0 for scalar properties).
    pub index: ArrayIndex,
}

impl DataPath {
    pub fn new(path: impl Into<String>, index: ArrayIndex) -> Self {
        Self {
            path: path.into(),
            index,
        }
    }
}
