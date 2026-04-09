//! Bone definition: the structural (edit-mode) bone data.

pub mod hierarchy;
pub mod properties;

pub use hierarchy::BoneHierarchy;
pub use properties::BoneProperties;

use crate::{BoneIndex, NO_PARENT, RotationMode};
use serde::{Deserialize, Serialize};

/// A bone in its rest (edit-mode) pose.
///
/// Bones define the skeleton topology and rest transforms.
/// At evaluation time, pose channels override these transforms.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bone {
    /// Unique name within the armature.
    pub name: String,
    /// Index of this bone in the armature's flat array.
    pub index: BoneIndex,
    /// Parent bone index, or `NO_PARENT` if root.
    pub parent: BoneIndex,
    /// Indices of child bones.
    pub children: Vec<BoneIndex>,

    // --- Rest pose transform (edit-mode) ---
    /// Head position in armature-local space.
    pub head: [f32; 3],
    /// Tail position in armature-local space.
    pub tail: [f32; 3],
    /// Roll angle around the bone's Y axis (radians).
    pub roll: f32,

    // --- Rest pose matrix (computed from head/tail/roll) ---
    /// Bone-to-armature transform (4x4, column-major).
    pub rest_matrix: [f32; 16],
    /// Armature-to-bone transform (inverse of rest_matrix).
    pub rest_matrix_inv: [f32; 16],

    /// Bone length (distance from head to tail).
    pub length: f32,

    /// Rotation mode for the associated pose channel.
    pub rotation_mode: RotationMode,

    /// Additional bone properties.
    pub properties: BoneProperties,
}

impl Bone {
    /// Create a new bone with head and tail positions.
    pub fn new(name: impl Into<String>, index: BoneIndex, head: [f32; 3], tail: [f32; 3]) -> Self {
        let dx = tail[0] - head[0];
        let dy = tail[1] - head[1];
        let dz = tail[2] - head[2];
        let length = (dx * dx + dy * dy + dz * dz).sqrt();

        Self {
            name: name.into(),
            index,
            parent: NO_PARENT,
            children: Vec::new(),
            head,
            tail,
            roll: 0.0,
            rest_matrix: identity_4x4(),
            rest_matrix_inv: identity_4x4(),
            length,
            rotation_mode: RotationMode::Quaternion,
            properties: BoneProperties::default(),
        }
    }

    /// Whether this bone is a root bone (no parent).
    #[inline]
    pub fn is_root(&self) -> bool {
        self.parent == NO_PARENT
    }

    /// Whether this bone has children.
    #[inline]
    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }
}

fn identity_4x4() -> [f32; 16] {
    [
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0,
    ]
}
