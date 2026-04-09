//! Collections group objects and nest hierarchically, analogous to Blender
//! collections or scene-graph groups.

use crate::object::ObjectHandle;
use forge3d_alloc::Handle;
use serde::{Deserialize, Serialize};

/// Handle to a collection inside the scene arena.
pub type CollectionHandle = Handle<Collection>;

/// Optional color tag for visual organization in the outliner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ColorTag {
    Red,
    Orange,
    Yellow,
    Green,
    Blue,
    Purple,
}

/// A named group that holds object references and child collections.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    /// Human-readable name.
    pub name: String,
    /// Child collections (nesting).
    pub children: Vec<CollectionHandle>,
    /// Direct object members.
    pub objects: Vec<ObjectHandle>,
    /// Parent collection (None for root).
    pub parent: Option<CollectionHandle>,
    /// Whether the collection is visible in the viewport.
    pub is_visible: bool,
    /// Whether objects in this collection can be selected.
    pub is_selectable: bool,
    /// Optional colour tag for outliner display.
    pub color_tag: Option<ColorTag>,
}

impl Collection {
    /// Create a new, empty collection.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            children: Vec::new(),
            objects: Vec::new(),
            parent: None,
            is_visible: true,
            is_selectable: true,
            color_tag: None,
        }
    }

    /// Add a child collection handle.
    pub fn add_child(&mut self, child: CollectionHandle) {
        if !self.children.contains(&child) {
            self.children.push(child);
        }
    }

    /// Add an object handle.
    pub fn add_object(&mut self, obj: ObjectHandle) {
        if !self.objects.contains(&obj) {
            self.objects.push(obj);
        }
    }

    /// Remove an object handle. Returns `true` if it was present.
    pub fn remove_object(&mut self, obj: ObjectHandle) -> bool {
        let before = self.objects.len();
        self.objects.retain(|&o| o != obj);
        self.objects.len() != before
    }
}
