pub mod bvh;
pub mod camera;
pub mod film;
pub mod light;
pub mod material;
pub mod sampling;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("Invalid resolution: {0}x{1}")]
    InvalidResolution(u32, u32),

    #[error("BVH construction error: {0}")]
    BvhConstruction(String),

    #[error("Material error: {0}")]
    Material(String),

    #[error("Sampling error: {0}")]
    Sampling(String),
}

pub type RenderResult<T> = Result<T, RenderError>;

pub use bvh::{Aabb, BvhNode, BvhTree};
pub use camera::{Camera, CameraController, Projection};
pub use film::Film;
pub use light::{DirectionalLight, Light, PointLight, SpotLight};
pub use material::{Material, MaterialType};
pub use sampling::{HaltonSampler, Sampler, StratifiedSampler};
