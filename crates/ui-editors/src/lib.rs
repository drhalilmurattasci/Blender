//! # forge3d-ui-editors
//!
//! Editor panels for the Forge3D application: 3D viewport, outliner,
//! properties panel, node editor, timeline, graph editor, image editor,
//! UV editor, text editor, and console.

pub mod console;
pub mod graph_editor;
pub mod image_editor;
pub mod node_editor;
pub mod outliner;
pub mod properties;
pub mod text_editor;
pub mod timeline;
pub mod uv_editor;
pub mod viewport;

pub use console::ConsoleEditor;
pub use graph_editor::GraphEditor;
pub use image_editor::ImageEditor;
pub use node_editor::NodeEditor;
pub use outliner::OutlinerEditor;
pub use properties::PropertiesEditor;
pub use text_editor::TextEditor;
pub use timeline::TimelineEditor;
pub use uv_editor::UVEditor;
pub use viewport::ViewportEditor;
