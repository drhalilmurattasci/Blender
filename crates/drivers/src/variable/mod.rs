//! Driver variables: named inputs to driver expressions.

use crate::source::DriverSource;

/// A named variable that feeds into a driver expression.
#[derive(Debug, Clone)]
pub struct DriverVariable {
    /// Display name of the variable.
    pub name: String,
    /// The source that provides this variable's value.
    pub source: DriverSource,
    /// Cached last-evaluated value.
    pub cached_value: f64,
    /// Whether the cache is valid.
    pub cache_valid: bool,
}

impl DriverVariable {
    /// Create a new driver variable.
    pub fn new(name: impl Into<String>, source: DriverSource) -> Self {
        Self {
            name: name.into(),
            source,
            cached_value: 0.0,
            cache_valid: false,
        }
    }

    /// Invalidate the cached value.
    pub fn invalidate(&mut self) {
        self.cache_valid = false;
    }
}

/// Transform channel that can be read from a target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransformChannel {
    LocationX,
    LocationY,
    LocationZ,
    RotationX,
    RotationY,
    RotationZ,
    RotationW,
    ScaleX,
    ScaleY,
    ScaleZ,
}

/// Transform space for reading values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransformSpace {
    /// World space.
    World,
    /// Object/armature local space.
    Local,
    /// Evaluated transform space.
    Transform,
}

impl Default for TransformSpace {
    fn default() -> Self {
        Self::World
    }
}
