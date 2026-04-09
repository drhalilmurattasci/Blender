//! Armature evaluation pipeline.

use crate::bone::hierarchy::BoneHierarchy;
use crate::evaluate::Armature;
use crate::{ArmatureResult, BoneIndex, NO_PARENT};

/// Temporary data used during armature evaluation.
pub struct ArmatureEvalData {
    /// Evaluation order (topological sort of bones).
    pub eval_order: Vec<BoneIndex>,
}

impl ArmatureEvalData {
    /// Prepare evaluation data for an armature.
    pub fn prepare(armature: &Armature) -> ArmatureResult<Self> {
        let hierarchy = BoneHierarchy::new(&armature.bones);
        hierarchy.validate()?;
        let eval_order = hierarchy.topological_order();
        Ok(Self { eval_order })
    }
}

/// Evaluate an entire armature: compute all bone pose matrices.
///
/// This is the main entry point for the armature evaluation pipeline:
/// 1. For each bone in topological order:
///    a. Build the channel (local) matrix from the pose channel's animated transform.
///    b. Multiply by the parent's pose matrix to get the bone's pose matrix.
///    c. Apply constraints.
pub fn evaluate_armature(armature: &mut Armature, eval_data: &ArmatureEvalData) -> ArmatureResult<()> {
    for &bone_idx in &eval_data.eval_order {
        let idx = bone_idx as usize;
        if idx >= armature.bones.len() {
            continue;
        }

        // Build channel matrix from pose channel's TRS.
        // Blender: connected bones do not add their own location (BKE_pchan_to_mat4).
        let is_connected = armature.bones[idx].properties.connected
            && armature.bones[idx].parent != NO_PARENT;
        let channel_matrix = build_channel_matrix(&armature.pose.channels[idx], is_connected);
        armature.pose.channels[idx].channel_matrix = channel_matrix;

        // Multiply rest (bone-to-armature offset) with channel (local animation).
        // Blender: pose_mat = parent_pose * bone_offset * chan_mat
        // rest_matrix acts as the bone offset; channel_matrix is the local anim.
        let rest = &armature.bones[idx].rest_matrix;
        let local = mul_4x4(rest, &channel_matrix);

        // Multiply with parent's pose matrix.
        let parent_idx = armature.bones[idx].parent;
        let pose_matrix = if parent_idx != NO_PARENT {
            let parent = parent_idx as usize;
            let parent_pose = &armature.pose.channels[parent].pose_matrix;
            mul_4x4(parent_pose, &local)
        } else {
            local
        };

        armature.pose.channels[idx].pose_matrix = pose_matrix;
    }

    Ok(())
}

/// Build a 4x4 column-major matrix from a pose channel's location, rotation, and scale.
///
/// `connected`: if true, the bone is connected to its parent and should not
/// apply its own location (Blender's `BKE_pchan_to_mat4` skips loc for
/// connected bones with `BONE_CONNECTED` flag).
fn build_channel_matrix(channel: &crate::pose::PoseChannel, connected: bool) -> [f32; 16] {
    let [qx, qy, qz, qw] = channel.effective_rotation_quat();
    let [sx, sy, sz] = channel.scale;
    let [tx, ty, tz] = if connected {
        [0.0, 0.0, 0.0]
    } else {
        channel.location
    };

    // Quaternion to rotation matrix, combined with scale and translation.
    let xx = qx * qx;
    let yy = qy * qy;
    let zz = qz * qz;
    let xy = qx * qy;
    let xz = qx * qz;
    let yz = qy * qz;
    let wx = qw * qx;
    let wy = qw * qy;
    let wz = qw * qz;

    [
        (1.0 - 2.0 * (yy + zz)) * sx,
        (2.0 * (xy + wz)) * sx,
        (2.0 * (xz - wy)) * sx,
        0.0,
        (2.0 * (xy - wz)) * sy,
        (1.0 - 2.0 * (xx + zz)) * sy,
        (2.0 * (yz + wx)) * sy,
        0.0,
        (2.0 * (xz + wy)) * sz,
        (2.0 * (yz - wx)) * sz,
        (1.0 - 2.0 * (xx + yy)) * sz,
        0.0,
        tx,
        ty,
        tz,
        1.0,
    ]
}

/// Multiply two 4x4 column-major matrices.
fn mul_4x4(a: &[f32; 16], b: &[f32; 16]) -> [f32; 16] {
    let mut result = [0.0_f32; 16];
    for col in 0..4 {
        for row in 0..4 {
            let mut sum = 0.0;
            for k in 0..4 {
                sum += a[row + k * 4] * b[k + col * 4];
            }
            result[row + col * 4] = sum;
        }
    }
    result
}
