//! Inverse Kinematics solvers.

mod ccd;
mod fabrik;

pub use ccd::CcdSolver;
pub use fabrik::FabrikSolver;

/// IK solver configuration.
#[derive(Debug, Clone)]
pub struct IkSettings {
    /// Maximum number of solver iterations.
    pub max_iterations: u32,
    /// Convergence tolerance (distance from target).
    pub tolerance: f32,
    /// Chain length (number of bones from the tip toward the root).
    pub chain_length: u16,
    /// Whether the chain is allowed to stretch.
    pub allow_stretch: bool,
    /// Stretch limit factor.
    pub stretch_limit: f32,
    /// Optional pole target position. When set, the solver orients the chain
    /// plane so that the first bone in the chain points toward this position.
    pub pole_target: Option<[f32; 3]>,
    /// Pole angle offset (radians) applied after pole target alignment.
    pub pole_angle: f32,
}

impl Default for IkSettings {
    fn default() -> Self {
        Self {
            max_iterations: 10,
            tolerance: 0.001,
            chain_length: 0, // 0 = entire chain to root.
            allow_stretch: false,
            stretch_limit: 1.0,
            pole_target: None,
            pole_angle: 0.0,
        }
    }
}

/// Trait for IK solvers.
pub trait IkSolver {
    /// Solve the IK chain for the given target position.
    ///
    /// `joint_positions` is a mutable slice of joint positions (3D) from root to tip.
    /// `bone_lengths` contains the length of each bone segment.
    /// `target` is the desired position of the end effector.
    ///
    /// Returns the number of iterations performed.
    fn solve(
        &self,
        joint_positions: &mut [[f32; 3]],
        bone_lengths: &[f32],
        target: [f32; 3],
        settings: &IkSettings,
    ) -> u32;
}
