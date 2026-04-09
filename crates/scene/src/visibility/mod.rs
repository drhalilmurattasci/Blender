//! Visibility and selection state for objects.

use serde::{Deserialize, Serialize};
use bitflags::bitflags;

bitflags! {
    /// Visibility flags controlling display in viewport and render.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct VisibilityFlags: u16 {
        /// Visible in the 3D viewport.
        const VIEWPORT  = 0b0000_0001;
        /// Visible in the final render.
        const RENDER    = 0b0000_0010;
        /// Participates in shadow casting.
        const SHADOW    = 0b0000_0100;
        /// Participates in ray-visibility (reflections, transmission).
        const RAY       = 0b0000_1000;
        /// Visible in diffuse light bounces.
        const DIFFUSE   = 0b0001_0000;
        /// Visible in glossy light bounces.
        const GLOSSY    = 0b0010_0000;
        /// Visible in transmission light bounces.
        const TRANSMISSION = 0b0100_0000;
        /// Visible in volume scatter bounces.
        const VOLUME    = 0b1000_0000;
    }
}

impl Default for VisibilityFlags {
    fn default() -> Self {
        Self::all()
    }
}

/// Object selection state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SelectState {
    /// Not selected.
    None,
    /// Selected but not active.
    Selected,
    /// The active (last-selected) object.
    Active,
}

impl Default for SelectState {
    fn default() -> Self {
        Self::None
    }
}
