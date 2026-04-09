//! Node definitions: built-in geometry node types.

use crate::graph::NodeId;
use crate::sockets::Socket;

/// Category of a node in the add-node menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeCategory {
    Input,
    Output,
    Geometry,
    Mesh,
    MeshPrimitives,
    Curve,
    CurvePrimitives,
    Point,
    Volume,
    Instances,
    Material,
    Texture,
    Utilities,
    Math,
    Vector,
    Color,
    String,
    Layout,
}

/// Identifies a node's behaviour.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeType {
    // I/O
    GroupInput,
    GroupOutput,

    // Geometry
    JoinGeometry,
    TransformGeometry,
    SetPosition,
    SetMaterial,
    BoundingBox,
    ConvexHull,
    DeleteGeometry,
    DuplicateElements,
    SeparateGeometry,
    GeometryToInstance,
    RealizeInstances,

    // Mesh
    SubdivideMesh,
    ExtrudeMesh,
    FlipFaces,
    MeshBoolean,
    MeshToPoints,
    SplitEdges,
    TriangulateMesh,
    DualMesh,

    // Curve
    CurveToMesh,
    CurveToPoints,
    FillCurve,
    FilletCurve,
    ResampleCurve,
    ReverseCurve,
    SubdivideCurve,
    TrimCurve,
    SetCurveRadius,
    SetCurveTilt,
    SetHandleType,
    SetSplineType,
    SetSplineCyclic,
    SetSplineResolution,
    CurveLength,
    CurveHandlePositions,
    CurveTangent,

    // Curve Primitives
    CurveBezierSegment,
    CurveCircle,
    CurveLine,
    CurveQuadrilateral,
    CurveStar,
    CurveSpiral,

    // Mesh Primitives
    MeshCircle,
    MeshCone,
    MeshCube,
    MeshCylinder,
    MeshGrid,
    MeshIcoSphere,
    MeshUVSphere,
    MeshLine,

    // Math / Utilities
    MathNode,
    VectorMathNode,
    CompareNode,
    BooleanMathNode,
    MapRange,
    Clamp,
    Mix,
    Switch,

    // Input
    Position,
    Normal,
    Index,
    ID,
    NamedAttribute,
    Value,
    RandomValue,
    ObjectInfo,

    // Instances
    InstanceOnPoints,
    RotateInstances,
    ScaleInstances,
    TranslateInstances,

    // Attribute
    StoreNamedAttribute,
    CaptureAttribute,
    RemoveNamedAttribute,

    // Point
    DistributePointsOnFaces,
    PointsToVertices,
    SetPointRadius,

    // Volume
    MeshToVolume,
    VolumeToMesh,

    // Material
    SetMaterialIndex,
    ReplaceMaterial,

    // Texture
    NoiseTexture,
    VoronoiTexture,
    MusgraveTexture,
    GradientTexture,
    WaveTexture,
    WhiteNoiseTexture,

    // Color
    ColorRamp,
    CombineColor,
    SeparateColor,

    // Vector
    CombineXYZ,
    SeparateXYZ,

    // String
    StringJoin,
    StringToNumber,
    ValueToString,

    // Custom / unknown
    Custom(u32),
}

/// A single node in the graph.
#[derive(Debug, Clone)]
pub struct Node {
    /// Runtime ID (assigned by the graph).
    pub id: NodeId,
    /// Node type.
    pub node_type: NodeType,
    /// Human-readable label (can be user-renamed).
    pub label: String,
    /// Category (for the add-node menu).
    pub category: NodeCategory,
    /// Input sockets.
    pub inputs: Vec<Socket>,
    /// Output sockets.
    pub outputs: Vec<Socket>,
    /// 2D position in the node editor (x, y).
    pub location: [f32; 2],
    /// Width of the node in the editor.
    pub width: f32,
    /// Whether the node is muted (pass-through).
    pub muted: bool,
    /// Whether the node is collapsed in the UI.
    pub collapsed: bool,
}

impl Node {
    /// Create a new node with the given type and label. Sockets must be
    /// added separately via the builder methods.
    pub fn new(node_type: NodeType, label: impl Into<String>, category: NodeCategory) -> Self {
        Self {
            id: 0,
            node_type,
            label: label.into(),
            category,
            inputs: Vec::new(),
            outputs: Vec::new(),
            location: [0.0, 0.0],
            width: 140.0,
            muted: false,
            collapsed: false,
        }
    }

    /// Builder: add an input socket.
    pub fn with_input(mut self, socket: Socket) -> Self {
        self.inputs.push(socket);
        self
    }

    /// Builder: add an output socket.
    pub fn with_output(mut self, socket: Socket) -> Self {
        self.outputs.push(socket);
        self
    }

    /// Builder: set location.
    pub fn at(mut self, x: f32, y: f32) -> Self {
        self.location = [x, y];
        self
    }

    /// Find an input socket by name.
    pub fn find_input(&self, name: &str) -> Option<(usize, &Socket)> {
        self.inputs.iter().enumerate().find(|(_, s)| s.name == name)
    }

    /// Find an output socket by name.
    pub fn find_output(&self, name: &str) -> Option<(usize, &Socket)> {
        self.outputs.iter().enumerate().find(|(_, s)| s.name == name)
    }
}
