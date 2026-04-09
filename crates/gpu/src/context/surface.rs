use crate::{GpuError, GpuResult};

/// Wraps a `wgpu::Surface` and its configuration.
pub struct GpuSurface<'window> {
    pub surface: wgpu::Surface<'window>,
    pub config: wgpu::SurfaceConfiguration,
}

impl<'window> GpuSurface<'window> {
    /// Create a new `GpuSurface` from an existing wgpu surface.
    pub fn new(
        surface: wgpu::Surface<'window>,
        adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        width: u32,
        height: u32,
    ) -> GpuResult<Self> {
        let caps = surface.get_capabilities(adapter);
        if caps.formats.is_empty() {
            return Err(GpuError::SurfaceConfig);
        }
        let format = caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(caps.formats[0]);

        let present_mode = if caps.present_modes.contains(&wgpu::PresentMode::Mailbox) {
            wgpu::PresentMode::Mailbox
        } else {
            wgpu::PresentMode::Fifo
        };

        let alpha_mode = caps
            .alpha_modes
            .first()
            .copied()
            .unwrap_or(wgpu::CompositeAlphaMode::Auto);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width,
            height,
            present_mode,
            alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(device, &config);

        Ok(Self { surface, config })
    }

    /// Resize the surface.
    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(device, &self.config);
        }
    }

    /// Get the current texture for rendering.
    pub fn get_current_texture(&self) -> GpuResult<wgpu::SurfaceTexture> {
        self.surface
            .get_current_texture()
            .map_err(GpuError::Surface)
    }

    /// Get the surface texture format.
    pub fn format(&self) -> wgpu::TextureFormat {
        self.config.format
    }

    /// Get the surface dimensions.
    pub fn size(&self) -> (u32, u32) {
        (self.config.width, self.config.height)
    }
}
