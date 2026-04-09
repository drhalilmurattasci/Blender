//! FABRIK (Forward And Backward Reaching Inverse Kinematics) solver.

use crate::ik::{IkSettings, IkSolver};

/// FABRIK solver.
///
/// A fast heuristic IK solver that alternates between forward and backward
/// reaching passes to converge on the target position.
#[derive(Debug, Clone)]
pub struct FabrikSolver;

impl IkSolver for FabrikSolver {
    fn solve(
        &self,
        joint_positions: &mut [[f32; 3]],
        bone_lengths: &[f32],
        target: [f32; 3],
        settings: &IkSettings,
    ) -> u32 {
        let n = joint_positions.len();
        if n < 2 || bone_lengths.len() < n - 1 {
            return 0;
        }

        let root = joint_positions[0];
        let end_idx = n - 1;

        // Check if target is reachable.
        let total_length: f32 = bone_lengths.iter().sum();
        let root_to_target = distance(root, target);

        if root_to_target > total_length && !settings.allow_stretch {
            // Target is unreachable: stretch toward it.
            for i in 0..end_idx {
                let dir = normalize(sub(target, joint_positions[i]));
                joint_positions[i + 1] = add(joint_positions[i], scale(dir, bone_lengths[i]));
            }
            return 1;
        }

        let mut iterations_done = settings.max_iterations;
        for iteration in 0..settings.max_iterations {
            // Check convergence.
            let dist = distance(joint_positions[end_idx], target);
            if dist < settings.tolerance {
                iterations_done = iteration;
                break;
            }

            // --- Forward reaching (from end to root) ---
            joint_positions[end_idx] = target;
            for i in (0..end_idx).rev() {
                let dir = normalize(sub(joint_positions[i], joint_positions[i + 1]));
                joint_positions[i] = add(joint_positions[i + 1], scale(dir, bone_lengths[i]));
            }

            // --- Backward reaching (from root to end) ---
            joint_positions[0] = root;
            for i in 0..end_idx {
                let dir = normalize(sub(joint_positions[i + 1], joint_positions[i]));
                joint_positions[i + 1] = add(joint_positions[i], scale(dir, bone_lengths[i]));
            }
        }

        // Apply pole target constraint if set (always, whether converged or not).
        if let Some(pole) = settings.pole_target {
            apply_pole_target(joint_positions, bone_lengths, target, pole, settings.pole_angle);
        }

        iterations_done
    }
}

/// Orient the chain plane toward the pole target by rotating around
/// the root-to-target axis. Same algorithm as CCD's pole target.
fn apply_pole_target(
    joints: &mut [[f32; 3]],
    bone_lengths: &[f32],
    target: [f32; 3],
    pole: [f32; 3],
    pole_angle: f32,
) {
    let n = joints.len();
    if n < 3 {
        return;
    }

    let root = joints[0];

    // Axis from root to target.
    let chain_axis = sub(target, root);
    let chain_len = (chain_axis[0] * chain_axis[0]
        + chain_axis[1] * chain_axis[1]
        + chain_axis[2] * chain_axis[2])
        .sqrt();
    if chain_len < f32::EPSILON {
        return;
    }
    let chain_axis = [
        chain_axis[0] / chain_len,
        chain_axis[1] / chain_len,
        chain_axis[2] / chain_len,
    ];

    let mid = n / 2;
    let to_mid = sub(joints[mid], root);

    let d = to_mid[0] * chain_axis[0] + to_mid[1] * chain_axis[1] + to_mid[2] * chain_axis[2];
    let proj_mid = [
        to_mid[0] - d * chain_axis[0],
        to_mid[1] - d * chain_axis[1],
        to_mid[2] - d * chain_axis[2],
    ];
    let proj_mid_len = (proj_mid[0] * proj_mid[0]
        + proj_mid[1] * proj_mid[1]
        + proj_mid[2] * proj_mid[2])
        .sqrt();
    if proj_mid_len < f32::EPSILON {
        return;
    }

    let to_pole = sub(pole, root);
    let dp = to_pole[0] * chain_axis[0] + to_pole[1] * chain_axis[1] + to_pole[2] * chain_axis[2];
    let proj_pole = [
        to_pole[0] - dp * chain_axis[0],
        to_pole[1] - dp * chain_axis[1],
        to_pole[2] - dp * chain_axis[2],
    ];
    let proj_pole_len = (proj_pole[0] * proj_pole[0]
        + proj_pole[1] * proj_pole[1]
        + proj_pole[2] * proj_pole[2])
        .sqrt();
    if proj_pole_len < f32::EPSILON {
        return;
    }

    let dot_val = (proj_mid[0] * proj_pole[0]
        + proj_mid[1] * proj_pole[1]
        + proj_mid[2] * proj_pole[2])
        / (proj_mid_len * proj_pole_len);
    let mut angle = dot_val.clamp(-1.0, 1.0).acos();

    let cx = proj_mid[1] * proj_pole[2] - proj_mid[2] * proj_pole[1];
    let cy = proj_mid[2] * proj_pole[0] - proj_mid[0] * proj_pole[2];
    let cz = proj_mid[0] * proj_pole[1] - proj_mid[1] * proj_pole[0];
    if cx * chain_axis[0] + cy * chain_axis[1] + cz * chain_axis[2] < 0.0 {
        angle = -angle;
    }

    angle += pole_angle;

    if angle.abs() < f32::EPSILON {
        return;
    }

    let cos_a = angle.cos();
    let sin_a = angle.sin();

    for j in 1..n {
        let rel = sub(joints[j], root);
        let d_axis = rel[0] * chain_axis[0] + rel[1] * chain_axis[1] + rel[2] * chain_axis[2];
        let cx2 = chain_axis[1] * rel[2] - chain_axis[2] * rel[1];
        let cy2 = chain_axis[2] * rel[0] - chain_axis[0] * rel[2];
        let cz2 = chain_axis[0] * rel[1] - chain_axis[1] * rel[0];
        joints[j] = add(root, [
            rel[0] * cos_a + cx2 * sin_a + chain_axis[0] * d_axis * (1.0 - cos_a),
            rel[1] * cos_a + cy2 * sin_a + chain_axis[1] * d_axis * (1.0 - cos_a),
            rel[2] * cos_a + cz2 * sin_a + chain_axis[2] * d_axis * (1.0 - cos_a),
        ]);
    }

    // Re-enforce bone lengths after pole rotation.
    for j in 0..(n - 1) {
        let dir = sub(joints[j + 1], joints[j]);
        let len = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
        if len > f32::EPSILON {
            let s = bone_lengths[j] / len;
            joints[j + 1] = add(joints[j], scale(dir, s));
        }
    }
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn scale(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

fn distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    let d = sub(a, b);
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

fn normalize(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < f32::EPSILON {
        // Return a valid unit vector to prevent joint collapse when
        // two joints are at the same position.
        [0.0, 1.0, 0.0]
    } else {
        [v[0] / len, v[1] / len, v[2] / len]
    }
}
