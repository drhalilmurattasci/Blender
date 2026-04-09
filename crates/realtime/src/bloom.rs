use forge3d_gpu::framebuffer::RenderTarget;
use forge3d_gpu::texture::format::hdr_color_format;

/// Bloom post-processing configuration.
#[derive(Debug, Clone)]
pub struct BloomConfig {
    pub threshold: f32,
    pub intensity: f32,
    pub mip_count: u32,
    pub radius: f32,
}

impl Default for BloomConfig {
    fn default() -> Self {
        Self {
            threshold: 1.0,
            intensity: 0.5,
            mip_count: 5,
            radius: 0.005,
        }
    }
}

/// Bloom pass using progressive downsampling and upsampling.
pub struct BloomPass {
    pub config: BloomConfig,
    pub downsample_targets: Vec<RenderTarget>,
    pub upsample_targets: Vec<RenderTarget>,
}

impl BloomPass {
    /// Create a new bloom pass.
    pub fn new(device: &wgpu::Device, width: u32, height: u32, config: BloomConfig) -> Self {
        let format = hdr_color_format();
        let mut downsample_targets = Vec::new();
        let mut upsample_targets = Vec::new();

        let mut w = width / 2;
        let mut h = height / 2;

        for i in 0..config.mip_count {
            downsample_targets.push(RenderTarget::new(
                device,
                &format!("bloom_down_{}", i),
                w.max(1),
                h.max(1),
                format,
            ));
            upsample_targets.push(RenderTarget::new(
                device,
                &format!("bloom_up_{}", i),
                w.max(1),
                h.max(1),
                format,
            ));
            w /= 2;
            h /= 2;
        }

        Self {
            config,
            downsample_targets,
            upsample_targets,
        }
    }

    /// Resize all bloom targets.
    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        let mut w = width / 2;
        let mut h = height / 2;

        for i in 0..self.config.mip_count as usize {
            if i < self.downsample_targets.len() {
                self.downsample_targets[i].resize(device, w.max(1), h.max(1));
                self.upsample_targets[i].resize(device, w.max(1), h.max(1));
            }
            w /= 2;
            h /= 2;
        }
    }

    /// Get the final bloom texture view (first upsample level).
    pub fn output_view(&self) -> Option<&wgpu::TextureView> {
        self.upsample_targets.first().map(|t| t.view())
    }

    /// Number of mip levels.
    pub fn mip_count(&self) -> u32 {
        self.config.mip_count
    }
}
