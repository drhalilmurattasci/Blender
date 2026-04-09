//! Keyframe types and handle modes.

mod handle;

pub use handle::{HandleType, KeyframeHandle};

use crate::interpolation::InterpolationMode;
use serde::{Deserialize, Serialize};

/// A single keyframe on an F-Curve.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Keyframe {
    /// Frame position (time).
    pub time: f32,
    /// Value at this keyframe.
    pub value: f32,
    /// Interpolation mode to the *next* keyframe.
    pub interpolation: InterpolationMode,
    /// Left (incoming) handle.
    pub handle_left: KeyframeHandle,
    /// Right (outgoing) handle.
    pub handle_right: KeyframeHandle,
    /// Easing mode for non-Bezier interpolations.
    pub easing: EasingMode,
    /// Amplitude for elastic/bounce easing.
    pub amplitude: f32,
    /// Period for elastic easing.
    pub period: f32,
    /// Back overshoot factor.
    pub back: f32,
    /// Selection flag (editor state, not serialized to file).
    #[serde(skip)]
    pub selected: bool,
}

impl Keyframe {
    /// Create a keyframe with default Bezier interpolation.
    pub fn new(time: f32, value: f32) -> Self {
        Self {
            time,
            value,
            interpolation: InterpolationMode::Bezier,
            handle_left: KeyframeHandle::new(time - 1.0, value),
            handle_right: KeyframeHandle::new(time + 1.0, value),
            easing: EasingMode::Auto,
            amplitude: 0.4,
            period: 0.3,
            back: 1.70158,
            selected: false,
        }
    }

    /// Create a keyframe with constant (step) interpolation.
    pub fn constant(time: f32, value: f32) -> Self {
        Self {
            interpolation: InterpolationMode::Constant,
            ..Self::new(time, value)
        }
    }

    /// Create a keyframe with linear interpolation.
    pub fn linear(time: f32, value: f32) -> Self {
        Self {
            interpolation: InterpolationMode::Linear,
            ..Self::new(time, value)
        }
    }
}

/// Easing mode for built-in easing functions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EasingMode {
    /// Automatically choose ease-in or ease-out based on context.
    Auto,
    EaseIn,
    EaseOut,
    EaseInOut,
}

impl Default for EasingMode {
    fn default() -> Self {
        Self::Auto
    }
}
