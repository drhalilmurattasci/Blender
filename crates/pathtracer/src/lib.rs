pub mod integrator;
pub mod kernel;
pub mod scene;
pub mod session;
pub mod tile;
pub mod wavefront;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum PathtraceError {
    #[error("Scene error: {0}")]
    Scene(String),

    #[error("Render cancelled")]
    Cancelled,

    #[error("Invalid configuration: {0}")]
    Config(String),

    #[error("Kernel error: {0}")]
    Kernel(String),
}

pub type PathtraceResult<T> = Result<T, PathtraceError>;

pub use integrator::PathIntegrator;
pub use kernel::RayKernel;
pub use scene::PathtracerScene;
pub use session::{RenderSession, RenderStatus};
pub use tile::TileScheduler;
pub use wavefront::WavefrontPathtracer;
