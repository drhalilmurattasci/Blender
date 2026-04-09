//! Modify modifiers: alter attributes without changing topology.

use crate::common::{Modifier, ModifierFlags, ModifierType};
use crate::ModifierResult;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Triangulate
// ---------------------------------------------------------------------------

/// Converts all faces to triangles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriangulateModifier {
    pub name: String,
    pub flags: ModifierFlags,
    /// Quad triangulation method.
    pub quad_method: TriangulateQuadMethod,
    /// N-gon triangulation method.
    pub ngon_method: TriangulateNgonMethod,
    /// Minimum number of face vertices to triangulate (e.g. 4 = quads+).
    pub min_vertices: u32,
    /// Keep existing custom normals.
    pub keep_normals: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TriangulateQuadMethod {
    Beauty,
    Fixed,
    FixedAlternate,
    ShortestDiagonal,
    LongestDiagonal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TriangulateNgonMethod {
    Beauty,
    EarClip,
}

impl Default for TriangulateModifier {
    fn default() -> Self {
        Self {
            name: "Triangulate".into(),
            flags: ModifierFlags::default(),
            quad_method: TriangulateQuadMethod::ShortestDiagonal,
            ngon_method: TriangulateNgonMethod::Beauty,
            min_vertices: 4,
            keep_normals: true,
        }
    }
}

impl Modifier for TriangulateModifier {
    fn modifier_type(&self) -> ModifierType { ModifierType::Triangulate }
    fn name(&self) -> &str { &self.name }
    fn flags(&self) -> ModifierFlags { self.flags }
    fn set_flags(&mut self, flags: ModifierFlags) { self.flags = flags; }

    fn apply(&self, _mesh_data: &mut dyn std::any::Any) -> ModifierResult<()> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Decimate
// ---------------------------------------------------------------------------

/// Reduces polygon count.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecimateModifier {
    pub name: String,
    pub flags: ModifierFlags,
    /// Decimation mode.
    pub mode: DecimateMode,
    /// Ratio (0..1) for Collapse mode: 1.0 = no reduction.
    pub ratio: f32,
    /// Angle threshold (radians) for Planar mode.
    pub angle_limit: f32,
    /// Number of un-subdivisions for Un-Subdivide mode.
    pub iterations: u32,
    /// Vertex group name for weighting.
    pub vertex_group: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DecimateMode {
    Collapse,
    UnSubdivide,
    Planar,
}

impl Default for DecimateModifier {
    fn default() -> Self {
        Self {
            name: "Decimate".into(),
            flags: ModifierFlags::default(),
            mode: DecimateMode::Collapse,
            ratio: 1.0,
            angle_limit: 5.0_f32.to_radians(),
            iterations: 0,
            vertex_group: None,
        }
    }
}

impl Modifier for DecimateModifier {
    fn modifier_type(&self) -> ModifierType { ModifierType::Decimate }
    fn name(&self) -> &str { &self.name }
    fn flags(&self) -> ModifierFlags { self.flags }
    fn set_flags(&mut self, flags: ModifierFlags) { self.flags = flags; }

    fn apply(&self, _mesh_data: &mut dyn std::any::Any) -> ModifierResult<()> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Weighted Normal
// ---------------------------------------------------------------------------

/// Adjusts normals based on face area or corner angle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightedNormalModifier {
    pub name: String,
    pub flags: ModifierFlags,
    /// Weighting mode.
    pub mode: WeightedNormalMode,
    /// Weight factor.
    pub weight: f32,
    /// Keep existing sharp edges.
    pub keep_sharp: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WeightedNormalMode {
    FaceArea,
    CornerAngle,
    FaceAreaAndCornerAngle,
}

impl Default for WeightedNormalModifier {
    fn default() -> Self {
        Self {
            name: "Weighted Normal".into(),
            flags: ModifierFlags::default(),
            mode: WeightedNormalMode::FaceAreaAndCornerAngle,
            weight: 50.0,
            keep_sharp: true,
        }
    }
}

impl Modifier for WeightedNormalModifier {
    fn modifier_type(&self) -> ModifierType { ModifierType::WeightedNormal }
    fn name(&self) -> &str { &self.name }
    fn flags(&self) -> ModifierFlags { self.flags }
    fn set_flags(&mut self, flags: ModifierFlags) { self.flags = flags; }

    fn apply(&self, _mesh_data: &mut dyn std::any::Any) -> ModifierResult<()> {
        Ok(())
    }
}
