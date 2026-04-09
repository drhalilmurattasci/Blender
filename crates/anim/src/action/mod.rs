//! Actions: named collections of F-Curves that can be assigned to objects.

mod group;

pub use group::ActionGroup;

use crate::fcurve::FCurve;
use crate::FrameTime;
use serde::{Deserialize, Serialize};

/// An Action is a reusable animation data-block containing one or more F-Curves
/// organized into groups (e.g., by bone name or property set).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    /// Unique name of the action.
    pub name: String,
    /// All F-Curves in this action.
    pub fcurves: Vec<FCurve>,
    /// Named groups that organize F-Curves.
    pub groups: Vec<ActionGroup>,
    /// Manual frame range override (if set, used instead of auto-computed range).
    pub frame_range: Option<(FrameTime, FrameTime)>,
    /// Whether to use the manual frame range.
    pub use_frame_range: bool,
    /// Unique identifier for this action within the blend file.
    pub id: u64,
}

impl Action {
    /// Create a new empty action.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            fcurves: Vec::new(),
            groups: Vec::new(),
            frame_range: None,
            use_frame_range: false,
            id: 0,
        }
    }

    /// Compute the frame range encompassing all F-Curves.
    pub fn computed_frame_range(&self) -> Option<(FrameTime, FrameTime)> {
        if self.use_frame_range {
            if let Some(range) = self.frame_range {
                return Some(range);
            }
        }

        let mut min_time = f32::MAX;
        let mut max_time = f32::MIN;
        let mut found = false;

        for fc in &self.fcurves {
            if let Some((start, end)) = fc.time_range() {
                min_time = min_time.min(start);
                max_time = max_time.max(end);
                found = true;
            }
        }

        if found {
            Some((min_time, max_time))
        } else {
            None
        }
    }

    /// Find all F-Curves matching a given data path string.
    pub fn fcurves_for_path(&self, path: &str) -> Vec<&FCurve> {
        self.fcurves
            .iter()
            .filter(|fc| fc.data_path.path == path)
            .collect()
    }

    /// Add an F-Curve to this action.
    pub fn add_fcurve(&mut self, fcurve: FCurve) {
        self.fcurves.push(fcurve);
    }

    /// Find or create a group by name, returning its index.
    pub fn ensure_group(&mut self, name: impl Into<String>) -> usize {
        let name = name.into();
        if let Some(idx) = self.groups.iter().position(|g| g.name == name) {
            idx
        } else {
            let idx = self.groups.len();
            self.groups.push(ActionGroup::new(name));
            idx
        }
    }
}
