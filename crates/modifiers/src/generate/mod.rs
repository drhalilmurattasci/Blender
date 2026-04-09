//! Generate modifiers: create or replicate geometry.

use crate::common::{Modifier, ModifierFlags, ModifierType};
use crate::ModifierResult;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Array modifier
// ---------------------------------------------------------------------------

/// How the array modifier determines the number of copies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ArrayFitType {
    /// Fixed number of copies.
    FixedCount,
    /// Fit copies within a specified length.
    FitLength,
    /// Fit copies along a curve.
    FitCurve,
}

/// Replicates mesh geometry with configurable offset and merging.
///
/// Supports three modes via [`ArrayFitType`]:
///   - **FixedCount**: `count` copies (minimum 1).
///   - **FitLength**: `count = (fit_length + epsilon) / offset_distance + 1`.
///   - **FitCurve**: uses `curve_object` path length to determine count.
///
/// Offset is computed by combining (if enabled):
///   - **Relative offset**: scales by object bounding box.
///   - **Constant offset**: direct translation added to offset matrix.
///   - **Object offset**: another object's transform relative to this object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArrayModifier {
    pub name: String,
    pub flags: ModifierFlags,
    /// How to determine the number of copies.
    pub fit_type: ArrayFitType,
    /// Number of copies (including the original) — used when fit_type is FixedCount.
    pub count: u32,
    /// Target length — used when fit_type is FitLength.
    pub fit_length: f32,
    /// Curve object name — used when fit_type is FitCurve.
    pub curve_object: Option<String>,
    /// Constant offset (X, Y, Z) between copies.
    pub constant_offset: [f32; 3],
    /// Relative offset as a fraction of the bounding box (X, Y, Z).
    pub relative_offset: [f32; 3],
    /// Whether to use relative offset.
    pub use_relative_offset: bool,
    /// Whether to use constant offset.
    pub use_constant_offset: bool,
    /// Whether to use an object offset.
    pub use_object_offset: bool,
    /// Object name used for object offset.
    pub offset_object: Option<String>,
    /// Whether to merge vertices at boundaries between adjacent copies.
    pub merge_vertices: bool,
    /// Whether to merge vertices between the first and last copies.
    pub merge_last: bool,
    /// Merge distance threshold.
    pub merge_distance: f32,
    /// Start cap mesh object name.
    pub start_cap: Option<String>,
    /// End cap mesh object name.
    pub end_cap: Option<String>,
}

impl Default for ArrayModifier {
    fn default() -> Self {
        Self {
            name: "Array".into(),
            flags: ModifierFlags::default(),
            fit_type: ArrayFitType::FixedCount,
            count: 2,
            fit_length: 0.0,
            curve_object: None,
            constant_offset: [0.0, 0.0, 0.0],
            relative_offset: [1.0, 0.0, 0.0],
            use_relative_offset: true,
            use_constant_offset: false,
            use_object_offset: false,
            offset_object: None,
            merge_vertices: false,
            merge_last: false,
            merge_distance: 0.0001,
            start_cap: None,
            end_cap: None,
        }
    }
}

impl Modifier for ArrayModifier {
    fn modifier_type(&self) -> ModifierType { ModifierType::Array }
    fn name(&self) -> &str { &self.name }
    fn flags(&self) -> ModifierFlags { self.flags }
    fn set_flags(&mut self, flags: ModifierFlags) { self.flags = flags; }

    fn apply(&self, _mesh_data: &mut dyn std::any::Any) -> ModifierResult<()> {
        // Actual implementation would duplicate geometry `count` times
        // with computed offsets and optionally merge boundary vertices.
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Mirror modifier
// ---------------------------------------------------------------------------

/// Mirrors mesh geometry across one or more axes.
///
/// Each enabled axis triggers a sequential mirroring pass. Vertices near the
/// mirror plane (within `merge_threshold`) are merged to avoid duplicates.
/// Bisect options allow cutting the mesh at the mirror plane before mirroring.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MirrorModifier {
    pub name: String,
    pub flags: ModifierFlags,
    /// Mirror across X axis.
    pub axis_x: bool,
    /// Mirror across Y axis.
    pub axis_y: bool,
    /// Mirror across Z axis.
    pub axis_z: bool,
    /// Merge vertices along the mirror plane.
    pub merge: bool,
    /// Merge distance.
    pub merge_threshold: f32,
    /// Flip UVs across the mirror axis.
    pub flip_u: bool,
    pub flip_v: bool,
    /// Offset for UV mirroring.
    pub uv_offset_u: f32,
    pub uv_offset_v: f32,
    /// Flip UDIM tiles for UV mirroring.
    pub use_mirror_udim: bool,
    /// Bisect the mesh on X axis before mirroring.
    pub bisect_axis_x: bool,
    /// Bisect the mesh on Y axis before mirroring.
    pub bisect_axis_y: bool,
    /// Bisect the mesh on Z axis before mirroring.
    pub bisect_axis_z: bool,
    /// Flip the bisect direction on each axis.
    pub bisect_flip_x: bool,
    pub bisect_flip_y: bool,
    pub bisect_flip_z: bool,
    /// Bisect distance threshold.
    pub bisect_threshold: f32,
    /// Prevent vertices from crossing the mirror plane.
    pub use_clip: bool,
    /// Optional mirror object name for object-relative mirroring.
    pub mirror_object: Option<String>,
}

impl Default for MirrorModifier {
    fn default() -> Self {
        Self {
            name: "Mirror".into(),
            flags: ModifierFlags::default(),
            axis_x: true,
            axis_y: false,
            axis_z: false,
            merge: true,
            merge_threshold: 0.001,
            flip_u: false,
            flip_v: false,
            uv_offset_u: 0.0,
            uv_offset_v: 0.0,
            use_mirror_udim: false,
            bisect_axis_x: false,
            bisect_axis_y: false,
            bisect_axis_z: false,
            bisect_flip_x: false,
            bisect_flip_y: false,
            bisect_flip_z: false,
            bisect_threshold: 0.001,
            use_clip: false,
            mirror_object: None,
        }
    }
}

impl Modifier for MirrorModifier {
    fn modifier_type(&self) -> ModifierType { ModifierType::Mirror }
    fn name(&self) -> &str { &self.name }
    fn flags(&self) -> ModifierFlags { self.flags }
    fn set_flags(&mut self, flags: ModifierFlags) { self.flags = flags; }

    fn apply(&self, _mesh_data: &mut dyn std::any::Any) -> ModifierResult<()> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Subdivision Surface modifier
// ---------------------------------------------------------------------------

/// UV smoothing mode for subdivision surfaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SubdivUvSmooth {
    /// No UV smoothing.
    None,
    /// Keep corners only.
    KeepCorners,
    /// Keep corners and junctions.
    KeepCornersJunctions,
    /// Keep corners, junctions, and concave angles.
    KeepCornersJunctionsConcave,
    /// Keep boundaries.
    KeepBoundaries,
    /// Smooth all UVs.
    All,
}

/// Boundary smoothing mode for subdivision surfaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SubdivBoundarySmooth {
    /// Preserve all corners.
    All,
    /// Preserve sharp corners only.
    KeepSharp,
}

/// Catmull-Clark or simple subdivision.
///
/// Catmull-Clark weights (applied per subdivision level):
///   - **Face point** = average of all vertices in the face.
///   - **Edge point** = (average of edge endpoints + average of adjacent face points) / 2.
///   - **Vertex point** = (F + 2*R + (n-3)*P) / n, where:
///       - F = average of face points for faces touching this vertex,
///       - R = average of edge midpoints for edges touching this vertex,
///       - P = original vertex position,
///       - n = vertex valence (number of adjacent edges).
///
/// Resolution per level: (1 << level) + 1 vertices per edge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubdivisionModifier {
    pub name: String,
    pub flags: ModifierFlags,
    /// Subdivision levels for viewport display.
    pub levels_viewport: u8,
    /// Subdivision levels for render.
    pub levels_render: u8,
    /// Use simple (linear) subdivision instead of Catmull-Clark.
    pub use_simple: bool,
    /// Quality setting for OpenSubdiv.
    pub quality: u8,
    /// Use creases.
    pub use_creases: bool,
    /// UV smoothing mode — controls how UVs are interpolated during subdivision.
    pub uv_smooth: SubdivUvSmooth,
    /// Boundary smoothing — controls smoothing of open boundary edges.
    pub boundary_smooth: SubdivBoundarySmooth,
    /// Preserve custom normals.
    pub use_custom_normals: bool,
}

impl Default for SubdivisionModifier {
    fn default() -> Self {
        Self {
            name: "Subdivision Surface".into(),
            flags: ModifierFlags::default(),
            levels_viewport: 1,
            levels_render: 2,
            use_simple: false,
            quality: 3,
            use_creases: true,
            uv_smooth: SubdivUvSmooth::KeepCorners,
            boundary_smooth: SubdivBoundarySmooth::All,
            use_custom_normals: false,
        }
    }
}

impl Modifier for SubdivisionModifier {
    fn modifier_type(&self) -> ModifierType { ModifierType::Subdivision }
    fn name(&self) -> &str { &self.name }
    fn flags(&self) -> ModifierFlags { self.flags }
    fn set_flags(&mut self, flags: ModifierFlags) { self.flags = flags; }

    fn apply(&self, _mesh_data: &mut dyn std::any::Any) -> ModifierResult<()> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Solidify modifier
// ---------------------------------------------------------------------------

/// Adds thickness to a surface.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolidifyModifier {
    pub name: String,
    pub flags: ModifierFlags,
    /// Thickness of the solidified shell.
    pub thickness: f32,
    /// Offset (-1..1): -1 = inward, 0 = centered, 1 = outward.
    pub offset: f32,
    /// Use even thickness.
    pub use_even_offset: bool,
    /// Fill the rim (edges of the open boundary).
    pub fill_rim: bool,
    /// Material index offset for shell / rim.
    pub material_offset: i32,
    pub material_offset_rim: i32,
}

impl Default for SolidifyModifier {
    fn default() -> Self {
        Self {
            name: "Solidify".into(),
            flags: ModifierFlags::default(),
            thickness: 0.01,
            offset: -1.0,
            use_even_offset: true,
            fill_rim: true,
            material_offset: 0,
            material_offset_rim: 0,
        }
    }
}

impl Modifier for SolidifyModifier {
    fn modifier_type(&self) -> ModifierType { ModifierType::Solidify }
    fn name(&self) -> &str { &self.name }
    fn flags(&self) -> ModifierFlags { self.flags }
    fn set_flags(&mut self, flags: ModifierFlags) { self.flags = flags; }

    fn apply(&self, _mesh_data: &mut dyn std::any::Any) -> ModifierResult<()> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Bevel modifier
// ---------------------------------------------------------------------------

/// Bevels edges or vertices.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BevelModifier {
    pub name: String,
    pub flags: ModifierFlags,
    /// Bevel width.
    pub width: f32,
    /// Number of segments.
    pub segments: u32,
    /// Profile curvature (0.0 = concave, 0.5 = flat, 1.0 = convex).
    pub profile: f32,
    /// Limit method.
    pub limit_method: BevelLimitMethod,
    /// Angle threshold (radians) when limit_method is Angle.
    pub angle_limit: f32,
    /// Whether to bevel vertices instead of edges.
    pub vertex_only: bool,
    /// Harden normals along beveled faces.
    pub harden_normals: bool,
}

/// How to limit which edges get beveled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BevelLimitMethod {
    None,
    Angle,
    Weight,
    VertexGroup,
}

impl Default for BevelModifier {
    fn default() -> Self {
        Self {
            name: "Bevel".into(),
            flags: ModifierFlags::default(),
            width: 0.1,
            segments: 1,
            profile: 0.5,
            limit_method: BevelLimitMethod::None,
            angle_limit: 30.0_f32.to_radians(),
            vertex_only: false,
            harden_normals: false,
        }
    }
}

impl Modifier for BevelModifier {
    fn modifier_type(&self) -> ModifierType { ModifierType::Bevel }
    fn name(&self) -> &str { &self.name }
    fn flags(&self) -> ModifierFlags { self.flags }
    fn set_flags(&mut self, flags: ModifierFlags) { self.flags = flags; }

    fn apply(&self, _mesh_data: &mut dyn std::any::Any) -> ModifierResult<()> {
        Ok(())
    }
}
