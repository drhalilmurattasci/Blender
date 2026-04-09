//! Constraint target references.

use serde::{Deserialize, Serialize};

/// A target reference for a constraint (object and optional bone).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintTarget {
    /// Name of the target object.
    pub object: String,
    /// Optional bone name within an armature target.
    pub bone: Option<String>,
    /// Vertex group name for vertex-based targets.
    pub vertex_group: Option<String>,
}

impl ConstraintTarget {
    /// Create a target referencing an object.
    pub fn object(name: impl Into<String>) -> Self {
        Self {
            object: name.into(),
            bone: None,
            vertex_group: None,
        }
    }

    /// Create a target referencing a specific bone on an armature.
    pub fn bone(object: impl Into<String>, bone: impl Into<String>) -> Self {
        Self {
            object: object.into(),
            bone: Some(bone.into()),
            vertex_group: None,
        }
    }

    /// Whether this target references a bone.
    #[inline]
    pub fn is_bone_target(&self) -> bool {
        self.bone.is_some()
    }
}
