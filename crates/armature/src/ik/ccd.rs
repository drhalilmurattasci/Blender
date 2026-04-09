//! Cyclic Coordinate Descent (CCD) IK solver.

use crate::ik::{IkSettings, IkSolver};

/// CCD (Cyclic Coordinate Descent) IK solver.
///
/// Simple and robust iterative solver that rotates each joint
/// one at a time to minimize the end-effector-to-target distance.
#[derive(Debug, Clone)]
pub struct CcdSolver;

impl IkSolver for CcdSolver {
    fn solve(
        &self,
        joint_positions: &mut [[f32; 3]],
        bone_lengths: &[f32],
        target: [f32; 3],
        settings: &IkSettings,
    ) -> u32 {
        let n = joint_positions.len();
        if n < 2 {
            return 0;
        }

        let end_idx = n - 1;

        for iteration in 0..settings.max_iterations {
            // Check convergence.
            let end = joint_positions[end_idx];
            let dist = distance(end, target);
            if dist < settings.tolerance {
                return iteration;
            }

            // Iterate from the second-to-last joint down to the root.
            for i in (0..end_idx).rev() {
                let end_effector = joint_positions[end_idx];
                let joint = joint_positions[i];

                // Vector from joint to end effector.
                let to_end = sub(end_effector, joint);
                let to_target = sub(target, joint);

                let len_end = vec_length(to_end);
                let len_target = vec_length(to_target);

                if len_end < f32::EPSILON || len_target < f32::EPSILON {
                    continue;
                }

                // Compute rotation angle.
                let cos_angle = dot(to_end, to_target) / (len_end * len_target);
                let cos_angle = cos_angle.clamp(-1.0, 1.0);
                let angle = cos_angle.acos();

                if angle.abs() < f32::EPSILON {
                    continue;
                }

                // Compute rotation axis.
                let axis = cross(to_end, to_target);
                let axis_len = vec_length(axis);
                if axis_len < f32::EPSILON {
                    continue;
                }
                let axis = [axis[0] / axis_len, axis[1] / axis_len, axis[2] / axis_len];

                // Rotate all joints from i+1 to end around joint[i].
                for j in (i + 1)..n {
                    let relative = sub(joint_positions[j], joint);
                    let rotated = rotate_around_axis(relative, axis, angle);
                    joint_positions[j] = add(joint, rotated);
                }

                // Enforce bone lengths after rotation to prevent drift.
                for j in i..end_idx {
                    let dir = sub(joint_positions[j + 1], joint_positions[j]);
                    let len = vec_length(dir);
                    if len > f32::EPSILON {
                        let desired = bone_lengths[j];
                        let scale = desired / len;
                        joint_positions[j + 1] = add(
                            joint_positions[j],
                            [dir[0] * scale, dir[1] * scale, dir[2] * scale],
                        );
                    }
                }
            }
        }

        // Apply pole target constraint if set.
        if let Some(pole) = settings.pole_target {
            apply_pole_target(joint_positions, bone_lengths, target, pole, settings.pole_angle);
        }

        settings.max_iterations
    }
}

/// Orient the chain plane so the mid-chain joint lies in the plane defined
/// by root->target and root->pole_target.
///
/// This matches Blender's pole target behavior: compute the angle between
/// the chain plane and the desired pole direction, then rotate the entire
/// chain around the root-to-target axis by that angle.
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
    let _end = joints[n - 1];

    // Axis from root to target (the "hinge" axis).
    let chain_axis = sub(target, root);
    let chain_len = vec_length(chain_axis);
    if chain_len < f32::EPSILON {
        return;
    }
    let chain_axis = [
        chain_axis[0] / chain_len,
        chain_axis[1] / chain_len,
        chain_axis[2] / chain_len,
    ];

    // Use the midpoint joint as the reference for the chain plane.
    let mid = n / 2;
    let to_mid = sub(joints[mid], root);

    // Project to_mid onto the plane perpendicular to chain_axis.
    let d = dot(to_mid, chain_axis);
    let proj_mid = [
        to_mid[0] - d * chain_axis[0],
        to_mid[1] - d * chain_axis[1],
        to_mid[2] - d * chain_axis[2],
    ];
    let proj_mid_len = vec_length(proj_mid);
    if proj_mid_len < f32::EPSILON {
        return;
    }

    // Project pole direction onto the same plane.
    let to_pole = sub(pole, root);
    let dp = dot(to_pole, chain_axis);
    let proj_pole = [
        to_pole[0] - dp * chain_axis[0],
        to_pole[1] - dp * chain_axis[1],
        to_pole[2] - dp * chain_axis[2],
    ];
    let proj_pole_len = vec_length(proj_pole);
    if proj_pole_len < f32::EPSILON {
        return;
    }

    // Compute the angle between the two projected vectors.
    let cos_angle = dot(proj_mid, proj_pole) / (proj_mid_len * proj_pole_len);
    let cos_angle = cos_angle.clamp(-1.0, 1.0);
    let mut angle = cos_angle.acos();

    // Determine sign using cross product.
    let c = cross(proj_mid, proj_pole);
    if dot(c, chain_axis) < 0.0 {
        angle = -angle;
    }

    // Add the pole angle offset.
    angle += pole_angle;

    if angle.abs() < f32::EPSILON {
        return;
    }

    // Rotate all joints (except root) around the chain_axis by this angle.
    for j in 1..n {
        let relative = sub(joints[j], root);
        let rotated = rotate_around_axis(relative, chain_axis, angle);
        joints[j] = add(root, rotated);
    }

    // Re-enforce bone lengths after pole rotation.
    for j in 0..(n - 1) {
        let dir = sub(joints[j + 1], joints[j]);
        let len = vec_length(dir);
        if len > f32::EPSILON {
            let scale = bone_lengths[j] / len;
            joints[j + 1] = add(joints[j], [dir[0] * scale, dir[1] * scale, dir[2] * scale]);
        }
    }
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn vec_length(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    vec_length(sub(a, b))
}

/// Rotate vector `v` around `axis` by `angle` (Rodrigues' rotation formula).
fn rotate_around_axis(v: [f32; 3], axis: [f32; 3], angle: f32) -> [f32; 3] {
    let cos_a = angle.cos();
    let sin_a = angle.sin();
    let d = dot(axis, v);
    let c = cross(axis, v);

    [
        v[0] * cos_a + c[0] * sin_a + axis[0] * d * (1.0 - cos_a),
        v[1] * cos_a + c[1] * sin_a + axis[1] * d * (1.0 - cos_a),
        v[2] * cos_a + c[2] * sin_a + axis[2] * d * (1.0 - cos_a),
    ]
}
