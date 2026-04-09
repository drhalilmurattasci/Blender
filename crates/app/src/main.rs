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

// #3: All faces are solid gray like Blender's default cube
#[rustfmt::skip]
const CUBE_VERTICES: &[CubeVertex] = &[
    // Front face
    CubeVertex { position: [-0.5, -0.5,  0.5], color: [0.62, 0.60, 0.58] },
    CubeVertex { position: [ 0.5, -0.5,  0.5], color: [0.62, 0.60, 0.58] },
    CubeVertex { position: [ 0.5,  0.5,  0.5], color: [0.62, 0.60, 0.58] },
    CubeVertex { position: [-0.5,  0.5,  0.5], color: [0.62, 0.60, 0.58] },
    // Back face
    CubeVertex { position: [-0.5, -0.5, -0.5], color: [0.62, 0.60, 0.58] },
    CubeVertex { position: [ 0.5, -0.5, -0.5], color: [0.62, 0.60, 0.58] },
    CubeVertex { position: [ 0.5,  0.5, -0.5], color: [0.62, 0.60, 0.58] },
    CubeVertex { position: [-0.5,  0.5, -0.5], color: [0.62, 0.60, 0.58] },
    // Top face
    CubeVertex { position: [-0.5,  0.5, -0.5], color: [0.62, 0.60, 0.58] },
    CubeVertex { position: [ 0.5,  0.5, -0.5], color: [0.62, 0.60, 0.58] },
    CubeVertex { position: [ 0.5,  0.5,  0.5], color: [0.62, 0.60, 0.58] },
    CubeVertex { position: [-0.5,  0.5,  0.5], color: [0.62, 0.60, 0.58] },
    // Bottom face
    CubeVertex { position: [-0.5, -0.5, -0.5], color: [0.62, 0.60, 0.58] },
    CubeVertex { position: [ 0.5, -0.5, -0.5], color: [0.62, 0.60, 0.58] },
    CubeVertex { position: [ 0.5, -0.5,  0.5], color: [0.62, 0.60, 0.58] },
    CubeVertex { position: [-0.5, -0.5,  0.5], color: [0.62, 0.60, 0.58] },
    // Right face
    CubeVertex { position: [ 0.5, -0.5, -0.5], color: [0.62, 0.60, 0.58] },
    CubeVertex { position: [ 0.5,  0.5, -0.5], color: [0.62, 0.60, 0.58] },
    CubeVertex { position: [ 0.5,  0.5,  0.5], color: [0.62, 0.60, 0.58] },
    CubeVertex { position: [ 0.5, -0.5,  0.5], color: [0.62, 0.60, 0.58] },
    // Left face
    CubeVertex { position: [-0.5, -0.5, -0.5], color: [0.62, 0.60, 0.58] },
    CubeVertex { position: [-0.5,  0.5, -0.5], color: [0.62, 0.60, 0.58] },
    CubeVertex { position: [-0.5,  0.5,  0.5], color: [0.62, 0.60, 0.58] },
    CubeVertex { position: [-0.5, -0.5,  0.5], color: [0.62, 0.60, 0.58] },
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
// Properties tab icons (#2)
// ---------------------------------------------------------------------------

const PROP_TAB_ICONS: &[&str] = &[
    "T",  // Active Tool
    "S",  // Scene
    "W",  // World
    "O",  // Object (orange)
    "M",  // Modifiers
    "P",  // Particles
    "Ph", // Physics
    "C",  // Constraints
    "D",  // Object Data
];

// ---------------------------------------------------------------------------
// Left toolbar icons (#1)
// ---------------------------------------------------------------------------

const TOOLBAR_ICONS: &[(&str, &str)] = &[
    ("W", "Cursor"),
    ("G", "Move"),
    ("R", "Rotate"),
    ("S", "Scale"),
    ("T", "Transform"),
    ("A", "Annotate"),
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
    active_prop_tab: usize,
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
            active_prop_tab: 3, // Object tab active by default
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
        self.draw_statusbar(ctx);   // #5: bottommost
        self.draw_timeline(ctx);    // #4: above statusbar
        self.draw_right_panel(ctx);
        self.draw_left_toolbar(ctx); // #1: left toolbar
        self.draw_viewport(ctx);
    }

    // -- Top bar (#10, #11) --
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

                    // #10: Menus with transparent background (use small_button style)
                    let menu_text = |t: &str| {
                        egui::RichText::new(t)
                            .color(Color32::from_rgb(200, 200, 200))
                            .size(12.0)
                    };
                    ui.menu_button(menu_text("File"), |ui| {
                        if ui.button("New").clicked() { ui.close_menu(); }
                        if ui.button("Open...").clicked() { ui.close_menu(); }
                        if ui.button("Save").clicked() { ui.close_menu(); }
                        if ui.button("Save As...").clicked() { ui.close_menu(); }
                        ui.separator();
                        if ui.button("Quit").clicked() { ui.close_menu(); }
                    });
                    ui.menu_button(menu_text("Edit"), |ui| {
                        if ui.button("Undo").clicked() { ui.close_menu(); }
                        if ui.button("Redo").clicked() { ui.close_menu(); }
                        ui.separator();
                        if ui.button("Preferences...").clicked() { ui.close_menu(); }
                    });
                    ui.menu_button(menu_text("Render"), |ui| {
                        if ui.button("Render Image").clicked() { ui.close_menu(); }
                        if ui.button("Render Animation").clicked() { ui.close_menu(); }
                    });
                    ui.menu_button(menu_text("Window"), |ui| {
                        if ui.button("New Window").clicked() { ui.close_menu(); }
                        if ui.button("Toggle Fullscreen").clicked() { ui.close_menu(); }
                    });
                    ui.menu_button(menu_text("Help"), |ui| {
                        if ui.button("About Forge3D").clicked() { ui.close_menu(); }
                    });

                    ui.separator();

                    // #11: Workspace tabs — active one has lighter bg
                    for (i, name) in WORKSPACE_TABS.iter().enumerate() {
                        let is_active = self.workspace == i;
                        let label = egui::RichText::new(*name)
                            .size(12.0)
                            .color(if is_active {
                                Color32::WHITE
                            } else {
                                Color32::from_rgb(160, 160, 160)
                            });
                        let btn = egui::Button::new(label)
                            .fill(if is_active {
                                Color32::from_rgb(58, 58, 58)
                            } else {
                                Color32::TRANSPARENT
                            })
                            .stroke(Stroke::NONE);
                        if ui.add(btn).clicked() {
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

    // -- #5: Status bar (very bottom) --
    fn draw_statusbar(&self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("statusbar")
            .exact_height(22.0)
            .frame(Frame::NONE.fill(Color32::from_rgb(42, 42, 42)))
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.label(
                        egui::RichText::new("Object Mode")
                            .color(Color32::from_rgb(180, 180, 180))
                            .size(11.0),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new("Verts:8  Faces:6  Tris:12 | Blender C++\u{2192}Rust conversion")
                                .color(Color32::from_rgb(140, 140, 140))
                                .size(11.0),
                        );
                    });
                });
            });
    }

    // -- #4: Timeline (thin, 32px) --
    fn draw_timeline(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("timeline")
            .exact_height(32.0)
            .frame(Frame::NONE.fill(Color32::from_rgb(42, 42, 42)))
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.spacing_mut().item_spacing.x = 2.0;

                    // Left: editor type label
                    ui.label(
                        egui::RichText::new("Dope Sheet")
                            .color(Color32::from_rgb(180, 180, 180))
                            .size(11.0),
                    );
                    ui.separator();

                    // Transport buttons
                    if ui.small_button("\u{23EE}").clicked() {
                        self.current_frame = self.start_frame;
                    }
                    if ui.small_button("\u{23EA}").clicked() {
                        self.current_frame = (self.current_frame - 1).max(self.start_frame);
                    }
                    let play_label = if self.playing { "\u{23F8}" } else { "\u{25B6}" };
                    if ui.small_button(play_label).clicked() {
                        self.playing = !self.playing;
                        if self.playing {
                            self.start_time = Instant::now();
                        }
                    }
                    if ui.small_button("\u{23E9}").clicked() {
                        self.current_frame = (self.current_frame + 1).min(self.end_frame);
                    }
                    if ui.small_button("\u{23ED}").clicked() {
                        self.current_frame = self.end_frame;
                    }

                    ui.separator();

                    // Frame counter
                    ui.add(egui::DragValue::new(&mut self.current_frame)
                        .range(self.start_frame..=self.end_frame)
                        .speed(0.5));

                    ui.separator();

                    ui.label(
                        egui::RichText::new(format!("{} / {}", self.start_frame, self.end_frame))
                            .color(Color32::from_rgb(140, 140, 140))
                            .size(11.0),
                    );
                });
            });
    }

    // -- #1: Left toolbar --
    fn draw_left_toolbar(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("toolbar")
            .exact_width(40.0)
            .frame(Frame::NONE.fill(Color32::from_rgb(40, 40, 40)))
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(4.0);
                    for (icon, _tooltip) in TOOLBAR_ICONS {
                        let btn = egui::Button::new(
                            egui::RichText::new(*icon)
                                .size(14.0)
                                .color(Color32::from_rgb(200, 200, 200)),
                        )
                        .min_size(egui::vec2(32.0, 28.0))
                        .fill(Color32::from_rgb(50, 50, 50))
                        .stroke(Stroke::new(0.5, Color32::from_rgb(35, 35, 35)));
                        ui.add(btn);
                        ui.add_space(1.0);
                    }
                });
            });
    }

    // -- Right panel: outliner + properties (#2, #8) --
    fn draw_right_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("right")
            .exact_width(300.0)
            .frame(Frame::NONE.fill(Color32::from_rgb(40, 40, 40)))
            .show(ctx, |ui| {
                // #8: Outliner header
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("Outliner")
                            .color(Color32::from_rgb(180, 180, 180))
                            .size(11.0),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new("View Layer")
                                .color(Color32::from_rgb(140, 140, 140))
                                .size(10.0),
                        );
                        ui.label(
                            egui::RichText::new("F")
                                .color(Color32::from_rgb(140, 140, 140))
                                .size(10.0),
                        );
                    });
                });
                ui.separator();

                // Outliner body
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

                ui.add_space(6.0);
                ui.separator();

                // #2: Properties section with vertical tab bar on the left
                ui.horizontal(|ui| {
                    // Vertical tab strip
                    ui.vertical(|ui| {
                        ui.set_width(28.0);
                        ui.spacing_mut().item_spacing.y = 1.0;
                        for (i, icon) in PROP_TAB_ICONS.iter().enumerate() {
                            let is_active = self.active_prop_tab == i;
                            let color = match i {
                                0 => Color32::WHITE,                      // T - Tool
                                1 => Color32::from_rgb(160, 160, 160),    // S - Scene
                                2 => Color32::from_rgb(180, 80, 80),      // W - World (red-ish)
                                3 => Color32::from_rgb(237, 154, 50),     // O - Object (orange)
                                4 => Color32::from_rgb(70, 130, 220),     // M - Modifiers (blue)
                                5 => Color32::from_rgb(120, 180, 220),    // P - Particles (light blue)
                                6 => Color32::from_rgb(120, 180, 220),    // Ph - Physics (light blue)
                                7 => Color32::from_rgb(200, 180, 50),     // C - Constraints (yellow)
                                8 => Color32::from_rgb(80, 200, 80),      // D - Data (green)
                                _ => Color32::from_rgb(180, 180, 180),
                            };
                            let label = egui::RichText::new(*icon)
                                .size(11.0)
                                .color(color);
                            let btn = egui::Button::new(label)
                                .min_size(egui::vec2(26.0, 22.0))
                                .fill(if is_active {
                                    Color32::from_rgb(60, 60, 60)
                                } else {
                                    Color32::TRANSPARENT
                                })
                                .stroke(Stroke::NONE);
                            if ui.add(btn).clicked() {
                                self.active_prop_tab = i;
                            }
                        }
                    });

                    ui.separator();

                    // Properties content
                    ui.vertical(|ui| {
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

                            // Collapsed sections matching Blender
                            let collapsed_sections = [
                                ("delta_transform", "Delta Transform"),
                                ("relations", "Relations"),
                                ("collections", "Collections"),
                                ("instancing", "Instancing"),
                                ("motion_paths", "Motion Paths"),
                                ("visibility", "Visibility"),
                                ("custom_properties", "Custom Properties"),
                            ];
                            for (id_str, label) in &collapsed_sections {
                                let sec_id = ui.make_persistent_id(*id_str);
                                egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), sec_id, false)
                                    .show_header(ui, |ui| {
                                        ui.label(
                                            egui::RichText::new(*label)
                                                .color(Color32::from_rgb(200, 200, 200)),
                                        );
                                    })
                                    .body(|ui| {
                                        ui.label(
                                            egui::RichText::new("(empty)")
                                                .color(Color32::from_rgb(120, 120, 120))
                                                .size(11.0),
                                        );
                                    });
                            }
                        }
                    });
                });
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

    // -- #7: Viewport header bar --
    fn draw_viewport_header(ui: &mut egui::Ui, active_shading: &mut usize) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;

            // Left: Object Mode menu
            ui.menu_button(
                egui::RichText::new("Object Mode \u{25BC}")
                    .color(Color32::from_rgb(200, 200, 200))
                    .size(11.0),
                |ui| {
                    if ui.button("Object Mode").clicked() { ui.close_menu(); }
                    if ui.button("Edit Mode").clicked() { ui.close_menu(); }
                    if ui.button("Sculpt Mode").clicked() { ui.close_menu(); }
                },
            );

            ui.separator();

            // Pivot, snap, proportional icons
            ui.label(egui::RichText::new("\u{2316}").color(Color32::from_rgb(180, 180, 180)).size(12.0)); // pivot
            ui.label(egui::RichText::new("\u{25C7}").color(Color32::from_rgb(180, 180, 180)).size(12.0)); // snap
            ui.label(egui::RichText::new("\u{25CE}").color(Color32::from_rgb(180, 180, 180)).size(12.0)); // proportional

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // X-Ray toggle
                ui.label(egui::RichText::new("X").color(Color32::from_rgb(140, 140, 140)).size(11.0));

                ui.separator();

                // Overlays menu
                ui.menu_button(
                    egui::RichText::new("Overlays \u{25BC}")
                        .color(Color32::from_rgb(180, 180, 180))
                        .size(11.0),
                    |ui| {
                        if ui.button("Show Overlays").clicked() { ui.close_menu(); }
                    },
                );

                ui.separator();

                // Shading mode circles
                let shading_icons = [
                    ("\u{25CB}", "Wireframe"),
                    ("\u{25CE}", "Solid"),
                    ("\u{25C9}", "Material"),
                    ("\u{25CF}", "Rendered"),
                ];
                for (i, (icon, _name)) in shading_icons.iter().enumerate().rev() {
                    let is_active = *active_shading == i;
                    let color = if is_active {
                        Color32::WHITE
                    } else {
                        Color32::from_rgb(140, 140, 140)
                    };
                    let btn = egui::Button::new(
                        egui::RichText::new(*icon).color(color).size(14.0),
                    )
                    .fill(Color32::TRANSPARENT)
                    .stroke(Stroke::NONE);
                    if ui.add(btn).clicked() {
                        *active_shading = i;
                    }
                }
            });
        });
    }

    // -- 3D Viewport --
    fn draw_viewport(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(Frame::NONE.fill(Color32::from_rgb(51, 51, 51)))
            .show(ctx, |ui| {
                // #7: Viewport header bar
                Self::draw_viewport_header(ui, &mut self.active_shading);
                ui.separator();

                // 3D cube render via callback
                let rect = ui.available_rect_before_wrap();
                let aspect = if rect.height() > 0.0 { rect.width() / rect.height() } else { 1.0 };

                let t = self.start_time.elapsed().as_secs_f32();
                let model = mat4_mul(&mat4_rotate_y(t * 0.7), &mat4_rotate_x(0.4));
                let view = mat4_translate(0.0, 0.0, -8.0);
                let proj = mat4_perspective(std::f32::consts::FRAC_PI_4, aspect, 0.1, 100.0);
                let mvp = mat4_mul(&proj, &mat4_mul(&view, &model));

                let cb = egui_wgpu::Callback::new_paint_callback(
                    rect,
                    CubeCallback { mvp },
                );
                ui.painter().add(cb);

                // #9: Grid overlay with perspective-like alpha fade
                let painter = ui.painter();
                let center = rect.center();
                let grid_spacing = 30.0;
                let sub_grid_spacing = 6.0;
                let max_dist = rect.width().max(rect.height()) * 0.5;

                // Sub-grid lines (very subtle)
                {
                    let mut y = center.y;
                    while y >= rect.top() {
                        let dist = (center.y - y).abs();
                        let alpha = (1.0 - (dist / max_dist).powf(0.6)).max(0.0);
                        let a = (20.0 * alpha) as u8;
                        if a > 1 {
                            let c = Color32::from_rgba_premultiplied(50, 50, 50, a);
                            painter.line_segment([egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)], Stroke::new(0.3, c));
                        }
                        y -= sub_grid_spacing;
                    }
                    y = center.y + sub_grid_spacing;
                    while y <= rect.bottom() {
                        let dist = (y - center.y).abs();
                        let alpha = (1.0 - (dist / max_dist).powf(0.6)).max(0.0);
                        let a = (20.0 * alpha) as u8;
                        if a > 1 {
                            let c = Color32::from_rgba_premultiplied(50, 50, 50, a);
                            painter.line_segment([egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)], Stroke::new(0.3, c));
                        }
                        y += sub_grid_spacing;
                    }
                    let mut x = center.x;
                    while x >= rect.left() {
                        let dist = (center.x - x).abs();
                        let alpha = (1.0 - (dist / max_dist).powf(0.6)).max(0.0);
                        let a = (20.0 * alpha) as u8;
                        if a > 1 {
                            let c = Color32::from_rgba_premultiplied(50, 50, 50, a);
                            painter.line_segment([egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())], Stroke::new(0.3, c));
                        }
                        x -= sub_grid_spacing;
                    }
                    x = center.x + sub_grid_spacing;
                    while x <= rect.right() {
                        let dist = (x - center.x).abs();
                        let alpha = (1.0 - (dist / max_dist).powf(0.6)).max(0.0);
                        let a = (20.0 * alpha) as u8;
                        if a > 1 {
                            let c = Color32::from_rgba_premultiplied(50, 50, 50, a);
                            painter.line_segment([egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())], Stroke::new(0.3, c));
                        }
                        x += sub_grid_spacing;
                    }
                }

                // Main grid lines
                let draw_grid_line = |p0: egui::Pos2, p1: egui::Pos2, dist_from_center: f32| {
                    let alpha = (1.0 - (dist_from_center / max_dist).powf(0.6)).max(0.0);
                    let a = (40.0 * alpha) as u8;
                    if a > 2 {
                        let c = Color32::from_rgba_premultiplied(60, 60, 60, a);
                        painter.line_segment([p0, p1], Stroke::new(0.5, c));
                    }
                };

                // Horizontal grid lines
                let mut y = center.y;
                let mut idx = 0.0_f32;
                while y >= rect.top() {
                    let dist = idx * grid_spacing;
                    draw_grid_line(egui::pos2(rect.left(), y), egui::pos2(rect.right(), y), dist);
                    y -= grid_spacing;
                    idx += 1.0;
                }
                y = center.y + grid_spacing;
                idx = 1.0;
                while y <= rect.bottom() {
                    let dist = idx * grid_spacing;
                    draw_grid_line(egui::pos2(rect.left(), y), egui::pos2(rect.right(), y), dist);
                    y += grid_spacing;
                    idx += 1.0;
                }

                // Vertical grid lines
                let mut x = center.x;
                idx = 0.0;
                while x >= rect.left() {
                    let dist = idx * grid_spacing;
                    draw_grid_line(egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom()), dist);
                    x -= grid_spacing;
                    idx += 1.0;
                }
                x = center.x + grid_spacing;
                idx = 1.0;
                while x <= rect.right() {
                    let dist = idx * grid_spacing;
                    draw_grid_line(egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom()), dist);
                    x += grid_spacing;
                    idx += 1.0;
                }

                // Colored axis lines: X = bright red, Y = bright green
                painter.line_segment(
                    [egui::pos2(rect.left(), center.y), egui::pos2(rect.right(), center.y)],
                    Stroke::new(1.5, Color32::from_rgb(180, 60, 60)),
                );
                painter.line_segment(
                    [egui::pos2(center.x, rect.top()), egui::pos2(center.x, rect.bottom())],
                    Stroke::new(1.5, Color32::from_rgb(60, 180, 60)),
                );

                // 3D cursor: crosshair at center
                let cursor_size = 6.0;
                painter.line_segment(
                    [egui::pos2(center.x - cursor_size, center.y), egui::pos2(center.x + cursor_size, center.y)],
                    Stroke::new(1.5, Color32::from_rgb(255, 50, 50)),
                );
                painter.line_segment(
                    [egui::pos2(center.x, center.y - cursor_size), egui::pos2(center.x, center.y + cursor_size)],
                    Stroke::new(1.5, Color32::from_rgb(255, 50, 50)),
                );
                painter.circle_stroke(center, 4.0, Stroke::new(1.0, Color32::WHITE));

                // Camera wireframe indicator
                let cam_pos = egui::pos2(center.x - 100.0, center.y - 30.0);
                let cam_color = Color32::from_rgb(100, 100, 100);
                // Small pyramid shape for camera
                painter.line_segment([egui::pos2(cam_pos.x, cam_pos.y - 6.0), egui::pos2(cam_pos.x - 8.0, cam_pos.y + 6.0)], Stroke::new(1.0, cam_color));
                painter.line_segment([egui::pos2(cam_pos.x, cam_pos.y - 6.0), egui::pos2(cam_pos.x + 8.0, cam_pos.y + 6.0)], Stroke::new(1.0, cam_color));
                painter.line_segment([egui::pos2(cam_pos.x - 8.0, cam_pos.y + 6.0), egui::pos2(cam_pos.x + 8.0, cam_pos.y + 6.0)], Stroke::new(1.0, cam_color));
                // Small triangle on top (viewfinder)
                painter.line_segment([egui::pos2(cam_pos.x - 3.0, cam_pos.y - 6.0), egui::pos2(cam_pos.x, cam_pos.y - 10.0)], Stroke::new(1.0, cam_color));
                painter.line_segment([egui::pos2(cam_pos.x + 3.0, cam_pos.y - 6.0), egui::pos2(cam_pos.x, cam_pos.y - 10.0)], Stroke::new(1.0, cam_color));

                // Light indicator (dot/circle)
                let light_pos = egui::pos2(center.x + 50.0, center.y - 80.0);
                painter.circle_filled(light_pos, 3.0, Color32::from_rgb(200, 180, 60));
                painter.circle_stroke(light_pos, 6.0, Stroke::new(0.8, Color32::from_rgb(200, 180, 60)));
                // Small rays
                for angle_deg in [0.0_f32, 45.0, 90.0, 135.0, 180.0, 225.0, 270.0, 315.0] {
                    let angle = angle_deg.to_radians();
                    let inner = 8.0;
                    let outer = 11.0;
                    painter.line_segment(
                        [
                            egui::pos2(light_pos.x + angle.cos() * inner, light_pos.y + angle.sin() * inner),
                            egui::pos2(light_pos.x + angle.cos() * outer, light_pos.y + angle.sin() * outer),
                        ],
                        Stroke::new(0.6, Color32::from_rgb(200, 180, 60)),
                    );
                }

                // "User Perspective" text in top-left of viewport
                painter.text(
                    egui::pos2(rect.left() + 10.0, rect.top() + 10.0),
                    egui::Align2::LEFT_TOP,
                    "User Perspective",
                    egui::FontId::proportional(11.0),
                    Color32::from_rgb(140, 140, 140),
                );
                painter.text(
                    egui::pos2(rect.left() + 10.0, rect.top() + 24.0),
                    egui::Align2::LEFT_TOP,
                    "Collection | Camera 1.0m",
                    egui::FontId::proportional(10.0),
                    Color32::from_rgb(110, 110, 110),
                );

                // #6: XYZ gizmo in TOP-RIGHT corner
                let gizmo_origin = egui::pos2(rect.right() - 50.0, rect.top() + 50.0);
                let gizmo_len = 30.0;
                // X axis
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
                // Y axis (up)
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
                // Z axis (diagonal)
                painter.line_segment(
                    [gizmo_origin, egui::pos2(gizmo_origin.x - gizmo_len * 0.5, gizmo_origin.y + gizmo_len * 0.5)],
                    Stroke::new(2.0, Color32::from_rgb(60, 100, 200)),
                );
                painter.text(
                    egui::pos2(gizmo_origin.x - gizmo_len * 0.5 - 10.0, gizmo_origin.y + gizmo_len * 0.5),
                    egui::Align2::RIGHT_CENTER,
                    "Z",
                    egui::FontId::proportional(10.0),
                    Color32::from_rgb(60, 100, 200),
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
