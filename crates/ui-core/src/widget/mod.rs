//! Widget trait and common widget types.

use crate::input::InputState;
use crate::layout::LayoutItem;
use crate::painter::{Painter, Rect};
use crate::theme::Theme;

/// Unique widget identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WidgetId(pub u64);

/// The response a widget returns after processing input / drawing.
#[derive(Debug, Clone, Copy)]
pub struct WidgetResponse {
    /// Whether the mouse is hovering this widget.
    pub hovered: bool,
    /// Whether the widget was clicked this frame.
    pub clicked: bool,
    /// Whether the widget's value changed this frame.
    pub changed: bool,
    /// Whether this widget has keyboard focus.
    pub has_focus: bool,
}

impl WidgetResponse {
    /// A default "nothing happened" response.
    pub const NONE: Self = Self {
        hovered: false,
        clicked: false,
        changed: false,
        has_focus: false,
    };
}

/// Context passed to widgets during draw / interaction.
pub struct WidgetContext<'a> {
    pub theme: &'a Theme,
    pub input: &'a InputState,
    pub painter: &'a mut Painter,
}

/// Trait implemented by all UI widgets.
pub trait Widget {
    /// Return the preferred layout item for sizing.
    fn layout_item(&self) -> LayoutItem {
        LayoutItem::default()
    }

    /// Draw the widget into the given rect and handle input.
    fn draw(&mut self, rect: Rect, ctx: &mut WidgetContext<'_>) -> WidgetResponse;
}

// ---------------------------------------------------------------------------
// Concrete widgets
// ---------------------------------------------------------------------------

/// A simple push-button.
pub struct Button {
    pub label: String,
    pub id: WidgetId,
}

impl Button {
    pub fn new(id: WidgetId, label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            id,
        }
    }
}

impl Widget for Button {
    fn draw(&mut self, rect: Rect, ctx: &mut WidgetContext<'_>) -> WidgetResponse {
        use crate::theme::ThemeColor;

        let hovered = rect.contains(ctx.input.mouse_x as f32, ctx.input.mouse_y as f32);
        let active = hovered && ctx.input.left_held();
        let clicked = hovered && ctx.input.left_pressed();

        let bg = if active {
            ctx.theme.color(ThemeColor::WidgetActive)
        } else if hovered {
            ctx.theme.color(ThemeColor::WidgetHover)
        } else {
            ctx.theme.color(ThemeColor::Widget)
        };

        ctx.painter.fill_rect(rect, bg, ctx.theme.corner_radius);
        ctx.painter.text(
            rect.x + ctx.theme.padding,
            rect.y + (rect.h - ctx.theme.font_size) * 0.5,
            &self.label,
            ctx.theme.color(ThemeColor::Text),
            ctx.theme.font_size,
        );

        WidgetResponse {
            hovered,
            clicked,
            changed: false,
            has_focus: false,
        }
    }
}

/// A text label (non-interactive).
pub struct Label {
    pub text: String,
}

impl Label {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}

impl Widget for Label {
    fn draw(&mut self, rect: Rect, ctx: &mut WidgetContext<'_>) -> WidgetResponse {
        use crate::theme::ThemeColor;
        ctx.painter.text(
            rect.x,
            rect.y + (rect.h - ctx.theme.font_size) * 0.5,
            &self.text,
            ctx.theme.color(ThemeColor::Text),
            ctx.theme.font_size,
        );
        WidgetResponse::NONE
    }
}

/// A checkbox / toggle.
pub struct Checkbox {
    pub id: WidgetId,
    pub label: String,
    pub checked: bool,
}

impl Checkbox {
    pub fn new(id: WidgetId, label: impl Into<String>, checked: bool) -> Self {
        Self {
            id,
            label: label.into(),
            checked,
        }
    }
}

impl Widget for Checkbox {
    fn draw(&mut self, rect: Rect, ctx: &mut WidgetContext<'_>) -> WidgetResponse {
        use crate::painter::Color;
        use crate::theme::ThemeColor;

        let hovered = rect.contains(ctx.input.mouse_x as f32, ctx.input.mouse_y as f32);
        let mut changed = false;

        // Toggle on press (single-fire), not while held.
        if hovered && ctx.input.left_pressed() {
            self.checked = !self.checked;
            changed = true;
        }

        let box_size = ctx.theme.widget_height - 4.0;
        let box_rect = Rect::new(rect.x + 2.0, rect.y + 2.0, box_size, box_size);

        let bg = if self.checked {
            ctx.theme.color(ThemeColor::Accent)
        } else {
            ctx.theme.color(ThemeColor::Widget)
        };
        ctx.painter.fill_rect(box_rect, bg, 3.0);
        ctx.painter.stroke_rect(box_rect, ctx.theme.color(ThemeColor::WidgetBorder), 1.0, 3.0);

        if self.checked {
            // Draw a simple check mark as two lines.
            let cx = box_rect.x + box_size * 0.3;
            let cy = box_rect.y + box_size * 0.6;
            ctx.painter.line(cx, cy, cx - box_size * 0.1, cy - box_size * 0.15, Color::WHITE, 2.0);
            ctx.painter.line(cx, cy, cx + box_size * 0.3, cy - box_size * 0.35, Color::WHITE, 2.0);
        }

        ctx.painter.text(
            rect.x + box_size + 8.0,
            rect.y + (rect.h - ctx.theme.font_size) * 0.5,
            &self.label,
            ctx.theme.color(ThemeColor::Text),
            ctx.theme.font_size,
        );

        WidgetResponse {
            hovered,
            clicked: changed,
            changed,
            has_focus: false,
        }
    }
}

/// A numeric slider.
pub struct Slider {
    pub id: WidgetId,
    pub label: String,
    pub value: f32,
    pub min: f32,
    pub max: f32,
}

impl Slider {
    pub fn new(id: WidgetId, label: impl Into<String>, value: f32, min: f32, max: f32) -> Self {
        Self {
            id,
            label: label.into(),
            value,
            min,
            max,
        }
    }

    fn normalized(&self) -> f32 {
        if (self.max - self.min).abs() < f32::EPSILON {
            0.0
        } else {
            ((self.value - self.min) / (self.max - self.min)).clamp(0.0, 1.0)
        }
    }
}

impl Widget for Slider {
    fn draw(&mut self, rect: Rect, ctx: &mut WidgetContext<'_>) -> WidgetResponse {
        use crate::theme::ThemeColor;

        let hovered = rect.contains(ctx.input.mouse_x as f32, ctx.input.mouse_y as f32);
        let mut changed = false;

        // Interactive dragging.
        if hovered && ctx.input.left_held() {
            let t = ((ctx.input.mouse_x as f32 - rect.x) / rect.w).clamp(0.0, 1.0);
            let new_val = self.min + t * (self.max - self.min);
            if (new_val - self.value).abs() > f32::EPSILON {
                self.value = new_val;
                changed = true;
            }
        }

        // Background track.
        ctx.painter.fill_rect(rect, ctx.theme.color(ThemeColor::Widget), ctx.theme.corner_radius);

        // Filled portion.
        let fill_w = rect.w * self.normalized();
        let fill_rect = Rect::new(rect.x, rect.y, fill_w, rect.h);
        ctx.painter.fill_rect(fill_rect, ctx.theme.color(ThemeColor::Accent), ctx.theme.corner_radius);

        // Label + value.
        let display = format!("{}: {:.2}", self.label, self.value);
        ctx.painter.text(
            rect.x + ctx.theme.padding,
            rect.y + (rect.h - ctx.theme.font_size) * 0.5,
            &display,
            ctx.theme.color(ThemeColor::Text),
            ctx.theme.font_size,
        );

        WidgetResponse {
            hovered,
            clicked: false,
            changed,
            has_focus: false,
        }
    }
}
