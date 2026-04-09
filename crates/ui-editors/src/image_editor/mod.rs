//! Image editor: displays images/textures and paint tools.

/// Image editor display mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImageDisplayMode {
    /// View an image (texture, render result).
    View,
    /// Texture paint mode.
    Paint,
    /// Mask editing mode.
    Mask,
}

impl Default for ImageDisplayMode {
    fn default() -> Self {
        Self::View
    }
}

/// Image editor state.
pub struct ImageEditor {
    /// Current display mode.
    pub mode: ImageDisplayMode,
    /// Name / path of the currently displayed image.
    pub image_name: Option<String>,
    /// Pan offset (x, y) in pixels.
    pub offset: [f32; 2],
    /// Zoom level (1.0 = 100%).
    pub zoom: f32,
    /// Whether to show alpha as checkerboard.
    pub show_alpha: bool,
    /// Whether to repeat the image (tile).
    pub repeat: bool,
}

impl ImageEditor {
    pub fn new() -> Self {
        Self {
            mode: ImageDisplayMode::default(),
            image_name: None,
            offset: [0.0, 0.0],
            zoom: 1.0,
            show_alpha: true,
            repeat: false,
        }
    }
}

impl Default for ImageEditor {
    fn default() -> Self {
        Self::new()
    }
}
