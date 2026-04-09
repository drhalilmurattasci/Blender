use forge3d_gpu::framebuffer::FrameBuffer;

/// G-Buffer layout for deferred rendering.
///
/// Targets:
/// 0: Albedo (RGBA8)
/// 1: Normal (RGBA16F) - world-space normals
/// 2: Material (RGBA8) - roughness, metallic, ao, flags
/// 3: Emission (RGBA16F)
/// + Depth
pub struct GBufferPass {
    pub framebuffer: FrameBuffer,
    pub width: u32,
    pub height: u32,
}

impl GBufferPass {
    /// Create a new G-Buffer pass.
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let framebuffer = FrameBuffer::new_mrt(
            device,
            width,
            height,
            &[
                wgpu::TextureFormat::Rgba8UnormSrgb,  // albedo
                wgpu::TextureFormat::Rgba16Float,      // normals
                wgpu::TextureFormat::Rgba8Unorm,       // material params
                wgpu::TextureFormat::Rgba16Float,      // emission
            ],
            true, // depth
        );

        Self {
            framebuffer,
            width,
            height,
        }
    }

    /// Resize the G-Buffer.
    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.framebuffer.resize(device, width, height);
    }

    /// Get color attachments for a render pass.
    pub fn color_attachments(&self) -> Vec<Option<wgpu::RenderPassColorAttachment<'_>>> {
        self.framebuffer
            .color_targets
            .iter()
            .map(|t| {
                Some(t.color_attachment(Some(wgpu::Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 0.0,
                })))
            })
            .collect()
    }

    /// Get the depth attachment.
    pub fn depth_attachment(&self) -> Option<wgpu::RenderPassDepthStencilAttachment<'_>> {
        self.framebuffer
            .depth_target
            .as_ref()
            .map(|d| d.depth_attachment(true))
    }

    /// Get the albedo texture view.
    pub fn albedo_view(&self) -> &wgpu::TextureView {
        self.framebuffer.color_targets[0].view()
    }

    /// Get the normal texture view.
    pub fn normal_view(&self) -> &wgpu::TextureView {
        self.framebuffer.color_targets[1].view()
    }

    /// Get the material texture view.
    pub fn material_view(&self) -> &wgpu::TextureView {
        self.framebuffer.color_targets[2].view()
    }

    /// Get the emission texture view.
    pub fn emission_view(&self) -> &wgpu::TextureView {
        self.framebuffer.color_targets[3].view()
    }

    /// Get the depth texture view.
    pub fn depth_view(&self) -> Option<&wgpu::TextureView> {
        self.framebuffer.depth_target.as_ref().map(|d| &d.texture.view)
    }
}
