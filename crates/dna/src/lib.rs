//! # forge3d-dna
//!
//! DNA (Data Nucleus Architecture) is the runtime type-information and
//! serialization backbone of Forge3D.
//!
//! It provides:
//!
//! - **[`DnaSchema`] / [`DnaField`] / [`DnaType`]** — runtime descriptions of
//!   struct layouts, including field names, types, and defaults.
//!
//! - **[`DnaValue`]** — a dynamic value type that can represent any DNA-typed
//!   value at runtime (similar in spirit to `serde_json::Value` but with
//!   math-type awareness).
//!
//! - **[`DnaRegistry`]** — a global, thread-safe catalogue of all registered
//!   schemas, initialised via [`LazyLock`](std::sync::LazyLock).
//!
//! - **[`PropertyPath`]** — dot/bracket path expressions for reading and
//!   writing nested values (e.g., `"transform.position.x"`).
//!
//! - **[`MigrationPlan`] / [`MigrationStep`]** — versioned schema migration:
//!   add, remove, rename, or retype fields with automatic diff support.

pub mod derive;
pub mod registry;
pub mod schema;
pub mod versioning;

// Re-exports for ergonomic access.
pub use derive::{PropertyPath, PropertyPathError};
pub use registry::DnaRegistry;
pub use schema::{DnaField, DnaSchema, DnaType, DnaValue};
pub use versioning::{MigrationError, MigrationPlan, MigrationStep};
