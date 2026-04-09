use forge3d_gpu::framebuffer::RenderTarget;

/// Screen-space ambient occlusion configuration.
#[derive(Debug, Clone)]
pub struct SsaoConfig {
    pub sample_count: u32,
    pub radius: f32,
    pub bias: f32,
    pub intensity: f32,
    pub blur_passes: u32,
    pub half_resolution: bool,
}

impl Default for SsaoConfig {
    fn default() -> Self {
        Self {
            sample_count: 32,
            radius: 0.5,
            bias: 0.025,
            intensity: 1.5,
            blur_passes: 2,
            half_resolution: true,
        }
    }
}

/// SSAO pass producing an occlusion texture.
pub struct SsaoPass {
    pub config: SsaoConfig,
    pub occlusion_target: RenderTarget,
    pub blur_target: RenderTarget,
    pub width: u32,
    pub height: u32,
}

impl SsaoPass {
    /// Create a new SSAO pass.
    pub fn new(device: &wgpu::Device, width: u32, height: u32, config: SsaoConfig) -> Self {
        let (w, h) = if config.half_resolution {
            ((width / 2).max(1), (height / 2).max(1))
        } else {
            (width, height)
        };

        let occlusion_target = RenderTarget::new(
            device,
            "ssao_occlusion",
            w,
            h,
            wgpu::TextureFormat::R8Unorm,
        );

        let blur_target = RenderTarget::new(
            device,
            "ssao_blur",
            w,
            h,
            wgpu::TextureFormat::R8Unorm,
        );

        Self {
            config,
            occlusion_target,
            blur_target,
            width: w,
            height: h,
        }
    }

    /// Resize the SSAO targets.
    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        let (w, h) = if self.config.half_resolution {
            ((width / 2).max(1), (height / 2).max(1))
        } else {
            (width, height)
        };
        self.width = w;
        self.height = h;
        self.occlusion_target.resize(device, w, h);
        self.blur_target.resize(device, w, h);
    }

    /// Get the final occlusion texture view (after blur).
    pub fn output_view(&self) -> &wgpu::TextureView {
        self.blur_target.view()
    }
}
