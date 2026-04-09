use crate::texture::GpuTexture;

/// A depth render target.
pub struct DepthTarget {
    pub texture: GpuTexture,
}

impl DepthTarget {
    /// Create a new depth target with Depth32Float format.
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let format = wgpu::TextureFormat::Depth32Float;
        let texture = GpuTexture::new(
            device,
            "depth_target",
            width,
            height,
            format,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        );

        Self { texture }
    }

    /// Create a depth target with a specific format.
    pub fn with_format(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
    ) -> Self {
        let texture = GpuTexture::new(
            device,
            "depth_target",
            width,
            height,
            format,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        );

        Self { texture }
    }

    /// Get the texture view for use as a depth attachment.
    pub fn view(&self) -> &wgpu::TextureView {
        &self.texture.view
    }

    /// Get a depth stencil attachment descriptor.
    pub fn depth_attachment(&self, clear: bool) -> wgpu::RenderPassDepthStencilAttachment<'_> {
        wgpu::RenderPassDepthStencilAttachment {
            view: &self.texture.view,
            depth_ops: Some(wgpu::Operations {
                load: if clear {
                    wgpu::LoadOp::Clear(1.0)
                } else {
                    wgpu::LoadOp::Load
                },
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        }
    }

    /// Resize the depth target.
    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.texture = GpuTexture::new(
                device,
                "depth_target",
                width,
                height,
                self.texture.format,
                wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            );
        }
    }
}
