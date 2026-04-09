//! # forge3d-alloc
//!
//! Memory allocation primitives for the Forge3D engine.
//!
//! This crate provides three core allocation strategies:
//!
//! - **[`Arena<T>`]** — A generational arena addressed by [`Handle<T>`].
//!   Supports O(1) insert, remove, and lookup with automatic stale-handle
//!   detection via generation counters.
//!
//! - **[`Pool<T>`]** — A fixed-capacity object pool with O(1) alloc/free and
//!   zero heap growth at runtime. Ideal for real-time workloads with bounded
//!   element counts (particles, audio buffers, etc.).
//!
//! - **[`ScratchArena`]** — A bump allocator (backed by `bumpalo`) for
//!   per-frame temporary data. Reset in O(1) at frame boundaries.

pub mod arena;
pub mod handle;
pub mod pool;
pub mod scratch;

// Re-exports for ergonomic access.
pub use arena::Arena;
pub use handle::Handle;
pub use pool::Pool;
pub use scratch::ScratchArena;
