//! Pose channel: per-bone animation state.

use crate::{BoneIndex, RotationMode};
use serde::{Deserialize, Serialize};

/// A single bone's pose (animation) state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoseChannel {
    /// Bone name (must match the corresponding `Bone::name`).
    pub name: String,
    /// Bone index.
    pub bone_index: BoneIndex,

    // --- Local transform (relative to rest pose) ---
    /// Location offset from rest pose.
    pub location: [f32; 3],
    /// Rotation quaternion (x, y, z, w).
    pub rotation_quaternion: [f32; 4],
    /// Euler rotation (x, y, z) in radians.
    pub rotation_euler: [f32; 3],
    /// Axis-angle rotation (axis x, y, z, angle).
    pub rotation_axis_angle: [f32; 4],
    /// Scale factor.
    pub scale: [f32; 3],
    /// Rotation mode.
    pub rotation_mode: RotationMode,

    // --- Evaluated matrices (computed during evaluation) ---
    /// Pose-space matrix (bone-to-armature, after animation + constraints).
    #[serde(skip)]
    pub pose_matrix: [f32; 16],
    /// Channel matrix (local transform, before parent multiplication).
    #[serde(skip)]
    pub channel_matrix: [f32; 16],

    // --- Constraint stack ---
    /// Constraint names applied to this bone (actual constraints stored separately).
    pub constraint_names: Vec<String>,

    // --- IK settings ---
    /// IK stretch factor for this bone.
    pub ik_stretch: f32,
    /// Minimum IK rotation limit per axis.
    pub ik_min: [f32; 3],
    /// Maximum IK rotation limit per axis.
    pub ik_max: [f32; 3],
    /// Whether IK limits are enabled per axis.
    pub ik_limit: [bool; 3],
    /// IK stiffness per axis [0, 1].
    pub ik_stiffness: [f32; 3],

    /// Custom shape object name (for viewport display).
    pub custom_shape: Option<String>,
}

impl PoseChannel {
    /// Create a new identity pose channel.
    pub fn new(name: &str, bone_index: BoneIndex) -> Self {
        Self {
            name: name.to_string(),
            bone_index,
            location: [0.0; 3],
            rotation_quaternion: [0.0, 0.0, 0.0, 1.0],
            rotation_euler: [0.0; 3],
            rotation_axis_angle: [0.0, 0.0, 1.0, 0.0],
            scale: [1.0; 3],
            rotation_mode: RotationMode::Quaternion,
            pose_matrix: identity_4x4(),
            channel_matrix: identity_4x4(),
            constraint_names: Vec::new(),
            ik_stretch: 0.0,
            ik_min: [-std::f32::consts::PI; 3],
            ik_max: [std::f32::consts::PI; 3],
            ik_limit: [false; 3],
            ik_stiffness: [0.0; 3],
            custom_shape: None,
        }
    }

    /// Reset to identity transform.
    pub fn reset(&mut self) {
        self.location = [0.0; 3];
        self.rotation_quaternion = [0.0, 0.0, 0.0, 1.0];
        self.rotation_euler = [0.0; 3];
        self.rotation_axis_angle = [0.0, 0.0, 1.0, 0.0];
        self.scale = [1.0; 3];
        self.pose_matrix = identity_4x4();
        self.channel_matrix = identity_4x4();
    }

    /// Get the active rotation as a quaternion (converting from euler/axis-angle if needed).
    pub fn effective_rotation_quat(&self) -> [f32; 4] {
        match self.rotation_mode {
            RotationMode::Quaternion => self.rotation_quaternion,
            RotationMode::AxisAngle => {
                let [ax, ay, az, angle] = self.rotation_axis_angle;
                axis_angle_to_quat(ax, ay, az, angle)
            }
            RotationMode::EulerXYZ => euler_to_quat(self.rotation_euler, [0, 1, 2]),
            RotationMode::EulerXZY => euler_to_quat(self.rotation_euler, [0, 2, 1]),
            RotationMode::EulerYXZ => euler_to_quat(self.rotation_euler, [1, 0, 2]),
            RotationMode::EulerYZX => euler_to_quat(self.rotation_euler, [1, 2, 0]),
            RotationMode::EulerZXY => euler_to_quat(self.rotation_euler, [2, 0, 1]),
            RotationMode::EulerZYX => euler_to_quat(self.rotation_euler, [2, 1, 0]),
        }
    }
}

fn identity_4x4() -> [f32; 16] {
    [
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0,
    ]
}

fn axis_angle_to_quat(ax: f32, ay: f32, az: f32, angle: f32) -> [f32; 4] {
    let half = angle * 0.5;
    let s = half.sin();
    let len = (ax * ax + ay * ay + az * az).sqrt();
    if len < f32::EPSILON {
        return [0.0, 0.0, 0.0, 1.0];
    }
    [ax / len * s, ay / len * s, az / len * s, half.cos()]
}

/// Convert euler angles to quaternion with arbitrary axis order.
/// `order` specifies which euler component maps to which axis:
/// e.g., [0,1,2] for XYZ, [2,1,0] for ZYX.
/// The rotation is applied as: R_order[2] * R_order[1] * R_order[0].
fn euler_to_quat(euler: [f32; 3], order: [usize; 3]) -> [f32; 4] {
    // Build individual axis quaternions and multiply in the correct order.
    let quats: [[f32; 4]; 3] = [
        axis_rotation_quat(0, euler[0]), // X rotation
        axis_rotation_quat(1, euler[1]), // Y rotation
        axis_rotation_quat(2, euler[2]), // Z rotation
    ];

    // Multiply in reverse order: order[2] * order[1] * order[0]
    let q01 = quat_mul(quats[order[2]], quats[order[1]]);
    quat_mul(q01, quats[order[0]])
}

/// Create a quaternion for rotation around a single axis (0=X, 1=Y, 2=Z).
fn axis_rotation_quat(axis: usize, angle: f32) -> [f32; 4] {
    let half = angle * 0.5;
    let s = half.sin();
    let c = half.cos();
    match axis {
        0 => [s, 0.0, 0.0, c],
        1 => [0.0, s, 0.0, c],
        2 => [0.0, 0.0, s, c],
        _ => [0.0, 0.0, 0.0, 1.0],
    }
}

/// Multiply two quaternions: a * b. [x, y, z, w] layout.
fn quat_mul(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    let [ax, ay, az, aw] = a;
    let [bx, by, bz, bw] = b;
    [
        aw * bx + ax * bw + ay * bz - az * by,
        aw * by - ax * bz + ay * bw + az * bx,
        aw * bz + ax * by - ay * bx + az * bw,
        aw * bw - ax * bx - ay * by - az * bz,
    ]
}
