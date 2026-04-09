//! Limit constraints: clamp location, rotation, scale, distance, and floor.

mod floor;
mod limit_distance;
mod limit_location;
mod limit_rotation;
mod limit_scale;

pub use floor::Floor;
pub use limit_distance::{DistanceClampMode, LimitDistance};
pub use limit_location::LimitLocation;
pub use limit_rotation::LimitRotation;
pub use limit_scale::LimitScale;
