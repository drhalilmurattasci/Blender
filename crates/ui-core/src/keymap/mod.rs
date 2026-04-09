//! Key-map: maps keyboard / mouse shortcuts to operator id-names.

use ahash::AHashMap;
use smallvec::SmallVec;

/// A single key binding entry.
#[derive(Debug, Clone)]
pub struct KeyBinding {
    /// Keyboard key name (e.g. "G", "S", "Delete").
    pub key: String,
    /// Modifier mask (shift, ctrl, alt, super).
    pub modifiers: u8,
    /// Operator id-name to invoke.
    pub operator_idname: String,
    /// Optional operator properties serialised as key=value pairs.
    pub properties: SmallVec<[(String, String); 2]>,
}

/// Modifier bit constants used by [`KeyBinding::modifiers`].
pub mod modifier {
    pub const SHIFT: u8 = 0b0001;
    pub const CTRL: u8 = 0b0010;
    pub const ALT: u8 = 0b0100;
    pub const SUPER: u8 = 0b1000;
}

/// A named keymap (e.g. "3D Viewport", "Outliner").
#[derive(Debug, Clone)]
pub struct KeyMap {
    /// Human-readable name.
    pub name: String,
    /// Bindings in priority order (first match wins).
    pub bindings: Vec<KeyBinding>,
}

impl KeyMap {
    /// Create a new, empty keymap.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            bindings: Vec::new(),
        }
    }

    /// Add a binding.
    pub fn add(&mut self, binding: KeyBinding) {
        self.bindings.push(binding);
    }

    /// Find the first binding that matches the given key and modifiers.
    pub fn find(&self, key: &str, modifiers: u8) -> Option<&KeyBinding> {
        self.bindings
            .iter()
            .find(|b| b.key == key && b.modifiers == modifiers)
    }
}

/// Global keymap registry: holds named keymaps for each editor context.
#[derive(Debug, Default)]
pub struct KeyMapRegistry {
    maps: AHashMap<String, KeyMap>,
}

impl KeyMapRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a keymap. Overwrites any existing keymap with the same name.
    pub fn register(&mut self, map: KeyMap) {
        self.maps.insert(map.name.clone(), map);
    }

    /// Look up a keymap by name.
    pub fn get(&self, name: &str) -> Option<&KeyMap> {
        self.maps.get(name)
    }

    /// Get a mutable keymap by name.
    pub fn get_mut(&mut self, name: &str) -> Option<&mut KeyMap> {
        self.maps.get_mut(name)
    }
}
