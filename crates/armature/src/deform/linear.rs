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
/// Same as `linear_blend_skinning` but for direction vectors (no translation).
pub fn linear_blend_skinning_normals(
    normals: &[[f32; 3]],
    influences: &[VertexInfluences],
    deform_matrices: &[[f32; 16]],
    output: &mut [[f32; 3]],
) {
    debug_assert_eq!(normals.len(), influences.len());
    debug_assert_eq!(normals.len(), output.len());

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
            if bone_idx >= deform_matrices.len() {
                continue;
            }
            let w = vw.weight * norm;
            let m = &deform_matrices[bone_idx];

            // Transform direction (no translation).
            let x = m[0] * normal[0] + m[4] * normal[1] + m[8] * normal[2];
            let y = m[1] * normal[0] + m[5] * normal[1] + m[9] * normal[2];
            let z = m[2] * normal[0] + m[6] * normal[1] + m[10] * normal[2];

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
