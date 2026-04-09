use forge3d_gpu::framebuffer::RenderTarget;
use forge3d_gpu::texture::format::hdr_color_format;

/// Deferred lighting pass that reads the G-Buffer and outputs lit HDR color.
pub struct LightingPass {
    pub output: RenderTarget,
    pub width: u32,
    pub height: u32,
}

impl LightingPass {
    /// Create a new lighting pass.
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let output = RenderTarget::new(
            device,
            "lighting_output",
            width,
            height,
            hdr_color_format(),
        );

        Self {
            output,
            width,
            height,
        }
    }

    /// Resize the output target.
    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.output.resize(device, width, height);
    }

    /// Get the output texture view.
    pub fn output_view(&self) -> &wgpu::TextureView {
        self.output.view()
    }

    /// Get the color attachment for the lighting pass.
    pub fn color_attachment(&self) -> wgpu::RenderPassColorAttachment<'_> {
        self.output.color_attachment(Some(wgpu::Color::BLACK))
    }
}
