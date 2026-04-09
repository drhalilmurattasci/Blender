//! Outliner editor: hierarchical tree view of scene objects and collections.

/// Display mode for the outliner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OutlinerDisplayMode {
    /// Show the scene hierarchy (collections + objects).
    ViewLayer,
    /// Show all scenes in the file.
    Scenes,
    /// Show the data-block browser.
    DataApi,
    /// Show orphan data-blocks.
    OrphanData,
}

impl Default for OutlinerDisplayMode {
    fn default() -> Self {
        Self::ViewLayer
    }
}

/// Filter settings for what the outliner shows.
#[derive(Debug, Clone)]
pub struct OutlinerFilter {
    /// Text search filter.
    pub search: String,
    /// Show meshes.
    pub show_meshes: bool,
    /// Show lights.
    pub show_lights: bool,
    /// Show cameras.
    pub show_cameras: bool,
    /// Show empties.
    pub show_empties: bool,
    /// Show armatures.
    pub show_armatures: bool,
    /// Show hidden objects.
    pub show_hidden: bool,
}

impl Default for OutlinerFilter {
    fn default() -> Self {
        Self {
            search: String::new(),
            show_meshes: true,
            show_lights: true,
            show_cameras: true,
            show_empties: true,
            show_armatures: true,
            show_hidden: false,
        }
    }
}

/// Outliner editor state.
pub struct OutlinerEditor {
    /// Current display mode.
    pub display_mode: OutlinerDisplayMode,
    /// Filter settings.
    pub filter: OutlinerFilter,
    /// Scroll position (vertical offset in pixels).
    pub scroll_y: f32,
    /// Row height in pixels.
    pub row_height: f32,
    /// Indent width per nesting level.
    pub indent: f32,
}

impl OutlinerEditor {
    pub fn new() -> Self {
        Self {
            display_mode: OutlinerDisplayMode::default(),
            filter: OutlinerFilter::default(),
            scroll_y: 0.0,
            row_height: 20.0,
            indent: 16.0,
        }
    }
}

impl Default for OutlinerEditor {
    fn default() -> Self {
        Self::new()
    }
}
