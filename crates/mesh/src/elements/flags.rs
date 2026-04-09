//! Element-level bitflags shared by all mesh element types.

use bitflags::bitflags;
use serde::{Deserialize, Serialize};

bitflags! {
    /// Flags that can be set on any mesh element (vert, edge, loop, face).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct ElemFlags: u32 {
        /// Element is selected.
        const SELECT   = 1 << 0;
        /// Element is hidden.
        const HIDE     = 1 << 1;
        /// Smooth shading (faces) or smooth normal (verts/edges).
        const SMOOTH   = 1 << 2;
        /// Marks a UV seam edge.
        const SEAM     = 1 << 3;
        /// Marks a sharp (crease) edge.
        const SHARP    = 1 << 4;
        /// General-purpose tag for algorithms.
        const TAG      = 1 << 5;
        /// Internal element, not directly editable by the user.
        const INTERNAL = 1 << 6;
    }
}

impl Default for ElemFlags {
    fn default() -> Self {
        ElemFlags::empty()
    }
}
