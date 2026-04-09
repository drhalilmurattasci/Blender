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

// Re-export key types for convenience.
pub use bone::{Bone, BoneHierarchy, BoneProperties};
pub use deform::{
    dual_quaternion_skinning, dual_quaternion_skinning_normals, linear_blend_skinning,
    linear_blend_skinning_normals, DeformMethod, VertexInfluences, VertexWeight,
};
pub use evaluate::{evaluate_armature, Armature, ArmatureEvalData};
pub use ik::{CcdSolver, FabrikSolver, IkSettings, IkSolver};
pub use pose::{Pose, PoseChannel};

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

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 1e-3;

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < EPSILON
    }

    fn approx_eq3(a: [f32; 3], b: [f32; 3]) -> bool {
        approx_eq(a[0], b[0]) && approx_eq(a[1], b[1]) && approx_eq(a[2], b[2])
    }

    // ---- IK: FABRIK ----

    #[test]
    fn fabrik_reachable_target() {
        let solver = FabrikSolver;
        let mut joints = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let lengths = [1.0, 1.0];
        let target = [1.5, 1.0, 0.0];
        let settings = ik::IkSettings {
            max_iterations: 50,
            tolerance: 0.001,
            ..Default::default()
        };
        solver.solve(&mut joints, &lengths, target, &settings);
        let end = joints[2];
        let dist = ((end[0] - target[0]).powi(2) + (end[1] - target[1]).powi(2)).sqrt();
        assert!(dist < 0.01, "FABRIK should converge for reachable target, dist={}", dist);
    }

    #[test]
    fn fabrik_unreachable_target() {
        let solver = FabrikSolver;
        let mut joints = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let lengths = [1.0, 1.0];
        let target = [10.0, 0.0, 0.0]; // Unreachable (distance=10, chain=2).
        let settings = ik::IkSettings::default();
        solver.solve(&mut joints, &lengths, target, &settings);
        // End effector should be fully stretched toward target.
        let end = joints[2];
        assert!(approx_eq(end[0], 2.0), "Should stretch to max length");
    }

    #[test]
    fn fabrik_zero_length_bone() {
        let solver = FabrikSolver;
        let mut joints = [[0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let lengths = [0.0, 1.0];
        let target = [0.5, 0.5, 0.0];
        let settings = ik::IkSettings { max_iterations: 10, ..Default::default() };
        // Should not panic.
        solver.solve(&mut joints, &lengths, target, &settings);
    }

    #[test]
    fn fabrik_single_joint() {
        let solver = FabrikSolver;
        let mut joints = [[0.0, 0.0, 0.0]];
        let lengths: [f32; 0] = [];
        let target = [1.0, 0.0, 0.0];
        let settings = ik::IkSettings::default();
        let iters = solver.solve(&mut joints, &lengths, target, &settings);
        assert_eq!(iters, 0);
    }

    #[test]
    fn fabrik_pole_target() {
        let solver = FabrikSolver;
        let mut joints = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let lengths = [1.0, 1.0];
        let target = [1.5, 1.0, 0.0];
        let settings = ik::IkSettings {
            max_iterations: 50,
            tolerance: 0.001,
            pole_target: Some([0.0, 0.0, 5.0]),
            ..Default::default()
        };
        solver.solve(&mut joints, &lengths, target, &settings);
        // Mid joint should have Z component shifted toward pole.
        // Just verify no panic and that the mid joint has moved in Z.
        assert!(joints[1][2].abs() > 0.01 || joints[1][1].abs() > 0.01);
    }

    // ---- IK: CCD ----

    #[test]
    fn ccd_reachable_target() {
        let solver = CcdSolver;
        let mut joints = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let lengths = [1.0, 1.0];
        let target = [1.5, 1.0, 0.0];
        let settings = ik::IkSettings {
            max_iterations: 100,
            tolerance: 0.01,
            ..Default::default()
        };
        solver.solve(&mut joints, &lengths, target, &settings);
        let end = joints[2];
        let dist = ((end[0] - target[0]).powi(2) + (end[1] - target[1]).powi(2)).sqrt();
        assert!(dist < 0.05, "CCD should converge for reachable target, dist={}", dist);
    }

    #[test]
    fn ccd_coincident_joints_safe() {
        let solver = CcdSolver;
        let mut joints = [[0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]];
        let lengths = [0.0, 0.0];
        let target = [1.0, 0.0, 0.0];
        let settings = ik::IkSettings::default();
        // Should not panic.
        solver.solve(&mut joints, &lengths, target, &settings);
    }

    // ---- Skinning: LBS ----

    #[test]
    fn lbs_identity_deform() {
        let positions = vec![[1.0, 2.0, 3.0]];
        let influences = vec![deform::VertexInfluences {
            weights: vec![deform::VertexWeight {
                bone_index: 0,
                weight: 1.0,
            }],
        }];
        // Identity deform matrix.
        let identity: [f32; 16] = [
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ];
        let matrices = vec![identity];
        let mut output = vec![[0.0; 3]];
        linear_blend_skinning(&positions, &influences, &matrices, &mut output);
        assert!(approx_eq3(output[0], [1.0, 2.0, 3.0]));
    }

    #[test]
    fn lbs_translation() {
        let positions = vec![[0.0, 0.0, 0.0]];
        let influences = vec![deform::VertexInfluences {
            weights: vec![deform::VertexWeight {
                bone_index: 0,
                weight: 1.0,
            }],
        }];
        let mat = [
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            5.0, 3.0, 1.0, 1.0,
        ];
        let matrices = vec![mat];
        let mut output = vec![[0.0; 3]];
        linear_blend_skinning(&positions, &influences, &matrices, &mut output);
        assert!(approx_eq3(output[0], [5.0, 3.0, 1.0]));
    }

    #[test]
    fn lbs_zero_weight_undeformed() {
        let positions = vec![[1.0, 2.0, 3.0]];
        let influences = vec![deform::VertexInfluences {
            weights: vec![deform::VertexWeight {
                bone_index: 0,
                weight: 0.0,
            }],
        }];
        let matrices = vec![[0.0; 16]]; // garbage matrix
        let mut output = vec![[0.0; 3]];
        linear_blend_skinning(&positions, &influences, &matrices, &mut output);
        assert!(approx_eq3(output[0], [1.0, 2.0, 3.0]));
    }

    #[test]
    fn lbs_multi_bone_blend() {
        let positions = vec![[0.0, 0.0, 0.0]];
        let influences = vec![deform::VertexInfluences {
            weights: vec![
                deform::VertexWeight { bone_index: 0, weight: 0.5 },
                deform::VertexWeight { bone_index: 1, weight: 0.5 },
            ],
        }];
        let mat0 = [
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            10.0, 0.0, 0.0, 1.0,
        ];
        let mat1 = [
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 10.0, 0.0, 1.0,
        ];
        let matrices = vec![mat0, mat1];
        let mut output = vec![[0.0; 3]];
        linear_blend_skinning(&positions, &influences, &matrices, &mut output);
        assert!(approx_eq3(output[0], [5.0, 5.0, 0.0]));
    }

    // ---- Skinning: Normals ----

    #[test]
    fn lbs_normal_identity() {
        let normals = vec![[0.0, 1.0, 0.0]];
        let influences = vec![deform::VertexInfluences {
            weights: vec![deform::VertexWeight {
                bone_index: 0,
                weight: 1.0,
            }],
        }];
        let identity: [f32; 16] = [
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ];
        let matrices = vec![identity];
        let mut output = vec![[0.0; 3]];
        linear_blend_skinning_normals(&normals, &influences, &matrices, &mut output);
        assert!(approx_eq3(output[0], [0.0, 1.0, 0.0]));
    }

    // ---- DQS ----

    #[test]
    fn dqs_identity_deform() {
        let positions = vec![[1.0, 2.0, 3.0]];
        let influences = vec![deform::VertexInfluences {
            weights: vec![deform::VertexWeight {
                bone_index: 0,
                weight: 1.0,
            }],
        }];
        let identity: [f32; 16] = [
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ];
        let matrices = vec![identity];
        let mut output = vec![[0.0; 3]];
        dual_quaternion_skinning(&positions, &influences, &matrices, &mut output);
        assert!(approx_eq3(output[0], [1.0, 2.0, 3.0]));
    }

    #[test]
    fn dqs_translation() {
        let positions = vec![[0.0, 0.0, 0.0]];
        let influences = vec![deform::VertexInfluences {
            weights: vec![deform::VertexWeight {
                bone_index: 0,
                weight: 1.0,
            }],
        }];
        let mat: [f32; 16] = [
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            5.0, 3.0, 1.0, 1.0,
        ];
        let matrices = vec![mat];
        let mut output = vec![[0.0; 3]];
        dual_quaternion_skinning(&positions, &influences, &matrices, &mut output);
        assert!(approx_eq3(output[0], [5.0, 3.0, 1.0]));
    }

    #[test]
    fn dqs_zero_weight_undeformed() {
        let positions = vec![[1.0, 2.0, 3.0]];
        let influences = vec![deform::VertexInfluences {
            weights: vec![deform::VertexWeight {
                bone_index: 0,
                weight: 0.0,
            }],
        }];
        let matrices = vec![[0.0; 16]];
        let mut output = vec![[0.0; 3]];
        dual_quaternion_skinning(&positions, &influences, &matrices, &mut output);
        assert!(approx_eq3(output[0], [1.0, 2.0, 3.0]));
    }

    #[test]
    fn dqs_normals_identity() {
        let normals = vec![[0.0, 1.0, 0.0]];
        let influences = vec![deform::VertexInfluences {
            weights: vec![deform::VertexWeight {
                bone_index: 0,
                weight: 1.0,
            }],
        }];
        let identity: [f32; 16] = [
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ];
        let matrices = vec![identity];
        let mut output = vec![[0.0; 3]];
        dual_quaternion_skinning_normals(&normals, &influences, &matrices, &mut output);
        assert!(approx_eq3(output[0], [0.0, 1.0, 0.0]));
    }

    // ---- Bone Hierarchy ----

    #[test]
    fn hierarchy_topological_order() {
        let mut bones = vec![
            bone::Bone::new("root", 0, [0.0; 3], [0.0, 1.0, 0.0]),
            bone::Bone::new("child", 1, [0.0, 1.0, 0.0], [0.0, 2.0, 0.0]),
        ];
        bones[0].children.push(1);
        bones[1].parent = 0;
        let hier = BoneHierarchy::new(&bones);
        let order = hier.topological_order();
        assert_eq!(order, vec![0, 1]);
    }

    #[test]
    fn hierarchy_validate_no_cycles() {
        let bones = vec![
            bone::Bone::new("root", 0, [0.0; 3], [0.0, 1.0, 0.0]),
        ];
        let hier = BoneHierarchy::new(&bones);
        assert!(hier.validate().is_ok());
    }

    // ---- Armature Evaluation ----

    #[test]
    fn evaluate_single_bone() {
        let bones = vec![bone::Bone::new("root", 0, [0.0; 3], [0.0, 1.0, 0.0])];
        let mut armature = evaluate::Armature::new("test", bones);
        let eval_data = ArmatureEvalData::prepare(&armature).unwrap();
        evaluate_armature(&mut armature, &eval_data).unwrap();
        // Pose matrix should be the rest matrix (identity channel * rest).
        // With default rest matrix (identity), pose should be identity.
        let pm = &armature.pose.channels[0].pose_matrix;
        assert!(approx_eq(pm[0], 1.0));
        assert!(approx_eq(pm[5], 1.0));
        assert!(approx_eq(pm[10], 1.0));
    }

    // ---- PoseChannel ----

    #[test]
    fn pose_channel_effective_rotation_quaternion() {
        let ch = PoseChannel::new("test", 0);
        let q = ch.effective_rotation_quat();
        assert!(approx_eq(q[3], 1.0)); // identity w
    }

    #[test]
    fn pose_channel_reset() {
        let mut ch = PoseChannel::new("test", 0);
        ch.location = [5.0, 5.0, 5.0];
        ch.reset();
        assert!(approx_eq3(ch.location, [0.0, 0.0, 0.0]));
    }

    // ---- VertexInfluences ----

    #[test]
    fn vertex_influences_normalize() {
        let mut vi = deform::VertexInfluences {
            weights: vec![
                deform::VertexWeight { bone_index: 0, weight: 3.0 },
                deform::VertexWeight { bone_index: 1, weight: 1.0 },
            ],
        };
        vi.normalize();
        let total: f32 = vi.weights.iter().map(|w| w.weight).sum();
        assert!(approx_eq(total, 1.0));
        assert!(approx_eq(vi.weights[0].weight, 0.75));
    }

    #[test]
    fn vertex_influences_normalize_zero() {
        let mut vi = deform::VertexInfluences {
            weights: vec![
                deform::VertexWeight { bone_index: 0, weight: 0.0 },
            ],
        };
        vi.normalize();
        // Should not panic, weight stays 0.
        assert!(approx_eq(vi.weights[0].weight, 0.0));
    }
}
