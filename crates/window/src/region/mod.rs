//! Regions are sub-rectangles inside an area (header, main content,
//! sidebar, footer, etc.).

use forge3d_alloc::Handle;

/// Handle into the region arena.
pub type RegionHandle = Handle<Region>;

/// Kind of region within an area.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RegionType {
    /// Main content region (the largest area).
    Main,
    /// Header bar (tool settings, menus).
    Header,
    /// Right-hand sidebar (properties, options).
    Sidebar,
    /// Tool-settings bar at the top of the main region.
    ToolHeader,
    /// Footer bar (status, info).
    Footer,
    /// Horizontal scrollbar region.
    HScroll,
    /// Vertical scrollbar region.
    VScroll,
}

/// A sub-rectangle inside an [`super::area::Area`].
#[derive(Debug)]
pub struct Region {
    /// Kind of region.
    pub region_type: RegionType,
    /// Pixel rectangle (x, y, width, height) relative to the parent area.
    pub rect: [u32; 4],
    /// Scroll offset in pixels (x, y).
    pub scroll: [f32; 2],
    /// Whether this region is currently visible.
    pub visible: bool,
}

impl Region {
    /// Create a new region.
    pub fn new(region_type: RegionType, rect: [u32; 4]) -> Self {
        Self {
            region_type,
            rect,
            scroll: [0.0, 0.0],
            visible: true,
        }
    }

    /// Width in pixels.
    pub fn width(&self) -> u32 {
        self.rect[2]
    }

    /// Height in pixels.
    pub fn height(&self) -> u32 {
        self.rect[3]
    }

    /// Returns `true` if the coordinate is inside this region.
    pub fn contains(&self, px: u32, py: u32) -> bool {
        let [x, y, w, h] = self.rect;
        px >= x && px < x + w && py >= y && py < y + h
    }
}
