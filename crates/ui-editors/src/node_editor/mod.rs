//! Node editor: visual graph editor for shaders, geometry nodes, and compositing.

/// Which node tree context is displayed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeEditorContext {
    Shader,
    Compositor,
    GeometryNodes,
    World,
    Texture,
}

impl Default for NodeEditorContext {
    fn default() -> Self {
        Self::Shader
    }
}

/// View state for the node editor canvas.
#[derive(Debug, Clone)]
pub struct NodeEditorView {
    /// Pan offset (x, y) in canvas units.
    pub offset: [f32; 2],
    /// Zoom level (1.0 = 100%).
    pub zoom: f32,
}

impl Default for NodeEditorView {
    fn default() -> Self {
        Self {
            offset: [0.0, 0.0],
            zoom: 1.0,
        }
    }
}

/// Node editor state.
pub struct NodeEditor {
    /// Which node tree is being edited.
    pub context: NodeEditorContext,
    /// Canvas view state.
    pub view: NodeEditorView,
    /// Whether the sidebar (N-panel) is open.
    pub show_sidebar: bool,
    /// Whether backdrop image is shown (compositor).
    pub show_backdrop: bool,
}

impl NodeEditor {
    pub fn new() -> Self {
        Self {
            context: NodeEditorContext::default(),
            view: NodeEditorView::default(),
            show_sidebar: false,
            show_backdrop: false,
        }
    }
}

impl Default for NodeEditor {
    fn default() -> Self {
        Self::new()
    }
}
