//! Dirty tags: bitflags indicating which aspects of a node need re-evaluation.

use std::ops::{BitOr, BitOrAssign};

/// Dirty tags for dependency graph nodes.
///
/// Each bit indicates a particular aspect that has changed and needs
/// re-evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DirtyTags(u32);

impl DirtyTags {
    /// No dirty flags.
    pub const fn empty() -> Self {
        Self(0)
    }

    /// Transform has changed.
    pub const TRANSFORM: Self = Self(1 << 0);
    /// Geometry has changed.
    pub const GEOMETRY: Self = Self(1 << 1);
    /// Animation data has changed.
    pub const ANIMATION: Self = Self(1 << 2);
    /// Time has changed (new frame).
    pub const TIME: Self = Self(1 << 3);
    /// Shading/material has changed.
    pub const SHADING: Self = Self(1 << 4);
    /// Constraint has changed.
    pub const CONSTRAINTS: Self = Self(1 << 5);
    /// Particle system has changed.
    pub const PARTICLES: Self = Self(1 << 6);
    /// Copy-on-write needed.
    pub const COPY_ON_WRITE: Self = Self(1 << 7);
    /// Parameters changed.
    pub const PARAMETERS: Self = Self(1 << 8);
    /// Selection changed.
    pub const SELECT: Self = Self(1 << 9);
    /// Visibility changed.
    pub const VISIBILITY: Self = Self(1 << 10);
    /// Everything is dirty.
    pub const ALL: Self = Self(0x7FF);

    /// Check if any flags are set.
    #[inline]
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Check if specific flags are set.
    #[inline]
    pub fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    /// Check if any of the specified flags are set.
    #[inline]
    pub fn intersects(self, other: Self) -> bool {
        (self.0 & other.0) != 0
    }

    /// Raw bits value.
    #[inline]
    pub fn bits(self) -> u32 {
        self.0
    }
}

impl BitOr for DirtyTags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for DirtyTags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

/// An update tag that can be sent to the depsgraph to notify of changes.
#[derive(Debug, Clone)]
pub struct UpdateTag {
    /// Name of the data block that changed.
    pub id_name: String,
    /// What changed.
    pub tags: DirtyTags,
}

impl UpdateTag {
    pub fn new(id_name: impl Into<String>, tags: DirtyTags) -> Self {
        Self {
            id_name: id_name.into(),
            tags,
        }
    }
}
