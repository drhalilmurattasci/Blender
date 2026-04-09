//! Tracking constraints: aim-at, track-to, damped-track, locked-track, stretch-to.

mod damped_track;
mod locked_track;
mod stretch_to;
mod track_to;

pub use damped_track::DampedTrack;
pub use locked_track::LockedTrack;
pub use stretch_to::{StretchTo, VolumeMode};
pub use track_to::TrackTo;

use serde::{Deserialize, Serialize};

/// Which axis of the owner points toward the target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TrackAxis {
    PosX,
    PosY,
    PosZ,
    NegX,
    NegY,
    NegZ,
}

impl Default for TrackAxis {
    fn default() -> Self {
        Self::NegZ
    }
}

/// Which axis of the owner stays "up".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UpAxis {
    X,
    Y,
    Z,
}

impl Default for UpAxis {
    fn default() -> Self {
        Self::Y
    }
}
