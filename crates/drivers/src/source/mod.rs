//! Driver sources: define where a driver variable reads its value from.

use crate::variable::{TransformChannel, TransformSpace};

/// Identifies an object in the scene by name or ID.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TargetId {
    /// Object name or unique identifier.
    pub name: String,
    /// Optional bone name (for armature targets).
    pub bone: Option<String>,
}

impl TargetId {
    pub fn object(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            bone: None,
        }
    }

    pub fn bone(object: impl Into<String>, bone: impl Into<String>) -> Self {
        Self {
            name: object.into(),
            bone: Some(bone.into()),
        }
    }
}

/// The source of a driver variable's value.
#[derive(Debug, Clone)]
pub enum DriverSource {
    /// Single property value (RNA path).
    SingleProperty {
        target: TargetId,
        data_path: String,
    },
    /// Transform channel of a target.
    TransformChannel {
        target: TargetId,
        channel: TransformChannel,
        space: TransformSpace,
    },
    /// Distance between two targets.
    Distance {
        target_a: TargetId,
        target_b: TargetId,
        space: TransformSpace,
    },
    /// Rotation difference between two bones (radians).
    RotationDifference {
        target_a: TargetId,
        target_b: TargetId,
    },
    /// Context property (current frame, scene property, etc.).
    ContextProperty {
        data_path: String,
    },
}
