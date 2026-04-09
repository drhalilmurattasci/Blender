//! Limit constraints: clamp location, rotation, scale, and distance.

mod limit_distance;
mod limit_location;
mod limit_rotation;
mod limit_scale;

pub use limit_distance::LimitDistance;
pub use limit_location::LimitLocation;
pub use limit_rotation::LimitRotation;
pub use limit_scale::LimitScale;
