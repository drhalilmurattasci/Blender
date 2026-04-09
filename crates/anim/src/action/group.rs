//! Action groups: named subsets of F-Curves within an Action.

use serde::{Deserialize, Serialize};

/// A named group of F-Curve indices within an Action.
///
/// Typically one group per bone, or per property set (e.g., "Location", "Rotation").
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionGroup {
    /// Display name of the group.
    pub name: String,
    /// Indices into the parent Action's `fcurves` vec.
    pub fcurve_indices: Vec<usize>,
    /// Color tag for the group in the UI.
    pub color_set: ColorSet,
    /// Whether the group is locked from editing.
    pub locked: bool,
    /// Whether the group is muted.
    pub muted: bool,
}

impl ActionGroup {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            fcurve_indices: Vec::new(),
            color_set: ColorSet::Default,
            locked: false,
            muted: false,
        }
    }

    /// Add an F-Curve index to this group.
    pub fn add_fcurve(&mut self, index: usize) {
        if !self.fcurve_indices.contains(&index) {
            self.fcurve_indices.push(index);
        }
    }
}

/// Predefined color sets for action groups (maps to theme colors).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ColorSet {
    Default,
    Set01,
    Set02,
    Set03,
    Set04,
    Set05,
    Set06,
    Set07,
    Set08,
    Set09,
    Set10,
    Set11,
    Set12,
    Set13,
    Set14,
    Set15,
    Set16,
    Set17,
    Set18,
    Set19,
    Set20,
    Custom,
}

impl Default for ColorSet {
    fn default() -> Self {
        Self::Default
    }
}
