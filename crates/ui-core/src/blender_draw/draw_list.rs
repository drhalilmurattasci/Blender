//! Immediate-mode draw list for batching colored triangles and line segments.
//!
//! [`DrawList`] accumulates geometry that can be uploaded to the GPU in a single
//! draw call per primitive type (filled triangles and line segments).

use crate::painter::Rect;
use super::roundbox::{self, RoundboxFlags};

/// A single vertex with 2D position and RGBA color.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct DrawVertex {
    pub pos: [f32; 2],
    pub color: [u8; 4],
}

impl DrawVertex {
    pub const fn new(x: f32, y: f32, color: [u8; 4]) -> Self {
        Self {
            pos: [x, y],
            color,
        }
    }
}

/// Accumulated draw geometry for a frame or widget.
///
/// Filled geometry uses indexed triangles; line geometry uses indexed line
/// segments (pairs of indices).
#[derive(Debug, Clone, Default)]
pub struct DrawList {
    /// Vertices for filled triangles.
    pub vertices: Vec<DrawVertex>,
    /// Triangle indices (groups of 3).
    pub indices: Vec<u32>,
    /// Vertices for line segments.
    pub line_vertices: Vec<DrawVertex>,
    /// Line indices (pairs of 2).
    pub line_indices: Vec<u32>,
}

impl DrawList {
    pub fn new() -> Self {
        Self::default()
    }

    /// Merge another draw list into this one.
    pub fn append(&mut self, other: &DrawList) {
        let v_off = self.vertices.len() as u32;
        self.vertices.extend_from_slice(&other.vertices);
        self.indices.extend(other.indices.iter().map(|i| i + v_off));

        let lv_off = self.line_vertices.len() as u32;
        self.line_vertices.extend_from_slice(&other.line_vertices);
        self.line_indices
            .extend(other.line_indices.iter().map(|i| i + lv_off));
    }

    /// Add a filled rounded rectangle as a triangle fan.
    ///
    /// `color_top` and `color_bottom` define a vertical gradient; the color at
    /// each vertex is interpolated by its Y position within the rect.
    pub fn add_filled_roundbox(
        &mut self,
        rect: Rect,
        radius: f32,
        corners: RoundboxFlags,
        color_top: [u8; 4],
        color_bottom: [u8; 4],
    ) {
        let ring = roundbox::roundbox_vertices(rect, radius, corners);
        if ring.is_empty() {
            return;
        }

        let base = self.vertices.len() as u32;
        let y_min = rect.y;
        let y_max = rect.bottom();
        let y_range = (y_max - y_min).max(1.0);

        // Center vertex for fan.
        let cx = rect.x + rect.w * 0.5;
        let cy = rect.y + rect.h * 0.5;
        let center_color = lerp_color_u8(color_top, color_bottom, 0.5);
        self.vertices.push(DrawVertex::new(cx, cy, center_color));

        // Ring vertices with gradient color.
        for p in &ring {
            let t = ((p[1] - y_min) / y_range).clamp(0.0, 1.0);
            let c = lerp_color_u8(color_top, color_bottom, t);
            self.vertices.push(DrawVertex::new(p[0], p[1], c));
        }

        // Fan triangles: center, ring[i], ring[i+1].
        let n = ring.len() as u32;
        for i in 0..n {
            self.indices.push(base); // center
            self.indices.push(base + 1 + i);
            self.indices.push(base + 1 + (i + 1) % n);
        }
    }

    /// Add the outline of a rounded rectangle as line segments.
    pub fn add_outline_roundbox(
        &mut self,
        rect: Rect,
        radius: f32,
        corners: RoundboxFlags,
        color: [u8; 4],
    ) {
        let path = roundbox::roundbox_path(rect, radius, corners);
        if path.len() < 2 {
            return;
        }

        let base = self.line_vertices.len() as u32;
        for p in &path {
            self.line_vertices.push(DrawVertex::new(p[0], p[1], color));
        }

        let n = path.len() as u32;
        for i in 0..n {
            self.line_indices.push(base + i);
            self.line_indices.push(base + (i + 1) % n);
        }
    }

    /// Add Blender's emboss effect: a 1px highlight along the top edge and a
    /// 1px shadow along the bottom edge of the rounded rectangle.
    pub fn add_emboss(
        &mut self,
        rect: Rect,
        radius: f32,
        corners: RoundboxFlags,
        highlight: [u8; 4],
        shadow: [u8; 4],
    ) {
        let path = roundbox::roundbox_path(rect, radius, corners);
        if path.len() < 4 {
            return;
        }

        let mid_y = rect.y + rect.h * 0.5;

        // Top edge segments (y < mid).
        let base_top = self.line_vertices.len() as u32;
        let mut top_count = 0u32;
        for p in &path {
            if p[1] <= mid_y + 0.5 {
                self.line_vertices
                    .push(DrawVertex::new(p[0], p[1], highlight));
                top_count += 1;
            }
        }
        for i in 0..top_count.saturating_sub(1) {
            self.line_indices.push(base_top + i);
            self.line_indices.push(base_top + i + 1);
        }

        // Bottom edge segments (y > mid).
        let base_bot = self.line_vertices.len() as u32;
        let mut bot_count = 0u32;
        for p in &path {
            if p[1] >= mid_y - 0.5 {
                self.line_vertices
                    .push(DrawVertex::new(p[0], p[1], shadow));
                bot_count += 1;
            }
        }
        for i in 0..bot_count.saturating_sub(1) {
            self.line_indices.push(base_bot + i);
            self.line_indices.push(base_bot + i + 1);
        }
    }

    /// Add a single line segment.
    pub fn add_line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, color: [u8; 4]) {
        let base = self.line_vertices.len() as u32;
        self.line_vertices.push(DrawVertex::new(x0, y0, color));
        self.line_vertices.push(DrawVertex::new(x1, y1, color));
        self.line_indices.push(base);
        self.line_indices.push(base + 1);
    }

    /// Add a filled axis-aligned rectangle (two triangles, no rounding).
    pub fn add_rect_filled(&mut self, rect: Rect, color: [u8; 4]) {
        let base = self.vertices.len() as u32;
        let x0 = rect.x;
        let y0 = rect.y;
        let x1 = rect.right();
        let y1 = rect.bottom();

        self.vertices.push(DrawVertex::new(x0, y0, color));
        self.vertices.push(DrawVertex::new(x1, y0, color));
        self.vertices.push(DrawVertex::new(x1, y1, color));
        self.vertices.push(DrawVertex::new(x0, y1, color));

        self.indices.push(base);
        self.indices.push(base + 1);
        self.indices.push(base + 2);
        self.indices.push(base);
        self.indices.push(base + 2);
        self.indices.push(base + 3);
    }

    /// Total number of triangles in the filled geometry.
    pub fn tri_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// Total number of line segments.
    pub fn line_count(&self) -> usize {
        self.line_indices.len() / 2
    }

    /// True if no geometry has been added.
    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty() && self.line_vertices.is_empty()
    }

    /// Reset all buffers.
    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
        self.line_vertices.clear();
        self.line_indices.clear();
    }
}

/// Linearly interpolate between two RGBA u8 colors.
fn lerp_color_u8(a: [u8; 4], b: [u8; 4], t: f32) -> [u8; 4] {
    let t = t.clamp(0.0, 1.0);
    [
        (a[0] as f32 + (b[0] as f32 - a[0] as f32) * t) as u8,
        (a[1] as f32 + (b[1] as f32 - a[1] as f32) * t) as u8,
        (a[2] as f32 + (b[2] as f32 - a[2] as f32) * t) as u8,
        (a[3] as f32 + (b[3] as f32 - a[3] as f32) * t) as u8,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draw_vertex_layout() {
        // Ensure DrawVertex is 12 bytes: 2*f32 + 4*u8.
        assert_eq!(std::mem::size_of::<DrawVertex>(), 12);
    }

    #[test]
    fn add_rect_filled_produces_two_triangles() {
        let mut dl = DrawList::new();
        dl.add_rect_filled(Rect::new(0.0, 0.0, 10.0, 10.0), [255, 0, 0, 255]);
        assert_eq!(dl.tri_count(), 2);
        assert_eq!(dl.vertices.len(), 4);
    }

    #[test]
    fn add_line_produces_one_segment() {
        let mut dl = DrawList::new();
        dl.add_line(0.0, 0.0, 10.0, 10.0, [255, 255, 255, 255]);
        assert_eq!(dl.line_count(), 1);
    }

    #[test]
    fn append_offsets_indices() {
        let mut a = DrawList::new();
        a.add_rect_filled(Rect::new(0.0, 0.0, 5.0, 5.0), [255, 0, 0, 255]);

        let mut b = DrawList::new();
        b.add_rect_filled(Rect::new(10.0, 10.0, 5.0, 5.0), [0, 255, 0, 255]);

        let a_verts = a.vertices.len();
        a.append(&b);
        assert_eq!(a.vertices.len(), a_verts + b.vertices.len());
        // All indices in the appended part should be offset.
        assert!(a.indices.iter().all(|&i| (i as usize) < a.vertices.len()));
    }

    #[test]
    fn lerp_color_endpoints() {
        let a = [0, 100, 200, 255];
        let b = [255, 50, 0, 128];
        assert_eq!(lerp_color_u8(a, b, 0.0), a);
        assert_eq!(lerp_color_u8(a, b, 1.0), b);
    }

    #[test]
    fn filled_roundbox_produces_geometry() {
        let mut dl = DrawList::new();
        dl.add_filled_roundbox(
            Rect::new(0.0, 0.0, 100.0, 50.0),
            8.0,
            RoundboxFlags::ALL,
            [100, 100, 100, 255],
            [60, 60, 60, 255],
        );
        assert!(dl.tri_count() > 0);
    }

    #[test]
    fn outline_roundbox_produces_lines() {
        let mut dl = DrawList::new();
        dl.add_outline_roundbox(
            Rect::new(0.0, 0.0, 100.0, 50.0),
            8.0,
            RoundboxFlags::ALL,
            [200, 200, 200, 255],
        );
        assert!(dl.line_count() > 0);
    }

    #[test]
    fn clear_empties_all() {
        let mut dl = DrawList::new();
        dl.add_rect_filled(Rect::new(0.0, 0.0, 10.0, 10.0), [0; 4]);
        dl.add_line(0.0, 0.0, 1.0, 1.0, [0; 4]);
        dl.clear();
        assert!(dl.is_empty());
    }
}
