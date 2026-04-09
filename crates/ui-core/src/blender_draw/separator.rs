//! Separator line drawing.

use super::draw_list::DrawList;
use super::theme::general;

/// Draw a horizontal 1px separator line between `x1` and `x2` at vertical
/// position `y`, using the theme separator color.
pub fn draw_separator(x1: f32, x2: f32, y: f32) -> DrawList {
    let mut dl = DrawList::new();
    dl.add_line(x1, y, x2, y, general::SEPARATOR);
    dl
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn separator_produces_one_line() {
        let dl = draw_separator(0.0, 300.0, 50.0);
        assert_eq!(dl.line_count(), 1);
        assert!(dl.vertices.is_empty(), "no fill geometry");
    }
}
