//! Forge3D — Blender-like interface with egui panels and a wgpu 3D viewport.
//!
//! Opens a window with:
//! - Top header bar with Forge3D title and menus
//! - 3D viewport (center) rendering a spinning colored cube
//! - Outliner panel (top-right) showing scene hierarchy
//! - Properties panel (right) showing selected object transforms
//! - Timeline strip (bottom) with play/pause and frame counter
//!
//! Press Escape to quit, Space to play/pause timeline.

use std::sync::Arc;
use std::time::Instant;

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};

// ---------------------------------------------------------------------------
// Vertex
// ---------------------------------------------------------------------------

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
        0 => Float32x3,
        1 => Float32x3,
    ];

    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

// ---------------------------------------------------------------------------
// Cube geometry
// ---------------------------------------------------------------------------

#[rustfmt::skip]
const VERTICES: &[Vertex] = &[
    // Front face (red-ish)
    Vertex { position: [-0.5, -0.5,  0.5], color: [1.0, 0.2, 0.2] },
    Vertex { position: [ 0.5, -0.5,  0.5], color: [1.0, 0.4, 0.2] },
    Vertex { position: [ 0.5,  0.5,  0.5], color: [1.0, 0.6, 0.4] },
    Vertex { position: [-0.5,  0.5,  0.5], color: [1.0, 0.3, 0.3] },
    // Back face (blue-ish)
    Vertex { position: [-0.5, -0.5, -0.5], color: [0.2, 0.2, 1.0] },
    Vertex { position: [ 0.5, -0.5, -0.5], color: [0.2, 0.4, 1.0] },
    Vertex { position: [ 0.5,  0.5, -0.5], color: [0.4, 0.6, 1.0] },
    Vertex { position: [-0.5,  0.5, -0.5], color: [0.3, 0.3, 1.0] },
    // Top face (green-ish)
    Vertex { position: [-0.5,  0.5, -0.5], color: [0.2, 1.0, 0.2] },
    Vertex { position: [ 0.5,  0.5, -0.5], color: [0.2, 1.0, 0.4] },
    Vertex { position: [ 0.5,  0.5,  0.5], color: [0.4, 1.0, 0.4] },
    Vertex { position: [-0.5,  0.5,  0.5], color: [0.3, 1.0, 0.3] },
    // Bottom face (yellow-ish)
    Vertex { position: [-0.5, -0.5, -0.5], color: [1.0, 1.0, 0.2] },
    Vertex { position: [ 0.5, -0.5, -0.5], color: [1.0, 1.0, 0.4] },
    Vertex { position: [ 0.5, -0.5,  0.5], color: [1.0, 1.0, 0.4] },
    Vertex { position: [-0.5, -0.5,  0.5], color: [1.0, 1.0, 0.2] },
    // Right face (cyan-ish)
    Vertex { position: [ 0.5, -0.5, -0.5], color: [0.2, 1.0, 1.0] },
    Vertex { position: [ 0.5,  0.5, -0.5], color: [0.4, 1.0, 1.0] },
    Vertex { position: [ 0.5,  0.5,  0.5], color: [0.4, 1.0, 1.0] },
    Vertex { position: [ 0.5, -0.5,  0.5], color: [0.2, 1.0, 1.0] },
    // Left face (magenta-ish)
    Vertex { position: [-0.5, -0.5, -0.5], color: [1.0, 0.2, 1.0] },
    Vertex { position: [-0.5,  0.5, -0.5], color: [1.0, 0.4, 1.0] },
    Vertex { position: [-0.5,  0.5,  0.5], color: [1.0, 0.4, 1.0] },
    Vertex { position: [-0.5, -0.5,  0.5], color: [1.0, 0.2, 1.0] },
];

#[rustfmt::skip]
const INDICES: &[u16] = &[
     0,  1,  2,  2,  3,  0, // front
     4,  6,  5,  6,  4,  7, // back
     8,  9, 10, 10, 11,  8, // top
    12, 14, 13, 14, 12, 15, // bottom
    16, 17, 18, 18, 19, 16, // right
    20, 22, 21, 22, 20, 23, // left
];

// ---------------------------------------------------------------------------
// WGSL Shader
// ---------------------------------------------------------------------------

const SHADER_SRC: &str = r#"
struct Uniforms {
    mvp: mat4x4<f32>,
};
@group(0) @binding(0) var<uniform> u: Uniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = u.mvp * vec4<f32>(in.position, 1.0);
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
"#;

// ---------------------------------------------------------------------------
// Simple math helpers (column-major 4x4 matrices)
// ---------------------------------------------------------------------------

fn mat4_identity() -> [f32; 16] {
    [
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0,
    ]
}

fn mat4_rotate_y(angle: f32) -> [f32; 16] {
    let c = angle.cos();
    let s = angle.sin();
    [
         c,  0.0,  -s, 0.0,
        0.0, 1.0, 0.0, 0.0,
         s,  0.0,   c, 0.0,
        0.0, 0.0, 0.0, 1.0,
    ]
}

fn mat4_rotate_x(angle: f32) -> [f32; 16] {
    let c = angle.cos();
    let s = angle.sin();
    [
        1.0, 0.0, 0.0, 0.0,
        0.0,  c,   s,  0.0,
        0.0, -s,   c,  0.0,
        0.0, 0.0, 0.0, 1.0,
    ]
}

fn mat4_rotate_z(angle: f32) -> [f32; 16] {
    let c = angle.cos();
    let s = angle.sin();
    [
         c,   s,  0.0, 0.0,
        -s,   c,  0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0,
    ]
}

fn mat4_translate(x: f32, y: f32, z: f32) -> [f32; 16] {
    [
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
          x,   y,   z, 1.0,
    ]
}

fn mat4_scale(sx: f32, sy: f32, sz: f32) -> [f32; 16] {
    [
         sx, 0.0, 0.0, 0.0,
        0.0,  sy, 0.0, 0.0,
        0.0, 0.0,  sz, 0.0,
        0.0, 0.0, 0.0, 1.0,
    ]
}

fn mat4_perspective(fov_y: f32, aspect: f32, near: f32, far: f32) -> [f32; 16] {
    let f = 1.0 / (fov_y / 2.0).tan();
    let nf = 1.0 / (near - far);
    [
        f / aspect, 0.0,  0.0,                    0.0,
        0.0,        f,    0.0,                    0.0,
        0.0,        0.0,  (far + near) * nf,     -1.0,
        0.0,        0.0,  2.0 * far * near * nf,  0.0,
    ]
}

fn mat4_mul(a: &[f32; 16], b: &[f32; 16]) -> [f32; 16] {
    let mut out = [0.0f32; 16];
    for col in 0..4 {
        for row in 0..4 {
            let mut sum = 0.0;
            for k in 0..4 {
                sum += a[k * 4 + row] * b[col * 4 + k];
            }
            out[col * 4 + row] = sum;
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Scene objects
// ---------------------------------------------------------------------------

struct SceneObject {
    name: String,
    obj_type: &'static str,
    location: [f32; 3],
    rotation: [f32; 3],
    scale: [f32; 3],
    #[allow(dead_code)]
    selected: bool,
}

impl SceneObject {
    fn new(name: &str, obj_type: &'static str, loc: [f32; 3], rot: [f32; 3], scl: [f32; 3]) -> Self {
        Self {
            name: name.to_string(),
            obj_type,
            location: loc,
            rotation: rot,
            scale: scl,
            selected: false,
        }
    }
}

// ---------------------------------------------------------------------------
// Cube render resources — stored in egui_wgpu CallbackResources
// ---------------------------------------------------------------------------

struct CubeResources {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    num_indices: u32,
    #[allow(dead_code)]
    depth_texture: Option<(wgpu::Texture, wgpu::TextureView, u32, u32)>,
}

impl CubeResources {
    fn create(device: &wgpu::Device, target_format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("cube-shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER_SRC.into()),
        });

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("cube-uniform"),
            contents: bytemuck::cast_slice(&mat4_identity()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("cube-bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("cube-bg"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("cube-pipeline-layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("cube-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[Vertex::layout()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            // No depth stencil — we render into the egui render pass which has no depth attachment
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("cube-vbo"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("cube-ibo"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            pipeline,
            vertex_buffer,
            index_buffer,
            uniform_buffer,
            bind_group,
            num_indices: INDICES.len() as u32,
            depth_texture: None,
        }
    }
}

// ---------------------------------------------------------------------------
// egui_wgpu paint callback for the 3D viewport
// ---------------------------------------------------------------------------

struct CubeCallback {
    mvp: [f32; 16],
}

impl egui_wgpu::CallbackTrait for CubeCallback {
    fn prepare(
        &self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &egui_wgpu::ScreenDescriptor,
        _encoder: &mut wgpu::CommandEncoder,
        callback_resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let resources: &CubeResources = callback_resources.get().unwrap();
        queue.write_buffer(&resources.uniform_buffer, 0, bytemuck::cast_slice(&self.mvp));
        Vec::new()
    }

    fn paint(
        &self,
        info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
        callback_resources: &egui_wgpu::CallbackResources,
    ) {
        let resources: &CubeResources = callback_resources.get().unwrap();

        // Set viewport to the callback rect (the central panel area)
        let rect = info.viewport_in_pixels();
        if rect.width_px > 0 && rect.height_px > 0 {
            render_pass.set_viewport(
                rect.left_px as f32,
                rect.top_px as f32,
                rect.width_px as f32,
                rect.height_px as f32,
                0.0,
                1.0,
            );
            // Set scissor rect too
            render_pass.set_scissor_rect(
                rect.left_px as u32,
                rect.top_px as u32,
                rect.width_px as u32,
                rect.height_px as u32,
            );
        }

        render_pass.set_pipeline(&resources.pipeline);
        render_pass.set_bind_group(0, &resources.bind_group, &[]);
        render_pass.set_vertex_buffer(0, resources.vertex_buffer.slice(..));
        render_pass.set_index_buffer(resources.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..resources.num_indices, 0, 0..1);
    }
}

// ---------------------------------------------------------------------------
// Application state
// ---------------------------------------------------------------------------

struct App {
    // Window & GPU
    window: Option<Arc<Window>>,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    surface: Option<wgpu::Surface<'static>>,
    surface_config: Option<wgpu::SurfaceConfiguration>,
    surface_format: wgpu::TextureFormat,

    // egui integration
    egui_ctx: egui::Context,
    egui_winit: Option<egui_winit::State>,
    egui_renderer: Option<egui_wgpu::Renderer>,

    // Scene
    objects: Vec<SceneObject>,
    selected_index: Option<usize>,

    // Timeline
    current_frame: i32,
    start_frame: i32,
    end_frame: i32,
    playing: bool,
    last_frame_time: Instant,

    // Timing
    start_time: Instant,
}

impl App {
    fn new() -> Self {
        let objects = vec![
            SceneObject::new("Cube", "Mesh", [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]),
            SceneObject::new("Camera", "Camera", [7.36, -6.93, 4.96], [63.6, 0.0, 46.7], [1.0, 1.0, 1.0]),
            SceneObject::new("Light", "Light", [4.08, 1.0, 5.90], [37.3, 3.16, 107.0], [1.0, 1.0, 1.0]),
        ];

        Self {
            window: None,
            device: None,
            queue: None,
            surface: None,
            surface_config: None,
            surface_format: wgpu::TextureFormat::Bgra8UnormSrgb,

            egui_ctx: egui::Context::default(),
            egui_winit: None,
            egui_renderer: None,

            objects,
            selected_index: Some(0), // Cube selected by default

            current_frame: 1,
            start_frame: 1,
            end_frame: 250,
            playing: false,
            last_frame_time: Instant::now(),

            start_time: Instant::now(),
        }
    }

    fn init_wgpu(&mut self, window: Arc<Window>) {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .expect("No suitable GPU adapter found");

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("forge3d-device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::Performance,
            },
            None,
        ))
        .expect("Failed to create device");

        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            desired_maximum_frame_latency: 2,
            alpha_mode: caps
                .alpha_modes
                .first()
                .copied()
                .unwrap_or(wgpu::CompositeAlphaMode::Auto),
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        self.surface_format = format;

        // Init egui renderer — no depth format needed (we render cube without depth in the egui pass)
        let egui_renderer = egui_wgpu::Renderer::new(&device, format, None, 1, true);

        // Init egui-winit state
        let egui_winit = egui_winit::State::new(
            self.egui_ctx.clone(),
            egui::ViewportId::ROOT,
            &window,
            Some(window.scale_factor() as f32),
            None,
            Some(device.limits().max_texture_dimension_2d as usize),
        );

        // Create cube resources and store in callback resources
        let cube_resources = CubeResources::create(&device, format);

        self.window = Some(window);
        self.device = Some(device);
        self.queue = Some(queue);
        self.surface = Some(surface);
        self.surface_config = Some(config);
        self.egui_winit = Some(egui_winit);
        self.egui_renderer = Some(egui_renderer);

        // Store cube resources in the egui renderer's callback resources
        self.egui_renderer
            .as_mut()
            .unwrap()
            .callback_resources
            .insert(cube_resources);
    }

    fn resize(&mut self, width: u32, height: u32) {
        if let (Some(config), Some(surface), Some(device)) =
            (&mut self.surface_config, &self.surface, &self.device)
        {
            config.width = width.max(1);
            config.height = height.max(1);
            surface.configure(device, config);
        }
    }

    fn configure_dark_theme(ctx: &egui::Context) {
        let mut visuals = egui::Visuals::dark();

        // Blender-like dark gray colors
        let _bg = egui::Color32::from_rgb(0x30, 0x30, 0x30);
        let panel_bg = egui::Color32::from_rgb(0x2D, 0x2D, 0x2D);
        let darker_bg = egui::Color32::from_rgb(0x25, 0x25, 0x25);
        let text_color = egui::Color32::from_rgb(0xE0, 0xE0, 0xE0);
        let dim_text = egui::Color32::from_rgb(0x99, 0x99, 0x99);
        let accent = egui::Color32::from_rgb(0x4C, 0x8B, 0xBF); // Blender selection blue
        let widget_bg = egui::Color32::from_rgb(0x3A, 0x3A, 0x3A);

        visuals.window_fill = panel_bg;
        visuals.panel_fill = panel_bg;
        visuals.extreme_bg_color = darker_bg;
        visuals.faint_bg_color = egui::Color32::from_rgb(0x35, 0x35, 0x35);

        visuals.override_text_color = Some(text_color);
        visuals.selection.bg_fill = accent;
        visuals.selection.stroke = egui::Stroke::new(1.0, accent);

        visuals.widgets.noninteractive.bg_fill = panel_bg;
        visuals.widgets.noninteractive.weak_bg_fill = panel_bg;
        visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(0x40, 0x40, 0x40));
        visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, dim_text);
        visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(2);

        visuals.widgets.inactive.bg_fill = widget_bg;
        visuals.widgets.inactive.weak_bg_fill = widget_bg;
        visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(0x50, 0x50, 0x50));
        visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, text_color);
        visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(2);

        visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(0x45, 0x45, 0x45);
        visuals.widgets.hovered.weak_bg_fill = egui::Color32::from_rgb(0x45, 0x45, 0x45);
        visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, accent);
        visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);
        visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(2);

        visuals.widgets.active.bg_fill = accent;
        visuals.widgets.active.weak_bg_fill = accent;
        visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, accent);
        visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);
        visuals.widgets.active.corner_radius = egui::CornerRadius::same(2);

        visuals.window_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(0x40, 0x40, 0x40));
        visuals.window_shadow = egui::Shadow::NONE;

        ctx.set_visuals(visuals);
    }

    fn draw_ui(&mut self, ctx: &egui::Context) {
        Self::configure_dark_theme(ctx);

        let accent = egui::Color32::from_rgb(0x4C, 0x8B, 0xBF);
        let header_bg = egui::Color32::from_rgb(0x2A, 0x2A, 0x2A);
        let panel_bg = egui::Color32::from_rgb(0x2D, 0x2D, 0x2D);
        let _separator_color = egui::Color32::from_rgb(0x1A, 0x1A, 0x1A);
        let text_color = egui::Color32::from_rgb(0xE0, 0xE0, 0xE0);
        let dim_text = egui::Color32::from_rgb(0x99, 0x99, 0x99);
        let icon_mesh = egui::Color32::from_rgb(0x4C, 0xAF, 0x50);
        let icon_camera = egui::Color32::from_rgb(0x42, 0xA5, 0xF5);
        let icon_light = egui::Color32::from_rgb(0xFF, 0xCA, 0x28);

        // ---- TOP HEADER BAR ----
        egui::TopBottomPanel::top("header")
            .exact_height(28.0)
            .frame(egui::Frame::new().fill(header_bg).inner_margin(egui::Margin::symmetric(8, 4)))
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.spacing_mut().item_spacing.x = 2.0;

                    // Forge3D logo/title
                    ui.colored_label(accent, egui::RichText::new("Forge3D").strong().size(14.0));
                    ui.add_space(12.0);

                    // Menus (visual only)
                    let _menu_style = egui::RichText::new("").size(12.0).color(dim_text);
                    egui::menu::bar(ui, |ui| {
                        ui.menu_button(egui::RichText::new("File").size(12.0), |ui| {
                            let _ = ui.button("New");
                            let _ = ui.button("Open...");
                            let _ = ui.button("Save");
                            let _ = ui.button("Save As...");
                            ui.separator();
                            let _ = ui.button("Import");
                            let _ = ui.button("Export");
                            ui.separator();
                            if ui.button("Quit").clicked() {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                        });
                        ui.menu_button(egui::RichText::new("Edit").size(12.0), |ui| {
                            let _ = ui.button("Undo");
                            let _ = ui.button("Redo");
                            ui.separator();
                            let _ = ui.button("Preferences...");
                        });
                        ui.menu_button(egui::RichText::new("Window").size(12.0), |ui| {
                            let _ = ui.button("Toggle Fullscreen");
                            let _ = ui.button("New Window");
                        });
                        ui.menu_button(egui::RichText::new("Help").size(12.0), |ui| {
                            let _ = ui.button("About Forge3D");
                            let _ = ui.button("Documentation");
                        });
                    });
                });
            });

        // ---- BOTTOM TIMELINE ----
        egui::TopBottomPanel::bottom("timeline")
            .exact_height(36.0)
            .frame(egui::Frame::new().fill(header_bg).inner_margin(egui::Margin::symmetric(8, 4)))
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;

                    // Transport controls
                    if ui.small_button(egui::RichText::new("\u{23EE}").size(14.0)).clicked() {
                        // Skip to start
                        self.current_frame = self.start_frame;
                    }
                    let play_icon = if self.playing { "\u{23F8}" } else { "\u{25B6}" };
                    if ui.small_button(egui::RichText::new(play_icon).size(14.0)).clicked() {
                        self.playing = !self.playing;
                        self.last_frame_time = Instant::now();
                    }
                    if ui.small_button(egui::RichText::new("\u{23F9}").size(14.0)).clicked() {
                        self.playing = false;
                        self.current_frame = self.start_frame;
                    }
                    if ui.small_button(egui::RichText::new("\u{23ED}").size(14.0)).clicked() {
                        // Skip to end
                        self.current_frame = self.end_frame;
                    }

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);

                    // Frame counter
                    ui.label(egui::RichText::new("Frame:").size(11.0).color(dim_text));
                    let mut frame_val = self.current_frame as f64;
                    let drag = egui::DragValue::new(&mut frame_val)
                        .range(self.start_frame as f64..=self.end_frame as f64)
                        .speed(1.0);
                    if ui.add(drag).changed() {
                        self.current_frame = frame_val as i32;
                    }

                    ui.add_space(16.0);

                    // Frame range
                    ui.label(egui::RichText::new("Start:").size(11.0).color(dim_text));
                    ui.add(egui::DragValue::new(&mut self.start_frame).speed(1.0));
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("End:").size(11.0).color(dim_text));
                    ui.add(egui::DragValue::new(&mut self.end_frame).speed(1.0));

                    ui.add_space(16.0);

                    // Frame slider spanning remaining width
                    let slider = egui::Slider::new(
                        &mut self.current_frame,
                        self.start_frame..=self.end_frame,
                    )
                    .show_value(false)
                    .trailing_fill(true);
                    ui.add(slider);
                });
            });

        // ---- RIGHT PANEL (Outliner + Properties) ----
        egui::SidePanel::right("right_panel")
            .default_width(280.0)
            .min_width(200.0)
            .max_width(500.0)
            .frame(egui::Frame::new().fill(panel_bg).inner_margin(egui::Margin::same(0)))
            .show(ctx, |ui| {
                // ---- OUTLINER (top portion) ----
                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    ui.add_space(6.0);
                    ui.colored_label(dim_text, egui::RichText::new("Outliner").size(11.0));
                });
                ui.separator();

                let available_height = ui.available_height();
                let outliner_height = available_height * 0.4;

                ui.allocate_ui_with_layout(
                    egui::vec2(ui.available_width(), outliner_height),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            ui.add_space(8.0);
                            ui.colored_label(dim_text, egui::RichText::new("\u{25BC}").size(10.0));
                            ui.label(egui::RichText::new("Scene Collection").size(12.0).color(text_color));
                        });

                        for i in 0..self.objects.len() {
                            let is_selected = self.selected_index == Some(i);
                            let obj_type = self.objects[i].obj_type;
                            let obj_name = self.objects[i].name.clone();

                            let icon_color = match obj_type {
                                "Mesh" => icon_mesh,
                                "Camera" => icon_camera,
                                "Light" => icon_light,
                                _ => dim_text,
                            };
                            let icon = match obj_type {
                                "Mesh" => "\u{25A0}",    // filled square
                                "Camera" => "\u{25A3}",  // square with inner square
                                "Light" => "\u{2600}",   // sun
                                _ => "\u{25CB}",
                            };

                            let bg_color = if is_selected {
                                egui::Color32::from_rgb(0x3A, 0x50, 0x68)
                            } else {
                                egui::Color32::TRANSPARENT
                            };

                            let response = ui.horizontal(|ui| {
                                ui.add_space(24.0);

                                // Draw selection background
                                let (rect, response) = ui.allocate_exact_size(
                                    egui::vec2(ui.available_width(), 20.0),
                                    egui::Sense::click(),
                                );
                                if response.clicked() {
                                    // Will be set after this closure
                                    return Some(i);
                                }
                                ui.painter().rect_filled(rect, 2.0, bg_color);
                                ui.painter().text(
                                    rect.left_center() + egui::vec2(4.0, 0.0),
                                    egui::Align2::LEFT_CENTER,
                                    icon,
                                    egui::FontId::proportional(12.0),
                                    icon_color,
                                );
                                ui.painter().text(
                                    rect.left_center() + egui::vec2(20.0, 0.0),
                                    egui::Align2::LEFT_CENTER,
                                    &obj_name,
                                    egui::FontId::proportional(12.0),
                                    if is_selected { egui::Color32::WHITE } else { text_color },
                                );
                                None
                            });

                            if let Some(idx) = response.inner {
                                self.selected_index = Some(idx);
                            }
                        }
                    },
                );

                // Separator between outliner and properties
                ui.add(egui::Separator::default().spacing(0.0));

                // ---- PROPERTIES (bottom portion) ----
                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    ui.add_space(6.0);
                    ui.colored_label(dim_text, egui::RichText::new("Properties").size(11.0));
                });
                ui.separator();

                if let Some(idx) = self.selected_index {
                    let obj = &mut self.objects[idx];
                    ui.add_space(6.0);

                    egui::Grid::new("properties_grid")
                        .num_columns(2)
                        .spacing([8.0, 6.0])
                        .min_col_width(70.0)
                        .show(ui, |ui| {
                            // Name
                            ui.label(egui::RichText::new("  Name").size(11.0).color(dim_text));
                            ui.add(egui::TextEdit::singleline(&mut obj.name).desired_width(160.0));
                            ui.end_row();

                            // Type
                            ui.label(egui::RichText::new("  Type").size(11.0).color(dim_text));
                            ui.label(egui::RichText::new(obj.obj_type).size(12.0).color(text_color));
                            ui.end_row();

                            ui.label("");
                            ui.label("");
                            ui.end_row();

                            // Location
                            ui.label(egui::RichText::new("  Location").size(11.0).color(dim_text));
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = 4.0;
                                ui.colored_label(egui::Color32::from_rgb(0xE0, 0x50, 0x50), "X");
                                ui.add(egui::DragValue::new(&mut obj.location[0]).speed(0.05).fixed_decimals(2));
                                ui.colored_label(egui::Color32::from_rgb(0x50, 0xC0, 0x50), "Y");
                                ui.add(egui::DragValue::new(&mut obj.location[1]).speed(0.05).fixed_decimals(2));
                                ui.colored_label(egui::Color32::from_rgb(0x50, 0x80, 0xE0), "Z");
                                ui.add(egui::DragValue::new(&mut obj.location[2]).speed(0.05).fixed_decimals(2));
                            });
                            ui.end_row();

                            // Rotation
                            ui.label(egui::RichText::new("  Rotation").size(11.0).color(dim_text));
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = 4.0;
                                ui.colored_label(egui::Color32::from_rgb(0xE0, 0x50, 0x50), "X");
                                ui.add(egui::DragValue::new(&mut obj.rotation[0]).speed(0.5).fixed_decimals(1).suffix("\u{00B0}"));
                                ui.colored_label(egui::Color32::from_rgb(0x50, 0xC0, 0x50), "Y");
                                ui.add(egui::DragValue::new(&mut obj.rotation[1]).speed(0.5).fixed_decimals(1).suffix("\u{00B0}"));
                                ui.colored_label(egui::Color32::from_rgb(0x50, 0x80, 0xE0), "Z");
                                ui.add(egui::DragValue::new(&mut obj.rotation[2]).speed(0.5).fixed_decimals(1).suffix("\u{00B0}"));
                            });
                            ui.end_row();

                            // Scale
                            ui.label(egui::RichText::new("  Scale").size(11.0).color(dim_text));
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = 4.0;
                                ui.colored_label(egui::Color32::from_rgb(0xE0, 0x50, 0x50), "X");
                                ui.add(egui::DragValue::new(&mut obj.scale[0]).speed(0.01).fixed_decimals(3));
                                ui.colored_label(egui::Color32::from_rgb(0x50, 0xC0, 0x50), "Y");
                                ui.add(egui::DragValue::new(&mut obj.scale[1]).speed(0.01).fixed_decimals(3));
                                ui.colored_label(egui::Color32::from_rgb(0x50, 0x80, 0xE0), "Z");
                                ui.add(egui::DragValue::new(&mut obj.scale[2]).speed(0.01).fixed_decimals(3));
                            });
                            ui.end_row();
                        });
                } else {
                    ui.add_space(20.0);
                    ui.centered_and_justified(|ui| {
                        ui.label(egui::RichText::new("No object selected").size(12.0).color(dim_text));
                    });
                }
            });

        // ---- CENTRAL PANEL (3D Viewport) ----
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(egui::Color32::from_rgb(0x1A, 0x1A, 0x22))
                    .inner_margin(egui::Margin::same(0)),
            )
            .show(ctx, |ui| {
                // Viewport header
                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("3D Viewport").size(11.0).color(dim_text));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(8.0);
                        ui.label(egui::RichText::new("Solid").size(10.0).color(dim_text));
                        ui.label(egui::RichText::new("|").size(10.0).color(egui::Color32::from_rgb(0x40, 0x40, 0x40)));
                        ui.label(egui::RichText::new("Perspective").size(10.0).color(dim_text));
                    });
                });

                // The remaining area is for the 3D viewport
                let available_rect = ui.available_rect_before_wrap();

                // Compute the MVP matrix using the cube object's transform
                let cube_obj = &self.objects[0];
                let t = self.start_time.elapsed().as_secs_f32();

                // Build model matrix from the cube object's properties
                let loc = cube_obj.location;
                let rot = cube_obj.rotation;
                let scl = cube_obj.scale;

                let translate = mat4_translate(loc[0], loc[1], loc[2]);
                let rotate_x = mat4_rotate_x(rot[0].to_radians());
                let rotate_y = mat4_rotate_y(rot[1].to_radians());
                let rotate_z = mat4_rotate_z(rot[2].to_radians());
                let scale = mat4_scale(scl[0], scl[1], scl[2]);

                // Apply rotation from properties, then add spinning animation
                let spin = mat4_mul(&mat4_rotate_y(t * 0.7), &mat4_rotate_x(t * 0.4));
                let model_static = mat4_mul(&translate, &mat4_mul(&rotate_z, &mat4_mul(&rotate_y, &mat4_mul(&rotate_x, &scale))));
                let model = mat4_mul(&model_static, &spin);

                let vp_width = available_rect.width();
                let vp_height = available_rect.height();
                let aspect = if vp_height > 0.0 { vp_width / vp_height } else { 1.0 };

                let view = mat4_translate(0.0, 0.0, -3.0);
                let proj = mat4_perspective(std::f32::consts::FRAC_PI_4, aspect, 0.1, 100.0);
                let mv = mat4_mul(&view, &model);
                let mvp = mat4_mul(&proj, &mv);

                // Paint callback for the 3D cube
                let callback = egui_wgpu::Callback::new_paint_callback(
                    available_rect,
                    CubeCallback { mvp },
                );

                ui.painter().add(callback);

                // Overlay: axes indicator in bottom-left
                let axes_origin = available_rect.left_bottom() + egui::vec2(30.0, -30.0);
                let axis_len = 18.0;
                let painter = ui.painter();
                // X axis (red)
                painter.line_segment(
                    [axes_origin, axes_origin + egui::vec2(axis_len, 0.0)],
                    egui::Stroke::new(2.0, egui::Color32::from_rgb(0xE0, 0x50, 0x50)),
                );
                painter.text(
                    axes_origin + egui::vec2(axis_len + 4.0, 0.0),
                    egui::Align2::LEFT_CENTER,
                    "X",
                    egui::FontId::proportional(10.0),
                    egui::Color32::from_rgb(0xE0, 0x50, 0x50),
                );
                // Y axis (green, going up)
                painter.line_segment(
                    [axes_origin, axes_origin + egui::vec2(0.0, -axis_len)],
                    egui::Stroke::new(2.0, egui::Color32::from_rgb(0x50, 0xC0, 0x50)),
                );
                painter.text(
                    axes_origin + egui::vec2(0.0, -axis_len - 8.0),
                    egui::Align2::CENTER_BOTTOM,
                    "Y",
                    egui::FontId::proportional(10.0),
                    egui::Color32::from_rgb(0x50, 0xC0, 0x50),
                );
                // Z axis (blue, diagonal)
                painter.line_segment(
                    [axes_origin, axes_origin + egui::vec2(-axis_len * 0.5, -axis_len * 0.5)],
                    egui::Stroke::new(2.0, egui::Color32::from_rgb(0x50, 0x80, 0xE0)),
                );
                painter.text(
                    axes_origin + egui::vec2(-axis_len * 0.5 - 6.0, -axis_len * 0.5 - 6.0),
                    egui::Align2::RIGHT_BOTTOM,
                    "Z",
                    egui::FontId::proportional(10.0),
                    egui::Color32::from_rgb(0x50, 0x80, 0xE0),
                );
            });
    }

    fn render(&mut self) {
        let window = match &self.window {
            Some(w) => w.clone(),
            None => return,
        };

        // Update timeline
        if self.playing {
            let now = Instant::now();
            let dt = now.duration_since(self.last_frame_time).as_secs_f32();
            // Advance at 24 fps
            if dt >= 1.0 / 24.0 {
                self.current_frame += 1;
                if self.current_frame > self.end_frame {
                    self.current_frame = self.start_frame;
                }
                self.last_frame_time = now;
            }
        }

        // Phase 1: Run the egui frame (this borrows self mutably for draw_ui)
        let raw_input = self.egui_winit.as_mut().unwrap().take_egui_input(&window);
        self.egui_ctx.begin_pass(raw_input);
        self.draw_ui(&self.egui_ctx.clone());
        let full_output = self.egui_ctx.end_pass();

        // Handle platform output
        self.egui_winit.as_mut().unwrap().handle_platform_output(
            &window,
            full_output.platform_output,
        );

        // Tessellate
        let pixels_per_point = full_output.pixels_per_point;
        let clipped_primitives = self.egui_ctx.tessellate(full_output.shapes, pixels_per_point);
        let textures_delta = full_output.textures_delta;

        // Phase 2: GPU rendering (now we can borrow device/queue/surface)
        let device = self.device.as_ref().unwrap();
        let queue = self.queue.as_ref().unwrap();
        let surface = self.surface.as_ref().unwrap();
        let config = self.surface_config.as_ref().unwrap();

        // Get surface texture
        let output_frame = match surface.get_current_texture() {
            Ok(tex) => tex,
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                self.resize(config.width, config.height);
                return;
            }
            Err(e) => {
                eprintln!("Surface error: {e}");
                return;
            }
        };

        let view = output_frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [config.width, config.height],
            pixels_per_point,
        };

        let renderer = self.egui_renderer.as_mut().unwrap();

        // Update textures
        for (id, image_delta) in &textures_delta.set {
            renderer.update_texture(device, queue, *id, image_delta);
        }

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("egui-encoder"),
        });

        // Update egui buffers (this also calls prepare on our CubeCallback)
        let user_cmd_bufs = renderer.update_buffers(
            device,
            queue,
            &mut encoder,
            &clipped_primitives,
            &screen_descriptor,
        );

        // Render pass
        {
            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui-render-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.188,
                            g: 0.188,
                            b: 0.188,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            renderer.render(
                &mut render_pass.forget_lifetime(),
                &clipped_primitives,
                &screen_descriptor,
            );
        }

        // Free textures
        for id in &textures_delta.free {
            renderer.free_texture(id);
        }

        // Submit
        let encoded = encoder.finish();
        let mut cmd_bufs: Vec<wgpu::CommandBuffer> = user_cmd_bufs;
        cmd_bufs.push(encoded);
        queue.submit(cmd_bufs);
        output_frame.present();
    }
}

// ---------------------------------------------------------------------------
// winit ApplicationHandler
// ---------------------------------------------------------------------------

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let attrs = Window::default_attributes()
            .with_title("Forge3D")
            .with_inner_size(winit::dpi::LogicalSize::new(1600, 900));

        let window = Arc::new(event_loop.create_window(attrs).expect("Failed to create window"));
        self.init_wgpu(window.clone());

        tracing::info!("Forge3D window opened");
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        // Let egui handle the event first
        if let Some(egui_winit) = &mut self.egui_winit {
            if let Some(window) = &self.window {
                let response = egui_winit.on_window_event(window, &event);
                if response.consumed {
                    return;
                }
            }
        }

        match event {
            WindowEvent::CloseRequested => {
                tracing::info!("Window closed");
                event_loop.exit();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Named(NamedKey::Escape),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                tracing::info!("Escape pressed — exiting");
                event_loop.exit();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Named(NamedKey::Space),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                self.playing = !self.playing;
                self.last_frame_time = Instant::now();
            }
            WindowEvent::Resized(size) => {
                self.resize(size.width, size.height);
            }
            WindowEvent::RedrawRequested => {
                self.render();
                if let Some(w) = &self.window {
                    w.request_redraw();
                }
            }
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() {
    tracing_subscriber::fmt()
        .with_target(true)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .init();

    tracing::info!("Starting Forge3D");

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let mut app = App::new();
    event_loop.run_app(&mut app).expect("Event loop error");
}
