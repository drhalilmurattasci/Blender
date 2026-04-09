use forge3d_gpu::framebuffer::DepthTarget;

/// Shadow map configuration.
#[derive(Debug, Clone)]
pub struct ShadowConfig {
    pub resolution: u32,
    pub cascade_count: u32,
    pub max_distance: f32,
    pub bias: f32,
    pub normal_bias: f32,
}

impl Default for ShadowConfig {
    fn default() -> Self {
        Self {
            resolution: 2048,
            cascade_count: 4,
            max_distance: 100.0,
            bias: 0.005,
            normal_bias: 0.02,
        }
    }
}

/// Cascaded shadow map pass.
pub struct ShadowPass {
    pub config: ShadowConfig,
    pub cascades: Vec<DepthTarget>,
    pub cascade_splits: Vec<f32>,
}

impl ShadowPass {
    /// Create a new shadow pass with cascaded shadow maps.
    pub fn new(device: &wgpu::Device, config: ShadowConfig) -> Self {
        let mut cascades = Vec::new();
        for _i in 0..config.cascade_count {
            cascades.push(DepthTarget::new(device, config.resolution, config.resolution));
        }

        let cascade_splits = Self::compute_cascade_splits(
            config.cascade_count,
            0.1,
            config.max_distance,
            0.5,
        );

        Self {
            config,
            cascades,
            cascade_splits,
        }
    }

    /// Compute logarithmic-linear cascade split distances.
    fn compute_cascade_splits(count: u32, near: f32, far: f32, lambda: f32) -> Vec<f32> {
        let mut splits = Vec::with_capacity(count as usize + 1);
        splits.push(near);

        for i in 1..=count {
            let p = i as f32 / count as f32;
            let log_split = near * (far / near).powf(p);
            let uniform_split = near + (far - near) * p;
            let split = lambda * log_split + (1.0 - lambda) * uniform_split;
            splits.push(split);
        }

        splits
    }

    /// Get the depth attachment for a specific cascade.
    pub fn cascade_depth_attachment(&self, cascade_index: usize) -> wgpu::RenderPassDepthStencilAttachment<'_> {
        self.cascades[cascade_index].depth_attachment(true)
    }

    /// Get the shadow map texture view for a specific cascade.
    pub fn cascade_view(&self, cascade_index: usize) -> &wgpu::TextureView {
        self.cascades[cascade_index].view()
    }

    /// Number of cascades.
    pub fn cascade_count(&self) -> usize {
        self.cascades.len()
    }
}
