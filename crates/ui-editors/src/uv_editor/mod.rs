//! UV Editor: displays and edits UV mappings of mesh objects.

/// UV editor display mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UVDisplayMode {
    /// Standard UV editing.
    UV,
    /// Paint mode for the UV space.
    Paint,
}

impl Default for UVDisplayMode {
    fn default() -> Self {
        Self::UV
    }
}

/// UV stretching visualization method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UVStretchDisplay {
    None,
    Angle,
    Area,
}

impl Default for UVStretchDisplay {
    fn default() -> Self {
        Self::None
    }
}

/// UV editor overlays.
#[derive(Debug, Clone)]
pub struct UVOverlays {
    /// Show UV stretch visualization.
    pub stretch: UVStretchDisplay,
    /// Show faces in the UV space.
    pub show_faces: bool,
    /// Show modified UVs (from modifiers).
    pub show_modified: bool,
    /// Opacity of faces.
    pub face_opacity: f32,
}

impl Default for UVOverlays {
    fn default() -> Self {
        Self {
            stretch: UVStretchDisplay::default(),
            show_faces: true,
            show_modified: true,
            face_opacity: 0.3,
        }
    }
}

/// UV editor state.
pub struct UVEditor {
    /// Display mode.
    pub mode: UVDisplayMode,
    /// Pan offset.
    pub offset: [f32; 2],
    /// Zoom level.
    pub zoom: f32,
    /// Overlay settings.
    pub overlays: UVOverlays,
    /// Whether to synchronize selection with the 3D viewport.
    pub sync_selection: bool,
    /// Pixel snap during transform.
    pub pixel_snap: bool,
}

impl UVEditor {
    pub fn new() -> Self {
        Self {
            mode: UVDisplayMode::default(),
            offset: [0.0, 0.0],
            zoom: 1.0,
            overlays: UVOverlays::default(),
            sync_selection: true,
            pixel_snap: false,
        }
    }
}

impl Default for UVEditor {
    fn default() -> Self {
        Self::new()
    }
}
