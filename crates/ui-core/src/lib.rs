//! # forge3d-ui-core
//!
//! Core UI framework for the Forge3D editor: widgets, layout engine,
//! operators, theming, input/keymap handling, and 2D painting.

pub mod input;
pub mod keymap;
pub mod layout;
pub mod operator;
pub mod painter;
pub mod theme;
pub mod widget;

pub use input::{DragState, InputState};
pub use keymap::{KeyBinding, KeyMap, KeyMapRegistry};
pub use layout::{Layout, LayoutDirection, LayoutItem};
pub use operator::{Operator, OperatorContext, OperatorResult, OperatorRegistry};
pub use painter::Painter;
pub use theme::{Theme, ThemeColor};
pub use widget::{Widget, WidgetId, WidgetResponse};

use thiserror::Error;

/// Errors originating from the UI subsystem.
#[derive(Debug, Error)]
pub enum UiError {
    #[error("widget `{0}` not found")]
    WidgetNotFound(String),

    #[error("operator `{idname}` failed: {reason}")]
    OperatorFailed { idname: String, reason: String },

    #[error("layout overflow: {0}")]
    LayoutOverflow(String),

    #[error("keymap conflict: `{0}` is already bound")]
    KeymapConflict(String),
}

/// Result alias for UI operations.
pub type UiResult<T> = Result<T, UiError>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::painter::{Color, Rect};

    #[test]
    fn slider_clamps_value_in_constructor() {
        let s = widget::Slider::new(widget::WidgetId(1), "Test", 200.0, 0.0, 100.0);
        assert_eq!(s.value, 100.0);

        let s2 = widget::Slider::new(widget::WidgetId(2), "Test", -10.0, 0.0, 100.0);
        assert_eq!(s2.value, 0.0);
    }

    #[test]
    fn slider_swapped_min_max() {
        let s = widget::Slider::new(widget::WidgetId(1), "Test", 50.0, 100.0, 0.0);
        assert_eq!(s.min, 0.0);
        assert_eq!(s.max, 100.0);
        assert_eq!(s.value, 50.0);
    }

    #[test]
    fn slider_equal_min_max() {
        let s = widget::Slider::new(widget::WidgetId(1), "Test", 5.0, 5.0, 5.0);
        assert_eq!(s.value, 5.0);
    }

    #[test]
    fn rect_contains() {
        let r = Rect::new(10.0, 20.0, 100.0, 50.0);
        assert!(r.contains(10.0, 20.0));
        assert!(r.contains(50.0, 40.0));
        assert!(!r.contains(9.9, 20.0));
        assert!(!r.contains(110.0, 20.0));
    }

    #[test]
    fn rect_right_bottom() {
        let r = Rect::new(5.0, 10.0, 20.0, 30.0);
        assert_eq!(r.right(), 25.0);
        assert_eq!(r.bottom(), 40.0);
    }

    #[test]
    fn color_constants() {
        assert_eq!(Color::WHITE.r, 1.0);
        assert_eq!(Color::BLACK.r, 0.0);
        assert_eq!(Color::TRANSPARENT.a, 0.0);
    }

    #[test]
    fn painter_drain() {
        let mut p = painter::Painter::new();
        p.fill_rect(Rect::new(0.0, 0.0, 10.0, 10.0), Color::WHITE, 0.0);
        assert_eq!(p.command_count(), 1);
        let cmds = p.drain();
        assert_eq!(cmds.len(), 1);
        assert_eq!(p.command_count(), 0);
    }

    #[test]
    fn input_state_edge_detection() {
        let mut input = InputState::new();
        // Simulate: button pressed on this frame.
        input.buttons[0] = true;
        input.begin_frame();
        assert!(input.left_pressed());
        assert!(input.left_held());
        assert!(!input.left_released());
        input.end_frame();

        // Next frame: still held, no longer "pressed".
        input.begin_frame();
        assert!(!input.left_pressed());
        assert!(input.left_held());
        input.end_frame();

        // Release.
        input.buttons[0] = false;
        input.begin_frame();
        assert!(input.left_released());
        assert!(!input.left_held());
    }

    #[test]
    fn drag_state_delta_and_distance() {
        let drag = DragState {
            start: [10.0, 20.0],
            current: [13.0, 24.0],
            button_index: 0,
        };
        assert_eq!(drag.delta(), [3.0, 4.0]);
        assert!((drag.distance() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn layout_compute_empty() {
        let mut layout = Layout::new(LayoutDirection::Horizontal);
        layout.compute(100.0, 50.0);
        assert!(layout.items.is_empty());
    }

    #[test]
    fn layout_compute_single_item() {
        let mut layout = Layout::new(LayoutDirection::Horizontal);
        layout.add(LayoutItem::default());
        layout.compute(100.0, 50.0);
        assert!(layout.items[0].computed_size[0] > 0.0);
    }

    #[test]
    fn operator_registry() {
        let mut reg = operator::OperatorRegistry::new();
        assert!(!reg.contains("TEST_OT_noop"));

        struct NoopOp;
        impl operator::Operator for NoopOp {
            fn idname(&self) -> &str { "TEST_OT_noop" }
            fn execute(&mut self, _ctx: &operator::OperatorContext<'_>) -> operator::OperatorResult {
                operator::OperatorResult::Finished
            }
        }

        reg.register("TEST_OT_noop", || Box::new(NoopOp));
        assert!(reg.contains("TEST_OT_noop"));
        let op = reg.create("TEST_OT_noop");
        assert!(op.is_some());
    }

    #[test]
    fn keymap_find() {
        let mut km = keymap::KeyMap::new("Test");
        km.add(keymap::KeyBinding {
            key: "G".to_string(),
            modifiers: 0,
            operator_idname: "TRANSFORM_OT_translate".to_string(),
            properties: smallvec::smallvec![],
        });
        assert!(km.find("G", 0).is_some());
        assert!(km.find("G", keymap::modifier::SHIFT).is_none());
        assert!(km.find("S", 0).is_none());
    }

    #[test]
    fn theme_default() {
        let theme = Theme::default();
        assert_eq!(theme.font_size, 13.0);
        assert_eq!(theme.widget_height, 24.0);
        // All theme colors should be present.
        let c = theme.color(theme::ThemeColor::Background);
        assert!(c.a > 0.0);
    }

    #[test]
    fn widget_response_none() {
        let r = widget::WidgetResponse::NONE;
        assert!(!r.hovered);
        assert!(!r.clicked);
        assert!(!r.changed);
        assert!(!r.has_focus);
    }
}
