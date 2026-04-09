//! Face element.

use forge3d_alloc::Handle;
use forge3d_math::Vec3;
use serde::{Deserialize, Serialize};

use super::flags::ElemFlags;
use super::loop_elem::LoopElem;

/// A mesh face defined by a loop cycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Face {
    /// Handle to the first loop in the face's boundary cycle.
    pub loop_first: Handle<LoopElem>,
    /// Number of vertices (and edges) in this face.
    pub len: u32,
    /// Face normal (computed from the loop cycle).
    pub normal: Vec3,
    /// Material slot index.
    pub material_index: u16,
    /// Element flags.
    pub flags: ElemFlags,
}

impl Face {
    /// Creates a new face with the given first loop handle and vertex count.
    pub fn new(loop_first: Handle<LoopElem>, len: u32) -> Self {
        Self {
            loop_first,
            len,
            normal: Vec3::ZERO,
            material_index: 0,
            flags: ElemFlags::empty(),
        }
    }
}
