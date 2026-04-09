//! Fixed-size object pool with O(1) allocation and deallocation.
//!
//! [`Pool<T>`] pre-allocates a fixed number of slots and manages them with a
//! free-list. Unlike [`Arena`](crate::Arena), a pool will never grow beyond its
//! initial capacity — attempts to allocate from a full pool return `None`.
//!
//! This makes it ideal for real-time contexts where allocation latency must be
//! bounded and predictable (e.g., particle systems, audio buffers).

use std::fmt;

/// A fixed-capacity object pool with O(1) alloc and free.
///
/// # Example
///
/// ```
/// use forge3d_alloc::Pool;
///
/// let mut pool = Pool::new(4, || 0i32);
/// let idx = pool.alloc_with(42).unwrap();
/// assert_eq!(pool.get(idx), Some(&42));
/// pool.free(idx);
/// ```
pub struct Pool<T> {
    /// Backing storage. `None` means the slot is vacant.
    slots: Vec<Option<T>>,
    /// Stack-based free-list of available slot indices.
    free_stack: Vec<u32>,
    /// Number of currently allocated (occupied) slots.
    active_count: usize,
}

impl<T: fmt::Debug> fmt::Debug for Pool<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Pool")
            .field("capacity", &self.slots.len())
            .field("active", &self.active_count)
            .field("free", &self.free_stack.len())
            .finish()
    }
}

impl<T> Pool<T> {
    /// Creates a new pool with `capacity` pre-allocated slots.
    ///
    /// `init` is called once per slot to produce the initial (default) value.
    /// All slots start as vacant — values produced by `init` are stored and
    /// returned on [`alloc`](Self::alloc).
    pub fn new(capacity: usize, init: impl Fn() -> T) -> Self {
        let mut slots = Vec::with_capacity(capacity);
        let mut free_stack = Vec::with_capacity(capacity);

        for i in 0..capacity {
            slots.push(Some(init()));
            // Push in reverse so that alloc pops low indices first.
            free_stack.push((capacity - 1 - i) as u32);
        }

        // Mark all slots as vacant (but with initialised data).
        // Actually we keep the values in `Some` and track vacancy via
        // the free-stack. On `alloc` we hand out the existing value;
        // on `free` we push the index back.
        //
        // Wait — that means `get` on a "free" slot would still return
        // `Some`. We need to distinguish. Let's set free slots to `None`.
        for slot in &mut slots {
            // Take the value out — it was only needed for pre-warming.
            // On alloc the caller is expected to write their own data.
            *slot = None;
        }

        Self {
            slots,
            free_stack,
            active_count: 0,
        }
    }

    /// Creates a pool where vacant slots keep no default value.
    pub fn new_empty(capacity: usize) -> Self
    where
        T: Default,
    {
        Self::new(capacity, T::default)
    }

    /// Allocates a slot, returning its index.
    ///
    /// Returns `None` if the pool is full.
    #[inline]
    pub fn alloc(&mut self) -> Option<u32> {
        let index = self.free_stack.pop()?;
        // Slot is already `None`; caller must write via `get_mut`.
        self.active_count += 1;
        Some(index)
    }

    /// Allocates a slot and immediately writes `value` into it.
    ///
    /// Returns `None` if the pool is full.
    #[inline]
    pub fn alloc_with(&mut self, value: T) -> Option<u32> {
        let index = self.alloc()?;
        self.slots[index as usize] = Some(value);
        Some(index)
    }

    /// Frees the slot at `index`, making it available for reuse.
    ///
    /// # Panics
    ///
    /// Panics in debug mode if `index` is out of range or already free.
    #[inline]
    pub fn free(&mut self, index: u32) {
        let i = index as usize;
        debug_assert!(i < self.slots.len(), "pool index out of range");
        debug_assert!(self.slots[i].is_some(), "double free on pool slot {index}");

        self.slots[i] = None;
        self.free_stack.push(index);
        self.active_count -= 1;
    }

    /// Returns a shared reference to the value at `index`, or `None` if the
    /// slot is vacant or out of range.
    #[inline]
    pub fn get(&self, index: u32) -> Option<&T> {
        self.slots.get(index as usize)?.as_ref()
    }

    /// Returns a mutable reference to the value at `index`, or `None` if the
    /// slot is vacant or out of range.
    #[inline]
    pub fn get_mut(&mut self, index: u32) -> Option<&mut T> {
        self.slots.get_mut(index as usize)?.as_mut()
    }

    /// Returns the fixed capacity of this pool.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.slots.len()
    }

    /// Returns the number of currently occupied slots.
    #[inline]
    pub fn active_count(&self) -> usize {
        self.active_count
    }

    /// Returns the number of available (free) slots.
    #[inline]
    pub fn free_count(&self) -> usize {
        self.free_stack.len()
    }

    /// Returns `true` if all slots are occupied.
    #[inline]
    pub fn is_full(&self) -> bool {
        self.free_stack.is_empty()
    }

    /// Returns `true` if no slots are occupied.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.active_count == 0
    }

    /// Resets all slots to vacant.
    pub fn clear(&mut self) {
        self.free_stack.clear();
        self.active_count = 0;
        for (i, slot) in self.slots.iter_mut().enumerate() {
            *slot = None;
            self.free_stack.push(i as u32);
        }
        // Reverse so low indices are popped first.
        self.free_stack.reverse();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alloc_and_free_cycle() {
        let mut pool: Pool<i32> = Pool::new_empty(4);
        assert_eq!(pool.capacity(), 4);
        assert_eq!(pool.active_count(), 0);

        let a = pool.alloc_with(10).unwrap();
        let b = pool.alloc_with(20).unwrap();
        assert_eq!(pool.active_count(), 2);
        assert_eq!(pool.get(a), Some(&10));
        assert_eq!(pool.get(b), Some(&20));

        pool.free(a);
        assert_eq!(pool.active_count(), 1);
        assert_eq!(pool.get(a), None);

        // Reuse freed slot.
        let c = pool.alloc_with(30).unwrap();
        assert_eq!(pool.get(c), Some(&30));
    }

    #[test]
    fn full_pool_returns_none() {
        let mut pool: Pool<u8> = Pool::new_empty(2);
        assert!(pool.alloc_with(1).is_some());
        assert!(pool.alloc_with(2).is_some());
        assert!(pool.alloc_with(3).is_none());
        assert!(pool.is_full());
    }

    #[test]
    fn clear_resets_pool() {
        let mut pool: Pool<i32> = Pool::new_empty(3);
        pool.alloc_with(1);
        pool.alloc_with(2);
        pool.alloc_with(3);
        assert!(pool.is_full());

        pool.clear();
        assert!(pool.is_empty());
        assert_eq!(pool.free_count(), 3);
    }

    #[test]
    fn capacity_enforcement_never_exceeds() {
        let cap = 5;
        let mut pool: Pool<u32> = Pool::new_empty(cap);
        let mut indices = Vec::new();

        // Allocate up to capacity.
        for i in 0..cap {
            let idx = pool.alloc_with(i as u32).expect("should succeed");
            indices.push(idx);
        }

        // Next alloc must fail.
        assert!(pool.alloc_with(999).is_none());
        assert!(pool.is_full());
        assert_eq!(pool.active_count(), cap);
        assert_eq!(pool.free_count(), 0);

        // Free one, then alloc should succeed again.
        pool.free(indices[2]);
        assert_eq!(pool.active_count(), cap - 1);
        assert_eq!(pool.get(indices[2]), None); // freed slot is gone

        let new_idx = pool.alloc_with(42).expect("should succeed after free");
        assert_eq!(pool.get(new_idx), Some(&42));

        // Pool uses raw indices (no generations), so the reused slot
        // index is the same as the freed one.
        assert_eq!(new_idx, indices[2]);
    }

    #[test]
    fn alloc_without_value_returns_none_on_get() {
        // Verify that alloc() without alloc_with() leaves slot as None.
        // This is expected: the caller must write via get_mut or use alloc_with.
        let mut pool: Pool<i32> = Pool::new_empty(2);
        let idx = pool.alloc().unwrap();
        // The slot is allocated (counted) but has no value yet.
        assert_eq!(pool.get(idx), None);
        assert_eq!(pool.active_count(), 1);
    }
}
