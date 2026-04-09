//! A screen represents one OS window and owns a grid of areas.

use crate::area::AreaHandle;
use forge3d_alloc::Handle;

/// Handle to a screen in the window manager.
pub type ScreenHandle = Handle<Screen>;

/// An OS-level window containing a layout of areas.
#[derive(Debug)]
pub struct Screen {
    /// Human-readable name (e.g. "Default", "Animation", "Scripting").
    pub name: String,
    /// Pixel rectangle (x, y, width, height).
    pub rect: [u32; 4],
    /// Areas laid out in this screen.
    pub areas: Vec<AreaHandle>,
    /// Edges used for splitting / resizing areas.
    pub edges: Vec<ScreenEdge>,
}

/// A vertical or horizontal divider between two areas.
#[derive(Debug, Clone)]
pub struct ScreenEdge {
    /// Is this a vertical split?
    pub vertical: bool,
    /// Position along the split axis in pixels.
    pub position: u32,
    /// Areas on the "left/top" side.
    pub areas_before: Vec<AreaHandle>,
    /// Areas on the "right/bottom" side.
    pub areas_after: Vec<AreaHandle>,
}

impl Screen {
    /// Create a new screen spanning the given rectangle.
    pub fn new(name: impl Into<String>, width: u32, height: u32) -> Self {
        Self {
            name: name.into(),
            rect: [0, 0, width, height],
            areas: Vec::new(),
            edges: Vec::new(),
        }
    }

    /// Add an area handle.
    pub fn add_area(&mut self, area: AreaHandle) {
        self.areas.push(area);
    }

    /// Total width.
    pub fn width(&self) -> u32 {
        self.rect[2]
    }

    /// Total height.
    pub fn height(&self) -> u32 {
        self.rect[3]
    }
}
