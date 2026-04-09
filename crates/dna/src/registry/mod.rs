//! Global DNA registry backed by [`LazyLock`].
//!
//! The registry is the central catalogue of all [`DnaSchema`]s known to the
//! engine. Schemas are registered once at startup (typically from static
//! initialisers or a plugin load hook) and can be looked up by name at any
//! point during the program's lifetime.

use std::sync::{LazyLock, RwLock};

use ahash::AHashMap;

use crate::schema::DnaSchema;

/// Global, lazily-initialised schema registry.
///
/// Uses a `RwLock` so that many readers can look up schemas concurrently while
/// registration (write) is exclusive. Registration only happens during startup,
/// so contention should be negligible in practice.
static GLOBAL_REGISTRY: LazyLock<RwLock<DnaRegistryInner>> =
    LazyLock::new(|| RwLock::new(DnaRegistryInner::new()));

#[derive(Debug)]
struct DnaRegistryInner {
    /// Schema name → schema.
    schemas: AHashMap<String, DnaSchema>,
}

impl DnaRegistryInner {
    fn new() -> Self {
        Self {
            schemas: AHashMap::new(),
        }
    }
}

/// The DNA registry — a process-wide catalogue of [`DnaSchema`]s.
///
/// All methods are static; the underlying storage is a global singleton.
///
/// # Example
///
/// ```
/// use forge3d_dna::{DnaRegistry, DnaSchema, DnaField, DnaType};
///
/// let schema = DnaSchema::new("MyStruct", 1)
///     .field(DnaField::new("x", DnaType::F32));
///
/// DnaRegistry::register(schema);
/// assert!(DnaRegistry::get("MyStruct").is_some());
/// ```
pub struct DnaRegistry;

impl DnaRegistry {
    /// Registers a schema. If a schema with the same name already exists it is
    /// replaced (useful for hot-reload / plugin updates).
    pub fn register(schema: DnaSchema) {
        let mut inner = GLOBAL_REGISTRY
            .write()
            .expect("DnaRegistry lock poisoned");
        inner.schemas.insert(schema.name.clone(), schema);
    }

    /// Registers multiple schemas at once.
    pub fn register_many(schemas: impl IntoIterator<Item = DnaSchema>) {
        let mut inner = GLOBAL_REGISTRY
            .write()
            .expect("DnaRegistry lock poisoned");
        for schema in schemas {
            inner.schemas.insert(schema.name.clone(), schema);
        }
    }

    /// Looks up a schema by name, returning a clone.
    ///
    /// Cloning is acceptable because schema lookup is infrequent (editor
    /// operations, serialization) rather than per-frame.
    pub fn get(name: &str) -> Option<DnaSchema> {
        let inner = GLOBAL_REGISTRY
            .read()
            .expect("DnaRegistry lock poisoned");
        inner.schemas.get(name).cloned()
    }

    /// Returns `true` if a schema with the given name is registered.
    pub fn contains(name: &str) -> bool {
        let inner = GLOBAL_REGISTRY
            .read()
            .expect("DnaRegistry lock poisoned");
        inner.schemas.contains_key(name)
    }

    /// Returns the names of all registered schemas (unordered).
    pub fn names() -> Vec<String> {
        let inner = GLOBAL_REGISTRY
            .read()
            .expect("DnaRegistry lock poisoned");
        inner.schemas.keys().cloned().collect()
    }

    /// Returns the total number of registered schemas.
    pub fn len() -> usize {
        let inner = GLOBAL_REGISTRY
            .read()
            .expect("DnaRegistry lock poisoned");
        inner.schemas.len()
    }

    /// Returns `true` if no schemas are registered.
    pub fn is_empty() -> bool {
        Self::len() == 0
    }

    /// Removes all registered schemas. Primarily useful for tests.
    pub fn clear() {
        let mut inner = GLOBAL_REGISTRY
            .write()
            .expect("DnaRegistry lock poisoned");
        inner.schemas.clear();
    }

    /// Executes a closure with shared access to a schema, avoiding a clone.
    ///
    /// Returns `None` if the schema is not found.
    pub fn with_schema<R>(name: &str, f: impl FnOnce(&DnaSchema) -> R) -> Option<R> {
        let inner = GLOBAL_REGISTRY
            .read()
            .expect("DnaRegistry lock poisoned");
        inner.schemas.get(name).map(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{DnaField, DnaType};

    fn make_test_schema(name: &str) -> DnaSchema {
        DnaSchema::new(name, 1).field(DnaField::new("value", DnaType::F32))
    }

    #[test]
    fn register_and_lookup() {
        DnaRegistry::clear();
        let schema = make_test_schema("test::RegisterLookup");
        DnaRegistry::register(schema);
        assert!(DnaRegistry::contains("test::RegisterLookup"));
        let s = DnaRegistry::get("test::RegisterLookup").unwrap();
        assert_eq!(s.version, 1);
    }

    #[test]
    fn register_replaces_existing() {
        DnaRegistry::clear();
        DnaRegistry::register(make_test_schema("test::Replace"));
        let updated = DnaSchema::new("test::Replace", 2)
            .field(DnaField::new("value", DnaType::F64));
        DnaRegistry::register(updated);

        let s = DnaRegistry::get("test::Replace").unwrap();
        assert_eq!(s.version, 2);
    }

    #[test]
    fn with_schema_avoids_clone() {
        DnaRegistry::clear();
        DnaRegistry::register(make_test_schema("test::WithSchema"));
        let field_count =
            DnaRegistry::with_schema("test::WithSchema", |s| s.fields.len());
        assert_eq!(field_count, Some(1));
    }
}
