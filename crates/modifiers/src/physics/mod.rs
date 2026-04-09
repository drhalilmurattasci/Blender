//! Physics modifier stubs: cloth, collision, softbody, fluid.
//!
//! These modifiers act as containers for physics settings. The actual
//! simulation runs in a dedicated physics solver; these types hold the
//! parameters and integration hooks.

use crate::common::{Modifier, ModifierFlags, ModifierType};
use crate::ModifierResult;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Cloth modifier
// ---------------------------------------------------------------------------

/// Settings container for cloth simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClothModifier {
    pub name: String,
    pub flags: ModifierFlags,
    /// Structural stiffness.
    pub structural_stiffness: f32,
    /// Bending stiffness.
    pub bending_stiffness: f32,
    /// Mass per vertex.
    pub mass: f32,
    /// Air damping.
    pub air_damping: f32,
    /// Quality (substeps per frame).
    pub quality: u32,
    /// Gravity vector.
    pub gravity: [f32; 3],
    /// Collision distance.
    pub collision_distance: f32,
    /// Self-collision enabled.
    pub self_collision: bool,
}

impl Default for ClothModifier {
    fn default() -> Self {
        Self {
            name: "Cloth".into(),
            flags: ModifierFlags::default(),
            structural_stiffness: 15.0,
            bending_stiffness: 0.5,
            mass: 0.3,
            air_damping: 1.0,
            quality: 5,
            gravity: [0.0, 0.0, -9.81],
            collision_distance: 0.015,
            self_collision: false,
        }
    }
}

impl Modifier for ClothModifier {
    fn modifier_type(&self) -> ModifierType { ModifierType::Cloth }
    fn name(&self) -> &str { &self.name }
    fn flags(&self) -> ModifierFlags { self.flags }
    fn set_flags(&mut self, flags: ModifierFlags) { self.flags = flags; }

    fn apply(&self, _mesh_data: &mut dyn std::any::Any) -> ModifierResult<()> {
        // Physics simulation results are baked externally;
        // this apply step would read cached vertex positions.
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Collision modifier
// ---------------------------------------------------------------------------

/// Marks an object as a collision surface for cloth / particles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollisionModifier {
    pub name: String,
    pub flags: ModifierFlags,
    /// Outer collision shell thickness.
    pub outer_thickness: f32,
    /// Inner collision shell thickness.
    pub inner_thickness: f32,
    /// Damping factor for collisions.
    pub damping: f32,
    /// Friction factor.
    pub friction: f32,
}

impl Default for CollisionModifier {
    fn default() -> Self {
        Self {
            name: "Collision".into(),
            flags: ModifierFlags::default(),
            outer_thickness: 0.02,
            inner_thickness: 0.2,
            damping: 0.0,
            friction: 0.0,
        }
    }
}

impl Modifier for CollisionModifier {
    fn modifier_type(&self) -> ModifierType { ModifierType::Collision }
    fn name(&self) -> &str { &self.name }
    fn flags(&self) -> ModifierFlags { self.flags }
    fn set_flags(&mut self, flags: ModifierFlags) { self.flags = flags; }

    fn apply(&self, _mesh_data: &mut dyn std::any::Any) -> ModifierResult<()> {
        Ok(())
    }
}
