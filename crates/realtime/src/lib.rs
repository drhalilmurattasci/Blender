pub mod bloom;
pub mod dof;
pub mod gbuffer;
pub mod lighting;
pub mod pipeline;
pub mod shadow;
pub mod ssao;
pub mod ssr;
pub mod taa;
pub mod volumetric;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum RealtimeError {
    #[error("GPU error: {0}")]
    Gpu(#[from] forge3d_gpu::GpuError),

    #[error("Pipeline error: {0}")]
    Pipeline(String),

    #[error("Resource error: {0}")]
    Resource(String),
}

pub type RealtimeResult<T> = Result<T, RealtimeError>;

pub use bloom::BloomPass;
pub use dof::DepthOfFieldPass;
pub use gbuffer::GBufferPass;
pub use lighting::LightingPass;
pub use pipeline::RealtimePipeline;
pub use shadow::ShadowPass;
pub use ssao::SsaoPass;
pub use ssr::SsrPass;
pub use taa::TaaPass;
pub use volumetric::VolumetricPass;
