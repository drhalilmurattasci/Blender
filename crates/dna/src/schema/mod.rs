//! DNA schema definitions: [`DnaType`], [`DnaField`], [`DnaSchema`], and
//! [`DnaValue`].
//!
//! A schema describes the runtime shape of a DNA struct — its fields, their
//! types, and default values. Schemas are registered in the global
//! [`DnaRegistry`](crate::registry::DnaRegistry) and used for serialization,
//! property-path access, and migration.

use std::fmt;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// Primitive and compound types representable in the DNA system.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DnaType {
    // -- Scalars --
    Bool,
    I32,
    I64,
    F32,
    F64,
    String,

    // -- Math (from forge3d-math) --
    Vec2,
    Vec3,
    Vec4,
    Mat4,
    Quat,
    Color4f,

    // -- Compound --
    /// An ordered list of homogeneous values.
    Array(Box<DnaType>),
    /// A reference to another named DNA struct.
    Struct(String),
    /// An optional value.
    Option(Box<DnaType>),
    /// An enum with named variants, each optionally carrying a type.
    Enum {
        name: String,
        variants: Vec<(String, Option<DnaType>)>,
    },
}

impl fmt::Display for DnaType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bool => write!(f, "bool"),
            Self::I32 => write!(f, "i32"),
            Self::I64 => write!(f, "i64"),
            Self::F32 => write!(f, "f32"),
            Self::F64 => write!(f, "f64"),
            Self::String => write!(f, "String"),
            Self::Vec2 => write!(f, "Vec2"),
            Self::Vec3 => write!(f, "Vec3"),
            Self::Vec4 => write!(f, "Vec4"),
            Self::Mat4 => write!(f, "Mat4"),
            Self::Quat => write!(f, "Quat"),
            Self::Color4f => write!(f, "Color4f"),
            Self::Array(inner) => write!(f, "[{inner}]"),
            Self::Struct(name) => write!(f, "{name}"),
            Self::Option(inner) => write!(f, "Option<{inner}>"),
            Self::Enum { name, .. } => write!(f, "enum {name}"),
        }
    }
}

/// A single field in a [`DnaSchema`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnaField {
    /// Field name (must be unique within the schema).
    pub name: String,
    /// The type of this field.
    pub ty: DnaType,
    /// Default value used when the field is missing during deserialization
    /// (e.g., after a schema migration adds a new field).
    pub default: Option<DnaValue>,
    /// Human-readable description (for editor tooltips, docs, etc.).
    pub description: Option<String>,
}

impl DnaField {
    /// Shorthand constructor.
    pub fn new(name: impl Into<String>, ty: DnaType) -> Self {
        Self {
            name: name.into(),
            ty,
            default: None,
            description: None,
        }
    }

    /// Builder: attach a default value.
    pub fn with_default(mut self, value: DnaValue) -> Self {
        self.default = Some(value);
        self
    }

    /// Builder: attach a description.
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
}

/// A concrete runtime value in the DNA type system.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DnaValue {
    Bool(bool),
    I32(i32),
    I64(i64),
    F32(f64), // stored as f64 for JSON round-trip fidelity
    F64(f64),
    String(String),

    // Math types serialized as arrays for JSON interop.
    Vec2([f64; 2]),
    Vec3([f64; 3]),
    Vec4([f64; 4]),
    Mat4([f64; 16]),
    Quat([f64; 4]),
    Color4f([f64; 4]),

    Array(Vec<DnaValue>),
    Struct(IndexMap<String, DnaValue>),
    Option(Option<Box<DnaValue>>),
    Enum {
        variant: String,
        value: Option<Box<DnaValue>>,
    },
    /// Represents an explicitly null / missing value.
    Null,
}

impl DnaValue {
    /// Returns `true` if this value is [`DnaValue::Null`].
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    /// Attempts to interpret this value as a `bool`.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(v) => Some(*v),
            _ => None,
        }
    }

    /// Attempts to interpret this value as an `f64`.
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::F32(v) | Self::F64(v) => Some(*v),
            Self::I32(v) => Some(*v as f64),
            Self::I64(v) => Some(*v as f64),
            _ => None,
        }
    }

    /// Attempts to interpret this value as a string slice.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    /// Returns a default [`DnaValue`] for the given [`DnaType`].
    pub fn default_for_type(ty: &DnaType) -> Self {
        match ty {
            DnaType::Bool => Self::Bool(false),
            DnaType::I32 => Self::I32(0),
            DnaType::I64 => Self::I64(0),
            DnaType::F32 => Self::F32(0.0),
            DnaType::F64 => Self::F64(0.0),
            DnaType::String => Self::String(String::new()),
            DnaType::Vec2 => Self::Vec2([0.0; 2]),
            DnaType::Vec3 => Self::Vec3([0.0; 3]),
            DnaType::Vec4 => Self::Vec4([0.0; 4]),
            DnaType::Mat4 => {
                // Identity matrix.
                let mut m = [0.0; 16];
                m[0] = 1.0;
                m[5] = 1.0;
                m[10] = 1.0;
                m[15] = 1.0;
                Self::Mat4(m)
            }
            DnaType::Quat => Self::Quat([0.0, 0.0, 0.0, 1.0]),
            DnaType::Color4f => Self::Color4f([0.0, 0.0, 0.0, 1.0]),
            DnaType::Array(_) => Self::Array(Vec::new()),
            DnaType::Struct(_) => Self::Struct(IndexMap::new()),
            DnaType::Option(_) => Self::Option(None),
            DnaType::Enum { variants, .. } => {
                if let Some((name, _)) = variants.first() {
                    Self::Enum {
                        variant: name.clone(),
                        value: None,
                    }
                } else {
                    Self::Null
                }
            }
        }
    }
}

/// Schema describing a DNA struct at runtime.
///
/// Schemas are typically registered once at startup via the global
/// [`DnaRegistry`](crate::registry::DnaRegistry).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnaSchema {
    /// Fully-qualified struct name (e.g., `"forge3d::scene::ObjectData"`).
    pub name: String,
    /// Schema version — incremented when fields change.
    pub version: u32,
    /// Ordered list of fields.
    pub fields: Vec<DnaField>,
}

impl DnaSchema {
    /// Creates a new schema with the given name and version.
    pub fn new(name: impl Into<String>, version: u32) -> Self {
        Self {
            name: name.into(),
            version,
            fields: Vec::new(),
        }
    }

    /// Builder: add a field.
    pub fn field(mut self, field: DnaField) -> Self {
        self.fields.push(field);
        self
    }

    /// Looks up a field by name.
    pub fn get_field(&self, name: &str) -> Option<&DnaField> {
        self.fields.iter().find(|f| f.name == name)
    }

    /// Returns an iterator over field names.
    pub fn field_names(&self) -> impl Iterator<Item = &str> {
        self.fields.iter().map(|f| f.name.as_str())
    }

    /// Constructs a [`DnaValue::Struct`] with all fields set to their defaults.
    pub fn default_value(&self) -> DnaValue {
        let mut map = IndexMap::new();
        for field in &self.fields {
            let value = field
                .default
                .clone()
                .unwrap_or_else(|| DnaValue::default_for_type(&field.ty));
            map.insert(field.name.clone(), value);
        }
        DnaValue::Struct(map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_default_value() {
        let schema = DnaSchema::new("TestStruct", 1)
            .field(DnaField::new("visible", DnaType::Bool).with_default(DnaValue::Bool(true)))
            .field(DnaField::new("name", DnaType::String))
            .field(DnaField::new("position", DnaType::Vec3));

        let val = schema.default_value();
        if let DnaValue::Struct(map) = &val {
            assert_eq!(map.get("visible"), Some(&DnaValue::Bool(true)));
            assert_eq!(
                map.get("name"),
                Some(&DnaValue::String(String::new()))
            );
            assert_eq!(map.get("position"), Some(&DnaValue::Vec3([0.0; 3])));
        } else {
            panic!("expected Struct");
        }
    }

    #[test]
    fn dna_type_display() {
        assert_eq!(DnaType::F32.to_string(), "f32");
        assert_eq!(DnaType::Array(Box::new(DnaType::Vec3)).to_string(), "[Vec3]");
        assert_eq!(
            DnaType::Option(Box::new(DnaType::String)).to_string(),
            "Option<String>"
        );
    }
}
