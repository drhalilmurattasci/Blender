//! Widget base drawing: the fundamental filled+outlined+embossed rounded
//! rectangle that underpins every Blender widget.

use crate::painter::Rect;
use super::draw_list::DrawList;
use super::roundbox::RoundboxFlags;

/// Color and shading parameters for a widget type, matching Blender's
/// `uiWidgetColors` struct.
#[derive(Debug, Clone, Copy)]
pub struct WidgetColors {
    /// Outline color.
    pub outline: [u8; 4],
    /// Inner gradient top color (normal state).
    pub inner: [u8; 4],
    /// Inner gradient top color (selected state).
    pub inner_sel: [u8; 4],
    /// Accent color for check marks, arrows, etc.
    pub item: [u8; 4],
    /// Text color (normal).
    pub text: [u8; 4],
    /// Text color (selected).
    pub text_sel: [u8; 4],
    /// If true, apply shade offsets to create a gradient.
    pub shaded: bool,
    /// HSV brightness offset for the top of the gradient (in 0-255 range).
    pub shadetop: i16,
    /// HSV brightness offset for the bottom of the gradient.
    pub shadedown: i16,
    /// Corner radius factor (0.0 = sharp, 1.0 = maximum rounding).
    pub roundness: f32,
}

impl Default for WidgetColors {
    fn default() -> Self {
        Self {
            outline: [50, 50, 50, 255],
            inner: [114, 114, 114, 255],
            inner_sel: [86, 128, 194, 255],
            item: [230, 230, 230, 255],
            text: [230, 230, 230, 255],
            text_sel: [255, 255, 255, 255],
            shaded: true,
            shadetop: 15,
            shadedown: -15,
            roundness: 0.2,
        }
    }
}

/// Interactive state of a widget.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WidgetState {
    Normal,
    Hover,
    Active,
    Disabled,
}

/// Emboss highlight and shadow colors (Blender defaults).
const EMBOSS_HIGHLIGHT: [u8; 4] = [255, 255, 255, 30];
const EMBOSS_SHADOW: [u8; 4] = [0, 0, 0, 50];

/// Draw the base of a Blender widget: filled rounded rect with gradient,
/// outline, and emboss.
///
/// Returns a [`DrawList`] containing colored triangles (fill) and line
/// segments (outline + emboss).
pub fn draw_widget_base(
    rect: Rect,
    colors: &WidgetColors,
    state: WidgetState,
    corner_radius: f32,
) -> DrawList {
    let mut dl = DrawList::new();
    let corners = RoundboxFlags::ALL;

    // Determine top/bottom colors based on state and shading.
    let (base_color, shaded) = match state {
        WidgetState::Normal => (colors.inner, colors.shaded),
        WidgetState::Hover => {
            // Hover: lighten inner slightly.
            (color_shade(colors.inner, 10), colors.shaded)
        }
        WidgetState::Active => (colors.inner_sel, colors.shaded),
        WidgetState::Disabled => {
            // Disabled: desaturate.
            let mut c = colors.inner;
            c[3] = (c[3] as u16 * 128 / 255) as u8; // half alpha
            (c, false)
        }
    };

    let (color_top, color_bottom) = if shaded {
        (
            color_shade(base_color, colors.shadetop),
            color_shade(base_color, colors.shadedown),
        )
    } else {
        (base_color, base_color)
    };

    // 1. Filled rounded rectangle with gradient.
    dl.add_filled_roundbox(rect, corner_radius, corners, color_top, color_bottom);

    // 2. Outline.
    let outline_color = match state {
        WidgetState::Active => color_shade(colors.outline, -20),
        WidgetState::Disabled => {
            let mut c = colors.outline;
            c[3] = 128;
            c
        }
        _ => colors.outline,
    };
    dl.add_outline_roundbox(rect, corner_radius, corners, outline_color);

    // 3. Emboss: highlight along top, shadow along bottom.
    if state != WidgetState::Disabled {
        dl.add_emboss(rect, corner_radius, corners, EMBOSS_HIGHLIGHT, EMBOSS_SHADOW);
    }

    dl
}

/// Apply an HSV brightness offset to an RGBA color.
///
/// This mimics Blender's `ui_colorshade` which simply clamps each RGB channel
/// by adding the offset.
pub fn color_shade(base: [u8; 4], offset: i16) -> [u8; 4] {
    [
        (base[0] as i16 + offset).clamp(0, 255) as u8,
        (base[1] as i16 + offset).clamp(0, 255) as u8,
        (base[2] as i16 + offset).clamp(0, 255) as u8,
        base[3],
    ]
}

/// Linearly blend two RGBA colors by `factor` (0.0 = a, 1.0 = b).
pub fn color_blend(a: [u8; 4], b: [u8; 4], factor: f32) -> [u8; 4] {
    let f = factor.clamp(0.0, 1.0);
    [
        (a[0] as f32 + (b[0] as f32 - a[0] as f32) * f) as u8,
        (a[1] as f32 + (b[1] as f32 - a[1] as f32) * f) as u8,
        (a[2] as f32 + (b[2] as f32 - a[2] as f32) * f) as u8,
        (a[3] as f32 + (b[3] as f32 - a[3] as f32) * f) as u8,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_shade_positive() {
        let c = color_shade([100, 100, 100, 255], 30);
        assert_eq!(c, [130, 130, 130, 255]);
    }

    #[test]
    fn color_shade_negative() {
        let c = color_shade([100, 100, 100, 255], -30);
        assert_eq!(c, [70, 70, 70, 255]);
    }

    #[test]
    fn color_shade_clamps_high() {
        let c = color_shade([250, 100, 100, 200], 20);
        assert_eq!(c[0], 255); // clamped
        assert_eq!(c[3], 200); // alpha unchanged
    }

    #[test]
    fn color_shade_clamps_low() {
        let c = color_shade([10, 100, 100, 255], -20);
        assert_eq!(c[0], 0); // clamped
    }

    #[test]
    fn color_blend_endpoints() {
        let a = [0, 0, 0, 255];
        let b = [255, 255, 255, 255];
        assert_eq!(color_blend(a, b, 0.0), a);
        assert_eq!(color_blend(a, b, 1.0), b);
    }

    #[test]
    fn color_blend_midpoint() {
        let a = [0, 0, 0, 0];
        let b = [200, 200, 200, 200];
        let mid = color_blend(a, b, 0.5);
        assert_eq!(mid, [100, 100, 100, 100]);
    }

    #[test]
    fn draw_widget_base_normal() {
        let dl = draw_widget_base(
            Rect::new(10.0, 10.0, 200.0, 28.0),
            &WidgetColors::default(),
            WidgetState::Normal,
            4.0,
        );
        assert!(dl.tri_count() > 0, "should have fill triangles");
        assert!(dl.line_count() > 0, "should have outline/emboss lines");
    }

    #[test]
    fn draw_widget_base_all_states() {
        let colors = WidgetColors::default();
        let rect = Rect::new(0.0, 0.0, 100.0, 28.0);
        for state in &[
            WidgetState::Normal,
            WidgetState::Hover,
            WidgetState::Active,
            WidgetState::Disabled,
        ] {
            let dl = draw_widget_base(rect, &colors, *state, 4.0);
            assert!(!dl.is_empty(), "state {:?} should produce geometry", state);
        }
    }
}
