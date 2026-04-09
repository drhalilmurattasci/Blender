//! Transform constraints: copy/maintain location, rotation, and scale.

pub mod copy_location;
pub mod copy_rotation;
pub mod copy_scale;
pub mod copy_transforms;
pub mod maintain_volume;
pub mod transformation;

pub use copy_location::CopyLocation;
pub use copy_rotation::CopyRotation;
pub use copy_scale::CopyScale;
pub use copy_transforms::CopyTransforms;
pub use maintain_volume::MaintainVolume;
pub use transformation::Transformation;

use bitflags::bitflags;
use serde::{Deserialize, Serialize};

bitflags! {
    /// Axis flags used by many transform constraints.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct AxisFlags: u8 {
        const X = 0b001;
        const Y = 0b010;
        const Z = 0b100;
        const ALL = Self::X.bits() | Self::Y.bits() | Self::Z.bits();
    }
}

impl Default for AxisFlags {
    fn default() -> Self {
        Self::ALL
    }
}
