//! Keyframe Bezier handle types.

use serde::{Deserialize, Serialize};

/// Type of a keyframe Bezier handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HandleType {
    /// Freely positioned by the user.
    Free,
    /// Aligned: left and right handles are collinear.
    Aligned,
    /// Vector: handle points toward the adjacent keyframe.
    Vector,
    /// Auto: automatically smoothed.
    Auto,
    /// Auto-clamped: like Auto but prevents overshoot.
    AutoClamped,
}

impl Default for HandleType {
    fn default() -> Self {
        Self::Auto
    }
}

/// A single Bezier handle (the control point on one side of a keyframe).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyframeHandle {
    /// Handle type determines auto-computation behavior.
    pub handle_type: HandleType,
    /// Time coordinate of the handle control point.
    pub x: f32,
    /// Value coordinate of the handle control point.
    pub y: f32,
}

impl KeyframeHandle {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            handle_type: HandleType::Auto,
            x,
            y,
        }
    }

    pub fn with_type(mut self, handle_type: HandleType) -> Self {
        self.handle_type = handle_type;
        self
    }
}
