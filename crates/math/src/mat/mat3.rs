use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::traits::{ApproxEq, Lerp};
use crate::vec::Vec3;

/// 3x3 matrix wrapping `glam::Mat3A` (SIMD-aligned columns).
///
/// Custom serde serializes as a flat 9-element array (column-major).
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(transparent)]
pub struct Mat3(pub glam::Mat3A);

impl Mat3 {
    pub const IDENTITY: Self = Self(glam::Mat3A::IDENTITY);
    pub const ZERO: Self = Self(glam::Mat3A::ZERO);

    #[inline]
    pub fn from_cols(x: Vec3, y: Vec3, z: Vec3) -> Self {
        Self(glam::Mat3A::from_cols(x.0, y.0, z.0))
    }

    #[inline]
    pub fn from_cols_array(m: &[f32; 9]) -> Self {
        Self(glam::Mat3A::from_cols_array(m))
    }

    #[inline]
    pub fn to_cols_array(&self) -> [f32; 9] {
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
    pub fn mul_vec3(&self, v: Vec3) -> Vec3 {
        Vec3(self.0.mul_vec3a(v.0))
    }

    #[inline]
    pub fn from_rotation_z(angle: f32) -> Self {
        Self(glam::Mat3A::from_rotation_z(angle))
    }

    #[inline]
    pub fn from_rotation_y(angle: f32) -> Self {
        Self(glam::Mat3A::from_rotation_y(angle))
    }

    #[inline]
    pub fn from_rotation_x(angle: f32) -> Self {
        Self(glam::Mat3A::from_rotation_x(angle))
    }

    #[inline]
    pub fn from_scale(scale: crate::vec::Vec2) -> Self {
        Self(glam::Mat3A::from_scale(scale.to_glam()))
    }
}

impl Default for Mat3 {
    #[inline]
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl std::fmt::Display for Mat3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let a = self.to_cols_array();
        write!(
            f,
            "Mat3([{}, {}, {} | {}, {}, {} | {}, {}, {}])",
            a[0], a[1], a[2], a[3], a[4], a[5], a[6], a[7], a[8]
        )
    }
}

impl From<glam::Mat3A> for Mat3 {
    #[inline]
    fn from(m: glam::Mat3A) -> Self {
        Self(m)
    }
}

impl From<Mat3> for glam::Mat3A {
    #[inline]
    fn from(m: Mat3) -> Self {
        m.0
    }
}

impl std::ops::Mul for Mat3 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self {
        Self(self.0 * rhs.0)
    }
}

impl std::ops::Mul<Vec3> for Mat3 {
    type Output = Vec3;
    #[inline]
    fn mul(self, rhs: Vec3) -> Vec3 {
        self.mul_vec3(rhs)
    }
}

// --- Custom Serde ---

impl Serialize for Mat3 {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let arr = self.to_cols_array();
        arr.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Mat3 {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let arr = <[f32; 9]>::deserialize(deserializer)?;
        Ok(Self::from_cols_array(&arr))
    }
}

// --- Trait impls ---

impl Lerp for Mat3 {
    #[inline]
    fn lerp(self, other: Self, t: f32) -> Self {
        let a = self.to_cols_array();
        let b = other.to_cols_array();
        let mut result = [0.0_f32; 9];
        for i in 0..9 {
            result[i] = a[i] + (b[i] - a[i]) * t;
        }
        Self::from_cols_array(&result)
    }
}

impl ApproxEq for Mat3 {
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
    fn test_identity() {
        let m = Mat3::IDENTITY;
        let v = Vec3::new(1.0, 2.0, 3.0);
        let r = m * v;
        assert!(r.approx_eq(&v, 1e-6));
    }

    #[test]
    fn test_serde_roundtrip() {
        let m = Mat3::from_rotation_z(std::f32::consts::FRAC_PI_4);
        let json = serde_json::to_string(&m).unwrap();
        let m2: Mat3 = serde_json::from_str(&json).unwrap();
        assert!(m.approx_eq(&m2, 1e-6));
    }

    #[test]
    fn test_determinant() {
        let m = Mat3::IDENTITY;
        assert!((m.determinant() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_inverse_roundtrip() {
        let m = Mat3::from_rotation_z(1.0);
        let inv = m.inverse();
        let product = m * inv;
        assert!(product.approx_eq(&Mat3::IDENTITY, 1e-5));
    }

    #[test]
    fn test_transpose() {
        let m = Mat3::from_rotation_z(0.5);
        let t = m.transpose().transpose();
        assert!(m.approx_eq(&t, 1e-6));
    }

    #[test]
    fn test_default_is_identity() {
        assert!(Mat3::default().approx_eq(&Mat3::IDENTITY, 1e-6));
    }

    #[test]
    fn test_display() {
        let m = Mat3::IDENTITY;
        let s = format!("{m}");
        assert!(s.starts_with("Mat3("));
    }
}
