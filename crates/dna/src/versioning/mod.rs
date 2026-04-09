//! Schema migration / versioning system.
//!
//! When a [`DnaSchema`] evolves (fields added, removed, renamed, or retyped),
//! a [`MigrationPlan`] describes how to transform old serialized data into the
//! new shape. Plans are built from a sequence of [`MigrationStep`]s and can be
//! applied to any [`DnaValue::Struct`].

use crate::schema::{DnaSchema, DnaType, DnaValue};

/// A single atomic transformation in a migration.
#[derive(Debug, Clone)]
pub enum MigrationStep {
    /// Add a new field with a default value.
    AddField {
        name: String,
        ty: DnaType,
        default: DnaValue,
    },
    /// Remove a field (its data is discarded).
    RemoveField {
        name: String,
    },
    /// Rename a field (preserving its value).
    RenameField {
        old_name: String,
        new_name: String,
    },
    /// Change a field's type, applying a conversion function.
    ChangeFieldType {
        name: String,
        new_ty: DnaType,
        convert: fn(&DnaValue) -> DnaValue,
    },
    /// Apply a custom transformation to the entire struct value.
    Custom {
        description: String,
        transform: fn(&mut DnaValue),
    },
}

/// A plan describing how to migrate a DNA value from `from_version` to
/// `to_version`.
///
/// # Example
///
/// ```
/// use forge3d_dna::{MigrationPlan, MigrationStep, DnaType, DnaValue};
///
/// let plan = MigrationPlan::new("MyStruct", 1, 2)
///     .step(MigrationStep::AddField {
///         name: "visible".into(),
///         ty: DnaType::Bool,
///         default: DnaValue::Bool(true),
///     })
///     .step(MigrationStep::RemoveField {
///         name: "obsolete_flag".into(),
///     });
///
/// assert_eq!(plan.step_count(), 2);
/// ```
#[derive(Debug, Clone)]
pub struct MigrationPlan {
    /// The schema name this plan applies to.
    pub schema_name: String,
    /// Source version.
    pub from_version: u32,
    /// Target version.
    pub to_version: u32,
    /// Ordered list of migration steps.
    steps: Vec<MigrationStep>,
}

/// Errors produced when applying a migration.
#[derive(Debug, Clone)]
pub enum MigrationError {
    /// The value is not a `DnaValue::Struct`.
    NotAStruct,
    /// A rename target already exists.
    FieldAlreadyExists(String),
    /// A type conversion produced an unexpected result.
    ConversionFailed(String),
}

impl std::fmt::Display for MigrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotAStruct => write!(f, "migration target is not a struct"),
            Self::FieldAlreadyExists(name) => {
                write!(f, "field already exists: {name}")
            }
            Self::ConversionFailed(msg) => write!(f, "conversion failed: {msg}"),
        }
    }
}

impl std::error::Error for MigrationError {}

impl MigrationPlan {
    /// Creates a new, empty migration plan.
    pub fn new(
        schema_name: impl Into<String>,
        from_version: u32,
        to_version: u32,
    ) -> Self {
        Self {
            schema_name: schema_name.into(),
            from_version,
            to_version,
            steps: Vec::new(),
        }
    }

    /// Builder: append a migration step.
    pub fn step(mut self, step: MigrationStep) -> Self {
        self.steps.push(step);
        self
    }

    /// Returns the number of steps in this plan.
    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    /// Returns a slice of all steps.
    pub fn steps(&self) -> &[MigrationStep] {
        &self.steps
    }

    /// Applies this migration plan to a [`DnaValue::Struct`] in place.
    pub fn apply(&self, value: &mut DnaValue) -> Result<(), MigrationError> {
        for step in &self.steps {
            match step {
                MigrationStep::AddField {
                    name,
                    default,
                    ..
                } => {
                    let map = Self::as_struct_mut(value)?;
                    if !map.contains_key(name.as_str()) {
                        map.insert(name.clone(), default.clone());
                    }
                }
                MigrationStep::RemoveField { name } => {
                    let map = Self::as_struct_mut(value)?;
                    map.shift_remove(name.as_str());
                }
                MigrationStep::RenameField { old_name, new_name } => {
                    let map = Self::as_struct_mut(value)?;
                    if map.contains_key(new_name.as_str()) {
                        return Err(MigrationError::FieldAlreadyExists(
                            new_name.clone(),
                        ));
                    }
                    if let Some(val) = map.shift_remove(old_name.as_str()) {
                        map.insert(new_name.clone(), val);
                    }
                }
                MigrationStep::ChangeFieldType {
                    name, convert, ..
                } => {
                    let map = Self::as_struct_mut(value)?;
                    if let Some(old_val) = map.get(name.as_str()) {
                        let new_val = convert(old_val);
                        map.insert(name.clone(), new_val);
                    }
                }
                MigrationStep::Custom { transform, .. } => {
                    transform(value);
                }
            }
        }
        Ok(())
    }

    fn as_struct_mut(
        value: &mut DnaValue,
    ) -> Result<&mut indexmap::IndexMap<String, DnaValue>, MigrationError> {
        match value {
            DnaValue::Struct(map) => Ok(map),
            _ => Err(MigrationError::NotAStruct),
        }
    }

    /// Automatically derives a migration plan by diffing two schemas.
    ///
    /// This handles the common cases:
    /// - Fields present in `new` but not `old` → `AddField`
    /// - Fields present in `old` but not `new` → `RemoveField`
    ///
    /// Renames and type changes cannot be auto-detected and must be specified
    /// manually.
    pub fn auto_diff(old: &DnaSchema, new: &DnaSchema) -> Self {
        let mut plan = Self::new(&new.name, old.version, new.version);

        let old_fields: std::collections::HashSet<&str> =
            old.fields.iter().map(|f| f.name.as_str()).collect();
        let new_fields: std::collections::HashSet<&str> =
            new.fields.iter().map(|f| f.name.as_str()).collect();

        // Fields added in new.
        for field in &new.fields {
            if !old_fields.contains(field.name.as_str()) {
                plan.steps.push(MigrationStep::AddField {
                    name: field.name.clone(),
                    ty: field.ty.clone(),
                    default: field
                        .default
                        .clone()
                        .unwrap_or_else(|| DnaValue::default_for_type(&field.ty)),
                });
            }
        }

        // Fields removed from old.
        for field in &old.fields {
            if !new_fields.contains(field.name.as_str()) {
                plan.steps.push(MigrationStep::RemoveField {
                    name: field.name.clone(),
                });
            }
        }

        plan
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::DnaField;
    use indexmap::IndexMap;

    #[test]
    fn add_field_migration() {
        let plan = MigrationPlan::new("Test", 1, 2).step(MigrationStep::AddField {
            name: "visible".into(),
            ty: DnaType::Bool,
            default: DnaValue::Bool(true),
        });

        let mut value = DnaValue::Struct({
            let mut m = IndexMap::new();
            m.insert("name".into(), DnaValue::String("obj".into()));
            m
        });

        plan.apply(&mut value).unwrap();

        if let DnaValue::Struct(map) = &value {
            assert_eq!(map.get("visible"), Some(&DnaValue::Bool(true)));
            assert_eq!(
                map.get("name"),
                Some(&DnaValue::String("obj".into()))
            );
        } else {
            panic!("expected struct");
        }
    }

    #[test]
    fn remove_field_migration() {
        let plan = MigrationPlan::new("Test", 1, 2)
            .step(MigrationStep::RemoveField {
                name: "obsolete".into(),
            });

        let mut value = DnaValue::Struct({
            let mut m = IndexMap::new();
            m.insert("obsolete".into(), DnaValue::I32(999));
            m.insert("keep".into(), DnaValue::Bool(true));
            m
        });

        plan.apply(&mut value).unwrap();

        if let DnaValue::Struct(map) = &value {
            assert!(!map.contains_key("obsolete"));
            assert!(map.contains_key("keep"));
        } else {
            panic!("expected struct");
        }
    }

    #[test]
    fn rename_field_migration() {
        let plan = MigrationPlan::new("Test", 1, 2)
            .step(MigrationStep::RenameField {
                old_name: "pos".into(),
                new_name: "position".into(),
            });

        let mut value = DnaValue::Struct({
            let mut m = IndexMap::new();
            m.insert("pos".into(), DnaValue::Vec3([1.0, 2.0, 3.0]));
            m
        });

        plan.apply(&mut value).unwrap();

        if let DnaValue::Struct(map) = &value {
            assert!(!map.contains_key("pos"));
            assert_eq!(
                map.get("position"),
                Some(&DnaValue::Vec3([1.0, 2.0, 3.0]))
            );
        } else {
            panic!("expected struct");
        }
    }

    #[test]
    fn auto_diff_detects_add_and_remove() {
        let old = DnaSchema::new("S", 1)
            .field(DnaField::new("a", DnaType::F32))
            .field(DnaField::new("b", DnaType::I32));

        let new = DnaSchema::new("S", 2)
            .field(DnaField::new("a", DnaType::F32))
            .field(DnaField::new("c", DnaType::Bool));

        let plan = MigrationPlan::auto_diff(&old, &new);
        assert_eq!(plan.from_version, 1);
        assert_eq!(plan.to_version, 2);
        assert_eq!(plan.step_count(), 2); // add c, remove b
    }

    #[test]
    fn change_field_type() {
        fn i32_to_f32(v: &DnaValue) -> DnaValue {
            match v {
                DnaValue::I32(n) => DnaValue::F32(*n as f64),
                other => other.clone(),
            }
        }

        let plan = MigrationPlan::new("Test", 1, 2)
            .step(MigrationStep::ChangeFieldType {
                name: "count".into(),
                new_ty: DnaType::F32,
                convert: i32_to_f32,
            });

        let mut value = DnaValue::Struct({
            let mut m = IndexMap::new();
            m.insert("count".into(), DnaValue::I32(42));
            m
        });

        plan.apply(&mut value).unwrap();

        if let DnaValue::Struct(map) = &value {
            assert_eq!(map.get("count"), Some(&DnaValue::F32(42.0)));
        } else {
            panic!("expected struct");
        }
    }
}
