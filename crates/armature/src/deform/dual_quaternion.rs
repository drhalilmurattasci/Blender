//! Dual quaternion skinning (DQS).
//!
//! Avoids the volume-loss artifacts of linear blend skinning
//! by blending in dual quaternion space.

use crate::deform::VertexInfluences;

/// A dual quaternion represented as real and dual parts.
#[derive(Debug, Clone, Copy)]
struct DualQuat {
    /// Real part (rotation quaternion: x, y, z, w).
    real: [f32; 4],
    /// Dual part (translation encoding: x, y, z, w).
    dual: [f32; 4],
}

impl DualQuat {
    fn identity() -> Self {
        Self {
            real: [0.0, 0.0, 0.0, 1.0],
            dual: [0.0, 0.0, 0.0, 0.0],
        }
    }

    /// Construct from a 4x4 column-major matrix.
    ///
    /// Strips scale from the rotation columns before extracting the quaternion,
    /// matching Blender's DQS which normalizes the matrix first.
    fn from_matrix(m: &[f32; 16]) -> Self {
        // Normalize rotation columns to strip scale before quaternion extraction.
        let mut normalized = *m;
        for col in 0..3 {
            let base = col * 4;
            let len = (normalized[base] * normalized[base]
                + normalized[base + 1] * normalized[base + 1]
                + normalized[base + 2] * normalized[base + 2])
                .sqrt();
            if len > f32::EPSILON {
                normalized[base] /= len;
                normalized[base + 1] /= len;
                normalized[base + 2] /= len;
            }
        }

        let quat = matrix_to_quat(&normalized);
        let tx = m[12];
        let ty = m[13];
        let tz = m[14];

        // Dual part = 0.5 * translation_quat * rotation_quat
        let [qx, qy, qz, qw] = quat;
        let dual = [
            0.5 * (tx * qw + ty * qz - tz * qy),
            0.5 * (-tx * qz + ty * qw + tz * qx),
            0.5 * (tx * qy - ty * qx + tz * qw),
            -0.5 * (tx * qx + ty * qy + tz * qz),
        ];

        Self { real: quat, dual }
    }

    fn scale(&self, s: f32) -> Self {
        Self {
            real: [self.real[0] * s, self.real[1] * s, self.real[2] * s, self.real[3] * s],
            dual: [self.dual[0] * s, self.dual[1] * s, self.dual[2] * s, self.dual[3] * s],
        }
    }

    fn add(&self, other: &DualQuat) -> Self {
        Self {
            real: [
                self.real[0] + other.real[0],
                self.real[1] + other.real[1],
                self.real[2] + other.real[2],
                self.real[3] + other.real[3],
            ],
            dual: [
                self.dual[0] + other.dual[0],
                self.dual[1] + other.dual[1],
                self.dual[2] + other.dual[2],
                self.dual[3] + other.dual[3],
            ],
        }
    }

    fn normalize(&mut self) {
        let len = quat_length(self.real);
        if len > f32::EPSILON {
            let inv = 1.0 / len;
            for v in &mut self.real { *v *= inv; }
            for v in &mut self.dual { *v *= inv; }
        }
    }

    /// Transform a position by this dual quaternion.
    fn transform_point(&self, p: [f32; 3]) -> [f32; 3] {
        // Rotate by real part.
        let rotated = quat_rotate(self.real, p);

        // Extract translation from dual part.
        let [dx, dy, dz, dw] = self.dual;
        let [rx, ry, rz, rw] = self.real;
        let tx = 2.0 * (-dw * rx + dx * rw - dy * rz + dz * ry);
        let ty = 2.0 * (-dw * ry + dx * rz + dy * rw - dz * rx);
        let tz = 2.0 * (-dw * rz - dx * ry + dy * rx + dz * rw);

        [rotated[0] + tx, rotated[1] + ty, rotated[2] + tz]
    }
}

/// Apply dual quaternion skinning to vertex positions.
pub fn dual_quaternion_skinning(
    positions: &[[f32; 3]],
    influences: &[VertexInfluences],
    deform_matrices: &[[f32; 16]],
    output: &mut [[f32; 3]],
) {
    debug_assert_eq!(positions.len(), influences.len());
    debug_assert_eq!(positions.len(), output.len());

    for (i, (pos, inf)) in positions.iter().zip(influences.iter()).enumerate() {
        if inf.weights.is_empty() {
            output[i] = *pos;
            continue;
        }

        // Compute total weight for normalization.
        let total_weight: f32 = inf
            .weights
            .iter()
            .filter(|vw| (vw.bone_index as usize) < deform_matrices.len())
            .map(|vw| vw.weight)
            .sum();

        // Zero total weight: vertex remains undeformed.
        if total_weight < f32::EPSILON {
            output[i] = *pos;
            continue;
        }

        let weight_norm = 1.0 / total_weight;

        // Blend dual quaternions.
        let mut blended = DualQuat::identity().scale(0.0);
        let mut first_dq = None;

        for vw in &inf.weights {
            let bone_idx = vw.bone_index as usize;
            if bone_idx >= deform_matrices.len() {
                continue;
            }

            let dq = DualQuat::from_matrix(&deform_matrices[bone_idx]);

            // Ensure shortest path (flip if dot with first DQ is negative).
            let dq = if let Some(ref first) = first_dq {
                if quat_dot(dq.real, *first) < 0.0 {
                    dq.scale(-1.0)
                } else {
                    dq
                }
            } else {
                first_dq = Some(dq.real);
                dq
            };

            blended = blended.add(&dq.scale(vw.weight * weight_norm));
        }

        blended.normalize();
        output[i] = blended.transform_point(*pos);
    }
}

fn matrix_to_quat(m: &[f32; 16]) -> [f32; 4] {
    let trace = m[0] + m[5] + m[10];

    if trace > 0.0 {
        let s = (trace + 1.0).sqrt() * 2.0;
        let inv_s = 1.0 / s;
        [
            (m[6] - m[9]) * inv_s,
            (m[8] - m[2]) * inv_s,
            (m[1] - m[4]) * inv_s,
            0.25 * s,
        ]
    } else if m[0] > m[5] && m[0] > m[10] {
        let s = (1.0 + m[0] - m[5] - m[10]).sqrt() * 2.0;
        let inv_s = 1.0 / s;
        [
            0.25 * s,
            (m[1] + m[4]) * inv_s,
            (m[8] + m[2]) * inv_s,
            (m[6] - m[9]) * inv_s,
        ]
    } else if m[5] > m[10] {
        let s = (1.0 + m[5] - m[0] - m[10]).sqrt() * 2.0;
        let inv_s = 1.0 / s;
        [
            (m[1] + m[4]) * inv_s,
            0.25 * s,
            (m[6] + m[9]) * inv_s,
            (m[8] - m[2]) * inv_s,
        ]
    } else {
        let s = (1.0 + m[10] - m[0] - m[5]).sqrt() * 2.0;
        let inv_s = 1.0 / s;
        [
            (m[8] + m[2]) * inv_s,
            (m[6] + m[9]) * inv_s,
            0.25 * s,
            (m[1] - m[4]) * inv_s,
        ]
    }
}

fn quat_dot(a: [f32; 4], b: [f32; 4]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2] + a[3] * b[3]
}

fn quat_length(q: [f32; 4]) -> f32 {
    quat_dot(q, q).sqrt()
}

fn quat_rotate(q: [f32; 4], v: [f32; 3]) -> [f32; 3] {
    let [qx, qy, qz, qw] = q;
    // t = 2 * cross(q.xyz, v)
    let tx = 2.0 * (qy * v[2] - qz * v[1]);
    let ty = 2.0 * (qz * v[0] - qx * v[2]);
    let tz = 2.0 * (qx * v[1] - qy * v[0]);

    [
        v[0] + qw * tx + (qy * tz - qz * ty),
        v[1] + qw * ty + (qz * tx - qx * tz),
        v[2] + qw * tz + (qx * ty - qy * tx),
    ]
}

/// Apply dual quaternion skinning to vertex normals.
///
/// DQS normals are transformed by only the rotation part of the blended
/// dual quaternion (no translation), then re-normalized.
pub fn dual_quaternion_skinning_normals(
    normals: &[[f32; 3]],
    influences: &[VertexInfluences],
    deform_matrices: &[[f32; 16]],
    output: &mut [[f32; 3]],
) {
    debug_assert_eq!(normals.len(), influences.len());
    debug_assert_eq!(normals.len(), output.len());

    for (i, (normal, inf)) in normals.iter().zip(influences.iter()).enumerate() {
        if inf.weights.is_empty() {
            output[i] = *normal;
            continue;
        }

        let total_weight: f32 = inf
            .weights
            .iter()
            .filter(|vw| (vw.bone_index as usize) < deform_matrices.len())
            .map(|vw| vw.weight)
            .sum();

        if total_weight < f32::EPSILON {
            output[i] = *normal;
            continue;
        }

        let weight_norm = 1.0 / total_weight;

        // Blend dual quaternions (same as position skinning).
        let mut blended = DualQuat::identity().scale(0.0);
        let mut first_dq = None;

        for vw in &inf.weights {
            let bone_idx = vw.bone_index as usize;
            if bone_idx >= deform_matrices.len() {
                continue;
            }

            let dq = DualQuat::from_matrix(&deform_matrices[bone_idx]);

            let dq = if let Some(ref first) = first_dq {
                if quat_dot(dq.real, *first) < 0.0 {
                    dq.scale(-1.0)
                } else {
                    dq
                }
            } else {
                first_dq = Some(dq.real);
                dq
            };

            blended = blended.add(&dq.scale(vw.weight * weight_norm));
        }

        blended.normalize();

        // For normals, only apply the rotation (no translation).
        let rotated = quat_rotate(blended.real, *normal);

        // Re-normalize.
        let len = (rotated[0] * rotated[0] + rotated[1] * rotated[1] + rotated[2] * rotated[2])
            .sqrt();
        if len > f32::EPSILON {
            output[i] = [rotated[0] / len, rotated[1] / len, rotated[2] / len];
        } else {
            output[i] = *normal;
        }
    }
}
