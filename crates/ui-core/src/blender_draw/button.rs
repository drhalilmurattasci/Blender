//! Button and control widget drawing functions.
//!
//! Each function returns a [`DrawList`] with the geometric primitives for that
//! widget. Text rendering is left to the caller's text pipeline; these
//! functions only produce the background, outline, emboss, and indicator
//! geometry.

use crate::painter::Rect;
use super::draw_list::{DrawList, DrawVertex};
use super::roundbox::RoundboxFlags;
use super::widget_base::{self, WidgetColors, WidgetState};

/// Default widget height in pixels (Blender uses ~20-28px depending on DPI).
pub const WIDGET_HEIGHT: f32 = 24.0;

/// Draw a standard push button.
///
/// The `_label` parameter is accepted for API completeness but text rendering
/// is handled by the caller. The returned `DrawList` contains the button
/// background, outline, and emboss.
pub fn draw_button(
    rect: Rect,
    _label: &str,
    state: WidgetState,
    colors: &WidgetColors,
) -> DrawList {
    let radius = colors.roundness * rect.h * 0.5;
    widget_base::draw_widget_base(rect, colors, state, radius)
}

/// Draw a number input field.
///
/// Renders the field background and small left/right arrow indicators
/// on the edges. The value text is rendered by the caller.
pub fn draw_number_field(
    rect: Rect,
    _value: f32,
    _label: &str,
    colors: &WidgetColors,
) -> DrawList {
    let radius = colors.roundness * rect.h * 0.5;
    let mut dl = widget_base::draw_widget_base(rect, colors, WidgetState::Normal, radius);

    // Left arrow indicator (small triangle).
    let arrow_w: f32 = 6.0;
    let cy = rect.y + rect.h * 0.5;
    let c = colors.item;
    {
        let base = dl.vertices.len() as u32;
        dl.vertices.push(DrawVertex::new(rect.x + 6.0, cy, c));
        dl.vertices
            .push(DrawVertex::new(rect.x + 6.0 + arrow_w, cy - 4.0, c));
        dl.vertices
            .push(DrawVertex::new(rect.x + 6.0 + arrow_w, cy + 4.0, c));
        dl.indices.push(base);
        dl.indices.push(base + 1);
        dl.indices.push(base + 2);
    }

    // Right arrow indicator.
    {
        let base = dl.vertices.len() as u32;
        let rx = rect.right() - 6.0;
        dl.vertices.push(DrawVertex::new(rx, cy, c));
        dl.vertices
            .push(DrawVertex::new(rx - arrow_w, cy - 4.0, c));
        dl.vertices
            .push(DrawVertex::new(rx - arrow_w, cy + 4.0, c));
        dl.indices.push(base);
        dl.indices.push(base + 1);
        dl.indices.push(base + 2);
    }

    dl
}

/// Draw a checkbox with optional check mark indicator.
///
/// The checkbox is rendered as a small rounded square on the left side of the
/// rect. When `checked`, a filled inner square is drawn using `colors.item`.
pub fn draw_checkbox(
    rect: Rect,
    checked: bool,
    _label: &str,
    colors: &WidgetColors,
) -> DrawList {
    let radius = colors.roundness * rect.h * 0.5;
    let mut dl = widget_base::draw_widget_base(rect, colors, WidgetState::Normal, radius);

    // Checkbox square area: 16x16, vertically centered, 4px left padding.
    let box_size: f32 = 16.0;
    let bx = rect.x + 4.0;
    let by = rect.y + (rect.h - box_size) * 0.5;
    let box_rect = Rect::new(bx, by, box_size, box_size);

    // Inner box outline.
    dl.add_outline_roundbox(box_rect, 2.0, RoundboxFlags::ALL, colors.outline);

    if checked {
        // Check mark: filled inner rect.
        let inset = 3.0;
        let inner_rect =
            Rect::new(bx + inset, by + inset, box_size - inset * 2.0, box_size - inset * 2.0);
        dl.add_filled_roundbox(
            inner_rect,
            1.0,
            RoundboxFlags::ALL,
            colors.item,
            colors.item,
        );
    }

    dl
}

/// Draw a menu / dropdown button.
///
/// Same as a regular button but with a small down-arrow indicator on the right
/// side to signal that clicking opens a menu.
pub fn draw_menu_button(
    rect: Rect,
    _label: &str,
    colors: &WidgetColors,
) -> DrawList {
    let radius = colors.roundness * rect.h * 0.5;
    let mut dl = widget_base::draw_widget_base(rect, colors, WidgetState::Normal, radius);

    // Down-arrow indicator (5px wide triangle on the right).
    let arrow_w: f32 = 5.0;
    let arrow_h: f32 = 3.0;
    let ax = rect.right() - 12.0;
    let ay = rect.y + (rect.h - arrow_h) * 0.5;
    let c = colors.item;
    let base = dl.vertices.len() as u32;
    dl.vertices.push(DrawVertex::new(ax, ay, c));
    dl.vertices
        .push(DrawVertex::new(ax + arrow_w * 2.0, ay, c));
    dl.vertices
        .push(DrawVertex::new(ax + arrow_w, ay + arrow_h * 2.0, c));
    dl.indices.push(base);
    dl.indices.push(base + 1);
    dl.indices.push(base + 2);

    dl
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_colors() -> WidgetColors {
        WidgetColors::default()
    }

    #[test]
    fn button_produces_geometry() {
        let dl = draw_button(
            Rect::new(0.0, 0.0, 120.0, WIDGET_HEIGHT),
            "OK",
            WidgetState::Normal,
            &default_colors(),
        );
        assert!(dl.tri_count() > 0);
        assert!(dl.line_count() > 0);
    }

    #[test]
    fn number_field_has_arrows() {
        let dl = draw_number_field(
            Rect::new(0.0, 0.0, 150.0, WIDGET_HEIGHT),
            42.0,
            "Scale",
            &default_colors(),
        );
        // Base widget + 2 arrow triangles.
        assert!(dl.tri_count() >= 3);
    }

    #[test]
    fn checkbox_checked_has_more_geometry() {
        let colors = default_colors();
        let rect = Rect::new(0.0, 0.0, 160.0, WIDGET_HEIGHT);
        let unchecked = draw_checkbox(rect, false, "Active", &colors);
        let checked = draw_checkbox(rect, true, "Active", &colors);
        assert!(checked.tri_count() > unchecked.tri_count());
    }

    #[test]
    fn menu_button_has_arrow() {
        let dl = draw_menu_button(
            Rect::new(0.0, 0.0, 140.0, WIDGET_HEIGHT),
            "File",
            &default_colors(),
        );
        // Base widget + arrow triangle.
        assert!(dl.tri_count() >= 2);
    }
}
