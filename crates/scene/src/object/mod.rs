//! Scene object: the fundamental entity in the scene graph.
//!
//! An [`Object`] combines a [`Transform`], visibility state, and a
//! reference to an underlying data block (mesh, camera, light, etc.).

use crate::transform::Transform;
use crate::visibility::{SelectState, VisibilityFlags};
use forge3d_alloc::Handle;
use serde::{Deserialize, Serialize};

/// Handle to an object inside the scene arena.
pub type ObjectHandle = Handle<Object>;

/// What kind of data block this object wraps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ObjectType {
    /// Polygon / mesh data.
    Mesh,
    /// Skeletal armature.
    Armature,
    /// Camera.
    Camera,
    /// Point, spot, area, or sun light.
    Light,
    /// Empty / null object (transform only).
    Empty,
    /// Curve / spline data.
    Curve,
    /// Volume / OpenVDB data.
    Volume,
    /// Grease-pencil strokes.
    GreasePencil,
}

/// Opaque reference to the data block.
///
/// The actual data block lives in its own arena (mesh arena, armature arena,
/// etc.). We store a raw `u64` index here so the scene crate does not need
/// to be generic over every possible data type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DataBlockRef {
    /// Arena index into the owning subsystem.
    pub index: u64,
    /// Generation counter for stale-reference detection.
    pub generation: u32,
}

/// Discriminated union of data references an object can hold.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObjectData {
    /// No data yet.
    None,
    /// Mesh data-block reference.
    Mesh(DataBlockRef),
    /// Armature data-block reference.
    Armature(DataBlockRef),
    /// Camera settings stored inline (lightweight).
    Camera(CameraData),
    /// Light settings stored inline (lightweight).
    Light(LightData),
    /// Empty (no data).
    Empty,
}

/// Inline camera parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraData {
    pub focal_length: f32,
    pub sensor_width: f32,
    pub clip_near: f32,
    pub clip_far: f32,
    pub is_orthographic: bool,
    pub ortho_scale: f32,
}

impl Default for CameraData {
    fn default() -> Self {
        Self {
            focal_length: 50.0,
            sensor_width: 36.0,
            clip_near: 0.1,
            clip_far: 1000.0,
            is_orthographic: false,
            ortho_scale: 6.0,
        }
    }
}

/// Inline light parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightData {
    pub kind: LightKind,
    pub color: [f32; 3],
    pub energy: f32,
    pub radius: f32,
    pub spot_angle: f32,
    pub spot_blend: f32,
    pub cast_shadow: bool,
}

/// Light variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LightKind {
    Point,
    Sun,
    Spot,
    Area,
}

impl Default for LightData {
    fn default() -> Self {
        Self {
            kind: LightKind::Point,
            color: [1.0, 1.0, 1.0],
            energy: 10.0,
            radius: 0.1,
            spot_angle: 45.0_f32.to_radians(),
            spot_blend: 0.15,
            cast_shadow: true,
        }
    }
}

/// A scene object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Object {
    /// Human-readable name.
    pub name: String,
    /// Object type tag.
    pub object_type: ObjectType,
    /// Local transform (relative to parent).
    pub transform: Transform,
    /// Data-block union.
    pub data: ObjectData,
    /// Optional parent object.
    pub parent: Option<ObjectHandle>,
    /// Children object handles.
    pub children: Vec<ObjectHandle>,
    /// Visibility flags.
    pub visibility: VisibilityFlags,
    /// Selection state.
    pub select_state: SelectState,
}

impl Object {
    /// Create a new empty object.
    pub fn empty(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            object_type: ObjectType::Empty,
            transform: Transform::IDENTITY,
            data: ObjectData::Empty,
            parent: None,
            children: Vec::new(),
            visibility: VisibilityFlags::default(),
            select_state: SelectState::None,
        }
    }

    /// Create a mesh object (caller supplies the data-block reference separately).
    pub fn mesh(name: impl Into<String>, data_ref: DataBlockRef) -> Self {
        Self {
            name: name.into(),
            object_type: ObjectType::Mesh,
            data: ObjectData::Mesh(data_ref),
            ..Self::empty("")
        }
    }

    /// Create a camera object with default settings.
    pub fn camera(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            object_type: ObjectType::Camera,
            data: ObjectData::Camera(CameraData::default()),
            ..Self::empty("")
        }
    }

    /// Create a light object.
    pub fn light(name: impl Into<String>, kind: LightKind) -> Self {
        Self {
            name: name.into(),
            object_type: ObjectType::Light,
            data: ObjectData::Light(LightData {
                kind,
                ..LightData::default()
            }),
            ..Self::empty("")
        }
    }
}
