//! Areas are rectangular subdivisions of a screen, each hosting one editor
//! type (3D viewport, outliner, properties, etc.).

use crate::region::RegionHandle;
use forge3d_alloc::Handle;

/// Handle into the area arena.
pub type AreaHandle = Handle<Area>;

/// Which editor occupies this area.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AreaType {
    Viewport3D,
    Outliner,
    Properties,
    Timeline,
    NodeEditor,
    ImageEditor,
    UVEditor,
    TextEditor,
    Console,
    GraphEditor,
    FileBrowser,
}

/// A rectangular area inside a screen.
#[derive(Debug)]
pub struct Area {
    /// Which editor is shown.
    pub area_type: AreaType,
    /// Pixel rectangle (x, y, width, height) within the screen.
    pub rect: [u32; 4],
    /// Regions owned by this area (header, main, sidebar, etc.).
    pub regions: Vec<RegionHandle>,
    /// Whether this area currently has keyboard focus.
    pub has_focus: bool,
}

impl Area {
    /// Create a new area with the given type and rectangle.
    pub fn new(area_type: AreaType, rect: [u32; 4]) -> Self {
        Self {
            area_type,
            rect,
            regions: Vec::new(),
            has_focus: false,
        }
    }

    /// Returns `true` if the pixel coordinate (px, py) is inside this area.
    pub fn contains(&self, px: u32, py: u32) -> bool {
        let [x, y, w, h] = self.rect;
        px >= x && px < x + w && py >= y && py < y + h
    }

    /// Width in pixels.
    pub fn width(&self) -> u32 {
        self.rect[2]
    }

    /// Height in pixels.
    pub fn height(&self) -> u32 {
        self.rect[3]
    }
}
