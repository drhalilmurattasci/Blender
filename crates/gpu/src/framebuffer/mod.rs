mod depth;
mod render_target;

pub use depth::DepthTarget;
pub use render_target::RenderTarget;

/// A complete framebuffer with color and optional depth attachments.
pub struct FrameBuffer {
    pub color_targets: Vec<RenderTarget>,
    pub depth_target: Option<DepthTarget>,
    pub width: u32,
    pub height: u32,
}

impl FrameBuffer {
    /// Create a new framebuffer with one color target and optional depth.
    pub fn new(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        color_format: wgpu::TextureFormat,
        with_depth: bool,
    ) -> Self {
        let color = RenderTarget::new(device, "fb_color_0", width, height, color_format);
        let depth = if with_depth {
            Some(DepthTarget::new(device, width, height))
        } else {
            None
        };

        Self {
            color_targets: vec![color],
            depth_target: depth,
            width,
            height,
        }
    }

    /// Create a framebuffer with multiple color targets (MRT).
    pub fn new_mrt(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        color_formats: &[wgpu::TextureFormat],
        with_depth: bool,
    ) -> Self {
        let color_targets = color_formats
            .iter()
            .enumerate()
            .map(|(i, &fmt)| {
                RenderTarget::new(device, &format!("fb_color_{}", i), width, height, fmt)
            })
            .collect();

        let depth = if with_depth {
            Some(DepthTarget::new(device, width, height))
        } else {
            None
        };

        Self {
            color_targets,
            depth_target: depth,
            width,
            height,
        }
    }

    /// Resize all targets in the framebuffer.
    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.width = width;
            self.height = height;
            for target in &mut self.color_targets {
                target.resize(device, width, height);
            }
            if let Some(ref mut depth) = self.depth_target {
                depth.resize(device, width, height);
            }
        }
    }
}
