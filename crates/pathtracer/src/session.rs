use forge3d_render_core::camera::Camera;
use forge3d_render_core::film::Film;

use crate::integrator::{IntegratorConfig, PathIntegrator};
use crate::kernel::RayKernel;
use crate::scene::PathtracerScene;
use crate::tile::TileScheduler;
use crate::{PathtraceError, PathtraceResult};

/// Status of the render session.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RenderStatus {
    Idle,
    Rendering,
    Completed,
    Cancelled,
}

/// Configuration for a render session.
#[derive(Debug, Clone)]
pub struct RenderConfig {
    pub width: u32,
    pub height: u32,
    pub samples_per_pixel: u32,
    pub tile_size: u32,
    pub integrator: IntegratorConfig,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 720,
            samples_per_pixel: 64,
            tile_size: 32,
            integrator: IntegratorConfig::default(),
        }
    }
}

/// A render session managing progressive path tracing.
pub struct RenderSession {
    pub config: RenderConfig,
    pub film: Film,
    pub status: RenderStatus,
    pub current_sample: u32,
    kernel: RayKernel,
    scheduler: TileScheduler,
}

impl RenderSession {
    /// Create a new render session.
    pub fn new(config: RenderConfig) -> PathtraceResult<Self> {
        if config.width == 0 || config.height == 0 {
            return Err(PathtraceError::Config("Resolution must be non-zero".into()));
        }

        let film = Film::new(config.width, config.height);
        let integrator = PathIntegrator::new(config.integrator.clone());
        let kernel = RayKernel::new(integrator, config.samples_per_pixel);
        let scheduler = TileScheduler::new(
            config.width,
            config.height,
            config.tile_size,
        );

        Ok(Self {
            config,
            film,
            status: RenderStatus::Idle,
            current_sample: 0,
            kernel,
            scheduler,
        })
    }

    /// Start or resume rendering.
    pub fn render_sample(
        &mut self,
        scene: &PathtracerScene,
        camera: &Camera,
    ) -> PathtraceResult<()> {
        if self.status == RenderStatus::Cancelled {
            return Err(PathtraceError::Cancelled);
        }

        if self.current_sample >= self.config.samples_per_pixel {
            self.status = RenderStatus::Completed;
            return Ok(());
        }

        self.status = RenderStatus::Rendering;

        let tiles = self.scheduler.generate_tiles();
        for tile in &tiles {
            self.kernel.render_tile_progressive(
                scene,
                camera,
                &mut self.film,
                tile,
                self.current_sample,
            );
        }

        self.current_sample += 1;

        if self.current_sample >= self.config.samples_per_pixel {
            self.status = RenderStatus::Completed;
        }

        Ok(())
    }

    /// Cancel the render.
    pub fn cancel(&mut self) {
        self.status = RenderStatus::Cancelled;
    }

    /// Reset the session for a new render.
    pub fn reset(&mut self) {
        self.film.clear();
        self.current_sample = 0;
        self.status = RenderStatus::Idle;
    }

    /// Get the current progress as a fraction [0, 1].
    pub fn progress(&self) -> f32 {
        self.current_sample as f32 / self.config.samples_per_pixel as f32
    }
}
