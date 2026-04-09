//! Forge3D — Blender-like interface using custom `blender_draw` module + egui text.
//!
//! Architecture:
//!   1. Backgrounds, panels, buttons, separators → blender_draw DrawList → 2D shader
//!   2. Text labels → egui with Frame::none() (no backgrounds)
//!   3. 3D viewport → spinning cube via egui_wgpu callback
//!
//! All dimensions from Blender DNA_screen_types.h:
//!   HEADERY=26, UI_UNIT_Y=20, UI_UNIT_X=20, ICON_DEFAULT=16, corner_radius=3

use std::sync::Arc;
use std::time::Instant;

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};

use forge3d_ui_core::blender_draw::{
    draw_button, draw_menu_button, draw_number_field, draw_panel_background,
    draw_panel_header, draw_separator, DrawList, DrawVertex, WidgetState,
};
use forge3d_ui_core::blender_draw::theme::{
    general, outliner, properties, timeline, view3d, wcol, PanelColors,
};
use forge3d_ui_core::painter::Rect;

// ---------------------------------------------------------------------------
// Blender exact dimensions
// ---------------------------------------------------------------------------

const HEADERY: f32 = 26.0;
const UI_UNIT_Y: f32 = 20.0;
const UI_UNIT_X: f32 = 20.0;
const ICON_DEFAULT: f32 = 16.0;
const RIGHT_PANEL_W: f32 = 320.0;
const TIMELINE_H: f32 = 100.0;
const OL_INDENT: f32 = 1.8 * UI_UNIT_X;

// ---------------------------------------------------------------------------
// 3D Cube Vertex (for spinning cube)
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
    CubeVertex { position: [-0.5, -0.5,  0.5], color: [1.0, 0.2, 0.2] },
    CubeVertex { position: [ 0.5, -0.5,  0.5], color: [1.0, 0.4, 0.2] },
    CubeVertex { position: [ 0.5,  0.5,  0.5], color: [1.0, 0.6, 0.4] },
    CubeVertex { position: [-0.5,  0.5,  0.5], color: [1.0, 0.3, 0.3] },
    CubeVertex { position: [-0.5, -0.5, -0.5], color: [0.2, 0.2, 1.0] },
    CubeVertex { position: [ 0.5, -0.5, -0.5], color: [0.2, 0.4, 1.0] },
    CubeVertex { position: [ 0.5,  0.5, -0.5], color: [0.4, 0.6, 1.0] },
    CubeVertex { position: [-0.5,  0.5, -0.5], color: [0.3, 0.3, 1.0] },
    CubeVertex { position: [-0.5,  0.5, -0.5], color: [0.2, 1.0, 0.2] },
    CubeVertex { position: [ 0.5,  0.5, -0.5], color: [0.2, 1.0, 0.4] },
    CubeVertex { position: [ 0.5,  0.5,  0.5], color: [0.4, 1.0, 0.4] },
    CubeVertex { position: [-0.5,  0.5,  0.5], color: [0.3, 1.0, 0.3] },
    CubeVertex { position: [-0.5, -0.5, -0.5], color: [1.0, 1.0, 0.2] },
    CubeVertex { position: [ 0.5, -0.5, -0.5], color: [1.0, 1.0, 0.4] },
    CubeVertex { position: [ 0.5, -0.5,  0.5], color: [1.0, 1.0, 0.4] },
    CubeVertex { position: [-0.5, -0.5,  0.5], color: [1.0, 1.0, 0.2] },
    CubeVertex { position: [ 0.5, -0.5, -0.5], color: [0.2, 1.0, 1.0] },
    CubeVertex { position: [ 0.5,  0.5, -0.5], color: [0.4, 1.0, 1.0] },
    CubeVertex { position: [ 0.5,  0.5,  0.5], color: [0.4, 1.0, 1.0] },
    CubeVertex { position: [ 0.5, -0.5,  0.5], color: [0.2, 1.0, 1.0] },
    CubeVertex { position: [-0.5, -0.5, -0.5], color: [1.0, 0.2, 1.0] },
    CubeVertex { position: [-0.5,  0.5, -0.5], color: [1.0, 0.4, 1.0] },
    CubeVertex { position: [-0.5,  0.5,  0.5], color: [1.0, 0.4, 1.0] },
    CubeVertex { position: [-0.5, -0.5,  0.5], color: [1.0, 0.2, 1.0] },
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
// 2D UI Shader (WGSL) — draws DrawVertex with pixel coords + u8 colors
// ---------------------------------------------------------------------------

const UI2D_SHADER_SRC: &str = r#"
struct Uniforms {
    screen_size: vec2<f32>,
    _pad: vec2<f32>,
};
@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) pos: vec2<f32>,
    @location(1) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let clip_x = in.pos.x / uniforms.screen_size.x * 2.0 - 1.0;
    let clip_y = 1.0 - in.pos.y / uniforms.screen_size.y * 2.0;
    out.clip_position = vec4<f32>(clip_x, clip_y, 0.0, 1.0);
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
"#;

// ---------------------------------------------------------------------------
// GPU vertex for 2D UI (matches DrawVertex layout but with Pod/Zeroable)
// ---------------------------------------------------------------------------

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct UiVertex {
    pos: [f32; 2],
    color: [u8; 4],
}

impl UiVertex {
    fn from_draw_vertex(dv: &DrawVertex) -> Self {
        Self {
            pos: dv.pos,
            color: dv.color,
        }
    }
}

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
    selectable: bool,
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
            selectable: true,
        }
    }
}

// ---------------------------------------------------------------------------
// Workspace tabs
// ---------------------------------------------------------------------------

const WORKSPACE_TABS: &[&str] = &[
    "Layout", "Modeling", "Sculpting", "UV Editing",
    "Texture Paint", "Shading", "Animation", "Rendering",
    "Compositing", "Geometry Nodes",
];

const PROP_TABS: &[(&str, &str)] = &[
    ("\u{1F527}", "Active Tool"),
    ("\u{1F3AC}", "Scene"),
    ("\u{1F30D}", "World"),
    ("\u{25A0}",  "Object"),
    ("\u{2699}",  "Modifiers"),
    ("\u{2728}",  "Particles"),
    ("\u{2301}",  "Physics"),
    ("\u{1F517}", "Constraints"),
    ("\u{25B3}",  "Object Data"),
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
// 2D UI GPU resources
// ---------------------------------------------------------------------------

struct Ui2dResources {
    tri_pipeline: wgpu::RenderPipeline,
    line_pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl Ui2dResources {
    fn create(device: &wgpu::Device, target_format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ui2d-shader"),
            source: wgpu::ShaderSource::Wgsl(UI2D_SHADER_SRC.into()),
        });

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("ui2d-uniform"),
            contents: bytemuck::cast_slice(&[1920.0f32, 1080.0, 0.0, 0.0]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ui2d-bgl"),
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
            label: Some("ui2d-bg"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ui2d-pipeline-layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // Vertex buffer layout: pos(f32x2) + color(u8x4 normalized)
        let vertex_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<UiVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 8,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Unorm8x4,
                },
            ],
        };

        let make_pipeline = |topology: wgpu::PrimitiveTopology, label: &str| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    compilation_options: Default::default(),
                    buffers: &[vertex_layout.clone()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: target_format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            })
        };

        let tri_pipeline = make_pipeline(wgpu::PrimitiveTopology::TriangleList, "ui2d-tri");
        let line_pipeline = make_pipeline(wgpu::PrimitiveTopology::LineList, "ui2d-line");

        Self {
            tri_pipeline,
            line_pipeline,
            uniform_buffer,
            bind_group,
        }
    }
}

// ---------------------------------------------------------------------------
// Application state
// ---------------------------------------------------------------------------

struct App {
    window: Option<Arc<Window>>,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    surface: Option<wgpu::Surface<'static>>,
    surface_config: Option<wgpu::SurfaceConfiguration>,
    surface_format: wgpu::TextureFormat,

    egui_ctx: egui::Context,
    egui_winit: Option<egui_winit::State>,
    egui_renderer: Option<egui_wgpu::Renderer>,

    ui2d: Option<Ui2dResources>,

    objects: Vec<SceneObject>,
    selected_index: Option<usize>,

    current_frame: i32,
    start_frame: i32,
    end_frame: i32,
    playing: bool,
    last_frame_time: Instant,
    #[allow(dead_code)]
    auto_keying: bool,

    start_time: Instant,

    active_workspace: usize,
    active_prop_tab: usize,
    active_shading: usize,
    overlays_on: bool,
    xray_on: bool,

    transform_open: bool,
    relations_open: bool,
    collections_open: bool,
    scene_collection_open: bool,
}

impl App {
    fn new() -> Self {
        let objects = vec![
            SceneObject::new("Cube",   "Mesh",   [0.0, 0.0, 0.0],  [0.0, 0.0, 0.0],     [1.0, 1.0, 1.0]),
            SceneObject::new("Camera", "Camera", [7.36, -6.93, 4.96], [63.6, 0.0, 46.7], [1.0, 1.0, 1.0]),
            SceneObject::new("Light",  "Light",  [4.08, 1.0, 5.90],  [37.3, 3.16, 107.0], [1.0, 1.0, 1.0]),
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

            ui2d: None,

            objects,
            selected_index: Some(0),

            current_frame: 1,
            start_frame: 1,
            end_frame: 250,
            playing: false,
            last_frame_time: Instant::now(),
            auto_keying: false,

            start_time: Instant::now(),

            active_workspace: 0,
            active_prop_tab: 3,
            active_shading: 1,
            overlays_on: true,
            xray_on: false,

            transform_open: true,
            relations_open: false,
            collections_open: false,
            scene_collection_open: true,
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
        let format = caps.formats.iter().find(|f| f.is_srgb()).copied().unwrap_or(caps.formats[0]);

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

        self.surface_format = format;

        let egui_renderer = egui_wgpu::Renderer::new(&device, format, None, 1, true);
        let egui_winit = egui_winit::State::new(
            self.egui_ctx.clone(),
            egui::ViewportId::ROOT,
            &window,
            Some(window.scale_factor() as f32),
            None,
            Some(device.limits().max_texture_dimension_2d as usize),
        );

        let cube_resources = CubeResources::create(&device, format);
        let ui2d_resources = Ui2dResources::create(&device, format);

        self.window = Some(window);
        self.ui2d = Some(ui2d_resources);
        self.device = Some(device);
        self.queue = Some(queue);
        self.surface = Some(surface);
        self.surface_config = Some(config);
        self.egui_winit = Some(egui_winit);
        self.egui_renderer = Some(egui_renderer);

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

    // -----------------------------------------------------------------------
    // Build the 2D UI DrawList using blender_draw primitives
    // -----------------------------------------------------------------------

    fn build_ui_draw_list(&self, w: f32, h: f32) -> DrawList {
        let mut dl = DrawList::new();
        let _panel_colors = PanelColors::default();

        // Layout regions
        let vp_right = w - RIGHT_PANEL_W;
        let ol_split_y = HEADERY + (h - TIMELINE_H - HEADERY) * 0.4;

        // 1. Top bar background
        dl.add_rect_filled(Rect::new(0.0, 0.0, w, HEADERY), general::HEADER);
        dl.append(&draw_separator(0.0, w, HEADERY));

        // 2. Viewport header (below top bar, left of right panel)
        dl.add_rect_filled(Rect::new(0.0, HEADERY, vp_right, HEADERY), view3d::HEADER);
        dl.append(&draw_separator(0.0, vp_right, HEADERY * 2.0));

        // 3. Top bar menu buttons: File, Edit, Render, Window, Help
        {
            let menu_labels = ["File", "Edit", "Render", "Window", "Help"];
            let mut bx = 28.0; // after logo area
            for label in &menu_labels {
                let bw = label.len() as f32 * 7.0 + 12.0;
                dl.append(&draw_button(
                    Rect::new(bx, 2.0, bw, HEADERY - 4.0),
                    label,
                    WidgetState::Normal,
                    &wcol::PULLDOWN,
                ));
                bx += bw + 2.0;
            }

            // Workspace tabs
            let mut tx = bx + 20.0;
            for (i, tab) in WORKSPACE_TABS.iter().enumerate() {
                let tw = tab.len() as f32 * 7.0 + 12.0;
                let state = if i == self.active_workspace {
                    WidgetState::Active
                } else {
                    WidgetState::Normal
                };
                dl.append(&draw_button(
                    Rect::new(tx, 2.0, tw, HEADERY - 4.0),
                    tab,
                    state,
                    &wcol::TAB,
                ));
                tx += tw + 2.0;
            }
        }

        // 4. Viewport header buttons
        {
            // "Object Mode" button
            dl.append(&draw_menu_button(
                Rect::new(4.0, HEADERY + 2.0, 90.0, HEADERY - 4.0),
                "Object Mode",
                &wcol::MENU,
            ));

            // Shading mode buttons (right side of viewport header)
            let shading_labels = ["Wire", "Solid", "Mat", "Rend"];
            let mut sx = vp_right - 4.0 * 50.0 - 20.0;
            for (i, label) in shading_labels.iter().enumerate() {
                let state = if i == self.active_shading {
                    WidgetState::Active
                } else {
                    WidgetState::Normal
                };
                dl.append(&draw_button(
                    Rect::new(sx, HEADERY + 2.0, 48.0, HEADERY - 4.0),
                    label,
                    state,
                    &wcol::RADIO,
                ));
                sx += 50.0;
            }
        }

        // 5. Outliner background (right panel, top 40%)
        dl.add_rect_filled(
            Rect::new(vp_right, HEADERY, RIGHT_PANEL_W, ol_split_y - HEADERY),
            outliner::BACK,
        );

        // Outliner header
        dl.append(&draw_panel_header(
            Rect::new(vp_right, HEADERY, RIGHT_PANEL_W, HEADERY),
            "Outliner",
            false,
            &PanelColors {
                header_back: outliner::HEADER,
                body_back: outliner::BACK,
                text: general::TITLE,
                triangle: [200, 200, 200, 255],
            },
        ));

        // "Display Mode: View Layer" row
        dl.add_rect_filled(
            Rect::new(vp_right, HEADERY * 2.0, RIGHT_PANEL_W, UI_UNIT_Y),
            [46, 46, 46, 255],
        );

        // Outliner rows
        if self.scene_collection_open {
            let row_start_y = HEADERY * 2.0 + UI_UNIT_Y + UI_UNIT_Y; // after header + mode row + scene collection row
            // Scene collection row
            dl.add_rect_filled(
                Rect::new(vp_right, HEADERY * 2.0 + UI_UNIT_Y, RIGHT_PANEL_W, UI_UNIT_Y),
                outliner::BACK,
            );

            for (i, obj) in self.objects.iter().enumerate() {
                let ry = row_start_y + i as f32 * UI_UNIT_Y;
                if ry + UI_UNIT_Y > ol_split_y {
                    break;
                }
                let is_selected = self.selected_index == Some(i);
                let row_color = if is_selected {
                    outliner::ACTIVE
                } else if i % 2 == 1 {
                    outliner::ROW_ALT
                } else {
                    outliner::BACK
                };
                dl.add_rect_filled(
                    Rect::new(vp_right, ry, RIGHT_PANEL_W, UI_UNIT_Y),
                    row_color,
                );
                // Selection outline
                if is_selected {
                    dl.add_line(
                        vp_right, ry,
                        vp_right + RIGHT_PANEL_W, ry,
                        outliner::SELECTED,
                    );
                    dl.add_line(
                        vp_right, ry + UI_UNIT_Y,
                        vp_right + RIGHT_PANEL_W, ry + UI_UNIT_Y,
                        outliner::SELECTED,
                    );
                }
                let _ = obj; // text rendered by egui
            }
        }

        // Separator between outliner and properties
        dl.append(&draw_separator(vp_right, w, ol_split_y));

        // 6. Properties background
        let props_y = ol_split_y;
        let props_h = h - TIMELINE_H - props_y;
        dl.add_rect_filled(
            Rect::new(vp_right, props_y, RIGHT_PANEL_W, props_h),
            properties::BACK,
        );

        // Properties header
        dl.append(&draw_panel_header(
            Rect::new(vp_right, props_y, RIGHT_PANEL_W, HEADERY),
            "Properties",
            false,
            &PanelColors {
                header_back: properties::HEADER,
                body_back: properties::BACK,
                text: general::TITLE,
                triangle: [200, 200, 200, 255],
            },
        ));

        // Property tab bar (vertical, 22px wide)
        let tab_bar_x = vp_right;
        let tab_bar_y = props_y + HEADERY;
        for (i, (_icon, _name)) in PROP_TABS.iter().enumerate() {
            let ty = tab_bar_y + 2.0 + i as f32 * (UI_UNIT_Y + 2.0);
            let state = if i == self.active_prop_tab {
                WidgetState::Active
            } else {
                WidgetState::Normal
            };
            dl.append(&draw_button(
                Rect::new(tab_bar_x + 2.0, ty, UI_UNIT_X, UI_UNIT_Y),
                "",
                state,
                &wcol::TAB,
            ));
        }

        // Vertical separator after tab bar
        dl.add_line(
            tab_bar_x + UI_UNIT_X + 4.0, props_y + HEADERY,
            tab_bar_x + UI_UNIT_X + 4.0, props_y + props_h,
            general::SEPARATOR,
        );

        // Property panels (Transform, Relations, Collections)
        let content_x = tab_bar_x + UI_UNIT_X + 6.0;
        let content_w = RIGHT_PANEL_W - UI_UNIT_X - 8.0;
        let mut py = props_y + HEADERY + 4.0;

        // Object name row
        dl.add_rect_filled(
            Rect::new(content_x, py, content_w, UI_UNIT_Y + 4.0),
            properties::PANEL,
        );
        py += UI_UNIT_Y + 6.0;

        // Transform panel header
        let transform_panel_colors = PanelColors {
            header_back: [55, 55, 55, 255],
            body_back: properties::PANEL,
            text: general::TITLE,
            triangle: [200, 200, 200, 255],
        };
        dl.append(&draw_panel_header(
            Rect::new(content_x, py, content_w, HEADERY),
            "Transform",
            !self.transform_open,
            &transform_panel_colors,
        ));
        py += HEADERY;

        if self.transform_open {
            // Transform body background
            let transform_body_h = 3.0 * (UI_UNIT_Y + 4.0) + 8.0;
            dl.append(&draw_panel_background(
                Rect::new(content_x, py, content_w, transform_body_h),
                &transform_panel_colors,
            ));

            // Number fields for Location, Rotation, Scale (3 rows x 3 fields)
            let field_labels = ["Location", "Rotation", "Scale"];
            for (row, _label) in field_labels.iter().enumerate() {
                let fy = py + 4.0 + row as f32 * (UI_UNIT_Y + 4.0);
                let field_x = content_x + 70.0;
                let field_w = (content_w - 74.0) / 3.0 - 2.0;
                for col in 0..3 {
                    let fx = field_x + col as f32 * (field_w + 2.0);
                    dl.append(&draw_number_field(
                        Rect::new(fx, fy, field_w, UI_UNIT_Y),
                        0.0,
                        "",
                        &wcol::NUM,
                    ));
                }
            }
            py += transform_body_h;
        }

        // Relations panel header
        dl.append(&draw_panel_header(
            Rect::new(content_x, py, content_w, HEADERY),
            "Relations",
            !self.relations_open,
            &transform_panel_colors,
        ));
        py += HEADERY;

        if self.relations_open {
            let body_h = UI_UNIT_Y + 8.0;
            dl.append(&draw_panel_background(
                Rect::new(content_x, py, content_w, body_h),
                &transform_panel_colors,
            ));
            py += body_h;
        }

        // Collections panel header
        dl.append(&draw_panel_header(
            Rect::new(content_x, py, content_w, HEADERY),
            "Collections",
            !self.collections_open,
            &transform_panel_colors,
        ));

        // 7. Timeline background
        dl.add_rect_filled(
            Rect::new(0.0, h - TIMELINE_H, w, TIMELINE_H),
            timeline::BACK,
        );

        // Timeline header
        dl.append(&draw_panel_header(
            Rect::new(0.0, h - TIMELINE_H, w, HEADERY),
            "Timeline",
            false,
            &PanelColors {
                header_back: timeline::HEADER,
                body_back: timeline::BACK,
                text: general::TITLE,
                triangle: [200, 200, 200, 255],
            },
        ));

        // Scrub bar background
        dl.add_rect_filled(
            Rect::new(0.0, h - TIMELINE_H + HEADERY, w, UI_UNIT_Y),
            timeline::SCRUB_BACK,
        );

        // Current frame indicator line
        {
            let total = (self.end_frame - self.start_frame).max(1) as f32;
            let frac = (self.current_frame - self.start_frame) as f32 / total;
            let x = frac * w;
            let scrub_y = h - TIMELINE_H + HEADERY;
            dl.add_line(x, scrub_y, x, scrub_y + UI_UNIT_Y, timeline::FRAME_CURRENT);
        }

        // Transport buttons row
        {
            let transport_y = h - TIMELINE_H + HEADERY + UI_UNIT_Y + 4.0;
            let btn_labels = ["|<<", "<", ">", ">>", ">>|"];
            let mut bx = 4.0;
            for label in &btn_labels {
                dl.append(&draw_button(
                    Rect::new(bx, transport_y, 28.0, UI_UNIT_Y),
                    label,
                    WidgetState::Normal,
                    &wcol::REGULAR,
                ));
                bx += 30.0;
            }

            // Frame number field
            dl.append(&draw_number_field(
                Rect::new(bx + 40.0, transport_y, 60.0, UI_UNIT_Y),
                self.current_frame as f32,
                "Frame",
                &wcol::NUM,
            ));

            // Start/End fields
            dl.append(&draw_number_field(
                Rect::new(bx + 140.0, transport_y, 50.0, UI_UNIT_Y),
                self.start_frame as f32,
                "Start",
                &wcol::NUM,
            ));
            dl.append(&draw_number_field(
                Rect::new(bx + 220.0, transport_y, 50.0, UI_UNIT_Y),
                self.end_frame as f32,
                "End",
                &wcol::NUM,
            ));
        }

        // Separator between viewport and timeline
        dl.append(&draw_separator(0.0, w, h - TIMELINE_H));

        dl
    }

    // -----------------------------------------------------------------------
    // Build egui text-only overlay (no backgrounds)
    // -----------------------------------------------------------------------

    fn draw_egui_text_overlay(&mut self, ctx: &egui::Context, w: f32, h: f32) {
        // Make egui fully transparent — text only
        let mut visuals = egui::Visuals::dark();
        visuals.window_fill = egui::Color32::TRANSPARENT;
        visuals.panel_fill = egui::Color32::TRANSPARENT;
        visuals.override_text_color = Some(egui::Color32::from_rgb(230, 230, 230));
        visuals.widgets.noninteractive.bg_fill = egui::Color32::TRANSPARENT;
        visuals.widgets.noninteractive.weak_bg_fill = egui::Color32::TRANSPARENT;
        visuals.widgets.noninteractive.bg_stroke = egui::Stroke::NONE;
        visuals.widgets.inactive.bg_fill = egui::Color32::TRANSPARENT;
        visuals.widgets.inactive.weak_bg_fill = egui::Color32::TRANSPARENT;
        visuals.widgets.inactive.bg_stroke = egui::Stroke::NONE;
        visuals.widgets.hovered.bg_fill = egui::Color32::TRANSPARENT;
        visuals.widgets.hovered.weak_bg_fill = egui::Color32::TRANSPARENT;
        visuals.widgets.active.bg_fill = egui::Color32::TRANSPARENT;
        visuals.widgets.active.weak_bg_fill = egui::Color32::TRANSPARENT;
        visuals.window_shadow = egui::Shadow::NONE;
        visuals.window_stroke = egui::Stroke::NONE;
        ctx.set_visuals(visuals);

        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(4.0, 2.0);
        style.spacing.button_padding = egui::vec2(4.0, 1.0);
        ctx.set_style(style);

        let vp_right = w - RIGHT_PANEL_W;
        let ol_split_y = HEADERY + (h - TIMELINE_H - HEADERY) * 0.4;

        let text_color = egui::Color32::from_rgb(230, 230, 230);
        let title_color = egui::Color32::from_rgb(238, 238, 238);
        let dim_color = egui::Color32::from_rgb(200, 200, 200);

        // Use egui::Area for text labels at exact pixel positions
        let painter = ctx.layer_painter(egui::LayerId::new(
            egui::Order::Foreground,
            egui::Id::new("ui_text"),
        ));

        // -- Top bar menu labels --
        let menu_labels = ["File", "Edit", "Render", "Window", "Help"];
        let mut bx = 34.0;
        for label in &menu_labels {
            let bw = label.len() as f32 * 7.0 + 12.0;
            painter.text(
                egui::pos2(bx + bw * 0.5, HEADERY * 0.5),
                egui::Align2::CENTER_CENTER,
                *label,
                egui::FontId::proportional(11.0),
                text_color,
            );
            bx += bw + 2.0;
        }

        // Logo icon
        painter.text(
            egui::pos2(14.0, HEADERY * 0.5),
            egui::Align2::CENTER_CENTER,
            "\u{2B22}",
            egui::FontId::proportional(14.0),
            egui::Color32::from_rgb(255, 160, 40),
        );

        // -- Workspace tab labels --
        let mut tx = bx + 20.0;
        for (i, tab) in WORKSPACE_TABS.iter().enumerate() {
            let tw = tab.len() as f32 * 7.0 + 12.0;
            let color = if i == self.active_workspace { egui::Color32::WHITE } else { text_color };
            painter.text(
                egui::pos2(tx + tw * 0.5, HEADERY * 0.5),
                egui::Align2::CENTER_CENTER,
                *tab,
                egui::FontId::proportional(11.0),
                color,
            );
            tx += tw + 2.0;
        }

        // Right side: Scene / ViewLayer
        painter.text(
            egui::pos2(w - 80.0, HEADERY * 0.5),
            egui::Align2::RIGHT_CENTER,
            "Scene  |  ViewLayer",
            egui::FontId::proportional(10.0),
            text_color,
        );

        // -- Viewport header text --
        painter.text(
            egui::pos2(50.0, HEADERY + HEADERY * 0.5),
            egui::Align2::CENTER_CENTER,
            "Object Mode",
            egui::FontId::proportional(11.0),
            text_color,
        );

        painter.text(
            egui::pos2(130.0, HEADERY + HEADERY * 0.5),
            egui::Align2::CENTER_CENTER,
            "Global",
            egui::FontId::proportional(11.0),
            text_color,
        );

        painter.text(
            egui::pos2(210.0, HEADERY + HEADERY * 0.5),
            egui::Align2::CENTER_CENTER,
            "Median Point",
            egui::FontId::proportional(11.0),
            text_color,
        );

        // Shading labels
        let shading_labels = ["Wire", "Solid", "Mat", "Rend"];
        let mut sx = vp_right - 4.0 * 50.0 - 20.0;
        for (i, label) in shading_labels.iter().enumerate() {
            let color = if i == self.active_shading { egui::Color32::WHITE } else { text_color };
            painter.text(
                egui::pos2(sx + 24.0, HEADERY + HEADERY * 0.5),
                egui::Align2::CENTER_CENTER,
                *label,
                egui::FontId::proportional(10.0),
                color,
            );
            sx += 50.0;
        }

        // Overlays / X-Ray text
        painter.text(
            egui::pos2(vp_right - 90.0, HEADERY + HEADERY * 0.5),
            egui::Align2::CENTER_CENTER,
            "Overlays",
            egui::FontId::proportional(10.0),
            if self.overlays_on { egui::Color32::WHITE } else { text_color },
        );
        painter.text(
            egui::pos2(vp_right - 30.0, HEADERY + HEADERY * 0.5),
            egui::Align2::CENTER_CENTER,
            "X-Ray",
            egui::FontId::proportional(10.0),
            if self.xray_on { egui::Color32::from_rgb(255, 160, 40) } else { text_color },
        );

        // -- Outliner text --
        painter.text(
            egui::pos2(vp_right + 28.0, HEADERY + HEADERY * 0.5),
            egui::Align2::LEFT_CENTER,
            "Outliner",
            egui::FontId::proportional(11.0),
            title_color,
        );

        // Display Mode text
        painter.text(
            egui::pos2(vp_right + 8.0, HEADERY * 2.0 + UI_UNIT_Y * 0.5),
            egui::Align2::LEFT_CENTER,
            "Display Mode: View Layer",
            egui::FontId::proportional(10.0),
            text_color,
        );

        // Scene Collection label
        painter.text(
            egui::pos2(vp_right + 40.0, HEADERY * 2.0 + UI_UNIT_Y + UI_UNIT_Y * 0.5),
            egui::Align2::LEFT_CENTER,
            "\u{25BC} \u{1F4C1} Scene Collection",
            egui::FontId::proportional(12.0),
            text_color,
        );

        // Object rows
        if self.scene_collection_open {
            let row_start_y = HEADERY * 2.0 + UI_UNIT_Y + UI_UNIT_Y;
            for (i, obj) in self.objects.iter().enumerate() {
                let ry = row_start_y + i as f32 * UI_UNIT_Y;
                if ry + UI_UNIT_Y > ol_split_y {
                    break;
                }
                let is_selected = self.selected_index == Some(i);
                let icon = match obj.obj_type {
                    "Mesh"   => "\u{25B3}",
                    "Camera" => "\u{1F3A5}",
                    "Light"  => "\u{2299}",
                    _        => "\u{25CB}",
                };
                let icon_color = match obj.obj_type {
                    "Mesh"   => egui::Color32::from_rgb(76, 175, 80),
                    "Camera" => egui::Color32::from_rgb(66, 165, 245),
                    "Light"  => egui::Color32::from_rgb(255, 202, 40),
                    _        => text_color,
                };

                let text_x = vp_right + OL_INDENT;
                painter.text(
                    egui::pos2(text_x, ry + UI_UNIT_Y * 0.5),
                    egui::Align2::LEFT_CENTER,
                    icon,
                    egui::FontId::proportional(ICON_DEFAULT - 4.0),
                    icon_color,
                );
                painter.text(
                    egui::pos2(text_x + ICON_DEFAULT + 2.0, ry + UI_UNIT_Y * 0.5),
                    egui::Align2::LEFT_CENTER,
                    &obj.name,
                    egui::FontId::proportional(12.0),
                    if is_selected { egui::Color32::WHITE } else { text_color },
                );

                // Visibility icons
                let eye = if obj.visible { "\u{1F441}" } else { "\u{2014}" };
                painter.text(
                    egui::pos2(w - 52.0, ry + UI_UNIT_Y * 0.5),
                    egui::Align2::CENTER_CENTER,
                    eye,
                    egui::FontId::proportional(11.0),
                    dim_color,
                );
                let cam = if obj.renderable { "\u{1F4F7}" } else { "\u{2014}" };
                painter.text(
                    egui::pos2(w - 32.0, ry + UI_UNIT_Y * 0.5),
                    egui::Align2::CENTER_CENTER,
                    cam,
                    egui::FontId::proportional(10.0),
                    dim_color,
                );
                let sel = if obj.selectable { "\u{2AFD}" } else { "\u{2014}" };
                painter.text(
                    egui::pos2(w - 12.0, ry + UI_UNIT_Y * 0.5),
                    egui::Align2::CENTER_CENTER,
                    sel,
                    egui::FontId::proportional(10.0),
                    dim_color,
                );
            }
        }

        // -- Properties text --
        let props_y = ol_split_y;
        painter.text(
            egui::pos2(vp_right + 28.0, props_y + HEADERY * 0.5),
            egui::Align2::LEFT_CENTER,
            "Properties",
            egui::FontId::proportional(11.0),
            title_color,
        );

        // Property tab icons
        let tab_bar_y = props_y + HEADERY;
        for (i, (icon, _name)) in PROP_TABS.iter().enumerate() {
            let ty = tab_bar_y + 2.0 + i as f32 * (UI_UNIT_Y + 2.0);
            let color = if i == self.active_prop_tab { egui::Color32::WHITE } else { text_color };
            painter.text(
                egui::pos2(vp_right + 2.0 + UI_UNIT_X * 0.5, ty + UI_UNIT_Y * 0.5),
                egui::Align2::CENTER_CENTER,
                *icon,
                egui::FontId::proportional(12.0),
                color,
            );
        }

        // Property content text
        let content_x = vp_right + UI_UNIT_X + 6.0;
        let content_w = RIGHT_PANEL_W - UI_UNIT_X - 8.0;
        let mut py = props_y + HEADERY + 4.0;

        if let Some(idx) = self.selected_index {
            let obj = &self.objects[idx];
            let type_icon = match obj.obj_type {
                "Mesh"   => "\u{25B3}",
                "Camera" => "\u{1F3A5}",
                "Light"  => "\u{2299}",
                _        => "\u{25CB}",
            };
            painter.text(
                egui::pos2(content_x + 4.0, py + (UI_UNIT_Y + 4.0) * 0.5),
                egui::Align2::LEFT_CENTER,
                &format!("{} {}", type_icon, obj.name),
                egui::FontId::proportional(12.0),
                egui::Color32::from_rgb(237, 87, 0),
            );
        }
        py += UI_UNIT_Y + 6.0;

        // Transform header text
        let tri = if self.transform_open { "\u{25BC}" } else { "\u{25B6}" };
        painter.text(
            egui::pos2(content_x + 22.0, py + HEADERY * 0.5),
            egui::Align2::LEFT_CENTER,
            &format!("{} Transform", tri),
            egui::FontId::proportional(12.0),
            title_color,
        );
        py += HEADERY;

        if self.transform_open {
            if let Some(idx) = self.selected_index {
                let obj = &self.objects[idx];
                let field_labels = [
                    ("Location", obj.location),
                    ("Rotation", obj.rotation),
                    ("Scale",    obj.scale),
                ];
                let axis_colors = [
                    egui::Color32::from_rgb(214, 67, 67),
                    egui::Color32::from_rgb(104, 188, 80),
                    egui::Color32::from_rgb(67, 133, 214),
                ];
                let axis_names = ["X", "Y", "Z"];

                for (row, (label, vals)) in field_labels.iter().enumerate() {
                    let fy = py + 4.0 + row as f32 * (UI_UNIT_Y + 4.0);
                    painter.text(
                        egui::pos2(content_x + 4.0, fy + UI_UNIT_Y * 0.5),
                        egui::Align2::LEFT_CENTER,
                        *label,
                        egui::FontId::proportional(11.0),
                        text_color,
                    );

                    let field_x = content_x + 70.0;
                    let field_w = (content_w - 74.0) / 3.0 - 2.0;
                    for col in 0..3 {
                        let fx = field_x + col as f32 * (field_w + 2.0);
                        painter.text(
                            egui::pos2(fx + 10.0, fy + UI_UNIT_Y * 0.5),
                            egui::Align2::LEFT_CENTER,
                            axis_names[col],
                            egui::FontId::proportional(9.0),
                            axis_colors[col],
                        );
                        let val_str = if *label == "Rotation" {
                            format!("{:.1}\u{00B0}", vals[col])
                        } else {
                            format!("{:.3}", vals[col])
                        };
                        painter.text(
                            egui::pos2(fx + field_w * 0.5 + 4.0, fy + UI_UNIT_Y * 0.5),
                            egui::Align2::CENTER_CENTER,
                            val_str,
                            egui::FontId::proportional(11.0),
                            text_color,
                        );
                    }
                }
                py += 3.0 * (UI_UNIT_Y + 4.0) + 8.0;
            }
        }

        // Relations header text
        let tri = if self.relations_open { "\u{25BC}" } else { "\u{25B6}" };
        painter.text(
            egui::pos2(content_x + 22.0, py + HEADERY * 0.5),
            egui::Align2::LEFT_CENTER,
            &format!("{} Relations", tri),
            egui::FontId::proportional(12.0),
            title_color,
        );
        py += HEADERY;

        if self.relations_open {
            painter.text(
                egui::pos2(content_x + 12.0, py + (UI_UNIT_Y + 8.0) * 0.5),
                egui::Align2::LEFT_CENTER,
                "Parent: None",
                egui::FontId::proportional(11.0),
                text_color,
            );
            py += UI_UNIT_Y + 8.0;
        }

        // Collections header text
        let tri = if self.collections_open { "\u{25BC}" } else { "\u{25B6}" };
        painter.text(
            egui::pos2(content_x + 22.0, py + HEADERY * 0.5),
            egui::Align2::LEFT_CENTER,
            &format!("{} Collections", tri),
            egui::FontId::proportional(12.0),
            title_color,
        );

        // -- Timeline text --
        let tl_y = h - TIMELINE_H;
        painter.text(
            egui::pos2(28.0, tl_y + HEADERY * 0.5),
            egui::Align2::LEFT_CENTER,
            "Timeline",
            egui::FontId::proportional(11.0),
            title_color,
        );

        painter.text(
            egui::pos2(w - 80.0, tl_y + HEADERY * 0.5),
            egui::Align2::RIGHT_CENTER,
            "Playback  Keying  View",
            egui::FontId::proportional(10.0),
            text_color,
        );

        // Scrub ruler tick labels
        {
            let scrub_y = tl_y + HEADERY;
            let total = (self.end_frame - self.start_frame).max(1) as f32;
            for f in (self.start_frame..=self.end_frame).step_by(50) {
                let frac = (f - self.start_frame) as f32 / total;
                let x = frac * w;
                painter.text(
                    egui::pos2(x, scrub_y + 2.0),
                    egui::Align2::CENTER_TOP,
                    format!("{f}"),
                    egui::FontId::proportional(9.0),
                    text_color,
                );
            }
        }

        // Transport button labels
        {
            let transport_y = tl_y + HEADERY + UI_UNIT_Y + 4.0;
            let btn_labels = ["\u{23EE}", "\u{25C0}", if self.playing { "\u{23F8}" } else { "\u{25B6}" }, "\u{25B6}\u{25B6}", "\u{23ED}"];
            let mut bx = 4.0;
            for label in &btn_labels {
                painter.text(
                    egui::pos2(bx + 14.0, transport_y + UI_UNIT_Y * 0.5),
                    egui::Align2::CENTER_CENTER,
                    *label,
                    egui::FontId::proportional(13.0),
                    text_color,
                );
                bx += 30.0;
            }

            // Frame: label + value
            painter.text(
                egui::pos2(bx + 20.0, transport_y + UI_UNIT_Y * 0.5),
                egui::Align2::LEFT_CENTER,
                "Frame:",
                egui::FontId::proportional(11.0),
                text_color,
            );
            painter.text(
                egui::pos2(bx + 70.0, transport_y + UI_UNIT_Y * 0.5),
                egui::Align2::CENTER_CENTER,
                format!("{}", self.current_frame),
                egui::FontId::proportional(11.0),
                text_color,
            );

            painter.text(
                egui::pos2(bx + 120.0, transport_y + UI_UNIT_Y * 0.5),
                egui::Align2::LEFT_CENTER,
                "Start:",
                egui::FontId::proportional(11.0),
                text_color,
            );
            painter.text(
                egui::pos2(bx + 170.0, transport_y + UI_UNIT_Y * 0.5),
                egui::Align2::CENTER_CENTER,
                format!("{}", self.start_frame),
                egui::FontId::proportional(11.0),
                text_color,
            );

            painter.text(
                egui::pos2(bx + 200.0, transport_y + UI_UNIT_Y * 0.5),
                egui::Align2::LEFT_CENTER,
                "End:",
                egui::FontId::proportional(11.0),
                text_color,
            );
            painter.text(
                egui::pos2(bx + 250.0, transport_y + UI_UNIT_Y * 0.5),
                egui::Align2::CENTER_CENTER,
                format!("{}", self.end_frame),
                egui::FontId::proportional(11.0),
                text_color,
            );
        }

        // -- 3D Viewport overlays (grid, gizmo, cursor, info text) --
        let vp_rect = egui::Rect::from_min_max(
            egui::pos2(0.0, HEADERY * 2.0),
            egui::pos2(vp_right, h - TIMELINE_H),
        );

        // Grid
        {
            let center = vp_rect.center();
            let grid_extent = 200.0;
            let grid_spacing = 20.0;
            let steps = (grid_extent / grid_spacing) as i32;
            let grid_color = egui::Color32::from_rgba_premultiplied(84, 84, 84, 128);

            for i in -steps..=steps {
                let y = center.y + i as f32 * grid_spacing;
                if y >= vp_rect.top() && y <= vp_rect.bottom() {
                    painter.line_segment(
                        [egui::pos2(vp_rect.left(), y), egui::pos2(vp_rect.right(), y)],
                        egui::Stroke::new(0.5, grid_color),
                    );
                }
            }
            for i in -steps..=steps {
                let x = center.x + i as f32 * grid_spacing;
                if x >= vp_rect.left() && x <= vp_rect.right() {
                    painter.line_segment(
                        [egui::pos2(x, vp_rect.top()), egui::pos2(x, vp_rect.bottom())],
                        egui::Stroke::new(0.5, grid_color),
                    );
                }
            }

            // Axis lines
            painter.line_segment(
                [egui::pos2(vp_rect.left(), center.y), egui::pos2(vp_rect.right(), center.y)],
                egui::Stroke::new(1.0, egui::Color32::from_rgba_premultiplied(180, 50, 50, 100)),
            );
            painter.line_segment(
                [egui::pos2(center.x, vp_rect.top()), egui::pos2(center.x, vp_rect.bottom())],
                egui::Stroke::new(1.0, egui::Color32::from_rgba_premultiplied(50, 180, 50, 100)),
            );
        }

        // 3D Cursor
        {
            let center = vp_rect.center();
            let size = 10.0;
            let cursor_red = egui::Color32::from_rgb(255, 0, 0);
            painter.line_segment(
                [center - egui::vec2(size, 0.0), center + egui::vec2(size, 0.0)],
                egui::Stroke::new(1.0, cursor_red),
            );
            painter.line_segment(
                [center - egui::vec2(0.0, size), center + egui::vec2(0.0, size)],
                egui::Stroke::new(1.0, cursor_red),
            );
            painter.circle_stroke(center, 3.0, egui::Stroke::new(1.0, egui::Color32::WHITE));
            painter.circle_filled(center, 1.5, cursor_red);
        }

        // Navigation gizmo
        {
            let gizmo_origin = vp_rect.left_bottom() + egui::vec2(40.0, -40.0);
            let axis_len = 24.0;
            let axis_x_color = egui::Color32::from_rgb(214, 67, 67);
            let axis_y_color = egui::Color32::from_rgb(104, 188, 80);
            let axis_z_color = egui::Color32::from_rgb(67, 133, 214);

            painter.circle_filled(gizmo_origin, 30.0, egui::Color32::from_rgba_premultiplied(30, 30, 30, 150));

            painter.line_segment(
                [gizmo_origin, gizmo_origin + egui::vec2(axis_len, 0.0)],
                egui::Stroke::new(2.5, axis_x_color),
            );
            painter.circle_filled(gizmo_origin + egui::vec2(axis_len + 3.0, 0.0), 4.0, axis_x_color);
            painter.text(gizmo_origin + egui::vec2(axis_len + 3.0, 0.0), egui::Align2::CENTER_CENTER, "X", egui::FontId::proportional(8.0), egui::Color32::WHITE);

            painter.line_segment(
                [gizmo_origin, gizmo_origin + egui::vec2(0.0, -axis_len)],
                egui::Stroke::new(2.5, axis_y_color),
            );
            painter.circle_filled(gizmo_origin + egui::vec2(0.0, -axis_len - 3.0), 4.0, axis_y_color);
            painter.text(gizmo_origin + egui::vec2(0.0, -axis_len - 3.0), egui::Align2::CENTER_CENTER, "Y", egui::FontId::proportional(8.0), egui::Color32::WHITE);

            painter.line_segment(
                [gizmo_origin, gizmo_origin + egui::vec2(-axis_len * 0.55, axis_len * 0.35)],
                egui::Stroke::new(2.5, axis_z_color),
            );
            painter.circle_filled(gizmo_origin + egui::vec2(-axis_len * 0.55 - 2.0, axis_len * 0.35 + 2.0), 4.0, axis_z_color);
            painter.text(gizmo_origin + egui::vec2(-axis_len * 0.55 - 2.0, axis_len * 0.35 + 2.0), egui::Align2::CENTER_CENTER, "Z", egui::FontId::proportional(8.0), egui::Color32::WHITE);

            painter.circle_filled(gizmo_origin, 3.0, egui::Color32::from_rgb(180, 180, 180));
        }

        // Viewport info text
        painter.text(
            vp_rect.right_bottom() + egui::vec2(-10.0, -8.0),
            egui::Align2::RIGHT_BOTTOM,
            "Verts: 8 | Faces: 6 | Tris: 12 | Objects: 1/3",
            egui::FontId::proportional(10.0),
            egui::Color32::from_rgb(140, 140, 140),
        );

        // 3D Viewport cube callback
        let cube_obj = &self.objects[0];
        let t = self.start_time.elapsed().as_secs_f32();

        let loc = cube_obj.location;
        let rot = cube_obj.rotation;
        let scl = cube_obj.scale;

        let translate = mat4_translate(loc[0], loc[1], loc[2]);
        let rotate_x = mat4_rotate_x(rot[0].to_radians());
        let rotate_y_m = mat4_rotate_y(rot[1].to_radians());
        let rotate_z = mat4_rotate_z(rot[2].to_radians());
        let scale = mat4_scale(scl[0], scl[1], scl[2]);

        let spin = mat4_mul(&mat4_rotate_y(t * 0.7), &mat4_rotate_x(t * 0.4));
        let model_static = mat4_mul(
            &translate,
            &mat4_mul(&rotate_z, &mat4_mul(&rotate_y_m, &mat4_mul(&rotate_x, &scale))),
        );
        let model = mat4_mul(&model_static, &spin);

        let vp_width = vp_rect.width();
        let vp_height = vp_rect.height();
        let aspect = if vp_height > 0.0 { vp_width / vp_height } else { 1.0 };

        let view = mat4_translate(0.0, 0.0, -3.0);
        let proj = mat4_perspective(std::f32::consts::FRAC_PI_4, aspect, 0.1, 100.0);
        let mv = mat4_mul(&view, &model);
        let mvp = mat4_mul(&proj, &mv);

        let callback = egui_wgpu::Callback::new_paint_callback(
            vp_rect,
            CubeCallback { mvp },
        );
        painter.add(callback);
    }

    // -----------------------------------------------------------------------
    // Render frame
    // -----------------------------------------------------------------------

    fn render(&mut self) {
        let window = match &self.window {
            Some(w) => w.clone(),
            None => return,
        };

        // Update timeline
        if self.playing {
            let now = Instant::now();
            let dt = now.duration_since(self.last_frame_time).as_secs_f32();
            if dt >= 1.0 / 24.0 {
                self.current_frame += 1;
                if self.current_frame > self.end_frame {
                    self.current_frame = self.start_frame;
                }
                self.last_frame_time = now;
            }
        }

        let (cfg_w, cfg_h) = {
            let config = self.surface_config.as_ref().unwrap();
            (config.width, config.height)
        };
        let w = cfg_w as f32;
        let h = cfg_h as f32;

        // Build DrawList for 2D UI
        let ui_draw_list = self.build_ui_draw_list(w, h);

        // egui frame (text-only overlay + 3D callback)
        let raw_input = self.egui_winit.as_mut().unwrap().take_egui_input(&window);
        self.egui_ctx.begin_pass(raw_input);
        self.draw_egui_text_overlay(&self.egui_ctx.clone(), w, h);
        let full_output = self.egui_ctx.end_pass();

        self.egui_winit
            .as_mut()
            .unwrap()
            .handle_platform_output(&window, full_output.platform_output);

        let pixels_per_point = full_output.pixels_per_point;
        let clipped_primitives = self.egui_ctx.tessellate(full_output.shapes, pixels_per_point);
        let textures_delta = full_output.textures_delta;

        // GPU rendering
        let device = self.device.as_ref().unwrap();
        let queue = self.queue.as_ref().unwrap();
        let surface = self.surface.as_ref().unwrap();

        // Update 2D UI uniform (screen size)
        if let Some(ui2d) = &self.ui2d {
            queue.write_buffer(
                &ui2d.uniform_buffer,
                0,
                bytemuck::cast_slice(&[w, h, 0.0f32, 0.0]),
            );
        }

        let output_frame = match surface.get_current_texture() {
            Ok(tex) => tex,
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                self.resize(cfg_w, cfg_h);
                return;
            }
            Err(e) => {
                eprintln!("Surface error: {e}");
                return;
            }
        };

        let view = output_frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [cfg_w, cfg_h],
            pixels_per_point,
        };

        let renderer = self.egui_renderer.as_mut().unwrap();

        for (id, image_delta) in &textures_delta.set {
            renderer.update_texture(device, queue, *id, image_delta);
        }

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("main-encoder"),
        });

        let user_cmd_bufs = renderer.update_buffers(
            device, queue, &mut encoder, &clipped_primitives, &screen_descriptor,
        );

        // PASS 1: Clear + 2D UI draw list
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ui2d-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 61.0 / 255.0,
                            g: 61.0 / 255.0,
                            b: 61.0 / 255.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            if let Some(ui2d) = &self.ui2d {
                // Upload and draw filled triangles
                if !ui_draw_list.vertices.is_empty() && !ui_draw_list.indices.is_empty() {
                    let verts: Vec<UiVertex> = ui_draw_list
                        .vertices
                        .iter()
                        .map(UiVertex::from_draw_vertex)
                        .collect();
                    let vbo = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("ui2d-vbo"),
                        contents: bytemuck::cast_slice(&verts),
                        usage: wgpu::BufferUsages::VERTEX,
                    });
                    let ibo = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("ui2d-ibo"),
                        contents: bytemuck::cast_slice(&ui_draw_list.indices),
                        usage: wgpu::BufferUsages::INDEX,
                    });

                    render_pass.set_pipeline(&ui2d.tri_pipeline);
                    render_pass.set_bind_group(0, &ui2d.bind_group, &[]);
                    render_pass.set_vertex_buffer(0, vbo.slice(..));
                    render_pass.set_index_buffer(ibo.slice(..), wgpu::IndexFormat::Uint32);
                    render_pass.draw_indexed(0..ui_draw_list.indices.len() as u32, 0, 0..1);
                }

                // Upload and draw line segments
                if !ui_draw_list.line_vertices.is_empty() && !ui_draw_list.line_indices.is_empty() {
                    let line_verts: Vec<UiVertex> = ui_draw_list
                        .line_vertices
                        .iter()
                        .map(UiVertex::from_draw_vertex)
                        .collect();
                    let line_vbo = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("ui2d-line-vbo"),
                        contents: bytemuck::cast_slice(&line_verts),
                        usage: wgpu::BufferUsages::VERTEX,
                    });
                    let line_ibo = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("ui2d-line-ibo"),
                        contents: bytemuck::cast_slice(&ui_draw_list.line_indices),
                        usage: wgpu::BufferUsages::INDEX,
                    });

                    render_pass.set_pipeline(&ui2d.line_pipeline);
                    render_pass.set_bind_group(0, &ui2d.bind_group, &[]);
                    render_pass.set_vertex_buffer(0, line_vbo.slice(..));
                    render_pass.set_index_buffer(line_ibo.slice(..), wgpu::IndexFormat::Uint32);
                    render_pass.draw_indexed(0..ui_draw_list.line_indices.len() as u32, 0, 0..1);
                }
            }
        }

        // PASS 2: egui (text + 3D cube callback) on top
        {
            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load, // Don't clear — draw on top
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

        for id in &textures_delta.free {
            renderer.free_texture(id);
        }

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
            .with_maximized(true)
            .with_inner_size(winit::dpi::LogicalSize::new(1920, 1080));

        let window = Arc::new(event_loop.create_window(attrs).expect("Failed to create window"));
        self.init_wgpu(window.clone());

        tracing::info!("Forge3D window opened (maximized)");
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
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
                event: KeyEvent {
                    logical_key: Key::Named(NamedKey::Escape),
                    state: ElementState::Pressed,
                    ..
                },
                ..
            } => {
                tracing::info!("Escape pressed - exiting");
                event_loop.exit();
            }
            WindowEvent::KeyboardInput {
                event: KeyEvent {
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
