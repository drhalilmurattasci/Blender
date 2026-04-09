//! Layout engine: arranges widgets in rows, columns, and grids.

/// Direction of a linear layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LayoutDirection {
    Horizontal,
    Vertical,
}

/// An item within a layout, contributing to size computation.
#[derive(Debug, Clone)]
pub struct LayoutItem {
    /// Minimum size (width, height).
    pub min_size: [f32; 2],
    /// Maximum size (width, height). `f32::INFINITY` means unconstrained.
    pub max_size: [f32; 2],
    /// Flex grow factor (0 = fixed size).
    pub flex: f32,
    /// Computed position after layout (x, y).
    pub position: [f32; 2],
    /// Computed size after layout (width, height).
    pub computed_size: [f32; 2],
}

impl Default for LayoutItem {
    fn default() -> Self {
        Self {
            min_size: [0.0, 0.0],
            max_size: [f32::INFINITY, f32::INFINITY],
            flex: 1.0,
            position: [0.0, 0.0],
            computed_size: [0.0, 0.0],
        }
    }
}

/// A layout container that arranges children linearly.
#[derive(Debug)]
pub struct Layout {
    pub direction: LayoutDirection,
    pub padding: f32,
    pub spacing: f32,
    pub items: Vec<LayoutItem>,
}

impl Layout {
    /// Create a new layout.
    pub fn new(direction: LayoutDirection) -> Self {
        Self {
            direction,
            padding: 4.0,
            spacing: 4.0,
            items: Vec::new(),
        }
    }

    /// Add an item and return its index.
    pub fn add(&mut self, item: LayoutItem) -> usize {
        let idx = self.items.len();
        self.items.push(item);
        idx
    }

    /// Compute positions and sizes given the available space.
    pub fn compute(&mut self, available_width: f32, available_height: f32) {
        if self.items.is_empty() {
            return;
        }

        let (main_avail, cross_avail) = match self.direction {
            LayoutDirection::Horizontal => (available_width, available_height),
            LayoutDirection::Vertical => (available_height, available_width),
        };

        let total_spacing = self.spacing * (self.items.len() as f32 - 1.0);
        let content_space = (main_avail - 2.0 * self.padding - total_spacing).max(0.0);

        // Sum flex factors.
        let total_flex: f32 = self.items.iter().map(|i| i.flex.max(0.0)).sum();
        let flex_unit = if total_flex > 0.0 {
            content_space / total_flex
        } else {
            0.0
        };

        let mut cursor = self.padding;
        for item in &mut self.items {
            let main_size = if item.flex > 0.0 {
                (flex_unit * item.flex)
                    .max(Self::main_min(item, self.direction))
                    .min(Self::main_max(item, self.direction))
            } else {
                Self::main_min(item, self.direction)
            };

            let cross_size = (cross_avail - 2.0 * self.padding)
                .max(0.0)
                .min(Self::cross_max(item, self.direction));

            match self.direction {
                LayoutDirection::Horizontal => {
                    item.position = [cursor, self.padding];
                    item.computed_size = [main_size, cross_size];
                }
                LayoutDirection::Vertical => {
                    item.position = [self.padding, cursor];
                    item.computed_size = [cross_size, main_size];
                }
            }

            cursor += main_size + self.spacing;
        }
    }

    fn main_min(item: &LayoutItem, dir: LayoutDirection) -> f32 {
        match dir {
            LayoutDirection::Horizontal => item.min_size[0],
            LayoutDirection::Vertical => item.min_size[1],
        }
    }

    fn main_max(item: &LayoutItem, dir: LayoutDirection) -> f32 {
        match dir {
            LayoutDirection::Horizontal => item.max_size[0],
            LayoutDirection::Vertical => item.max_size[1],
        }
    }

    fn cross_max(item: &LayoutItem, dir: LayoutDirection) -> f32 {
        match dir {
            LayoutDirection::Horizontal => item.max_size[1],
            LayoutDirection::Vertical => item.max_size[0],
        }
    }
}
