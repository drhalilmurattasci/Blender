//! NLA blending: combines values from multiple strips/tracks.

use crate::NlaBlendMode;

/// Blend a new strip value into the accumulated result.
///
/// `accumulated`: the current accumulated value from lower tracks/strips.
/// `strip_value`: the value from the current strip.
/// `influence`: the strip's effective influence (including blend-in/out).
/// `mode`: the blending mode.
/// `rest_value`: the rest/default value for this channel.
pub fn blend_value(
    accumulated: f32,
    strip_value: f32,
    influence: f32,
    mode: NlaBlendMode,
    rest_value: f32,
) -> f32 {
    let influence = influence.clamp(0.0, 1.0);

    match mode {
        NlaBlendMode::Replace => {
            // Lerp between accumulated and strip value.
            // Blender: lower * (1 - influence) + strip * influence
            accumulated * (1.0 - influence) + strip_value * influence
        }
        NlaBlendMode::Combine => {
            // Add the strip's delta from rest pose to the accumulated value.
            // Blender: lower + (strip - rest) * influence
            let delta = strip_value - rest_value;
            accumulated + delta * influence
        }
        NlaBlendMode::Add => {
            // Additive: add strip value scaled by influence directly.
            // Blender: lower + (strip * influence)
            accumulated + strip_value * influence
        }
        NlaBlendMode::Subtract => {
            // Subtractive: subtract strip value scaled by influence directly.
            // Blender: lower - (strip * influence)
            accumulated - strip_value * influence
        }
        NlaBlendMode::Multiply => {
            // Multiplicative: blend between identity (no effect) and full multiply.
            // Blender: influence * (lower * strip) + (1 - influence) * lower
            influence * (accumulated * strip_value) + (1.0 - influence) * accumulated
        }
    }
}

/// Blend quaternion rotations for NLA.
///
/// Returns the blended quaternion [x, y, z, w].
pub fn blend_quaternion(
    accumulated: [f32; 4],
    strip_value: [f32; 4],
    influence: f32,
    mode: NlaBlendMode,
) -> [f32; 4] {
    let influence = influence.clamp(0.0, 1.0);

    match mode {
        NlaBlendMode::Replace => nlerp(accumulated, strip_value, influence),
        NlaBlendMode::Add | NlaBlendMode::Combine => {
            // For additive: multiply the strip rotation (scaled by influence) onto accumulated.
            // Simplified: use nlerp with identity.
            let identity = [0.0, 0.0, 0.0, 1.0_f32];
            let partial = nlerp(identity, strip_value, influence);
            quat_mul(accumulated, partial)
        }
        NlaBlendMode::Subtract => {
            let identity = [0.0, 0.0, 0.0, 1.0_f32];
            let partial = nlerp(identity, strip_value, influence);
            let inv = quat_conjugate(partial);
            quat_mul(accumulated, inv)
        }
        NlaBlendMode::Multiply => {
            // Multiply mode for quaternions is the same as replace.
            nlerp(accumulated, strip_value, influence)
        }
    }
}

fn nlerp(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    let dot = a[0] * b[0] + a[1] * b[1] + a[2] * b[2] + a[3] * b[3];
    let b = if dot < 0.0 { [-b[0], -b[1], -b[2], -b[3]] } else { b };
    let inv = 1.0 - t;
    let mut r = [
        a[0] * inv + b[0] * t,
        a[1] * inv + b[1] * t,
        a[2] * inv + b[2] * t,
        a[3] * inv + b[3] * t,
    ];
    let len = (r[0] * r[0] + r[1] * r[1] + r[2] * r[2] + r[3] * r[3]).sqrt();
    if len > f32::EPSILON {
        for v in &mut r {
            *v /= len;
        }
    }
    r
}

fn quat_mul(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    [
        a[3] * b[0] + a[0] * b[3] + a[1] * b[2] - a[2] * b[1],
        a[3] * b[1] - a[0] * b[2] + a[1] * b[3] + a[2] * b[0],
        a[3] * b[2] + a[0] * b[1] - a[1] * b[0] + a[2] * b[3],
        a[3] * b[3] - a[0] * b[0] - a[1] * b[1] - a[2] * b[2],
    ]
}

fn quat_conjugate(q: [f32; 4]) -> [f32; 4] {
    [-q[0], -q[1], -q[2], q[3]]
}
