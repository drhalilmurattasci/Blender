//! Graph editor: displays and edits F-Curves for animation.

/// Channel filter / visibility.
#[derive(Debug, Clone)]
pub struct GraphChannelFilter {
    /// Text search.
    pub search: String,
    /// Show location curves.
    pub show_location: bool,
    /// Show rotation curves.
    pub show_rotation: bool,
    /// Show scale curves.
    pub show_scale: bool,
    /// Show custom property curves.
    pub show_custom: bool,
    /// Only show selected channels.
    pub only_selected: bool,
}

impl Default for GraphChannelFilter {
    fn default() -> Self {
        Self {
            search: String::new(),
            show_location: true,
            show_rotation: true,
            show_scale: true,
            show_custom: true,
            only_selected: false,
        }
    }
}

/// Graph editor state.
pub struct GraphEditor {
    /// Visible frame range.
    pub frame_range: [f32; 2],
    /// Visible value range.
    pub value_range: [f32; 2],
    /// Zoom levels (pixels per frame, pixels per unit value).
    pub zoom: [f32; 2],
    /// Pan offset.
    pub offset: [f32; 2],
    /// Channel filter settings.
    pub filter: GraphChannelFilter,
    /// Whether to show handles.
    pub show_handles: bool,
    /// Cursor frame (vertical green line).
    pub cursor_frame: f32,
    /// Whether auto-normalization is on.
    pub normalize: bool,
}

impl GraphEditor {
    pub fn new() -> Self {
        Self {
            frame_range: [1.0, 250.0],
            value_range: [-1.0, 1.0],
            zoom: [8.0, 50.0],
            offset: [0.0, 0.0],
            filter: GraphChannelFilter::default(),
            show_handles: true,
            cursor_frame: 1.0,
            normalize: false,
        }
    }
}

impl Default for GraphEditor {
    fn default() -> Self {
        Self::new()
    }
}
