//! Interpolation modes and evaluation functions.

mod bezier;
mod easing;

pub use bezier::evaluate_bezier;
pub use easing::{ease_in, ease_in_out, ease_out};

use serde::{Deserialize, Serialize};

/// Interpolation mode between two keyframes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InterpolationMode {
    /// No interpolation; value holds until the next keyframe.
    Constant,
    /// Linear interpolation between keyframes.
    Linear,
    /// Cubic Bezier interpolation using handles.
    Bezier,
    /// Sine easing.
    Sine,
    /// Quadratic easing.
    Quad,
    /// Cubic easing.
    Cubic,
    /// Quartic easing.
    Quart,
    /// Quintic easing.
    Quint,
    /// Exponential easing.
    Expo,
    /// Circular easing.
    Circ,
    /// Back easing (overshoot).
    Back,
    /// Bounce easing.
    Bounce,
    /// Elastic easing.
    Elastic,
}

impl Default for InterpolationMode {
    fn default() -> Self {
        Self::Bezier
    }
}

/// Linearly interpolate between two values.
#[inline]
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Compute the interpolation factor `t` in `[0, 1]` for a time between two keyframes.
#[inline]
pub fn time_factor(time: f32, start: f32, end: f32) -> f32 {
    if (end - start).abs() < f32::EPSILON {
        0.0
    } else {
        ((time - start) / (end - start)).clamp(0.0, 1.0)
    }
}
