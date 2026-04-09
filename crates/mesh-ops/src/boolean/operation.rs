//! Boolean operation type definitions.

/// The type of boolean operation to perform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BooleanOp {
    /// Keep geometry inside both meshes (intersection).
    Intersect,
    /// Combine both meshes, removing interior geometry (union).
    Union,
    /// Subtract mesh B from mesh A (difference).
    Difference,
}
