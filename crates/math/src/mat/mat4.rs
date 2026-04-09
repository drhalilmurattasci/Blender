use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::quat::Quat;
use crate::traits::{ApproxEq, Lerp};
use crate::vec::Vec3;

/// 4x4 matrix wrapping `glam::Mat4`.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(transparent)]
pub struct Mat4(pub glam::Mat4);

impl Mat4 {
    pub const IDENTITY: Self = Self(glam::Mat4::IDENTITY);
    pub const ZERO: Self = Self(glam::Mat4::ZERO);

    #[inline]
    pub fn from_cols_array(m: &[f32; 16]) -> Self {
        Self(glam::Mat4::from_cols_array(m))
    }

    #[inline]
    pub fn to_cols_array(&self) -> [f32; 16] {
        self.0.to_cols_array()
    }

    #[inline]
    pub fn transpose(&self) -> Self {
        Self(self.0.transpose())
    }

    #[inline]
    pub fn determinant(&self) -> f32 {
        self.0.determinant()
    }

    #[inline]
    pub fn inverse(&self) -> Self {
        Self(self.0.inverse())
    }

    #[inline]
    pub fn mul_vec4(&self, v: crate::vec::Vec4) -> crate::vec::Vec4 {
        crate::vec::Vec4::from_glam(self.0.mul_vec4(v.to_glam()))
    }

    /// Transform a 3D point (w=1).
    #[inline]
    pub fn transform_point3(&self, p: Vec3) -> Vec3 {
        Vec3(self.0.transform_point3a(p.0))
    }

    /// Transform a 3D vector (w=0).
    #[inline]
    pub fn transform_vector3(&self, v: Vec3) -> Vec3 {
        Vec3(self.0.transform_vector3a(v.0))
    }

    // --- Projection matrices ---

    /// Create a perspective projection matrix (right-handed, zero-to-one depth).
    #[inline]
    pub fn perspective_rh(fov_y_radians: f32, aspect_ratio: f32, z_near: f32, z_far: f32) -> Self {
        Self(glam::Mat4::perspective_rh(fov_y_radians, aspect_ratio, z_near, z_far))
    }

    /// Create an infinite perspective projection matrix (right-handed, zero-to-one depth).
    #[inline]
    pub fn perspective_infinite_rh(fov_y_radians: f32, aspect_ratio: f32, z_near: f32) -> Self {
        Self(glam::Mat4::perspective_infinite_rh(fov_y_radians, aspect_ratio, z_near))
    }

    /// Create an orthographic projection matrix (right-handed, zero-to-one depth).
    #[inline]
    pub fn orthographic_rh(
        left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32,
    ) -> Self {
        Self(glam::Mat4::orthographic_rh(left, right, bottom, top, near, far))
    }

    // --- View matrices ---

    /// Create a right-handed look-at view matrix.
    #[inline]
    pub fn look_at_rh(eye: Vec3, center: Vec3, up: Vec3) -> Self {
        Self(glam::Mat4::look_at_rh(eye.0.into(), center.0.into(), up.0.into()))
    }

    /// Create a left-handed look-at view matrix.
    #[inline]
    pub fn look_at_lh(eye: Vec3, center: Vec3, up: Vec3) -> Self {
        Self(glam::Mat4::look_at_lh(eye.0.into(), center.0.into(), up.0.into()))
    }

    // --- SRT composition / decomposition ---

    /// Build a matrix from scale, rotation, translation.
    #[inline]
    pub fn from_scale_rotation_translation(scale: Vec3, rotation: Quat, translation: Vec3) -> Self {
        Self(glam::Mat4::from_scale_rotation_translation(
            glam::Vec3::from(scale),
            rotation.0,
            glam::Vec3::from(translation),
        ))
    }

    /// Build from translation only.
    #[inline]
    pub fn from_translation(translation: Vec3) -> Self {
        Self(glam::Mat4::from_translation(glam::Vec3::from(translation)))
    }

    /// Build from rotation only.
    #[inline]
    pub fn from_quat(rotation: Quat) -> Self {
        Self(glam::Mat4::from_quat(rotation.0))
    }

    /// Build from scale only.
    #[inline]
    pub fn from_scale(scale: Vec3) -> Self {
        Self(glam::Mat4::from_scale(glam::Vec3::from(scale)))
    }

    /// Decompose into (scale, rotation, translation).
    ///
    /// Correctly handles negative scales by checking the determinant of the
    /// upper-left 3x3 submatrix. When the determinant is negative, the X
    /// scale component is negated to preserve orientation parity, matching
    /// Blender's decomposition convention.
    ///
    /// Returns `None` if any scale axis has near-zero length (degenerate).
    pub fn to_scale_rotation_translation(&self) -> Option<(Vec3, Quat, Vec3)> {
        let col0 = glam::Vec3::new(
            self.0.x_axis.x, self.0.x_axis.y, self.0.x_axis.z,
        );
        let col1 = glam::Vec3::new(
            self.0.y_axis.x, self.0.y_axis.y, self.0.y_axis.z,
        );
        let col2 = glam::Vec3::new(
            self.0.z_axis.x, self.0.z_axis.y, self.0.z_axis.z,
        );

        let mut sx = col0.length();
        let sy = col1.length();
        let sz = col2.length();

        if sx < 1e-10 || sy < 1e-10 || sz < 1e-10 {
            return None;
        }

        // Check for negative scale via determinant of upper-3x3.
        // If det < 0, the matrix includes a reflection. We absorb it
        // into the X scale so the rotation quaternion stays proper.
        let det = col0.dot(col1.cross(col2));
        if det < 0.0 {
            sx = -sx;
        }

        // Build the rotation matrix by dividing out scale.
        let rot_mat = glam::Mat3::from_cols(
            col0 / sx,
            col1 / sy,
            col2 / sz,
        );
        let rotation = glam::Quat::from_mat3(&rot_mat).normalize();

        let translation = Vec3::new(self.0.w_axis.x, self.0.w_axis.y, self.0.w_axis.z);
        let scale = Vec3::new(sx, sy, sz);

        Some((scale, Quat(rotation), translation))
    }
}

impl Default for Mat4 {
    #[inline]
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl std::fmt::Display for Mat4 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let a = self.to_cols_array();
        write!(
            f,
            "Mat4([{}, {}, {}, {} | {}, {}, {}, {} | {}, {}, {}, {} | {}, {}, {}, {}])",
            a[0], a[1], a[2], a[3], a[4], a[5], a[6], a[7],
            a[8], a[9], a[10], a[11], a[12], a[13], a[14], a[15]
        )
    }
}

impl From<glam::Mat4> for Mat4 {
    #[inline]
    fn from(m: glam::Mat4) -> Self {
        Self(m)
    }
}

impl From<Mat4> for glam::Mat4 {
    #[inline]
    fn from(m: Mat4) -> Self {
        m.0
    }
}

impl std::ops::Mul for Mat4 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self {
        Self(self.0 * rhs.0)
    }
}

impl std::ops::Mul<crate::vec::Vec4> for Mat4 {
    type Output = crate::vec::Vec4;
    #[inline]
    fn mul(self, rhs: crate::vec::Vec4) -> crate::vec::Vec4 {
        self.mul_vec4(rhs)
    }
}

// --- Custom Serde ---

impl Serialize for Mat4 {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let arr = self.to_cols_array();
        arr.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Mat4 {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let arr = <[f32; 16]>::deserialize(deserializer)?;
        Ok(Self::from_cols_array(&arr))
    }
}

// --- Trait impls ---

impl Lerp for Mat4 {
    #[inline]
    fn lerp(self, other: Self, t: f32) -> Self {
        let a = self.to_cols_array();
        let b = other.to_cols_array();
        let mut result = [0.0_f32; 16];
        for i in 0..16 {
            result[i] = a[i] + (b[i] - a[i]) * t;
        }
        Self::from_cols_array(&result)
    }
}

impl ApproxEq for Mat4 {
    #[inline]
    fn approx_eq(&self, other: &Self, epsilon: f32) -> bool {
        let a = self.to_cols_array();
        let b = other.to_cols_array();
        a.iter().zip(b.iter()).all(|(x, y)| (x - y).abs() <= epsilon)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_transform() {
        let m = Mat4::IDENTITY;
        let p = Vec3::new(1.0, 2.0, 3.0);
        let r = m.transform_point3(p);
        assert!(r.approx_eq(&p, 1e-6));
    }

    #[test]
    fn test_srt_roundtrip() {
        let s = Vec3::new(2.0, 3.0, 4.0);
        let r = Quat::from_axis_angle(Vec3::Y, std::f32::consts::FRAC_PI_4);
        let t = Vec3::new(10.0, 20.0, 30.0);
        let m = Mat4::from_scale_rotation_translation(s, r, t);
        let (s2, r2, t2) = m.to_scale_rotation_translation().unwrap();
        assert!(s2.approx_eq(&s, 1e-4));
        assert!(r2.approx_eq(&r, 1e-4));
        assert!(t2.approx_eq(&t, 1e-4));
    }

    #[test]
    fn test_serde_roundtrip() {
        let m = Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0));
        let json = serde_json::to_string(&m).unwrap();
        let m2: Mat4 = serde_json::from_str(&json).unwrap();
        assert!(m.approx_eq(&m2, 1e-6));
    }

    #[test]
    fn test_negative_scale_decomposition() {
        // Build a matrix with a negative X scale (mirror).
        let s = Vec3::new(-2.0, 3.0, 4.0);
        let r = Quat::from_axis_angle(Vec3::Y, std::f32::consts::FRAC_PI_4);
        let t = Vec3::new(10.0, 20.0, 30.0);
        let m = Mat4::from_scale_rotation_translation(s, r, t);

        let (s2, r2, t2) = m.to_scale_rotation_translation().unwrap();

        // Scale x should be negative.
        assert!(s2.x() < 0.0, "expected negative x scale, got {}", s2.x());
        assert!((s2.x() - (-2.0)).abs() < 1e-4);
        assert!((s2.y() - 3.0).abs() < 1e-4);
        assert!((s2.z() - 4.0).abs() < 1e-4);
        assert!(t2.approx_eq(&t, 1e-4));

        // Verify the recomposed matrix matches.
        let m2 = Mat4::from_scale_rotation_translation(s2, r2, t2);
        assert!(m.approx_eq(&m2, 1e-3));
    }

    #[test]
    fn test_degenerate_scale_returns_none() {
        // A matrix with a zero column should fail decomposition.
        let m = Mat4::from_cols_array(&[
            0.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ]);
        assert!(m.to_scale_rotation_translation().is_none());
    }
}
