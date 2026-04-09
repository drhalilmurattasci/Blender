use crate::texture::GpuTexture;

/// A color render target backed by a texture.
pub struct RenderTarget {
    pub texture: GpuTexture,
}

impl RenderTarget {
    /// Create a new render target with the given format.
    pub fn new(
        device: &wgpu::Device,
        label: &str,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
    ) -> Self {
        let texture = GpuTexture::new(
            device,
            label,
            width,
            height,
            format,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        );

        Self { texture }
    }

    /// Get the texture view for use as a render attachment.
    pub fn view(&self) -> &wgpu::TextureView {
        &self.texture.view
    }

    /// Get a color attachment descriptor for a render pass.
    pub fn color_attachment(&self, clear: Option<wgpu::Color>) -> wgpu::RenderPassColorAttachment<'_> {
        wgpu::RenderPassColorAttachment {
            view: &self.texture.view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: match clear {
                    Some(color) => wgpu::LoadOp::Clear(color),
                    None => wgpu::LoadOp::Load,
                },
                store: wgpu::StoreOp::Store,
            },
        }
    }

    /// Resize the render target.
    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.texture = GpuTexture::new(
                device,
                "render_target",
                width,
                height,
                self.texture.format,
                wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            );
        }
    }
}
