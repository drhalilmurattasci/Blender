//! Properties editor: tabbed panel showing context-sensitive properties
//! (active tool, render, output, scene, world, object, modifiers, particles,
//! physics, constraints, data, material, etc.).

/// Property panel tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PropertiesTab {
    ActiveTool,
    Render,
    Output,
    ViewLayer,
    Scene,
    World,
    Object,
    Modifiers,
    Particles,
    Physics,
    Constraints,
    ObjectData,
    Material,
}

/// Properties editor state.
pub struct PropertiesEditor {
    /// Currently active tab.
    pub active_tab: PropertiesTab,
    /// Scroll position within the current tab.
    pub scroll_y: f32,
    /// Whether the pin icon is active (locks context to the current object).
    pub pinned: bool,
}

impl PropertiesEditor {
    pub fn new() -> Self {
        Self {
            active_tab: PropertiesTab::Object,
            scroll_y: 0.0,
            pinned: false,
        }
    }

    /// Switch to a different tab, resetting scroll.
    pub fn set_tab(&mut self, tab: PropertiesTab) {
        self.active_tab = tab;
        self.scroll_y = 0.0;
    }
}

impl Default for PropertiesEditor {
    fn default() -> Self {
        Self::new()
    }
}
