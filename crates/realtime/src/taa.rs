use forge3d_gpu::framebuffer::RenderTarget;
use forge3d_gpu::texture::format::hdr_color_format;

/// Temporal anti-aliasing configuration.
#[derive(Debug, Clone)]
pub struct TaaConfig {
    pub feedback_factor: f32,
    pub jitter_scale: f32,
    pub clamp_mode: ClampMode,
    pub motion_rejection: bool,
}

/// Neighborhood clamping mode for TAA.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClampMode {
    Aabb,
    Variance,
}

impl Default for TaaConfig {
    fn default() -> Self {
        Self {
            feedback_factor: 0.9,
            jitter_scale: 1.0,
            clamp_mode: ClampMode::Variance,
            motion_rejection: true,
        }
    }
}

/// Temporal anti-aliasing pass.
pub struct TaaPass {
    pub config: TaaConfig,
    pub history: [RenderTarget; 2],
    pub current_frame: usize,
    pub jitter_index: u32,
    pub width: u32,
    pub height: u32,
}

impl TaaPass {
    /// Create a new TAA pass.
    pub fn new(device: &wgpu::Device, width: u32, height: u32, config: TaaConfig) -> Self {
        let format = hdr_color_format();
        let history = [
            RenderTarget::new(device, "taa_history_0", width, height, format),
            RenderTarget::new(device, "taa_history_1", width, height, format),
        ];

        Self {
            config,
            history,
            current_frame: 0,
            jitter_index: 0,
            width,
            height,
        }
    }

    /// Resize the TAA targets.
    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        for h in &mut self.history {
            h.resize(device, width, height);
        }
        self.current_frame = 0;
    }

    /// Get the current frame's output target view.
    pub fn current_view(&self) -> &wgpu::TextureView {
        self.history[self.current_frame % 2].view()
    }

    /// Get the previous frame's history target view.
    pub fn history_view(&self) -> &wgpu::TextureView {
        self.history[(self.current_frame + 1) % 2].view()
    }

    /// Advance to the next frame.
    pub fn advance_frame(&mut self) {
        self.current_frame += 1;
        self.jitter_index = (self.jitter_index + 1) % 16;
    }

    /// Get the current jitter offset in pixels.
    pub fn jitter_offset(&self) -> (f32, f32) {
        // Halton(2,3) sequence for sub-pixel jitter
        let x = halton(self.jitter_index + 1, 2) - 0.5;
        let y = halton(self.jitter_index + 1, 3) - 0.5;
        (x * self.config.jitter_scale, y * self.config.jitter_scale)
    }
}

fn halton(index: u32, base: u32) -> f32 {
    let mut f = 1.0f32;
    let mut r = 0.0f32;
    let mut i = index;
    let b = base as f32;
    while i > 0 {
        f /= b;
        r += f * (i % base) as f32;
        i /= base;
    }
    r
}
