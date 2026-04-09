//! 2D drawing / painting primitives for the UI layer.
//!
//! The [`Painter`] accumulates draw commands that are later flushed to the
//! GPU layer. This keeps the widget code free of direct GPU calls.

/// RGBA color (linear, premultiplied alpha).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const WHITE: Self = Self { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
    pub const BLACK: Self = Self { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const TRANSPARENT: Self = Self { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };

    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
}

/// A 2D rectangle defined by top-left corner and size.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    pub fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && px < self.x + self.w && py >= self.y && py < self.y + self.h
    }

    pub fn right(&self) -> f32 {
        self.x + self.w
    }

    pub fn bottom(&self) -> f32 {
        self.y + self.h
    }
}

/// A single draw command.
#[derive(Debug, Clone)]
pub enum DrawCommand {
    /// Filled rectangle.
    FillRect { rect: Rect, color: Color, corner_radius: f32 },
    /// Rectangle outline.
    StrokeRect { rect: Rect, color: Color, width: f32, corner_radius: f32 },
    /// Line from (x0,y0) to (x1,y1).
    Line { x0: f32, y0: f32, x1: f32, y1: f32, color: Color, width: f32 },
    /// Text string at position.
    Text { x: f32, y: f32, text: String, color: Color, size: f32 },
    /// Filled circle.
    FillCircle { cx: f32, cy: f32, radius: f32, color: Color },
    /// Push a clip rectangle (intersect with current clip).
    PushClip(Rect),
    /// Pop the most recent clip rectangle.
    PopClip,
}

/// Accumulates draw commands for the current frame.
#[derive(Debug, Default)]
pub struct Painter {
    commands: Vec<DrawCommand>,
}

impl Painter {
    /// Create a new, empty painter.
    pub fn new() -> Self {
        Self {
            commands: Vec::with_capacity(256),
        }
    }

    /// Draw a filled rectangle.
    pub fn fill_rect(&mut self, rect: Rect, color: Color, corner_radius: f32) {
        self.commands.push(DrawCommand::FillRect { rect, color, corner_radius });
    }

    /// Draw a rectangle outline.
    pub fn stroke_rect(&mut self, rect: Rect, color: Color, width: f32, corner_radius: f32) {
        self.commands.push(DrawCommand::StrokeRect { rect, color, width, corner_radius });
    }

    /// Draw a line.
    pub fn line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, color: Color, width: f32) {
        self.commands.push(DrawCommand::Line { x0, y0, x1, y1, color, width });
    }

    /// Draw text at a position.
    pub fn text(&mut self, x: f32, y: f32, text: impl Into<String>, color: Color, size: f32) {
        self.commands.push(DrawCommand::Text { x, y, text: text.into(), color, size });
    }

    /// Draw a filled circle.
    pub fn fill_circle(&mut self, cx: f32, cy: f32, radius: f32, color: Color) {
        self.commands.push(DrawCommand::FillCircle { cx, cy, radius, color });
    }

    /// Push a clip rectangle.
    pub fn push_clip(&mut self, rect: Rect) {
        self.commands.push(DrawCommand::PushClip(rect));
    }

    /// Pop the most recent clip rectangle.
    pub fn pop_clip(&mut self) {
        self.commands.push(DrawCommand::PopClip);
    }

    /// Take all accumulated commands, leaving the painter empty.
    pub fn drain(&mut self) -> Vec<DrawCommand> {
        std::mem::take(&mut self.commands)
    }

    /// Number of queued commands.
    pub fn command_count(&self) -> usize {
        self.commands.len()
    }
}
