//! Forge3D — Clean egui-only Blender-like interface.
//!
//! Architecture:
//!   1. winit window + wgpu surface (NON-sRGB: Bgra8Unorm)
//!   2. egui-winit + egui-wgpu for ALL UI rendering
//!   3. egui_wgpu::Callback for the 3D cube in the central panel
//!   4. One clean render pass — no custom 2D shader, no DrawList

use std::sync::Arc;
use std::time::Instant;

use bytemuck::{Pod, Zeroable};
use egui::{
    Color32, FontId, Frame, Stroke, TextStyle,
};
use wgpu::util::DeviceExt;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};

// ---------------------------------------------------------------------------
// 3D Cube Vertex
// ---------------------------------------------------------------------------

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct CubeVertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl CubeVertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
        0 => Float32x3,
        1 => Float32x3,
    ];

    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<CubeVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[rustfmt::skip]
const CUBE_VERTICES: &[CubeVertex] = &[
    // Front face (muted red)
    CubeVertex { position: [-0.5, -0.5,  0.5], color: [0.8, 0.3, 0.3] },
    CubeVertex { position: [ 0.5, -0.5,  0.5], color: [0.8, 0.3, 0.3] },
    CubeVertex { position: [ 0.5,  0.5,  0.5], color: [0.8, 0.3, 0.3] },
    CubeVertex { position: [-0.5,  0.5,  0.5], color: [0.8, 0.3, 0.3] },
    // Back face (muted blue)
    CubeVertex { position: [-0.5, -0.5, -0.5], color: [0.3, 0.3, 0.8] },
    CubeVertex { position: [ 0.5, -0.5, -0.5], color: [0.3, 0.3, 0.8] },
    CubeVertex { position: [ 0.5,  0.5, -0.5], color: [0.3, 0.3, 0.8] },
    CubeVertex { position: [-0.5,  0.5, -0.5], color: [0.3, 0.3, 0.8] },
    // Top face (muted green)
    CubeVertex { position: [-0.5,  0.5, -0.5], color: [0.3, 0.7, 0.3] },
    CubeVertex { position: [ 0.5,  0.5, -0.5], color: [0.3, 0.7, 0.3] },
    CubeVertex { position: [ 0.5,  0.5,  0.5], color: [0.3, 0.7, 0.3] },
    CubeVertex { position: [-0.5,  0.5,  0.5], color: [0.3, 0.7, 0.3] },
    // Bottom face (muted yellow)
    CubeVertex { position: [-0.5, -0.5, -0.5], color: [0.7, 0.7, 0.3] },
    CubeVertex { position: [ 0.5, -0.5, -0.5], color: [0.7, 0.7, 0.3] },
    CubeVertex { position: [ 0.5, -0.5,  0.5], color: [0.7, 0.7, 0.3] },
    CubeVertex { position: [-0.5, -0.5,  0.5], color: [0.7, 0.7, 0.3] },
    // Right face (muted cyan)
    CubeVertex { position: [ 0.5, -0.5, -0.5], color: [0.3, 0.7, 0.7] },
    CubeVertex { position: [ 0.5,  0.5, -0.5], color: [0.3, 0.7, 0.7] },
    CubeVertex { position: [ 0.5,  0.5,  0.5], color: [0.3, 0.7, 0.7] },
    CubeVertex { position: [ 0.5, -0.5,  0.5], color: [0.3, 0.7, 0.7] },
    // Left face (muted magenta)
    CubeVertex { position: [-0.5, -0.5, -0.5], color: [0.7, 0.3, 0.7] },
    CubeVertex { position: [-0.5,  0.5, -0.5], color: [0.7, 0.3, 0.7] },
    CubeVertex { position: [-0.5,  0.5,  0.5], color: [0.7, 0.3, 0.7] },
    CubeVertex { position: [-0.5, -0.5,  0.5], color: [0.7, 0.3, 0.7] },
];

#[rustfmt::skip]
const CUBE_INDICES: &[u16] = &[
     0,  1,  2,  2,  3,  0,
     4,  6,  5,  6,  4,  7,
     8,  9, 10, 10, 11,  8,
    12, 14, 13, 14, 12, 15,
    16, 17, 18, 18, 19, 16,
    20, 22, 21, 22, 20, 23,
];

// ---------------------------------------------------------------------------
// 3D Cube Shader (WGSL)
// ---------------------------------------------------------------------------

const CUBE_SHADER_SRC: &str = r#"
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
// Column-major 4x4 matrix helpers
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
    let (s, c) = (angle.sin(), angle.cos());
    [
         c,  0.0, -s,  0.0,
        0.0, 1.0, 0.0, 0.0,
         s,  0.0,  c,  0.0,
        0.0, 0.0, 0.0, 1.0,
    ]
}

fn mat4_rotate_x(angle: f32) -> [f32; 16] {
    let (s, c) = (angle.sin(), angle.cos());
    [
        1.0, 0.0, 0.0, 0.0,
        0.0,  c,   s,  0.0,
        0.0, -s,   c,  0.0,
        0.0, 0.0, 0.0, 1.0,
    ]
}

#[allow(dead_code)]
fn mat4_rotate_z(angle: f32) -> [f32; 16] {
    let (s, c) = (angle.sin(), angle.cos());
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

#[allow(dead_code)]
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
        f / aspect, 0.0, 0.0,                    0.0,
        0.0,        f,   0.0,                    0.0,
        0.0,        0.0, (far + near) * nf,     -1.0,
        0.0,        0.0, 2.0 * far * near * nf,  0.0,
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
    visible: bool,
    renderable: bool,
}

impl SceneObject {
    fn new(name: &str, obj_type: &'static str, loc: [f32; 3], rot: [f32; 3], scl: [f32; 3]) -> Self {
        Self {
            name: name.to_string(),
            obj_type,
            location: loc,
            rotation: rot,
            scale: scl,
            visible: true,
            renderable: true,
        }
    }

    fn icon(&self) -> &'static str {
        match self.obj_type {
            "Mesh" => "\u{1F536}",   // orange diamond
            "Camera" => "\u{1F4F7}", // camera
            "Light" => "\u{1F4A1}",  // light bulb
            _ => "\u{25CF}",
        }
    }
}

// ---------------------------------------------------------------------------
// Workspace tabs
// ---------------------------------------------------------------------------

const WORKSPACE_TABS: &[&str] = &[
    "Layout", "Modeling", "Sculpting", "UV Editing",
    "Shading", "Animation", "Rendering",
];

// ---------------------------------------------------------------------------
// Cube render resources (stored in egui_wgpu CallbackResources)
// ---------------------------------------------------------------------------

struct CubeResources {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    num_indices: u32,
}

impl CubeResources {
    fn create(device: &wgpu::Device, target_format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("cube-shader"),
            source: wgpu::ShaderSource::Wgsl(CUBE_SHADER_SRC.into()),
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
                buffers: &[CubeVertex::layout()],
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
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("cube-vbo"),
            contents: bytemuck::cast_slice(CUBE_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("cube-ibo"),
            contents: bytemuck::cast_slice(CUBE_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            pipeline,
            vertex_buffer,
            index_buffer,
            uniform_buffer,
            bind_group,
            num_indices: CUBE_INDICES.len() as u32,
        }
    }
}

// ---------------------------------------------------------------------------
// egui_wgpu paint callback for 3D viewport
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
        let rect = info.viewport_in_pixels();
        if rect.width_px > 0 && rect.height_px > 0 {
            render_pass.set_viewport(
                rect.left_px as f32, rect.top_px as f32,
                rect.width_px as f32, rect.height_px as f32,
                0.0, 1.0,
            );
            render_pass.set_scissor_rect(
                rect.left_px as u32, rect.top_px as u32,
                rect.width_px as u32, rect.height_px as u32,
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
// GPU state
// ---------------------------------------------------------------------------

struct GpuState {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
}

// ---------------------------------------------------------------------------
// Egui state
// ---------------------------------------------------------------------------

struct EguiState {
    winit_state: egui_winit::State,
    renderer: egui_wgpu::Renderer,
}

// ---------------------------------------------------------------------------
// Application
// ---------------------------------------------------------------------------

struct App {
    window: Option<Arc<Window>>,
    gpu: Option<GpuState>,
    egui_state: Option<EguiState>,
    egui_ctx: egui::Context,

    // Scene
    objects: Vec<SceneObject>,
    selected: usize,

    // Animation
    current_frame: i32,
    start_frame: i32,
    end_frame: i32,
    playing: bool,
    start_time: Instant,

    // UI state
    workspace: usize,
    active_shading: usize,

    // Panel collapse states (used by egui CollapsingState default_open)
    #[allow(dead_code)]
    transform_open: bool,
    #[allow(dead_code)]
    relations_open: bool,
    #[allow(dead_code)]
    collections_open: bool,
    #[allow(dead_code)]
    scene_collection_open: bool,
}

impl App {
    fn new() -> Self {
        let objects = vec![
            SceneObject::new("Cube",   "Mesh",   [0.0, 0.0, 0.0],     [0.0, 0.0, 0.0],     [1.0, 1.0, 1.0]),
            SceneObject::new("Camera", "Camera", [7.36, -6.93, 4.96], [63.6, 0.0, 46.7],   [1.0, 1.0, 1.0]),
            SceneObject::new("Light",  "Light",  [4.08, 1.0, 5.90],   [37.3, 3.16, 107.0], [1.0, 1.0, 1.0]),
        ];

        Self {
            window: None,
            gpu: None,
            egui_state: None,
            egui_ctx: egui::Context::default(),

            objects,
            selected: 0,

            current_frame: 1,
            start_frame: 1,
            end_frame: 250,
            playing: false,
            start_time: Instant::now(),

            workspace: 0,
            active_shading: 1,

            transform_open: true,
            relations_open: false,
            collections_open: false,
            scene_collection_open: true,
        }
    }

    fn init_gpu(&mut self, window: Arc<Window>) {
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
        // NON-sRGB format so theme colors display correctly
        let format = caps.formats.iter().find(|f| !f.is_srgb()).copied().unwrap_or(caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            desired_maximum_frame_latency: 2,
            alpha_mode: caps.alpha_modes.first().copied().unwrap_or(wgpu::CompositeAlphaMode::Auto),
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        // egui
        let renderer = egui_wgpu::Renderer::new(&device, format, None, 1, true);
        let winit_state = egui_winit::State::new(
            self.egui_ctx.clone(),
            egui::ViewportId::ROOT,
            &window,
            Some(window.scale_factor() as f32),
            None,
            Some(device.limits().max_texture_dimension_2d as usize),
        );

        // Cube resources into callback resources
        let cube_resources = CubeResources::create(&device, format);

        let mut egui_state = EguiState { winit_state, renderer };
        egui_state.renderer.callback_resources.insert(cube_resources);

        self.window = Some(window);
        self.gpu = Some(GpuState { device, queue, surface, config });
        self.egui_state = Some(egui_state);

        setup_blender_theme(&self.egui_ctx);
    }

    fn resize(&mut self, width: u32, height: u32) {
        if let Some(gpu) = &mut self.gpu {
            gpu.config.width = width.max(1);
            gpu.config.height = height.max(1);
            gpu.surface.configure(&gpu.device, &gpu.config);
        }
    }

    // -----------------------------------------------------------------------
    // Full UI layout
    // -----------------------------------------------------------------------

    fn draw_ui(&mut self, ctx: &egui::Context) {
        self.draw_top_bar(ctx);
        self.draw_timeline(ctx);
        self.draw_right_panel(ctx);
        self.draw_viewport(ctx);
    }

    // -- Top bar --
    fn draw_top_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("topbar")
            .exact_height(26.0)
            .frame(Frame::NONE.fill(Color32::from_rgb(42, 42, 42)))
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.spacing_mut().item_spacing.x = 2.0;

                    // Logo
                    ui.label(
                        egui::RichText::new("Forge3D")
                            .strong()
                            .color(Color32::from_rgb(200, 200, 200))
                            .size(13.0),
                    );

                    ui.separator();

                    // Menus
                    ui.menu_button("File", |ui| {
                        if ui.button("New").clicked() { ui.close_menu(); }
                        if ui.button("Open...").clicked() { ui.close_menu(); }
                        if ui.button("Save").clicked() { ui.close_menu(); }
                        if ui.button("Save As...").clicked() { ui.close_menu(); }
                        ui.separator();
                        if ui.button("Quit").clicked() { ui.close_menu(); }
                    });
                    ui.menu_button("Edit", |ui| {
                        if ui.button("Undo").clicked() { ui.close_menu(); }
                        if ui.button("Redo").clicked() { ui.close_menu(); }
                        ui.separator();
                        if ui.button("Preferences...").clicked() { ui.close_menu(); }
                    });
                    ui.menu_button("Render", |ui| {
                        if ui.button("Render Image").clicked() { ui.close_menu(); }
                        if ui.button("Render Animation").clicked() { ui.close_menu(); }
                    });
                    ui.menu_button("Window", |ui| {
                        if ui.button("New Window").clicked() { ui.close_menu(); }
                        if ui.button("Toggle Fullscreen").clicked() { ui.close_menu(); }
                    });
                    ui.menu_button("Help", |ui| {
                        if ui.button("About Forge3D").clicked() { ui.close_menu(); }
                    });

                    ui.separator();

                    // Workspace tabs
                    for (i, name) in WORKSPACE_TABS.iter().enumerate() {
                        if ui.selectable_label(self.workspace == i, *name).clicked() {
                            self.workspace = i;
                        }
                    }

                    ui.separator();

                    // Scene / ViewLayer labels
                    ui.label(
                        egui::RichText::new("Scene")
                            .color(Color32::from_rgb(180, 180, 180))
                            .size(11.0),
                    );
                    ui.label(
                        egui::RichText::new("ViewLayer")
                            .color(Color32::from_rgb(180, 180, 180))
                            .size(11.0),
                    );
                });
            });
    }

    // -- Timeline --
    fn draw_timeline(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("timeline")
            .exact_height(80.0)
            .frame(Frame::NONE.fill(Color32::from_rgb(42, 42, 42)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 2.0;

                    // Transport buttons
                    if ui.button("\u{23EE}").clicked() {
                        self.current_frame = self.start_frame;
                    }
                    if ui.button("\u{23EA}").clicked() {
                        self.current_frame = (self.current_frame - 1).max(self.start_frame);
                    }
                    let play_label = if self.playing { "\u{23F8}" } else { "\u{25B6}" };
                    if ui.button(play_label).clicked() {
                        self.playing = !self.playing;
                        if self.playing {
                            self.start_time = Instant::now();
                        }
                    }
                    if ui.button("\u{23E9}").clicked() {
                        self.current_frame = (self.current_frame + 1).min(self.end_frame);
                    }
                    if ui.button("\u{23ED}").clicked() {
                        self.current_frame = self.end_frame;
                    }

                    ui.separator();

                    ui.label("Frame:");
                    ui.add(egui::DragValue::new(&mut self.current_frame)
                        .range(self.start_frame..=self.end_frame)
                        .speed(0.5));

                    ui.separator();

                    ui.label("Start:");
                    ui.add(egui::DragValue::new(&mut self.start_frame).speed(1.0));
                    ui.label("End:");
                    ui.add(egui::DragValue::new(&mut self.end_frame).speed(1.0));
                });

                // Timeline scrub slider
                ui.add_space(4.0);
                let mut frame_f = self.current_frame as f32;
                let slider = egui::Slider::new(
                    &mut frame_f,
                    self.start_frame as f32..=self.end_frame as f32,
                )
                .show_value(false);
                if ui.add(slider).changed() {
                    self.current_frame = frame_f as i32;
                }

                // Frame ruler marks
                ui.horizontal(|ui| {
                    let available = ui.available_width();
                    let range = (self.end_frame - self.start_frame).max(1) as f32;
                    let step = (range / 10.0).max(1.0) as i32;
                    for i in 0..=10 {
                        let f = self.start_frame + i * step;
                        let x_frac = (f - self.start_frame) as f32 / range;
                        let _ = available; // use for layout reference
                        ui.label(
                            egui::RichText::new(format!("{f}"))
                                .color(Color32::from_rgb(140, 140, 140))
                                .size(9.0),
                        );
                        if x_frac < 0.95 {
                            ui.add_space(available / 12.0 - 20.0);
                        }
                    }
                });
            });
    }

    // -- Right panel: outliner + properties --
    fn draw_right_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("right")
            .exact_width(300.0)
            .frame(Frame::NONE.fill(Color32::from_rgb(40, 40, 40)))
            .show(ctx, |ui| {
                // Outliner section
                ui.label(
                    egui::RichText::new("Outliner")
                        .color(Color32::from_rgb(180, 180, 180))
                        .size(11.0),
                );
                ui.separator();

                egui::CollapsingHeader::new(
                        egui::RichText::new("Scene Collection")
                            .color(Color32::from_rgb(200, 200, 200)),
                    )
                    .default_open(true)
                    .show(ui, |ui| {
                        let mut new_selected = self.selected;
                        for (i, obj) in self.objects.iter().enumerate() {
                            ui.horizontal(|ui| {
                                let label_text = format!("{} {}", obj.icon(), obj.name);
                                let response = ui.selectable_label(
                                    self.selected == i,
                                    egui::RichText::new(&label_text)
                                        .color(if self.selected == i {
                                            Color32::WHITE
                                        } else {
                                            Color32::from_rgb(200, 200, 200)
                                        }),
                                );
                                if response.clicked() {
                                    new_selected = i;
                                }

                                // Visibility / render toggles on the right
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    let vis_label = if obj.visible { "\u{1F441}" } else { "\u{1F441}\u{200D}\u{1F5E8}" };
                                    ui.label(
                                        egui::RichText::new(vis_label)
                                            .color(Color32::from_rgb(160, 160, 160))
                                            .size(11.0),
                                    );
                                    let cam_label = if obj.renderable { "\u{1F4F7}" } else { " " };
                                    ui.label(
                                        egui::RichText::new(cam_label)
                                            .color(Color32::from_rgb(160, 160, 160))
                                            .size(11.0),
                                    );
                                });
                            });
                        }
                        self.selected = new_selected;
                    });

                ui.add_space(10.0);
                ui.separator();

                // Properties section
                ui.label(
                    egui::RichText::new("Properties")
                        .color(Color32::from_rgb(180, 180, 180))
                        .size(11.0),
                );
                ui.separator();

                if let Some(obj) = self.objects.get_mut(self.selected) {
                    // Transform
                    let transform_id = ui.make_persistent_id("transform_section");
                    egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), transform_id, true)
                        .show_header(ui, |ui| {
                            ui.label(
                                egui::RichText::new("Transform")
                                    .color(Color32::from_rgb(200, 200, 200)),
                            );
                        })
                        .body(|ui| {
                            Self::draw_xyz_row(ui, "Location", &mut obj.location, 0.01);
                            Self::draw_xyz_row(ui, "Rotation", &mut obj.rotation, 0.5);
                            Self::draw_xyz_row(ui, "Scale", &mut obj.scale, 0.01);
                        });

                    // Relations
                    let relations_id = ui.make_persistent_id("relations_section");
                    egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), relations_id, false)
                        .show_header(ui, |ui| {
                            ui.label(
                                egui::RichText::new("Relations")
                                    .color(Color32::from_rgb(200, 200, 200)),
                            );
                        })
                        .body(|ui| {
                            ui.label("Parent: None");
                        });

                    // Collections
                    let collections_id = ui.make_persistent_id("collections_section");
                    egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), collections_id, false)
                        .show_header(ui, |ui| {
                            ui.label(
                                egui::RichText::new("Collections")
                                    .color(Color32::from_rgb(200, 200, 200)),
                            );
                        })
                        .body(|ui| {
                            ui.label("Scene Collection");
                        });
                }
            });
    }

    fn draw_xyz_row(ui: &mut egui::Ui, label: &str, values: &mut [f32; 3], speed: f32) {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(label)
                    .color(Color32::from_rgb(180, 180, 180))
                    .size(11.0),
            );
            ui.colored_label(Color32::from_rgb(220, 70, 70), "X");
            ui.add(egui::DragValue::new(&mut values[0]).speed(speed).max_decimals(3));
            ui.colored_label(Color32::from_rgb(70, 200, 70), "Y");
            ui.add(egui::DragValue::new(&mut values[1]).speed(speed).max_decimals(3));
            ui.colored_label(Color32::from_rgb(70, 120, 220), "Z");
            ui.add(egui::DragValue::new(&mut values[2]).speed(speed).max_decimals(3));
        });
    }

    // -- 3D Viewport --
    fn draw_viewport(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(Frame::NONE.fill(Color32::from_rgb(51, 51, 51)))
            .show(ctx, |ui| {
                // Viewport header
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 2.0;
                    ui.label(
                        egui::RichText::new("Object Mode")
                            .color(Color32::from_rgb(200, 200, 200))
                            .size(11.0),
                    );
                    ui.separator();

                    let shading_labels = ["Wire", "Solid", "Material", "Rendered"];
                    for (i, label) in shading_labels.iter().enumerate() {
                        if ui.selectable_label(self.active_shading == i, *label).clicked() {
                            self.active_shading = i;
                        }
                    }
                });
                ui.separator();

                // 3D cube render via callback
                let rect = ui.available_rect_before_wrap();
                let aspect = if rect.height() > 0.0 { rect.width() / rect.height() } else { 1.0 };

                let t = self.start_time.elapsed().as_secs_f32();
                let model = mat4_mul(&mat4_rotate_y(t * 0.7), &mat4_rotate_x(0.4));
                let view = mat4_translate(0.0, 0.0, -3.0);
                let proj = mat4_perspective(std::f32::consts::FRAC_PI_4, aspect, 0.1, 100.0);
                let mvp = mat4_mul(&proj, &mat4_mul(&view, &model));

                let cb = egui_wgpu::Callback::new_paint_callback(
                    rect,
                    CubeCallback { mvp },
                );
                ui.painter().add(cb);

                // Grid overlay
                let painter = ui.painter();
                let grid_color = Color32::from_rgba_premultiplied(70, 70, 70, 60);
                let center = rect.center();
                let grid_spacing = 50.0;

                // Horizontal lines
                let mut y = center.y;
                while y >= rect.top() {
                    painter.line_segment(
                        [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                        Stroke::new(0.5, grid_color),
                    );
                    y -= grid_spacing;
                }
                y = center.y + grid_spacing;
                while y <= rect.bottom() {
                    painter.line_segment(
                        [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                        Stroke::new(0.5, grid_color),
                    );
                    y += grid_spacing;
                }

                // Vertical lines
                let mut x = center.x;
                while x >= rect.left() {
                    painter.line_segment(
                        [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                        Stroke::new(0.5, grid_color),
                    );
                    x -= grid_spacing;
                }
                x = center.x + grid_spacing;
                while x <= rect.right() {
                    painter.line_segment(
                        [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                        Stroke::new(0.5, grid_color),
                    );
                    x += grid_spacing;
                }

                // Axis lines (X = red, Y = green)
                painter.line_segment(
                    [egui::pos2(rect.left(), center.y), egui::pos2(rect.right(), center.y)],
                    Stroke::new(1.0, Color32::from_rgba_premultiplied(150, 50, 50, 100)),
                );
                painter.line_segment(
                    [egui::pos2(center.x, rect.top()), egui::pos2(center.x, rect.bottom())],
                    Stroke::new(1.0, Color32::from_rgba_premultiplied(50, 150, 50, 100)),
                );

                // XYZ gizmo in bottom-left
                let gizmo_origin = egui::pos2(rect.left() + 40.0, rect.bottom() - 40.0);
                let gizmo_len = 30.0;
                painter.line_segment(
                    [gizmo_origin, egui::pos2(gizmo_origin.x + gizmo_len, gizmo_origin.y)],
                    Stroke::new(2.0, Color32::from_rgb(200, 60, 60)),
                );
                painter.text(
                    egui::pos2(gizmo_origin.x + gizmo_len + 4.0, gizmo_origin.y),
                    egui::Align2::LEFT_CENTER,
                    "X",
                    egui::FontId::proportional(10.0),
                    Color32::from_rgb(200, 60, 60),
                );
                painter.line_segment(
                    [gizmo_origin, egui::pos2(gizmo_origin.x, gizmo_origin.y - gizmo_len)],
                    Stroke::new(2.0, Color32::from_rgb(60, 200, 60)),
                );
                painter.text(
                    egui::pos2(gizmo_origin.x, gizmo_origin.y - gizmo_len - 10.0),
                    egui::Align2::CENTER_BOTTOM,
                    "Y",
                    egui::FontId::proportional(10.0),
                    Color32::from_rgb(60, 200, 60),
                );
                // Z goes diagonally
                painter.line_segment(
                    [gizmo_origin, egui::pos2(gizmo_origin.x - gizmo_len * 0.5, gizmo_origin.y - gizmo_len * 0.5)],
                    Stroke::new(2.0, Color32::from_rgb(60, 100, 200)),
                );
                painter.text(
                    egui::pos2(gizmo_origin.x - gizmo_len * 0.5 - 10.0, gizmo_origin.y - gizmo_len * 0.5),
                    egui::Align2::RIGHT_CENTER,
                    "Z",
                    egui::FontId::proportional(10.0),
                    Color32::from_rgb(60, 100, 200),
                );

                // "Object Mode" overlay top-left
                painter.text(
                    egui::pos2(rect.left() + 8.0, rect.top() + 8.0),
                    egui::Align2::LEFT_TOP,
                    "Object Mode",
                    egui::FontId::proportional(12.0),
                    Color32::from_rgb(200, 200, 200),
                );

                // Stats overlay bottom-right
                painter.text(
                    egui::pos2(rect.right() - 8.0, rect.bottom() - 8.0),
                    egui::Align2::RIGHT_BOTTOM,
                    "Verts: 8  Faces: 6",
                    egui::FontId::proportional(11.0),
                    Color32::from_rgb(160, 160, 160),
                );
            });
    }

    // -----------------------------------------------------------------------
    // Render frame
    // -----------------------------------------------------------------------

    fn render(&mut self) {
        if self.gpu.is_none() || self.egui_state.is_none() || self.window.is_none() {
            return;
        }

        // Animation update
        if self.playing {
            let elapsed = self.start_time.elapsed().as_secs_f32();
            let fps = 24.0;
            let frame_offset = (elapsed * fps) as i32;
            let range = (self.end_frame - self.start_frame).max(1);
            self.current_frame = self.start_frame + (frame_offset % range);
        }

        // Get surface texture
        let output = {
            let gpu = self.gpu.as_ref().unwrap();
            match gpu.surface.get_current_texture() {
                Ok(o) => o,
                Err(wgpu::SurfaceError::Outdated | wgpu::SurfaceError::Lost) => {
                    let size = self.window.as_ref().unwrap().inner_size();
                    let gpu = self.gpu.as_mut().unwrap();
                    gpu.config.width = size.width.max(1);
                    gpu.config.height = size.height.max(1);
                    gpu.surface.configure(&gpu.device, &gpu.config);
                    return;
                }
                Err(_) => return,
            }
        };

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Run egui — take_egui_input, run UI, handle output
        let raw_input = {
            let egui_st = self.egui_state.as_mut().unwrap();
            let window = self.window.as_ref().unwrap();
            egui_st.winit_state.take_egui_input(window)
        };

        let ctx = self.egui_ctx.clone();
        let full_output = ctx.run(raw_input, |ctx| {
            self.draw_ui(ctx);
        });

        {
            let egui_st = self.egui_state.as_mut().unwrap();
            let window = self.window.as_ref().unwrap();
            egui_st.winit_state.handle_platform_output(window, full_output.platform_output.clone());
        }

        let primitives = ctx.tessellate(full_output.shapes, full_output.pixels_per_point);

        let gpu = self.gpu.as_ref().unwrap();
        let egui_st = self.egui_state.as_mut().unwrap();

        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [gpu.config.width, gpu.config.height],
            pixels_per_point: full_output.pixels_per_point,
        };

        let mut encoder = gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("forge3d-encoder"),
        });

        // Upload egui textures
        for (id, delta) in &full_output.textures_delta.set {
            egui_st.renderer.update_texture(&gpu.device, &gpu.queue, *id, delta);
        }
        egui_st.renderer.update_buffers(&gpu.device, &gpu.queue, &mut encoder, &primitives, &screen_descriptor);

        // Single render pass
        {
            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui-render-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.2, g: 0.2, b: 0.2, a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            egui_st.renderer.render(
                &mut render_pass.forget_lifetime(),
                &primitives,
                &screen_descriptor,
            );
        }

        // Free textures
        for id in &full_output.textures_delta.free {
            egui_st.renderer.free_texture(id);
        }

        gpu.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        // Request repaint for animation
        if self.playing {
            if let Some(w) = &self.window {
                w.request_redraw();
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Blender dark theme
// ---------------------------------------------------------------------------

fn setup_blender_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    let mut visuals = egui::Visuals::dark();

    // Window/panel backgrounds
    visuals.window_fill = Color32::from_rgb(48, 48, 48);
    visuals.panel_fill = Color32::from_rgb(48, 48, 48);
    visuals.faint_bg_color = Color32::from_rgb(40, 40, 40);
    visuals.extreme_bg_color = Color32::from_rgb(30, 30, 30);

    // Widget colors
    visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(61, 61, 61);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, Color32::from_rgb(230, 230, 230));
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(36, 36, 36));
    visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(3);

    visuals.widgets.inactive.bg_fill = Color32::from_rgb(70, 70, 70);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, Color32::from_rgb(220, 220, 220));
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(55, 55, 55));
    visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(3);

    visuals.widgets.hovered.bg_fill = Color32::from_rgb(80, 80, 80);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, Color32::from_rgb(255, 255, 255));
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, Color32::from_rgb(100, 100, 100));

    visuals.widgets.active.bg_fill = Color32::from_rgb(71, 114, 179);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, Color32::from_rgb(255, 255, 255));

    // Selection
    visuals.selection.bg_fill = Color32::from_rgb(71, 114, 179);
    visuals.selection.stroke = Stroke::new(1.0, Color32::from_rgb(100, 150, 220));

    // Text styles
    style.text_styles.insert(TextStyle::Body, FontId::proportional(13.0));
    style.text_styles.insert(TextStyle::Small, FontId::proportional(11.0));
    style.text_styles.insert(TextStyle::Button, FontId::proportional(12.0));
    style.text_styles.insert(TextStyle::Heading, FontId::proportional(14.0));

    // Spacing
    style.spacing.item_spacing = egui::vec2(4.0, 3.0);
    style.spacing.button_padding = egui::vec2(6.0, 2.0);

    style.visuals = visuals;
    ctx.set_style(style);
}

// ---------------------------------------------------------------------------
// ApplicationHandler
// ---------------------------------------------------------------------------

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let attrs = Window::default_attributes()
            .with_title("Forge3D")
            .with_inner_size(winit::dpi::LogicalSize::new(1600.0, 900.0))
            .with_maximized(true);

        let window = Arc::new(event_loop.create_window(attrs).unwrap());
        self.init_gpu(window);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        // Let egui handle events first
        if let Some(egui_st) = &mut self.egui_state {
            let response = egui_st.winit_state.on_window_event(self.window.as_ref().unwrap(), &event);
            if response.consumed {
                if let Some(w) = &self.window {
                    w.request_redraw();
                }
                return;
            }
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                self.resize(size.width, size.height);
                if let Some(w) = &self.window {
                    w.request_redraw();
                }
            }
            WindowEvent::KeyboardInput {
                event: KeyEvent {
                    logical_key,
                    state: ElementState::Pressed,
                    ..
                },
                ..
            } => {
                match logical_key {
                    Key::Named(NamedKey::Escape) => event_loop.exit(),
                    Key::Named(NamedKey::Space) => {
                        self.playing = !self.playing;
                        if self.playing {
                            self.start_time = Instant::now();
                        }
                        if let Some(w) = &self.window {
                            w.request_redraw();
                        }
                    }
                    _ => {}
                }
            }
            WindowEvent::RedrawRequested => {
                self.render();
                // Keep redrawing when playing
                if self.playing {
                    if let Some(w) = &self.window {
                        w.request_redraw();
                    }
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // Always request redraw for continuous animation of cube
        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }
}

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------

fn main() {
    tracing_subscriber::fmt::init();

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let mut app = App::new();
    event_loop.run_app(&mut app).expect("Event loop failed");
}
