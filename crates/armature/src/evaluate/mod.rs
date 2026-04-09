//! Armature evaluation: compute final bone transforms from animation and constraints.

mod pipeline;

pub use pipeline::{evaluate_armature, ArmatureEvalData};

use crate::bone::Bone;
use crate::pose::Pose;
use serde::{Deserialize, Serialize};

/// An armature data block containing the skeleton definition and pose.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Armature {
    /// Unique name of the armature.
    pub name: String,
    /// The bones that define the skeleton topology.
    pub bones: Vec<Bone>,
    /// The current pose (animation state).
    pub pose: Pose,
    /// Armature-to-world transform (4x4, column-major).
    #[serde(skip)]
    pub world_matrix: [f32; 16],
}

impl Armature {
    /// Create a new armature with the given bones.
    pub fn new(name: impl Into<String>, bones: Vec<Bone>) -> Self {
        let pose = Pose::from_bones(&bones);
        Self {
            name: name.into(),
            bones,
            pose,
            world_matrix: identity_4x4(),
        }
    }

    /// Number of bones.
    #[inline]
    pub fn bone_count(&self) -> usize {
        self.bones.len()
    }

    /// Find a bone by name.
    pub fn find_bone(&self, name: &str) -> Option<&Bone> {
        self.bones.iter().find(|b| b.name == name)
    }

    /// Get the final deformation matrix for a bone (pose_matrix * rest_matrix_inv).
    ///
    /// This is the matrix used to deform mesh vertices.
    pub fn deform_matrix(&self, bone_index: usize) -> Option<[f32; 16]> {
        if bone_index >= self.bones.len() {
            return None;
        }
        let pose_mat = &self.pose.channels[bone_index].pose_matrix;
        let rest_inv = &self.bones[bone_index].rest_matrix_inv;
        Some(mul_4x4(pose_mat, rest_inv))
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

/// Multiply two 4x4 column-major matrices.
fn mul_4x4(a: &[f32; 16], b: &[f32; 16]) -> [f32; 16] {
    let mut result = [0.0_f32; 16];
    for col in 0..4 {
        for row in 0..4 {
            let mut sum = 0.0;
            for k in 0..4 {
                sum += a[row + k * 4] * b[k + col * 4];
            }
            result[row + col * 4] = sum;
        }
    }
    result
}
