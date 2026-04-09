use forge3d_gpu::framebuffer::RenderTarget;
use forge3d_gpu::texture::format::hdr_color_format;

/// Depth of field configuration.
#[derive(Debug, Clone)]
pub struct DofConfig {
    pub focus_distance: f32,
    pub aperture: f32,
    pub focal_length: f32,
    pub bokeh_shape: BokehShape,
    pub max_blur_radius: f32,
}

/// Shape of the bokeh highlight.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BokehShape {
    Circle,
    Hexagon,
}

impl Default for DofConfig {
    fn default() -> Self {
        Self {
            focus_distance: 5.0,
            aperture: 2.8,
            focal_length: 50.0,
            bokeh_shape: BokehShape::Circle,
            max_blur_radius: 10.0,
        }
    }
}

/// Depth of field post-processing pass.
pub struct DepthOfFieldPass {
    pub config: DofConfig,
    pub coc_target: RenderTarget,
    pub near_field: RenderTarget,
    pub far_field: RenderTarget,
    pub output: RenderTarget,
    pub width: u32,
    pub height: u32,
}

impl DepthOfFieldPass {
    /// Create a new depth of field pass.
    pub fn new(device: &wgpu::Device, width: u32, height: u32, config: DofConfig) -> Self {
        let coc_target = RenderTarget::new(
            device,
            "dof_coc",
            width,
            height,
            wgpu::TextureFormat::R16Float,
        );

        let near_field = RenderTarget::new(
            device,
            "dof_near",
            (width / 2).max(1),
            (height / 2).max(1),
            hdr_color_format(),
        );

        let far_field = RenderTarget::new(
            device,
            "dof_far",
            (width / 2).max(1),
            (height / 2).max(1),
            hdr_color_format(),
        );

        let output = RenderTarget::new(
            device,
            "dof_output",
            width,
            height,
            hdr_color_format(),
        );

        Self {
            config,
            coc_target,
            near_field,
            far_field,
            output,
            width,
            height,
        }
    }

    /// Resize all DOF targets.
    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.coc_target.resize(device, width, height);
        self.near_field.resize(device, (width / 2).max(1), (height / 2).max(1));
        self.far_field.resize(device, (width / 2).max(1), (height / 2).max(1));
        self.output.resize(device, width, height);
    }

    /// Compute the circle of confusion diameter for a given depth.
    ///
    /// CoC = |A * f * (d - S)| / (d * (S - f))
    /// where A = f/N (aperture diameter), f = focal length, S = focus distance, d = depth.
    pub fn compute_coc(&self, depth: f32) -> f32 {
        let f = self.config.focal_length / 1000.0; // mm to m
        let a = f / self.config.aperture; // aperture diameter
        let s = self.config.focus_distance;
        let denom = depth * (s - f);
        if denom.abs() < 1e-8 || depth <= 0.0 {
            return 0.0;
        }
        (a * f * (depth - s).abs() / denom).abs()
    }

    /// Get the output texture view.
    pub fn output_view(&self) -> &wgpu::TextureView {
        self.output.view()
    }
}
