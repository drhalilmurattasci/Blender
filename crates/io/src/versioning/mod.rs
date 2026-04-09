//! File versioning: tracks schema changes across Forge3D releases so
//! that older files can be migrated forward.

use crate::IoResult;

/// The current file-format version.
pub const CURRENT_VERSION: u32 = 1;

/// A migration step that upgrades data from one version to the next.
pub trait Migration: Send + Sync {
    /// The version this migration upgrades *from*.
    fn from_version(&self) -> u32;
    /// The version this migration upgrades *to*.
    fn to_version(&self) -> u32;
    /// Apply the migration to raw JSON data (used during import).
    fn apply(&self, data: &mut serde_json::Value) -> IoResult<()>;
    /// Human-readable description.
    fn description(&self) -> &str;
}

/// Registry of migrations, applied in order.
pub struct VersioningPipeline {
    migrations: Vec<Box<dyn Migration>>,
}

impl VersioningPipeline {
    /// Create an empty pipeline.
    pub fn new() -> Self {
        Self {
            migrations: Vec::new(),
        }
    }

    /// Register a migration step.
    pub fn register(&mut self, migration: Box<dyn Migration>) {
        self.migrations.push(migration);
        self.migrations.sort_by_key(|m| m.from_version());
    }

    /// Upgrade data from `file_version` to `CURRENT_VERSION`.
    pub fn upgrade(
        &self,
        data: &mut serde_json::Value,
        file_version: u32,
    ) -> IoResult<()> {
        let mut version = file_version;
        while version < CURRENT_VERSION {
            let migration = self
                .migrations
                .iter()
                .find(|m| m.from_version() == version)
                .ok_or_else(|| crate::IoError::VersionMismatch {
                    file_version: version,
                    expected_version: CURRENT_VERSION,
                })?;

            tracing::debug!(
                from = version,
                to = migration.to_version(),
                desc = migration.description(),
                "applying migration"
            );

            migration.apply(data)?;
            version = migration.to_version();
        }
        Ok(())
    }

    /// Number of registered migrations.
    pub fn len(&self) -> usize {
        self.migrations.len()
    }

    /// Whether the pipeline has no migrations.
    pub fn is_empty(&self) -> bool {
        self.migrations.is_empty()
    }
}

impl Default for VersioningPipeline {
    fn default() -> Self {
        Self::new()
    }
}
