//! Generational arena — a contiguous collection addressed by [`Handle<T>`].
//!
//! The arena stores elements in a dense `Vec` and recycles freed slots via a
//! free-list. Each slot carries a generation counter so that stale handles are
//! safely rejected rather than silently aliasing a different element.

use crate::Handle;

/// Entry in the arena's backing storage.
#[derive(Debug)]
enum Slot<T> {
    /// An occupied slot holding a live value.
    Occupied { value: T, generation: u32 },
    /// A vacant slot that has been freed. `next_free` points to the next
    /// vacant slot (or `u32::MAX` if this is the tail of the free-list).
    Vacant { next_free: u32, generation: u32 },
}

/// A generational arena that maps [`Handle<T>`] → `T`.
///
/// # Complexity
///
/// | Operation | Time  |
/// |-----------|-------|
/// | insert    | O(1) amortised |
/// | remove    | O(1) |
/// | get / get_mut | O(1) |
///
/// # Example
///
/// ```
/// use forge3d_alloc::Arena;
///
/// let mut arena = Arena::new();
/// let h = arena.insert(42);
/// assert_eq!(arena.get(h), Some(&42));
/// arena.remove(h);
/// assert_eq!(arena.get(h), None);
/// ```
#[derive(Debug)]
pub struct Arena<T> {
    slots: Vec<Slot<T>>,
    /// Head of the free-list (`u32::MAX` when empty).
    free_head: u32,
    /// Number of currently occupied slots.
    len: usize,
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Arena<T> {
    /// Creates an empty arena.
    pub fn new() -> Self {
        Self {
            slots: Vec::new(),
            free_head: u32::MAX,
            len: 0,
        }
    }

    /// Creates an arena pre-allocated for at least `capacity` elements.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            slots: Vec::with_capacity(capacity),
            free_head: u32::MAX,
            len: 0,
        }
    }

    /// Returns the number of live elements.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the arena contains no live elements.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the total slot capacity (occupied + vacant).
    #[inline]
    pub fn capacity(&self) -> usize {
        self.slots.len()
    }

    /// Inserts a value and returns a [`Handle`] to it.
    pub fn insert(&mut self, value: T) -> Handle<T> {
        if self.free_head != u32::MAX {
            // Reuse a vacant slot.
            let index = self.free_head;
            let slot = &mut self.slots[index as usize];
            match slot {
                Slot::Vacant {
                    next_free,
                    generation,
                } => {
                    let current_gen = *generation;
                    self.free_head = *next_free;
                    *slot = Slot::Occupied {
                        value,
                        generation: current_gen,
                    };
                    self.len += 1;
                    Handle::new(index, current_gen)
                }
                Slot::Occupied { .. } => unreachable!("free-list pointed to an occupied slot"),
            }
        } else {
            // Grow.
            let index = self.slots.len() as u32;
            let generation = 0;
            self.slots.push(Slot::Occupied { value, generation });
            self.len += 1;
            Handle::new(index, generation)
        }
    }

    /// Removes the element addressed by `handle` and returns it, or `None` if
    /// the handle is stale / out of range.
    pub fn remove(&mut self, handle: Handle<T>) -> Option<T> {
        let index = handle.index() as usize;
        if index >= self.slots.len() {
            return None;
        }

        match &self.slots[index] {
            Slot::Occupied { generation, .. } if *generation == handle.generation() => {}
            _ => return None,
        }

        // Take the value out, converting the slot to Vacant.
        let old_slot = std::mem::replace(
            &mut self.slots[index],
            Slot::Vacant {
                next_free: self.free_head,
                // Bump generation so old handles become stale.
                generation: handle.generation() + 1,
            },
        );

        self.free_head = handle.index();
        self.len -= 1;

        match old_slot {
            Slot::Occupied { value, .. } => Some(value),
            Slot::Vacant { .. } => unreachable!(),
        }
    }

    /// Returns a shared reference to the element, or `None` if the handle is
    /// stale / out of range.
    #[inline]
    pub fn get(&self, handle: Handle<T>) -> Option<&T> {
        let index = handle.index() as usize;
        match self.slots.get(index)? {
            Slot::Occupied { value, generation } if *generation == handle.generation() => {
                Some(value)
            }
            _ => None,
        }
    }

    /// Returns an exclusive reference to the element, or `None` if the handle
    /// is stale / out of range.
    #[inline]
    pub fn get_mut(&mut self, handle: Handle<T>) -> Option<&mut T> {
        let index = handle.index() as usize;
        match self.slots.get_mut(index)? {
            Slot::Occupied { value, generation } if *generation == handle.generation() => {
                Some(value)
            }
            _ => None,
        }
    }

    /// Returns `true` if `handle` refers to a live element.
    #[inline]
    pub fn contains(&self, handle: Handle<T>) -> bool {
        self.get(handle).is_some()
    }

    /// Returns an iterator over `(Handle<T>, &T)` pairs for all live elements.
    pub fn iter(&self) -> ArenaIter<'_, T> {
        ArenaIter {
            slots: &self.slots,
            index: 0,
            remaining: self.len,
        }
    }

    /// Returns a mutable iterator over `(Handle<T>, &mut T)` pairs.
    pub fn iter_mut(&mut self) -> ArenaIterMut<'_, T> {
        let remaining = self.len;
        ArenaIterMut {
            slots: self.slots.iter_mut(),
            index: 0,
            remaining,
        }
    }

    /// Removes all elements, leaving the arena empty. Generations are preserved
    /// so that outstanding handles remain stale.
    pub fn clear(&mut self) {
        self.free_head = u32::MAX;
        self.len = 0;
        for (i, slot) in self.slots.iter_mut().enumerate() {
            if let Slot::Occupied { generation, .. } = slot {
                let next_gen = *generation + 1;
                *slot = Slot::Vacant {
                    next_free: self.free_head,
                    generation: next_gen,
                };
                self.free_head = i as u32;
            }
        }
    }
}

// --- Iterators ---

/// Shared iterator over live arena entries.
pub struct ArenaIter<'a, T> {
    slots: &'a [Slot<T>],
    index: usize,
    remaining: usize,
}

impl<'a, T> Iterator for ArenaIter<'a, T> {
    type Item = (Handle<T>, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.slots.len() && self.remaining > 0 {
            let i = self.index;
            self.index += 1;
            if let Slot::Occupied { value, generation } = &self.slots[i] {
                self.remaining -= 1;
                return Some((Handle::new(i as u32, *generation), value));
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<T> ExactSizeIterator for ArenaIter<'_, T> {}

/// Mutable iterator over live arena entries.
pub struct ArenaIterMut<'a, T> {
    slots: std::slice::IterMut<'a, Slot<T>>,
    index: usize,
    remaining: usize,
}

impl<'a, T> Iterator for ArenaIterMut<'a, T> {
    type Item = (Handle<T>, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        while self.remaining > 0 {
            let i = self.index;
            self.index += 1;
            let slot = self.slots.next()?;
            if let Slot::Occupied { value, generation } = slot {
                self.remaining -= 1;
                return Some((Handle::new(i as u32, *generation), value));
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<T> ExactSizeIterator for ArenaIterMut<'_, T> {}

// --- IntoIterator impls ---

impl<'a, T> IntoIterator for &'a Arena<T> {
    type Item = (Handle<T>, &'a T);
    type IntoIter = ArenaIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_get() {
        let mut arena = Arena::new();
        let h = arena.insert("hello");
        assert_eq!(arena.get(h), Some(&"hello"));
        assert_eq!(arena.len(), 1);
    }

    #[test]
    fn remove_invalidates_handle() {
        let mut arena = Arena::new();
        let h = arena.insert(10);
        assert_eq!(arena.remove(h), Some(10));
        assert_eq!(arena.get(h), None);
        assert!(arena.is_empty());
    }

    #[test]
    fn slot_reuse_bumps_generation() {
        let mut arena = Arena::new();
        let h1 = arena.insert(1);
        arena.remove(h1);
        let h2 = arena.insert(2);

        // Same index, different generation.
        assert_eq!(h1.index(), h2.index());
        assert_ne!(h1.generation(), h2.generation());

        // Old handle is stale.
        assert_eq!(arena.get(h1), None);
        assert_eq!(arena.get(h2), Some(&2));
    }

    #[test]
    fn iter_yields_live_elements() {
        let mut arena = Arena::new();
        let _h0 = arena.insert(0);
        let h1 = arena.insert(1);
        let _h2 = arena.insert(2);
        arena.remove(h1);

        let values: Vec<_> = arena.iter().map(|(_, v)| *v).collect();
        assert_eq!(values.len(), 2);
        assert!(values.contains(&0));
        assert!(values.contains(&2));
    }

    #[test]
    fn clear_invalidates_all_handles() {
        let mut arena = Arena::new();
        let h = arena.insert(42);
        arena.clear();
        assert!(arena.is_empty());
        assert_eq!(arena.get(h), None);
    }

    #[test]
    fn with_capacity_preallocates() {
        let arena = Arena::<i32>::with_capacity(128);
        assert!(arena.is_empty());
    }

    #[test]
    fn insert_remove_insert_old_handle_fails() {
        // This is the critical generational correctness test:
        // after insert -> remove -> insert, the OLD handle MUST fail on get().
        let mut arena = Arena::new();

        // Insert, get a handle.
        let old_handle = arena.insert(100);
        assert_eq!(arena.get(old_handle), Some(&100));

        // Remove.
        let removed = arena.remove(old_handle);
        assert_eq!(removed, Some(100));
        assert_eq!(arena.get(old_handle), None);

        // Insert again -- reuses same slot but with bumped generation.
        let new_handle = arena.insert(200);
        assert_eq!(arena.get(new_handle), Some(&200));

        // Old handle MUST fail.
        assert_eq!(
            arena.get(old_handle),
            None,
            "stale handle must not resolve after slot reuse"
        );
        // And they must have the same index but different generation.
        assert_eq!(old_handle.index(), new_handle.index());
        assert_ne!(old_handle.generation(), new_handle.generation());
    }

    #[test]
    fn iter_no_skip_no_double_visit() {
        let mut arena = Arena::new();
        let h0 = arena.insert(0);
        let _h1 = arena.insert(1);
        let h2 = arena.insert(2);
        let _h3 = arena.insert(3);
        let h4 = arena.insert(4);

        // Remove some elements to create holes.
        arena.remove(h0);
        arena.remove(h2);
        arena.remove(h4);

        // Remaining: 1, 3
        let collected: Vec<i32> = arena.iter().map(|(_, v)| *v).collect();
        assert_eq!(collected.len(), 2);
        assert!(collected.contains(&1));
        assert!(collected.contains(&3));

        // Verify no doubles.
        let mut seen = std::collections::HashSet::new();
        for (handle, _) in arena.iter() {
            assert!(
                seen.insert(handle.index()),
                "handle visited twice: index {}",
                handle.index()
            );
        }
    }

    #[test]
    fn iter_mut_visits_all_live_elements() {
        let mut arena = Arena::new();
        let _h0 = arena.insert(10);
        let h1 = arena.insert(20);
        let _h2 = arena.insert(30);

        arena.remove(h1);

        // Mutate all live elements.
        for (_, val) in arena.iter_mut() {
            *val *= 2;
        }

        let values: Vec<_> = arena.iter().map(|(_, v)| *v).collect();
        assert_eq!(values.len(), 2);
        assert!(values.contains(&20)); // 10 * 2
        assert!(values.contains(&60)); // 30 * 2
    }

    #[test]
    fn many_insert_remove_cycles() {
        let mut arena = Arena::new();
        let mut handles = Vec::new();

        // Do many cycles to stress the generation counter.
        for round in 0..100 {
            let h = arena.insert(round);
            handles.push(h);
        }

        // Remove every other.
        for i in (0..100).step_by(2) {
            arena.remove(handles[i]);
        }

        assert_eq!(arena.len(), 50);

        // Stale handles must not resolve.
        for i in (0..100).step_by(2) {
            assert_eq!(arena.get(handles[i]), None);
        }

        // Live handles must resolve.
        for i in (1..100).step_by(2) {
            assert_eq!(arena.get(handles[i]), Some(&i));
        }
    }
}
