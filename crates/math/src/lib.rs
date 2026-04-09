pub mod traits;
pub mod vec;
pub mod mat;
pub mod quat;
pub mod color;
pub mod geometry;

// Re-export everything flat for convenience.
pub use traits::{ApproxEq, Lerp};

pub use vec::{Vec2, Vec3, Vec4};
pub use mat::{Mat3, Mat4};
pub use quat::{EulerOrder, Quat};
pub use color::{Color3f, Color4f};
pub use geometry::{Aabb, Plane, Ray, Transform};
pub use geometry::{
    closest_point_on_tri, closest_to_line_segment_v3, closest_to_line_v3,
    dist_squared_to_line_segment_v3, interp_weights_poly_v3, isect_ray_tri_v3,
};
