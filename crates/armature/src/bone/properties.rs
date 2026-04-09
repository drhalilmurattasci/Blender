//! Bone property flags and display settings.

use serde::{Deserialize, Serialize};

/// Bone display type in the viewport.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BoneDisplayType {
    Octahedral,
    Stick,
    BoneShape,
    Envelope,
    Wire,
}

impl Default for BoneDisplayType {
    fn default() -> Self {
        Self::Octahedral
    }
}

/// Additional bone properties.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoneProperties {
    /// Whether the bone is connected to its parent (head snaps to parent's tail).
    pub connected: bool,
    /// Whether the bone inherits parent rotation.
    pub inherit_rotation: bool,
    /// Whether the bone inherits parent scale.
    pub inherit_scale: InheritScale,
    /// Whether the bone is visible.
    pub visible: bool,
    /// Whether the bone is selectable.
    pub selectable: bool,
    /// Whether the bone is used as a deform bone.
    pub deform: bool,
    /// Bone layers (32-bit bitmask).
    pub layers: u32,
    /// Display type.
    pub display_type: BoneDisplayType,
    /// Envelope distance for envelope display/deformation.
    pub envelope_distance: f32,
    /// Envelope weight.
    pub envelope_weight: f32,
    /// Head radius for envelope.
    pub head_radius: f32,
    /// Tail radius for envelope.
    pub tail_radius: f32,
}

/// How a bone inherits its parent's scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InheritScale {
    /// Full inheritance.
    Full,
    /// Fix shear (remove parent shear from the result).
    FixShear,
    /// Aligned (inherit magnitude, not orientation).
    Aligned,
    /// Average (use the average of parent's scale axes).
    Average,
    /// None (do not inherit scale).
    None,
    /// No scale, same as None but preserves offset.
    NoneLocal,
}

impl Default for InheritScale {
    fn default() -> Self {
        Self::Full
    }
}

impl Default for BoneProperties {
    fn default() -> Self {
        Self {
            connected: false,
            inherit_rotation: true,
            inherit_scale: InheritScale::Full,
            visible: true,
            selectable: true,
            deform: true,
            layers: 1,
            display_type: BoneDisplayType::Octahedral,
            envelope_distance: 0.25,
            envelope_weight: 1.0,
            head_radius: 0.1,
            tail_radius: 0.1,
        }
    }
}
