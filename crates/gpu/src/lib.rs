pub mod batch;
pub mod buffer;
pub mod context;
pub mod framebuffer;
pub mod pipeline;
pub mod shader;
pub mod texture;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum GpuError {
    #[error("Failed to request adapter: no suitable GPU found")]
    AdapterNotFound,

    #[error("Failed to request device: {0}")]
    DeviceRequest(#[from] wgpu::RequestDeviceError),

    #[error("Surface error: {0}")]
    Surface(#[from] wgpu::SurfaceError),

    #[error("Surface configuration error: surface not compatible with adapter")]
    SurfaceConfig,

    #[error("Shader compilation error: {0}")]
    ShaderCompilation(String),

    #[error("Buffer creation error: {0}")]
    BufferCreation(String),

    #[error("Texture creation error: {0}")]
    TextureCreation(String),

    #[error("Pipeline creation error: {0}")]
    PipelineCreation(String),

    #[error("Render target error: {0}")]
    RenderTarget(String),

    #[error("Create surface error: {0}")]
    CreateSurface(#[from] wgpu::CreateSurfaceError),
}

pub type GpuResult<T> = Result<T, GpuError>;

pub use batch::{DrawCommand, GpuBatch, PosNormTexTanVertex, PosNormTexVertex, PosVertex, Vertex};
pub use buffer::{StorageBuffer, UniformBuffer, VertexBuffer};
pub use context::{GpuContext, GpuDevice, GpuSurface};
pub use framebuffer::{DepthTarget, FrameBuffer, RenderTarget};
pub use pipeline::{ComputePipelineWrapper, RenderPipelineWrapper};
pub use shader::{ComputePipelineDesc, GpuShader, RenderPipelineDesc};
pub use texture::{GpuTexture, SamplerPreset};
