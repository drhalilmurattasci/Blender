//! Blender-accurate widget drawing system.
//!
//! This module implements the exact drawing primitives that Blender uses
//! for its UI: rounded rectangles with per-corner radii, two-color gradient
//! fills, outline strokes, emboss effects, and themed widget colors.
//!
//! Every widget in Blender is built from these primitives.

pub mod button;
pub mod draw_list;
pub mod panel;
pub mod roundbox;
pub mod separator;
pub mod theme;
pub mod widget_base;

pub use button::{draw_button, draw_checkbox, draw_menu_button, draw_number_field};
pub use draw_list::{DrawList, DrawVertex};
pub use panel::{draw_panel_background, draw_panel_header};
pub use roundbox::{RoundboxFlags, CORNER_VEC};
pub use separator::draw_separator;
pub use theme::Theme as BlenderTheme;
pub use widget_base::{WidgetColors, WidgetState};
