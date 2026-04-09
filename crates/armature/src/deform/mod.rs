//! Mesh deformation (skinning): apply bone transforms to mesh vertices.

mod linear;
mod dual_quaternion;

pub use linear::{linear_blend_skinning, linear_blend_skinning_normals};
pub use dual_quaternion::{dual_quaternion_skinning, dual_quaternion_skinning_normals};

/// A vertex weight: which bone influences this vertex and by how much.
#[derive(Debug, Clone, Copy)]
pub struct VertexWeight {
    /// Bone index.
    pub bone_index: u16,
    /// Weight in `[0, 1]`.
    pub weight: f32,
}

/// A vertex group entry: all bone influences on a single vertex.
#[derive(Debug, Clone)]
pub struct VertexInfluences {
    /// The weights for this vertex (typically 1-4 bones).
    pub weights: Vec<VertexWeight>,
}

impl VertexInfluences {
    /// Normalize weights so they sum to 1.0.
    pub fn normalize(&mut self) {
        let total: f32 = self.weights.iter().map(|w| w.weight).sum();
        if total > f32::EPSILON {
            for w in &mut self.weights {
                w.weight /= total;
            }
        }
    }
}

/// Deformation method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeformMethod {
    /// Linear blend skinning (LBS / vertex blending).
    Linear,
    /// Dual quaternion skinning (DQS).
    DualQuaternion,
}

impl Default for DeformMethod {
    fn default() -> Self {
        Self::Linear
    }
}
