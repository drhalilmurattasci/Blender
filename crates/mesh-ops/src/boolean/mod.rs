//! Boolean / CSG operations on meshes.

pub mod csg;
pub mod operation;

pub use csg::{csg_boolean, IntersectionSegment};
pub use operation::BooleanOp;
