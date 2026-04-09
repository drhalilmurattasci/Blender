//! # forge3d-nla
//!
//! Non-Linear Animation (NLA) system for Forge3D.
//!
//! Provides NLA tracks, strips, blending modes, and evaluation
//! to layer and combine multiple animation actions.

pub mod blend;
pub mod evaluate;
pub mod strip;
pub mod track;

/// Blend mode for NLA strips.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum NlaBlendMode {
    /// Replace: the strip's value replaces the accumulated value.
    Replace,
    /// Combine: the strip's value is added to the rest pose, then combined.
    Combine,
    /// Add: the strip's delta from its rest pose is added.
    Add,
    /// Subtract: the strip's delta is subtracted.
    Subtract,
    /// Multiply: the accumulated value is multiplied by the strip's value.
    Multiply,
}

impl Default for NlaBlendMode {
    fn default() -> Self {
        Self::Replace
    }
}

/// Extrapolation mode for NLA strips outside their time range.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum NlaExtrapolation {
    /// No value outside the strip's range.
    Nothing,
    /// Hold the boundary value.
    Hold,
    /// Hold the forward boundary only.
    HoldForward,
}

impl Default for NlaExtrapolation {
    fn default() -> Self {
        Self::Hold
    }
}
