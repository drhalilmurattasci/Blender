//! Linear blend skinning (LBS / vertex blending).

use crate::deform::VertexInfluences;

/// Apply linear blend skinning to a set of vertices.
///
/// `positions`: input vertex positions (3 floats per vertex).
/// `influences`: per-vertex bone influences.
/// `deform_matrices`: one 4x4 column-major matrix per bone (pose * rest_inverse).
/// `output`: output vertex positions (same layout as `positions`).
pub fn linear_blend_skinning(
    positions: &[[f32; 3]],
    influences: &[VertexInfluences],
    deform_matrices: &[[f32; 16]],
    output: &mut [[f32; 3]],
) {
    debug_assert_eq!(positions.len(), influences.len());
    debug_assert_eq!(positions.len(), output.len());

    for (i, (pos, inf)) in positions.iter().zip(influences.iter()).enumerate() {
        // Compute total weight for normalization (Blender normalizes weights).
        let total_weight: f32 = inf
            .weights
            .iter()
            .filter(|vw| (vw.bone_index as usize) < deform_matrices.len())
            .map(|vw| vw.weight)
            .sum();

        // Zero total weight: vertex remains undeformed (Blender behavior).
        if total_weight < f32::EPSILON {
            output[i] = *pos;
            continue;
        }

        let norm = 1.0 / total_weight;
        let mut result = [0.0_f32; 3];

        for vw in &inf.weights {
            let bone_idx = vw.bone_index as usize;
            if bone_idx >= deform_matrices.len() {
                continue;
            }
            let w = vw.weight * norm;
            let m = &deform_matrices[bone_idx];

            // Transform position by the deformation matrix.
            let x = m[0] * pos[0] + m[4] * pos[1] + m[8] * pos[2] + m[12];
            let y = m[1] * pos[0] + m[5] * pos[1] + m[9] * pos[2] + m[13];
            let z = m[2] * pos[0] + m[6] * pos[1] + m[10] * pos[2] + m[14];

            result[0] += x * w;
            result[1] += y * w;
            result[2] += z * w;
        }

        output[i] = result;
    }
}

/// Apply linear blend skinning to vertex normals.
///
/// Normals must be transformed by the **transpose of the inverse** of the
/// upper-left 3x3 of each deform matrix. Using the plain matrix is only
/// correct for pure rotations; with non-uniform scale it distorts normals.
pub fn linear_blend_skinning_normals(
    normals: &[[f32; 3]],
    influences: &[VertexInfluences],
    deform_matrices: &[[f32; 16]],
    output: &mut [[f32; 3]],
) {
    debug_assert_eq!(normals.len(), influences.len());
    debug_assert_eq!(normals.len(), output.len());

    // Pre-compute the inverse-transpose of the upper-left 3x3 for each bone.
    let normal_matrices: Vec<[f32; 9]> = deform_matrices
        .iter()
        .map(|m| {
            let mat3 = [
                m[0], m[1], m[2], // column 0
                m[4], m[5], m[6], // column 1
                m[8], m[9], m[10], // column 2
            ];
            inverse_transpose_3x3(mat3)
        })
        .collect();

    for (i, (normal, inf)) in normals.iter().zip(influences.iter()).enumerate() {
        // Compute total weight for normalization.
        let total_weight: f32 = inf
            .weights
            .iter()
            .filter(|vw| (vw.bone_index as usize) < deform_matrices.len())
            .map(|vw| vw.weight)
            .sum();

        // Zero total weight: normal remains undeformed.
        if total_weight < f32::EPSILON {
            output[i] = *normal;
            continue;
        }

        let norm = 1.0 / total_weight;
        let mut result = [0.0_f32; 3];

        for vw in &inf.weights {
            let bone_idx = vw.bone_index as usize;
            if bone_idx >= normal_matrices.len() {
                continue;
            }
            let w = vw.weight * norm;
            let n = &normal_matrices[bone_idx];

            // Transform normal by inverse-transpose 3x3 (column-major: n[col*3+row]).
            let x = n[0] * normal[0] + n[3] * normal[1] + n[6] * normal[2];
            let y = n[1] * normal[0] + n[4] * normal[1] + n[7] * normal[2];
            let z = n[2] * normal[0] + n[5] * normal[1] + n[8] * normal[2];

            result[0] += x * w;
            result[1] += y * w;
            result[2] += z * w;
        }

        // Normalize the result.
        let len = (result[0] * result[0] + result[1] * result[1] + result[2] * result[2]).sqrt();
        if len > f32::EPSILON {
            result[0] /= len;
            result[1] /= len;
            result[2] /= len;
        }

        output[i] = result;
    }
}

/// Compute the inverse-transpose of a 3x3 column-major matrix.
/// Falls back to the original matrix if the determinant is near zero
/// (degenerate scale), which preserves correct behavior for pure rotations.
fn inverse_transpose_3x3(m: [f32; 9]) -> [f32; 9] {
    // m is column-major: m[col*3+row]
    // m[0..3] = col0, m[3..6] = col1, m[6..9] = col2
    let a00 = m[0]; let a10 = m[1]; let a20 = m[2];
    let a01 = m[3]; let a11 = m[4]; let a21 = m[5];
    let a02 = m[6]; let a12 = m[7]; let a22 = m[8];

    // Cofactor matrix (the transpose of the adjugate is the inverse-transpose * det).
    let c00 = a11 * a22 - a12 * a21;
    let c01 = a12 * a20 - a10 * a22;
    let c02 = a10 * a21 - a11 * a20;
    let c10 = a02 * a21 - a01 * a22;
    let c11 = a00 * a22 - a02 * a20;
    let c12 = a01 * a20 - a00 * a21;
    let c20 = a01 * a12 - a02 * a11;
    let c21 = a02 * a10 - a00 * a12;
    let c22 = a00 * a11 - a01 * a10;

    let det = a00 * c00 + a01 * c01 + a02 * c02;

    if det.abs() < f32::EPSILON {
        // Singular matrix; fall back to the original (works for pure rotations).
        return m;
    }

    let inv_det = 1.0 / det;

    // The cofactor matrix IS the transpose of the adjugate, so the cofactor
    // matrix divided by det gives us (M^-1)^T directly, stored column-major.
    [
        c00 * inv_det, c01 * inv_det, c02 * inv_det, // column 0
        c10 * inv_det, c11 * inv_det, c12 * inv_det, // column 1
        c20 * inv_det, c21 * inv_det, c22 * inv_det, // column 2
    ]
}
