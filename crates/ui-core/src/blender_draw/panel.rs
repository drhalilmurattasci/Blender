//! Panel drawing: header bar with collapse triangle and title, plus
//! the panel body background.

use crate::painter::Rect;
use super::draw_list::DrawList;
use super::theme::PanelColors;

/// Standard panel header height in pixels (matching Blender).
pub const PANEL_HEADER_HEIGHT: f32 = 26.0;

/// Draw a panel header bar with a collapse triangle and title text.
///
/// The triangle is a small filled indicator on the left side:
/// - Pointing right when `collapsed` is true.
/// - Pointing down when `collapsed` is false.
///
/// Note: text rendering is represented as a placeholder rectangle in the
/// draw list since actual glyph rasterization lives in the GPU text pipeline.
/// The caller is expected to render the title string separately using the
/// text subsystem; the returned `DrawList` provides the geometric background
/// and collapse indicator.
pub fn draw_panel_header(
    rect: Rect,
    _title: &str,
    collapsed: bool,
    theme: &PanelColors,
) -> DrawList {
    let mut dl = DrawList::new();

    // Header background.
    dl.add_rect_filled(rect, theme.header_back);

    // Collapse triangle (8x8 px, vertically centered, 8px left margin).
    let tri_size: f32 = 8.0;
    let tri_x = rect.x + 8.0;
    let tri_cy = rect.y + rect.h * 0.5;

    if collapsed {
        // Right-pointing triangle.
        let base = dl.vertices.len() as u32;
        let c = theme.triangle;
        dl.vertices.push(super::draw_list::DrawVertex::new(tri_x, tri_cy - tri_size * 0.5, c));
        dl.vertices.push(super::draw_list::DrawVertex::new(tri_x + tri_size, tri_cy, c));
        dl.vertices.push(super::draw_list::DrawVertex::new(tri_x, tri_cy + tri_size * 0.5, c));
        dl.indices.push(base);
        dl.indices.push(base + 1);
        dl.indices.push(base + 2);
    } else {
        // Down-pointing triangle.
        let base = dl.vertices.len() as u32;
        let c = theme.triangle;
        dl.vertices.push(super::draw_list::DrawVertex::new(tri_x, tri_cy - tri_size * 0.25, c));
        dl.vertices.push(super::draw_list::DrawVertex::new(tri_x + tri_size, tri_cy - tri_size * 0.25, c));
        dl.vertices.push(super::draw_list::DrawVertex::new(tri_x + tri_size * 0.5, tri_cy + tri_size * 0.5, c));
        dl.indices.push(base);
        dl.indices.push(base + 1);
        dl.indices.push(base + 2);
    }

    // Bottom separator line.
    dl.add_line(
        rect.x,
        rect.bottom(),
        rect.right(),
        rect.bottom(),
        [36, 36, 36, 255],
    );

    dl
}

/// Draw the panel body background.
pub fn draw_panel_background(rect: Rect, theme: &PanelColors) -> DrawList {
    let mut dl = DrawList::new();
    dl.add_rect_filled(rect, theme.body_back);
    dl
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn panel_header_collapsed() {
        let rect = Rect::new(0.0, 0.0, 300.0, PANEL_HEADER_HEIGHT);
        let theme = PanelColors::default();
        let dl = draw_panel_header(rect, "Transform", true, &theme);
        // Should have background quad (2 tris) + triangle (1 tri) + separator line.
        assert!(dl.tri_count() >= 3);
        assert!(dl.line_count() >= 1);
    }

    #[test]
    fn panel_header_expanded() {
        let rect = Rect::new(0.0, 0.0, 300.0, PANEL_HEADER_HEIGHT);
        let theme = PanelColors::default();
        let dl = draw_panel_header(rect, "Transform", false, &theme);
        assert!(dl.tri_count() >= 3);
    }

    #[test]
    fn panel_background_produces_rect() {
        let rect = Rect::new(0.0, 26.0, 300.0, 200.0);
        let theme = PanelColors::default();
        let dl = draw_panel_background(rect, &theme);
        assert_eq!(dl.tri_count(), 2);
    }
}
