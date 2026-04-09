//! Spatial transform: location, rotation (Euler / quaternion), scale, and
//! the derived 4x4 matrix.

use serde::{Deserialize, Serialize};

/// Euler rotation order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RotationOrder {
    XYZ,
    XZY,
    YXZ,
    YZX,
    ZXY,
    ZYX,
}

impl Default for RotationOrder {
    fn default() -> Self {
        Self::XYZ
    }
}

/// Rotation representation selector.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RotationMode {
    /// Euler angles (radians) with a given order.
    Euler {
        angles: [f32; 3],
        order: RotationOrder,
    },
    /// Unit quaternion (x, y, z, w).
    Quaternion([f32; 4]),
    /// Axis-angle: (axis_x, axis_y, axis_z, angle_rad).
    AxisAngle([f32; 4]),
}

impl Default for RotationMode {
    fn default() -> Self {
        Self::Euler {
            angles: [0.0; 3],
            order: RotationOrder::default(),
        }
    }
}

/// Local transform of an object.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    /// Translation (X, Y, Z).
    pub location: [f32; 3],
    /// Rotation.
    pub rotation: RotationMode,
    /// Non-uniform scale.
    pub scale: [f32; 3],
    /// Pre-computed local-to-parent 4x4 matrix (column-major).
    /// Recomputed by calling [`Transform::recompute_matrix`].
    pub matrix_local: [[f32; 4]; 4],
}

impl Transform {
    /// Identity transform constant.
    pub const IDENTITY: Self = Self {
        location: [0.0, 0.0, 0.0],
        rotation: RotationMode::Euler {
            angles: [0.0, 0.0, 0.0],
            order: RotationOrder::XYZ,
        },
        scale: [1.0, 1.0, 1.0],
        matrix_local: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
    };

    /// Recompute `matrix_local` from location / rotation / scale.
    ///
    /// Blender's transform order is **T * R * S** (Scale applied first,
    /// then Rotation, then Translation). The resulting 4x4 matrix is stored
    /// in column-major order: `matrix_local[col][row]`.
    pub fn recompute_matrix(&mut self) {
        let [sx, sy, sz] = self.scale;
        let rot = self.rotation_matrix_3x3();

        // Column-major assembly: column i = R * scale_basis_i
        // rot[row][col] is row-major, so R[row][col] = rot[row][col].
        self.matrix_local = [
            [rot[0][0] * sx, rot[1][0] * sx, rot[2][0] * sx, 0.0],  // column 0
            [rot[0][1] * sy, rot[1][1] * sy, rot[2][1] * sy, 0.0],  // column 1
            [rot[0][2] * sz, rot[1][2] * sz, rot[2][2] * sz, 0.0],  // column 2
            [self.location[0], self.location[1], self.location[2], 1.0], // column 3 (translation)
        ];
    }

    /// Build a 3x3 rotation matrix from the current rotation mode.
    fn rotation_matrix_3x3(&self) -> [[f32; 3]; 3] {
        match &self.rotation {
            RotationMode::Euler { angles, order } => {
                Self::euler_to_matrix(*angles, *order)
            }
            RotationMode::Quaternion(q) => Self::quat_to_matrix(*q),
            RotationMode::AxisAngle(aa) => {
                let axis = [aa[0], aa[1], aa[2]];
                let angle = aa[3];
                Self::axis_angle_to_matrix(axis, angle)
            }
        }
    }

    /// Build a row-major 3x3 rotation matrix from Euler angles.
    ///
    /// Blender's convention: for order XYZ, the combined rotation is
    /// `R = Rz * Ry * Rx` (intrinsic rotations: X applied first).
    /// The individual matrices follow the standard right-hand-rule convention.
    fn euler_to_matrix(angles: [f32; 3], order: RotationOrder) -> [[f32; 3]; 3] {
        let (sx, cx) = angles[0].sin_cos();
        let (sy, cy) = angles[1].sin_cos();
        let (sz, cz) = angles[2].sin_cos();

        // Standard rotation matrices (row-major: mat[row][col]).
        let rx = [[1.0, 0.0, 0.0], [0.0, cx, -sx], [0.0, sx, cx]];
        let ry = [[cy, 0.0, sy], [0.0, 1.0, 0.0], [-sy, 0.0, cy]];
        let rz = [[cz, -sz, 0.0], [sz, cz, 0.0], [0.0, 0.0, 1.0]];

        // For intrinsic rotations in order ABC, the combined matrix is
        // Rc * Rb * Ra (last axis applied outermost / leftmost).
        let (a, b, c) = match order {
            RotationOrder::XYZ => (rx, ry, rz),
            RotationOrder::XZY => (rx, rz, ry),
            RotationOrder::YXZ => (ry, rx, rz),
            RotationOrder::YZX => (ry, rz, rx),
            RotationOrder::ZXY => (rz, rx, ry),
            RotationOrder::ZYX => (rz, ry, rx),
        };

        // Result = c * b * a  (rightmost applied first).
        Self::mat3_mul(Self::mat3_mul(c, b), a)
    }

    /// Convert unit quaternion `[x, y, z, w]` to a row-major 3x3 rotation matrix.
    ///
    /// Uses the standard formula: `R[row][col]`, compatible with Blender's
    /// `quat_to_mat3` which stores `q = (w, x, y, z)`.
    fn quat_to_matrix(q: [f32; 4]) -> [[f32; 3]; 3] {
        let [x, y, z, w] = q;
        let x2 = x + x;
        let y2 = y + y;
        let z2 = z + z;
        let xx = x * x2;
        let xy = x * y2;
        let xz = x * z2;
        let yy = y * y2;
        let yz = y * z2;
        let zz = z * z2;
        let wx = w * x2;
        let wy = w * y2;
        let wz = w * z2;

        [
            [1.0 - (yy + zz), xy - wz, xz + wy],
            [xy + wz, 1.0 - (xx + zz), yz - wx],
            [xz - wy, yz + wx, 1.0 - (xx + yy)],
        ]
    }

    /// Axis-angle to row-major 3x3 rotation matrix (Rodrigues formula).
    ///
    /// The axis should be normalized. Matches Blender's `axis_angle_to_mat3`.
    fn axis_angle_to_matrix(axis: [f32; 3], angle: f32) -> [[f32; 3]; 3] {
        let (s, c) = angle.sin_cos();
        let t = 1.0 - c;
        let [x, y, z] = axis;
        [
            [t * x * x + c,     t * x * y - s * z, t * x * z + s * y],
            [t * x * y + s * z, t * y * y + c,      t * y * z - s * x],
            [t * x * z - s * y, t * y * z + s * x, t * z * z + c],
        ]
    }

    fn mat3_mul(a: [[f32; 3]; 3], b: [[f32; 3]; 3]) -> [[f32; 3]; 3] {
        let mut out = [[0.0f32; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                out[i][j] = a[i][0] * b[0][j] + a[i][1] * b[1][j] + a[i][2] * b[2][j];
            }
        }
        out
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::IDENTITY
    }
}
