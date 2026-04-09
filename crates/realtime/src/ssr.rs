use forge3d_gpu::framebuffer::RenderTarget;
use forge3d_gpu::texture::format::hdr_color_format;

/// Screen-space reflections configuration.
#[derive(Debug, Clone)]
pub struct SsrConfig {
    pub max_steps: u32,
    pub max_distance: f32,
    pub thickness: f32,
    pub stride: f32,
    pub jitter: bool,
    pub half_resolution: bool,
}

impl Default for SsrConfig {
    fn default() -> Self {
        Self {
            max_steps: 64,
            max_distance: 50.0,
            thickness: 0.1,
            stride: 1.0,
            jitter: true,
            half_resolution: true,
        }
    }
}

/// Screen-space reflections pass.
pub struct SsrPass {
    pub config: SsrConfig,
    pub reflection_target: RenderTarget,
    pub width: u32,
    pub height: u32,
}

impl SsrPass {
    /// Create a new SSR pass.
    pub fn new(device: &wgpu::Device, width: u32, height: u32, config: SsrConfig) -> Self {
        let (w, h) = if config.half_resolution {
            (width / 2, height / 2)
        } else {
            (width, height)
        };

        let reflection_target = RenderTarget::new(
            device,
            "ssr_reflections",
            w,
            h,
            hdr_color_format(),
        );

        Self {
            config,
            reflection_target,
            width: w,
            height: h,
        }
    }

    /// Resize the SSR target.
    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        let (w, h) = if self.config.half_resolution {
            (width / 2, height / 2)
        } else {
            (width, height)
        };
        self.width = w;
        self.height = h;
        self.reflection_target.resize(device, w, h);
    }

    /// Get the reflection output texture view.
    pub fn output_view(&self) -> &wgpu::TextureView {
        self.reflection_target.view()
    }
}
