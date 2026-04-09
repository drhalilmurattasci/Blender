//! Rounded rectangle vertex generation matching Blender's `cornervec` system.
//!
//! Blender precomputes 9 normalized points per quarter-arc and uses them
//! to build rounded rectangles with configurable per-corner radii.

use crate::painter::Rect;

/// Blender's precomputed 9-point normalized corner arc (quarter circle).
/// Each entry is `(cos, sin)` from 0 to 90 degrees in 9 steps.
pub const CORNER_VEC: [[f32; 2]; 9] = [
    [0.0, 1.0],     // 0 degrees (top of arc)
    [0.195, 0.981],  // ~11.25 degrees
    [0.383, 0.924],  // ~22.5 degrees
    [0.556, 0.831],  // ~33.75 degrees
    [0.707, 0.707],  // 45 degrees
    [0.831, 0.556],  // ~56.25 degrees
    [0.924, 0.383],  // ~67.5 degrees
    [0.981, 0.195],  // ~78.75 degrees
    [1.0, 0.0],     // 90 degrees (end of arc)
];

bitflags::bitflags! {
    /// Which corners of a rounded rectangle should be rounded.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct RoundboxFlags: u8 {
        const TOP_LEFT     = 0b0001;
        const TOP_RIGHT    = 0b0010;
        const BOTTOM_RIGHT = 0b0100;
        const BOTTOM_LEFT  = 0b1000;
        const ALL          = 0b1111;
        const NONE         = 0b0000;
    }
}

impl Default for RoundboxFlags {
    fn default() -> Self {
        Self::ALL
    }
}

/// Generate the closed vertex ring for a filled rounded rectangle.
///
/// Vertices are emitted counter-clockwise starting from the bottom-left corner.
/// The ring can be triangulated with a fan from the center for fill rendering.
///
/// If a corner's flag is not set, a single sharp corner vertex is emitted instead
/// of the 9-point arc.
pub fn roundbox_vertices(rect: Rect, radius: f32, corners: RoundboxFlags) -> Vec<[f32; 2]> {
    let mut verts = Vec::with_capacity(4 * 9 + 1);

    let x_min = rect.x;
    let x_max = rect.x + rect.w;
    let y_min = rect.y;
    let y_max = rect.y + rect.h;

    // Clamp radius to half the smallest dimension.
    let max_rad = (rect.w * 0.5).min(rect.h * 0.5);
    let rad = radius.min(max_rad).max(0.0);

    // Bottom-left corner (arc goes from 180 to 270 degrees).
    if corners.contains(RoundboxFlags::BOTTOM_LEFT) && rad > 0.0 {
        let cx = x_min + rad;
        let cy = y_max - rad;
        for i in 0..9 {
            let vx = cx - rad * CORNER_VEC[i][0]; // negate x for left side
            let vy = cy + rad * CORNER_VEC[i][1]; // positive y = down
            verts.push([vx, vy]);
        }
    } else {
        verts.push([x_min, y_max]);
    }

    // Top-left corner (arc goes from 90 to 180 degrees).
    if corners.contains(RoundboxFlags::TOP_LEFT) && rad > 0.0 {
        let cx = x_min + rad;
        let cy = y_min + rad;
        for i in 0..9 {
            let vx = cx - rad * CORNER_VEC[8 - i][0];
            let vy = cy - rad * CORNER_VEC[8 - i][1];
            verts.push([vx, vy]);
        }
    } else {
        verts.push([x_min, y_min]);
    }

    // Top-right corner (arc goes from 0 to 90 degrees).
    if corners.contains(RoundboxFlags::TOP_RIGHT) && rad > 0.0 {
        let cx = x_max - rad;
        let cy = y_min + rad;
        for i in 0..9 {
            let vx = cx + rad * CORNER_VEC[i][0];
            let vy = cy - rad * CORNER_VEC[i][1];
            verts.push([vx, vy]);
        }
    } else {
        verts.push([x_max, y_min]);
    }

    // Bottom-right corner (arc goes from 270 to 360 degrees).
    if corners.contains(RoundboxFlags::BOTTOM_RIGHT) && rad > 0.0 {
        let cx = x_max - rad;
        let cy = y_max - rad;
        for i in 0..9 {
            let vx = cx + rad * CORNER_VEC[8 - i][0];
            let vy = cy + rad * CORNER_VEC[8 - i][1];
            verts.push([vx, vy]);
        }
    } else {
        verts.push([x_max, y_max]);
    }

    verts
}

/// Generate the outline path (open polyline) for a rounded rectangle.
///
/// Same vertex positions as [`roundbox_vertices`] but intended for line-strip
/// rendering. The caller should close the path by connecting last -> first.
pub fn roundbox_path(rect: Rect, radius: f32, corners: RoundboxFlags) -> Vec<[f32; 2]> {
    // The outline path is the same vertex ring as the fill.
    roundbox_vertices(rect, radius, corners)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cornervec_endpoints() {
        // First point should be (0, 1) and last should be (1, 0).
        assert_eq!(CORNER_VEC[0], [0.0, 1.0]);
        assert_eq!(CORNER_VEC[8], [1.0, 0.0]);
    }

    #[test]
    fn cornervec_midpoint_is_45_degrees() {
        let mid = CORNER_VEC[4];
        assert!((mid[0] - 0.707).abs() < 0.001);
        assert!((mid[1] - 0.707).abs() < 0.001);
    }

    #[test]
    fn roundbox_all_corners_vertex_count() {
        let rect = Rect::new(0.0, 0.0, 100.0, 50.0);
        let verts = roundbox_vertices(rect, 10.0, RoundboxFlags::ALL);
        // 4 corners * 9 points each = 36
        assert_eq!(verts.len(), 36);
    }

    #[test]
    fn roundbox_no_corners_vertex_count() {
        let rect = Rect::new(0.0, 0.0, 100.0, 50.0);
        let verts = roundbox_vertices(rect, 10.0, RoundboxFlags::NONE);
        // 4 sharp corners
        assert_eq!(verts.len(), 4);
    }

    #[test]
    fn roundbox_zero_radius() {
        let rect = Rect::new(10.0, 20.0, 80.0, 40.0);
        let verts = roundbox_vertices(rect, 0.0, RoundboxFlags::ALL);
        // Zero radius => sharp corners
        assert_eq!(verts.len(), 4);
    }

    #[test]
    fn roundbox_vertices_within_rect() {
        let rect = Rect::new(10.0, 20.0, 80.0, 40.0);
        let verts = roundbox_vertices(rect, 5.0, RoundboxFlags::ALL);
        for v in &verts {
            assert!(v[0] >= rect.x - 0.01, "x={} < rect.x={}", v[0], rect.x);
            assert!(v[0] <= rect.right() + 0.01, "x={} > right={}", v[0], rect.right());
            assert!(v[1] >= rect.y - 0.01, "y={} < rect.y={}", v[1], rect.y);
            assert!(v[1] <= rect.bottom() + 0.01, "y={} > bottom={}", v[1], rect.bottom());
        }
    }

    #[test]
    fn roundbox_radius_clamped() {
        // Radius larger than half the height should be clamped.
        let rect = Rect::new(0.0, 0.0, 100.0, 20.0);
        let verts = roundbox_vertices(rect, 50.0, RoundboxFlags::ALL);
        for v in &verts {
            assert!(v[1] >= -0.01);
            assert!(v[1] <= 20.01);
        }
    }
}
