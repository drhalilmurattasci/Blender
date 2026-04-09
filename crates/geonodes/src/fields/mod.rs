//! Fields: lazy, per-element evaluation of attributes.
//!
//! A [`Field`] represents a computation that can be evaluated for each
//! element in a domain (point, edge, face, corner, instance, etc.).

/// Element domain over which a field is evaluated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FieldDomain {
    Point,
    Edge,
    Face,
    Corner,
    Curve,
    /// Per-spline domain (each spline in a curve is one element).
    Spline,
    Instance,
}

/// The kind of data a field produces per element.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FieldDataType {
    Float,
    Int,
    Bool,
    Vector,
    Color,
}

/// A composable field node (lazy evaluation tree).
#[derive(Debug, Clone)]
pub enum Field {
    /// A constant value broadcast to all elements.
    Constant(FieldValue),
    /// Read a named attribute from the geometry.
    Attribute {
        name: String,
        domain: FieldDomain,
        data_type: FieldDataType,
    },
    /// Unary math operation on a child field.
    UnaryMath {
        op: UnaryMathOp,
        input: Box<Field>,
    },
    /// Binary math operation on two child fields.
    BinaryMath {
        op: BinaryMathOp,
        a: Box<Field>,
        b: Box<Field>,
    },
    /// Map range: remap a float field from [from_min, from_max] to [to_min, to_max].
    MapRange {
        value: Box<Field>,
        from_min: f32,
        from_max: f32,
        to_min: f32,
        to_max: f32,
    },
    /// Position built-in field.
    Position,
    /// Normal built-in field.
    Normal,
    /// Index built-in field.
    Index,
    /// Random value per element.
    Random { seed: i32, min: f32, max: f32 },
}

/// Concrete field value.
#[derive(Debug, Clone)]
pub enum FieldValue {
    Float(f32),
    Int(i32),
    Bool(bool),
    Vector([f32; 3]),
    Color([f32; 4]),
}

/// Unary math operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnaryMathOp {
    Negate,
    Abs,
    Sqrt,
    Floor,
    Ceil,
    Round,
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan,
    Exp,
    Log,
    Sign,
    Normalize,
    Length,
}

/// Binary math operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinaryMathOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Power,
    Min,
    Max,
    Modulo,
    Atan2,
    Dot,
    Cross,
    Distance,
    Compare,
}

impl Field {
    /// Create a constant float field.
    pub fn constant_float(value: f32) -> Self {
        Self::Constant(FieldValue::Float(value))
    }

    /// Create a constant vector field.
    pub fn constant_vector(value: [f32; 3]) -> Self {
        Self::Constant(FieldValue::Vector(value))
    }

    /// Create an attribute read field.
    pub fn attribute(name: impl Into<String>, domain: FieldDomain, data_type: FieldDataType) -> Self {
        Self::Attribute {
            name: name.into(),
            domain,
            data_type,
        }
    }

    /// Compose: add two fields.
    pub fn add(self, other: Field) -> Self {
        Self::BinaryMath {
            op: BinaryMathOp::Add,
            a: Box::new(self),
            b: Box::new(other),
        }
    }

    /// Compose: multiply two fields.
    pub fn multiply(self, other: Field) -> Self {
        Self::BinaryMath {
            op: BinaryMathOp::Multiply,
            a: Box::new(self),
            b: Box::new(other),
        }
    }

    /// Compose: negate a field.
    pub fn negate(self) -> Self {
        Self::UnaryMath {
            op: UnaryMathOp::Negate,
            input: Box::new(self),
        }
    }
}
