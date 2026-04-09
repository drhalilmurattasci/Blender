//! 3D Viewport editor: displays the scene from a camera and handles
//! navigation (orbit, pan, zoom), selection, and gizmos.

/// Viewport shading mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShadingMode {
    Wireframe,
    Solid,
    MaterialPreview,
    Rendered,
}

impl Default for ShadingMode {
    fn default() -> Self {
        Self::Solid
    }
}

/// Viewport overlay settings.
#[derive(Debug, Clone)]
pub struct ViewportOverlays {
    /// Show the grid floor.
    pub show_grid: bool,
    /// Show object origins.
    pub show_origins: bool,
    /// Show object relationships (parent lines).
    pub show_relationships: bool,
    /// Show wireframe overlay.
    pub show_wireframe: bool,
    /// Wireframe overlay opacity (0..1).
    pub wireframe_opacity: f32,
    /// Show normals.
    pub show_normals: bool,
    /// Normal display length.
    pub normal_length: f32,
    /// Show statistics (vert/edge/face counts).
    pub show_statistics: bool,
}

impl Default for ViewportOverlays {
    fn default() -> Self {
        Self {
            show_grid: true,
            show_origins: true,
            show_relationships: true,
            show_wireframe: false,
            wireframe_opacity: 0.5,
            show_normals: false,
            normal_length: 0.1,
            show_statistics: false,
        }
    }
}

/// Viewport gizmo settings.
#[derive(Debug, Clone)]
pub struct ViewportGizmos {
    /// Show move/rotate/scale gizmo.
    pub show_transform: bool,
    /// Active transform type.
    pub transform_type: TransformGizmoType,
    /// Show navigation gizmo (orientation cube).
    pub show_navigate: bool,
}

/// Which transform gizmo is active.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransformGizmoType {
    Translate,
    Rotate,
    Scale,
}

impl Default for ViewportGizmos {
    fn default() -> Self {
        Self {
            show_transform: true,
            transform_type: TransformGizmoType::Translate,
            show_navigate: true,
        }
    }
}

/// Camera view parameters for the viewport.
#[derive(Debug, Clone)]
pub struct ViewportCamera {
    /// Camera pivot / look-at point.
    pub target: [f32; 3],
    /// Orbit distance from target.
    pub distance: f32,
    /// Orbit rotation as quaternion (x, y, z, w).
    pub rotation: [f32; 4],
    /// Whether in orthographic projection.
    pub orthographic: bool,
    /// Field of view in degrees (perspective only).
    pub fov: f32,
    /// Near clip plane.
    pub clip_near: f32,
    /// Far clip plane.
    pub clip_far: f32,
}

impl Default for ViewportCamera {
    fn default() -> Self {
        Self {
            target: [0.0, 0.0, 0.0],
            distance: 10.0,
            rotation: [0.0, 0.0, 0.0, 1.0],
            orthographic: false,
            fov: 50.0,
            clip_near: 0.1,
            clip_far: 1000.0,
        }
    }
}

/// The 3D viewport editor state.
pub struct ViewportEditor {
    /// Current shading mode.
    pub shading: ShadingMode,
    /// Overlay settings.
    pub overlays: ViewportOverlays,
    /// Gizmo settings.
    pub gizmos: ViewportGizmos,
    /// Camera view state.
    pub camera: ViewportCamera,
    /// Whether the viewport is in camera view (locked to scene camera).
    pub use_scene_camera: bool,
    /// Whether local view (isolate selection) is active.
    pub local_view: bool,
}

impl ViewportEditor {
    pub fn new() -> Self {
        Self {
            shading: ShadingMode::default(),
            overlays: ViewportOverlays::default(),
            gizmos: ViewportGizmos::default(),
            camera: ViewportCamera::default(),
            use_scene_camera: false,
            local_view: false,
        }
    }
}

impl Default for ViewportEditor {
    fn default() -> Self {
        Self::new()
    }
}
