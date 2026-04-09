//! Scratch allocator — a thin wrapper around [`bumpalo::Bump`] for per-frame
//! temporary allocations.
//!
//! The scratch allocator is ideal for short-lived data that is allocated during
//! a frame and discarded at the end (e.g., intermediate geometry, command
//! lists, transient strings).
//!
//! Call [`ScratchArena::reset`] at frame boundaries to reclaim all memory in
//! O(1) without running destructors.

use bumpalo::Bump;

/// A resettable bump allocator for temporary per-frame data.
///
/// # Example
///
/// ```
/// use forge3d_alloc::ScratchArena;
///
/// let mut scratch = ScratchArena::new();
/// let v = scratch.alloc(42);
/// assert_eq!(*v, 42);
/// scratch.reset(); // O(1), reclaims all memory
/// ```
pub struct ScratchArena {
    bump: Bump,
}

impl Default for ScratchArena {
    fn default() -> Self {
        Self::new()
    }
}

impl ScratchArena {
    /// Creates a new scratch allocator.
    pub fn new() -> Self {
        Self { bump: Bump::new() }
    }

    /// Creates a scratch allocator with at least `capacity` bytes pre-allocated.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            bump: Bump::with_capacity(capacity),
        }
    }

    /// Allocates a single value in the scratch arena.
    #[inline]
    pub fn alloc<T>(&self, value: T) -> &mut T {
        self.bump.alloc(value)
    }

    /// Allocates a slice by copying from `src`.
    #[inline]
    pub fn alloc_slice_copy<T: Copy>(&self, src: &[T]) -> &mut [T] {
        self.bump.alloc_slice_copy(src)
    }

    /// Allocates a slice by cloning from `src`.
    #[inline]
    pub fn alloc_slice_clone<T: Clone>(&self, src: &[T]) -> &mut [T] {
        self.bump.alloc_slice_clone(src)
    }

    /// Allocates a string slice.
    #[inline]
    pub fn alloc_str(&self, s: &str) -> &mut str {
        self.bump.alloc_str(s)
    }

    /// Resets the allocator, reclaiming all memory in O(1).
    ///
    /// **Warning:** This does *not* run destructors on allocated values.
    /// Only use this allocator for `Copy` types or types where leaking is
    /// acceptable.
    #[inline]
    pub fn reset(&mut self) {
        self.bump.reset();
    }

    /// Returns the total number of bytes currently allocated in this arena
    /// (including alignment padding and bookkeeping overhead).
    #[inline]
    pub fn allocated_bytes(&self) -> usize {
        self.bump.allocated_bytes()
    }

    /// Provides direct access to the underlying `bumpalo::Bump` allocator for
    /// advanced use cases (e.g., `bumpalo::collections::Vec`).
    #[inline]
    pub fn inner(&self) -> &Bump {
        &self.bump
    }
}

impl std::fmt::Debug for ScratchArena {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScratchArena")
            .field("allocated_bytes", &self.allocated_bytes())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alloc_and_reset() {
        let mut scratch = ScratchArena::new();
        let a = scratch.alloc(1u32);
        let b = scratch.alloc(2u32);
        assert_eq!(*a, 1);
        assert_eq!(*b, 2);

        scratch.reset();
        // After reset we can allocate again; old references are invalidated
        // (but we don't hold them).
        let c = scratch.alloc(3u32);
        assert_eq!(*c, 3);
    }

    #[test]
    fn alloc_slice() {
        let scratch = ScratchArena::new();
        let data = scratch.alloc_slice_copy(&[1, 2, 3, 4]);
        assert_eq!(data, &[1, 2, 3, 4]);
    }

    #[test]
    fn alloc_str() {
        let scratch = ScratchArena::new();
        let s = scratch.alloc_str("hello");
        assert_eq!(s, "hello");
    }

    #[test]
    fn with_capacity() {
        let scratch = ScratchArena::with_capacity(4096);
        assert!(scratch.allocated_bytes() == 0 || scratch.allocated_bytes() > 0);
        // Just verifying it doesn't panic.
    }
}
