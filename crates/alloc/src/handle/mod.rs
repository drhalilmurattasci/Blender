//! Generational handle types for safe indirect references.
//!
//! A [`Handle<T>`] is a lightweight, copyable identifier that pairs an index
//! with a generation counter. This allows safe reuse of storage slots: after
//! an element is removed and its slot recycled, stale handles are detected
//! because their generation no longer matches.

use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

/// A generational index into an [`Arena<T>`](crate::Arena).
///
/// Handles are lightweight (8 bytes) and `Copy`. They do not borrow the arena,
/// so they can be freely stored, passed around, and compared.
///
/// A handle becomes *stale* once the element it pointed to has been removed and
/// the slot has been reused with a newer generation.
#[derive(Serialize, Deserialize)]
#[serde(bound = "")]
pub struct Handle<T> {
    /// Slot index in the backing storage.
    index: u32,
    /// Generation counter — incremented each time a slot is recycled.
    generation: u32,
    _marker: PhantomData<fn() -> T>,
}

// --- Manual trait impls (PhantomData prevents auto-derive) ---

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Handle<T> {}

impl<T> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && self.generation == other.generation
    }
}

impl<T> Eq for Handle<T> {}

impl<T> Hash for Handle<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index.hash(state);
        self.generation.hash(state);
    }
}

impl<T> fmt::Debug for Handle<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Handle<{}>({}, gen={})",
            std::any::type_name::<T>(),
            self.index,
            self.generation,
        )
    }
}

impl<T> fmt::Display for Handle<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_dangling() {
            write!(f, "Handle(dangling)")
        } else {
            write!(f, "Handle({}, gen={})", self.index, self.generation)
        }
    }
}

impl<T> Handle<T> {
    /// Creates a new handle from raw parts.
    ///
    /// This is only intended for use by the arena implementation.
    #[inline]
    pub(crate) fn new(index: u32, generation: u32) -> Self {
        Self {
            index,
            generation,
            _marker: PhantomData,
        }
    }

    /// Returns the slot index.
    #[inline]
    pub fn index(self) -> u32 {
        self.index
    }

    /// Returns the generation counter.
    #[inline]
    pub fn generation(self) -> u32 {
        self.generation
    }

    /// Returns a *dangling* handle that will never match any live entry.
    ///
    /// Useful as a default / sentinel value.
    #[inline]
    pub fn dangling() -> Self {
        Self::new(u32::MAX, 0)
    }

    /// Returns `true` if this handle is the dangling sentinel.
    #[inline]
    pub fn is_dangling(self) -> bool {
        self.index == u32::MAX
    }
}

impl<T> Default for Handle<T> {
    /// The default handle is [`Handle::dangling()`].
    fn default() -> Self {
        Self::dangling()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dangling_handle_is_detectable() {
        let h = Handle::<u32>::dangling();
        assert!(h.is_dangling());
        assert_eq!(h, Handle::default());
    }

    #[test]
    fn handles_with_different_generations_are_not_equal() {
        let a = Handle::<u32>::new(0, 1);
        let b = Handle::<u32>::new(0, 2);
        assert_ne!(a, b);
    }

    #[test]
    fn handle_is_copy() {
        let a = Handle::<u32>::new(1, 1);
        let b = a; // copy
        assert_eq!(a, b);
    }

    #[test]
    fn handle_display() {
        let h = Handle::<u32>::new(5, 3);
        let s = format!("{h}");
        assert!(s.contains("5"));
        assert!(s.contains("3"));

        let d = Handle::<u32>::dangling();
        let s = format!("{d}");
        assert!(s.contains("dangling"));
    }

    #[test]
    fn handle_hash_consistency() {
        use std::collections::HashSet;
        let h1 = Handle::<u32>::new(1, 1);
        let h2 = Handle::<u32>::new(1, 1);
        let mut set = HashSet::new();
        set.insert(h1);
        assert!(set.contains(&h2));
    }
}
