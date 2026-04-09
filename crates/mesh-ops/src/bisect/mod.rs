//! Bisect operations — cut the mesh along a plane.

pub mod plane_cut;

pub use plane_cut::plane_cut;

use forge3d_math::Vec3;

/// Parameters for the bisect/plane-cut operation.
#[derive(Debug, Clone)]
pub struct BisectParams {
    /// A point on the cutting plane.
    pub plane_co: Vec3,
    /// Normal direction of the cutting plane.
    pub plane_no: Vec3,
    /// If `true`, remove geometry on the positive (outer) side of the plane.
    pub clear_outer: bool,
    /// If `true`, remove geometry on the negative (inner) side of the plane.
    pub clear_inner: bool,
    /// Distance threshold for considering a vertex on the plane.
    pub epsilon: f32,
}

impl Default for BisectParams {
    fn default() -> Self {
        Self {
            plane_co: Vec3::ZERO,
            plane_no: Vec3::Z,
            clear_outer: false,
            clear_inner: false,
            epsilon: 1e-5,
        }
    }
}
