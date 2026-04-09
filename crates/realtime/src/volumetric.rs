use forge3d_gpu::framebuffer::RenderTarget;
use forge3d_gpu::texture::format::hdr_color_format;

/// Volumetric lighting / fog configuration.
#[derive(Debug, Clone)]
pub struct VolumetricConfig {
    pub steps: u32,
    pub max_distance: f32,
    pub scattering: f32,
    pub absorption: f32,
    pub density: f32,
    pub anisotropy: f32,
    pub half_resolution: bool,
}

impl Default for VolumetricConfig {
    fn default() -> Self {
        Self {
            steps: 64,
            max_distance: 50.0,
            scattering: 0.1,
            absorption: 0.01,
            density: 0.05,
            anisotropy: 0.3,
            half_resolution: true,
        }
    }
}

/// Volumetric lighting and fog pass.
pub struct VolumetricPass {
    pub config: VolumetricConfig,
    pub scatter_target: RenderTarget,
    pub transmittance_target: RenderTarget,
    pub width: u32,
    pub height: u32,
}

impl VolumetricPass {
    /// Create a new volumetric pass.
    pub fn new(device: &wgpu::Device, width: u32, height: u32, config: VolumetricConfig) -> Self {
        let (w, h) = if config.half_resolution {
            (width / 2, height / 2)
        } else {
            (width, height)
        };

        let scatter_target = RenderTarget::new(
            device,
            "volumetric_scatter",
            w,
            h,
            hdr_color_format(),
        );

        let transmittance_target = RenderTarget::new(
            device,
            "volumetric_transmittance",
            w,
            h,
            wgpu::TextureFormat::R16Float,
        );

        Self {
            config,
            scatter_target,
            transmittance_target,
            width: w,
            height: h,
        }
    }

    /// Resize the volumetric targets.
    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        let (w, h) = if self.config.half_resolution {
            (width / 2, height / 2)
        } else {
            (width, height)
        };
        self.width = w;
        self.height = h;
        self.scatter_target.resize(device, w, h);
        self.transmittance_target.resize(device, w, h);
    }

    /// Get the in-scattering texture view.
    pub fn scatter_view(&self) -> &wgpu::TextureView {
        self.scatter_target.view()
    }

    /// Get the transmittance texture view.
    pub fn transmittance_view(&self) -> &wgpu::TextureView {
        self.transmittance_target.view()
    }

    /// Compute the Henyey-Greenstein phase function.
    pub fn phase_hg(cos_theta: f32, g: f32) -> f32 {
        let g2 = g * g;
        let denom = 1.0 + g2 - 2.0 * g * cos_theta;
        (1.0 - g2) / (4.0 * std::f32::consts::PI * denom * denom.sqrt())
    }
}
