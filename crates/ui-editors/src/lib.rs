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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn console_submit_empty_input() {
        let mut c = ConsoleEditor::new();
        let result = c.submit();
        assert!(result.is_empty());
        // Empty input should not add to command history.
        assert!(c.command_history.is_empty());
        // But it should add to display history.
        assert_eq!(c.history.len(), 1);
    }

    #[test]
    fn console_submit_and_history() {
        let mut c = ConsoleEditor::new();
        c.input = "print('hello')".to_string();
        let submitted = c.submit();
        assert_eq!(submitted, "print('hello')");
        assert_eq!(c.command_history.len(), 1);
        assert!(c.input.is_empty());
    }

    #[test]
    fn console_history_navigation() {
        let mut c = ConsoleEditor::new();
        c.input = "cmd1".to_string();
        c.submit();
        c.input = "cmd2".to_string();
        c.submit();

        c.history_up();
        assert_eq!(c.input, "cmd2");
        c.history_up();
        assert_eq!(c.input, "cmd1");
        c.history_up(); // Should stay at first
        assert_eq!(c.input, "cmd1");

        c.history_down();
        assert_eq!(c.input, "cmd2");
        c.history_down(); // Past end clears input
        assert!(c.input.is_empty());
    }

    #[test]
    fn console_history_up_empty() {
        let mut c = ConsoleEditor::new();
        c.history_up(); // should not panic
        assert!(c.input.is_empty());
    }

    #[test]
    fn timeline_step_forward_loop() {
        let mut t = TimelineEditor::new();
        t.frame_current = 250.0;
        t.loop_playback = true;
        let f = t.step_forward();
        assert_eq!(f, 1.0); // Should loop back to start.
    }

    #[test]
    fn timeline_step_backward_loop() {
        let mut t = TimelineEditor::new();
        t.frame_current = 1.0;
        t.loop_playback = true;
        let f = t.step_backward();
        assert_eq!(f, 250.0); // Should loop to end.
    }

    #[test]
    fn timeline_toggle_playback() {
        let mut t = TimelineEditor::new();
        assert_eq!(t.playback_state, timeline::PlaybackState::Stopped);
        t.toggle_playback();
        assert_eq!(t.playback_state, timeline::PlaybackState::Playing);
        t.toggle_playback();
        assert_eq!(t.playback_state, timeline::PlaybackState::Stopped);
    }

    #[test]
    fn graph_editor_defaults() {
        let g = GraphEditor::new();
        assert_eq!(g.frame_range, [1.0, 250.0]);
        assert!(g.show_handles);
        assert!(!g.normalize);
    }

    #[test]
    fn viewport_editor_defaults() {
        let v = ViewportEditor::new();
        assert_eq!(v.shading, viewport::ShadingMode::Solid);
        assert!(v.overlays.show_grid);
        assert!(!v.use_scene_camera);
        assert!(!v.local_view);
    }

    #[test]
    fn properties_set_tab_resets_scroll() {
        let mut p = PropertiesEditor::new();
        p.scroll_y = 100.0;
        p.set_tab(properties::PropertiesTab::Render);
        assert_eq!(p.scroll_y, 0.0);
        assert_eq!(p.active_tab, properties::PropertiesTab::Render);
    }

    #[test]
    fn outliner_defaults() {
        let o = OutlinerEditor::new();
        assert_eq!(o.display_mode, outliner::OutlinerDisplayMode::ViewLayer);
        assert!(o.filter.show_meshes);
        assert!(!o.filter.show_hidden);
    }

    #[test]
    fn text_editor_set_cursor_clears_selection() {
        let mut t = TextEditor::new();
        t.begin_selection();
        assert!(t.has_selection());
        t.set_cursor(5, 10);
        assert!(!t.has_selection());
        assert_eq!(t.cursor_line, 5);
        assert_eq!(t.cursor_column, 10);
    }

    #[test]
    fn image_editor_defaults() {
        let ie = ImageEditor::new();
        assert_eq!(ie.mode, image_editor::ImageDisplayMode::View);
        assert_eq!(ie.zoom, 1.0);
        assert!(ie.show_alpha);
    }

    #[test]
    fn node_editor_defaults() {
        let ne = NodeEditor::new();
        assert_eq!(ne.context, node_editor::NodeEditorContext::Shader);
        assert_eq!(ne.view.zoom, 1.0);
    }

    #[test]
    fn uv_editor_defaults() {
        let uv = UVEditor::new();
        assert_eq!(uv.mode, uv_editor::UVDisplayMode::UV);
        assert!(uv.sync_selection);
    }
}
