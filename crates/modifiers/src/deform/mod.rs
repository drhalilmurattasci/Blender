//! Deform modifiers: reshape vertex positions without changing topology.

use crate::common::{Modifier, ModifierFlags, ModifierType};
use crate::ModifierResult;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Simple Deform
// ---------------------------------------------------------------------------

/// Simple deform operations: twist, bend, taper, stretch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleDeformModifier {
    pub name: String,
    pub flags: ModifierFlags,
    /// Deform mode.
    pub mode: SimpleDeformMode,
    /// Deformation factor (angle in radians for twist/bend, factor for taper/stretch).
    pub factor: f32,
    /// Deform axis.
    pub axis: DeformAxis,
    /// Lower limit (0..1 range within the bounding box).
    pub limit_min: f32,
    /// Upper limit (0..1 range within the bounding box).
    pub limit_max: f32,
    /// Lock the axis orthogonal to deform.
    pub lock_x: bool,
    pub lock_y: bool,
}

/// Simple deform modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SimpleDeformMode {
    Twist,
    Bend,
    Taper,
    Stretch,
}

/// Deform axis selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeformAxis {
    X,
    Y,
    Z,
}

impl Default for SimpleDeformModifier {
    fn default() -> Self {
        Self {
            name: "Simple Deform".into(),
            flags: ModifierFlags::default(),
            mode: SimpleDeformMode::Twist,
            factor: 0.0,
            axis: DeformAxis::Z,
            limit_min: 0.0,
            limit_max: 1.0,
            lock_x: false,
            lock_y: false,
        }
    }
}

impl Modifier for SimpleDeformModifier {
    fn modifier_type(&self) -> ModifierType { ModifierType::SimpleDeform }
    fn name(&self) -> &str { &self.name }
    fn flags(&self) -> ModifierFlags { self.flags }
    fn set_flags(&mut self, flags: ModifierFlags) { self.flags = flags; }

    fn apply(&self, _mesh_data: &mut dyn std::any::Any) -> ModifierResult<()> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Smooth modifier
// ---------------------------------------------------------------------------

/// Laplacian or simple smooth.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmoothModifier {
    pub name: String,
    pub flags: ModifierFlags,
    /// Smoothing factor.
    pub factor: f32,
    /// Number of iterations.
    pub iterations: u32,
    /// Smooth along X.
    pub axis_x: bool,
    /// Smooth along Y.
    pub axis_y: bool,
    /// Smooth along Z.
    pub axis_z: bool,
}

impl Default for SmoothModifier {
    fn default() -> Self {
        Self {
            name: "Smooth".into(),
            flags: ModifierFlags::default(),
            factor: 0.5,
            iterations: 1,
            axis_x: true,
            axis_y: true,
            axis_z: true,
        }
    }
}

impl Modifier for SmoothModifier {
    fn modifier_type(&self) -> ModifierType { ModifierType::SmoothCorrectiveSmooth }
    fn name(&self) -> &str { &self.name }
    fn flags(&self) -> ModifierFlags { self.flags }
    fn set_flags(&mut self, flags: ModifierFlags) { self.flags = flags; }

    fn apply(&self, _mesh_data: &mut dyn std::any::Any) -> ModifierResult<()> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Shrinkwrap modifier
// ---------------------------------------------------------------------------

/// Projects vertices onto a target surface.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShrinkwrapModifier {
    pub name: String,
    pub flags: ModifierFlags,
    /// Wrap mode.
    pub wrap_mode: ShrinkwrapMode,
    /// Offset distance from the target surface.
    pub offset: f32,
    /// Projection axis (for Project mode).
    pub project_axis: DeformAxis,
    /// Whether to project in both directions.
    pub project_bidirectional: bool,
}

/// Shrinkwrap projection method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShrinkwrapMode {
    NearestSurfacePoint,
    Project,
    NearestVertex,
    TargetNormalProject,
}

impl Default for ShrinkwrapModifier {
    fn default() -> Self {
        Self {
            name: "Shrinkwrap".into(),
            flags: ModifierFlags::default(),
            wrap_mode: ShrinkwrapMode::NearestSurfacePoint,
            offset: 0.0,
            project_axis: DeformAxis::Z,
            project_bidirectional: false,
        }
    }
}

impl Modifier for ShrinkwrapModifier {
    fn modifier_type(&self) -> ModifierType { ModifierType::Shrinkwrap }
    fn name(&self) -> &str { &self.name }
    fn flags(&self) -> ModifierFlags { self.flags }
    fn set_flags(&mut self, flags: ModifierFlags) { self.flags = flags; }

    fn apply(&self, _mesh_data: &mut dyn std::any::Any) -> ModifierResult<()> {
        Ok(())
    }
}
