//! # forge3d-armature
//!
//! Armature (skeleton) system for Forge3D.
//!
//! Provides bones, pose channels, IK solvers, armature evaluation,
//! and mesh deformation (skinning).

pub mod bone;
pub mod deform;
pub mod evaluate;
pub mod ik;
pub mod pose;

use thiserror::Error;

/// Errors from armature operations.
#[derive(Debug, Error)]
pub enum ArmatureError {
    #[error("bone `{0}` not found")]
    BoneNotFound(String),

    #[error("bone index {index} out of range (count: {count})")]
    BoneIndexOutOfRange { index: usize, count: usize },

    #[error("cyclic bone hierarchy detected at bone `{0}`")]
    CyclicHierarchy(String),

    #[error("IK solver failed: {0}")]
    IkSolverFailed(String),

    #[error("deformation error: {0}")]
    DeformError(String),
}

pub type ArmatureResult<T> = Result<T, ArmatureError>;

/// Rotation mode for bones and pose channels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum RotationMode {
    /// Quaternion (WXYZ).
    Quaternion,
    /// Euler XYZ.
    EulerXYZ,
    /// Euler XZY.
    EulerXZY,
    /// Euler YXZ.
    EulerYXZ,
    /// Euler YZX.
    EulerYZX,
    /// Euler ZXY.
    EulerZXY,
    /// Euler ZYX.
    EulerZYX,
    /// Axis-angle representation.
    AxisAngle,
}

impl Default for RotationMode {
    fn default() -> Self {
        Self::Quaternion
    }
}

/// Index of a bone within an armature.
pub type BoneIndex = u16;

/// Sentinel value for "no parent bone".
pub const NO_PARENT: BoneIndex = BoneIndex::MAX;
