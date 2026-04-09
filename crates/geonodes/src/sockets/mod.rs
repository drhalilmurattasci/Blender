//! Socket types, values, and connection metadata.

/// Direction of a socket (input or output).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SocketDirection {
    Input,
    Output,
}

/// Data type carried by a socket.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SocketType {
    /// Single float.
    Float,
    /// 3-component vector.
    Vector,
    /// RGBA color.
    Color,
    /// Boolean.
    Bool,
    /// 32-bit signed integer.
    Int,
    /// Geometry data (mesh, curve, point cloud, etc.).
    Geometry,
    /// A string value.
    String,
    /// An object reference.
    Object,
    /// A collection reference.
    Collection,
    /// A material reference.
    Material,
    /// An image reference.
    Image,
    /// A texture reference.
    Texture,
}

/// Concrete value stored in or passed through a socket.
#[derive(Debug, Clone)]
pub enum SocketValue {
    Float(f32),
    Vector([f32; 3]),
    Color([f32; 4]),
    Bool(bool),
    Int(i32),
    String(std::string::String),
    /// Geometry is opaque at this level; the evaluator handles it.
    Geometry,
    /// No value (unconnected optional socket).
    None,
}

impl SocketValue {
    /// Returns the socket type tag for this value.
    pub fn socket_type(&self) -> SocketType {
        match self {
            Self::Float(_) => SocketType::Float,
            Self::Vector(_) => SocketType::Vector,
            Self::Color(_) => SocketType::Color,
            Self::Bool(_) => SocketType::Bool,
            Self::Int(_) => SocketType::Int,
            Self::String(_) => SocketType::String,
            Self::Geometry => SocketType::Geometry,
            Self::None => SocketType::Float, // default fallback
        }
    }

    /// Try to read as `f32`.
    pub fn as_float(&self) -> Option<f32> {
        match self {
            Self::Float(v) => Some(*v),
            Self::Int(v) => Some(*v as f32),
            Self::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
            _ => None,
        }
    }

    /// Try to read as `[f32; 3]`.
    pub fn as_vector(&self) -> Option<[f32; 3]> {
        match self {
            Self::Vector(v) => Some(*v),
            Self::Float(f) => Some([*f, *f, *f]),
            _ => None,
        }
    }
}

/// A socket definition on a node.
#[derive(Debug, Clone)]
pub struct Socket {
    /// Socket name (unique within the node).
    pub name: std::string::String,
    /// Direction.
    pub direction: SocketDirection,
    /// Data type.
    pub socket_type: SocketType,
    /// Default value (used when no link is connected).
    pub default_value: SocketValue,
    /// Whether this socket is hidden in the UI.
    pub hidden: bool,
    /// Minimum value (for numeric types).
    pub min: Option<f32>,
    /// Maximum value (for numeric types).
    pub max: Option<f32>,
}

impl Socket {
    /// Create an input socket.
    pub fn input(name: impl Into<std::string::String>, socket_type: SocketType, default: SocketValue) -> Self {
        Self {
            name: name.into(),
            direction: SocketDirection::Input,
            socket_type,
            default_value: default,
            hidden: false,
            min: None,
            max: None,
        }
    }

    /// Create an output socket.
    pub fn output(name: impl Into<std::string::String>, socket_type: SocketType) -> Self {
        Self {
            name: name.into(),
            direction: SocketDirection::Output,
            socket_type,
            default_value: SocketValue::None,
            hidden: false,
            min: None,
            max: None,
        }
    }
}
