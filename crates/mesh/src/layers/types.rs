//! Custom data layer type descriptors.

use serde::{Deserialize, Serialize};

/// Describes the data type stored in a custom data layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LayerType {
    /// Single `f32` value.
    Float,
    /// Two-component `[f32; 2]` (e.g. UV coordinates).
    Vec2,
    /// Three-component `[f32; 3]` (e.g. vertex colors RGB, displacement).
    Vec3,
    /// Four-component `[f32; 4]` color (RGBA).
    Color4f,
    /// Signed 32-bit integer.
    Int,
    /// Boolean flag.
    Bool,
}

impl LayerType {
    /// Returns the byte size of a single element of this type.
    pub fn element_size(self) -> usize {
        match self {
            LayerType::Float => 4,
            LayerType::Vec2 => 8,
            LayerType::Vec3 => 12,
            LayerType::Color4f => 16,
            LayerType::Int => 4,
            LayerType::Bool => 1,
        }
    }
}
