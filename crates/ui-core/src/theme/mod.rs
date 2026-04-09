//! Theming: named colors, font sizes, and spacing values for the UI.

use crate::painter::Color;
use ahash::AHashMap;

/// Semantic color roles used by widgets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ThemeColor {
    /// Primary background.
    Background,
    /// Slightly lighter panel background.
    BackgroundPanel,
    /// Text color.
    Text,
    /// Dimmed / secondary text.
    TextSecondary,
    /// Widget background (buttons, fields).
    Widget,
    /// Widget background when hovered.
    WidgetHover,
    /// Widget background when pressed / active.
    WidgetActive,
    /// Widget outline / border.
    WidgetBorder,
    /// Selected / highlighted item.
    Select,
    /// Active accent color.
    Accent,
    /// Error / danger.
    Error,
    /// Warning.
    Warning,
    /// Success / info.
    Success,
    /// Header / toolbar background.
    Header,
    /// Scrollbar thumb.
    ScrollThumb,
}

/// A complete UI theme.
#[derive(Debug, Clone)]
pub struct Theme {
    /// Named color map.
    pub colors: AHashMap<ThemeColor, Color>,
    /// Default font size in pixels.
    pub font_size: f32,
    /// Default widget height in pixels.
    pub widget_height: f32,
    /// Default corner radius.
    pub corner_radius: f32,
    /// Default spacing between items.
    pub spacing: f32,
    /// Default padding inside widgets.
    pub padding: f32,
}

impl Theme {
    /// Create the default dark theme (similar to Blender's default).
    pub fn dark() -> Self {
        let mut colors = AHashMap::new();
        colors.insert(ThemeColor::Background, Color::new(0.188, 0.188, 0.188, 1.0));
        colors.insert(ThemeColor::BackgroundPanel, Color::new(0.212, 0.212, 0.212, 1.0));
        colors.insert(ThemeColor::Text, Color::new(0.9, 0.9, 0.9, 1.0));
        colors.insert(ThemeColor::TextSecondary, Color::new(0.6, 0.6, 0.6, 1.0));
        colors.insert(ThemeColor::Widget, Color::new(0.278, 0.278, 0.278, 1.0));
        colors.insert(ThemeColor::WidgetHover, Color::new(0.337, 0.337, 0.337, 1.0));
        colors.insert(ThemeColor::WidgetActive, Color::new(0.176, 0.420, 0.694, 1.0));
        colors.insert(ThemeColor::WidgetBorder, Color::new(0.098, 0.098, 0.098, 1.0));
        colors.insert(ThemeColor::Select, Color::new(0.227, 0.490, 0.749, 1.0));
        colors.insert(ThemeColor::Accent, Color::new(0.306, 0.604, 0.902, 1.0));
        colors.insert(ThemeColor::Error, Color::new(0.8, 0.2, 0.2, 1.0));
        colors.insert(ThemeColor::Warning, Color::new(0.85, 0.65, 0.15, 1.0));
        colors.insert(ThemeColor::Success, Color::new(0.2, 0.7, 0.3, 1.0));
        colors.insert(ThemeColor::Header, Color::new(0.165, 0.165, 0.165, 1.0));
        colors.insert(ThemeColor::ScrollThumb, Color::new(0.4, 0.4, 0.4, 0.6));

        Self {
            colors,
            font_size: 13.0,
            widget_height: 24.0,
            corner_radius: 4.0,
            spacing: 4.0,
            padding: 6.0,
        }
    }

    /// Look up a theme color.
    pub fn color(&self, role: ThemeColor) -> Color {
        self.colors.get(&role).copied().unwrap_or(Color::WHITE)
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}
