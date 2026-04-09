use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::ops::{Add, AddAssign, Deref, DerefMut, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use crate::traits::{ApproxEq, Lerp};

/// 3D vector wrapping `glam::Vec3A` (16-byte SIMD-aligned).
///
/// Because `Vec3A` is 16 bytes (not 12), we cannot derive `Pod`.
/// Custom serde serializes only x/y/z.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(transparent)]
pub struct Vec3(pub glam::Vec3A);

impl Vec3 {
    pub const ZERO: Self = Self(glam::Vec3A::ZERO);
    pub const ONE: Self = Self(glam::Vec3A::ONE);
    pub const X: Self = Self(glam::Vec3A::X);
    pub const Y: Self = Self(glam::Vec3A::Y);
    pub const Z: Self = Self(glam::Vec3A::Z);
    pub const NEG_X: Self = Self(glam::Vec3A::NEG_X);
    pub const NEG_Y: Self = Self(glam::Vec3A::NEG_Y);
    pub const NEG_Z: Self = Self(glam::Vec3A::NEG_Z);

    #[inline]
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self(glam::Vec3A::new(x, y, z))
    }

    #[inline]
    pub fn splat(v: f32) -> Self {
        Self(glam::Vec3A::splat(v))
    }

    #[inline]
    pub fn x(self) -> f32 {
        self.0.x
    }

    #[inline]
    pub fn y(self) -> f32 {
        self.0.y
    }

    #[inline]
    pub fn z(self) -> f32 {
        self.0.z
    }

    #[inline]
    pub fn length(self) -> f32 {
        self.0.length()
    }

    #[inline]
    pub fn length_squared(self) -> f32 {
        self.0.length_squared()
    }

    #[inline]
    pub fn normalize(self) -> Self {
        Self(self.0.normalize())
    }

    #[inline]
    pub fn normalize_or_zero(self) -> Self {
        Self(self.0.normalize_or_zero())
    }

    #[inline]
    pub fn dot(self, rhs: Self) -> f32 {
        self.0.dot(rhs.0)
    }

    #[inline]
    pub fn cross(self, rhs: Self) -> Self {
        Self(self.0.cross(rhs.0))
    }

    #[inline]
    pub fn distance(self, rhs: Self) -> f32 {
        self.0.distance(rhs.0)
    }

    #[inline]
    pub fn min(self, rhs: Self) -> Self {
        Self(self.0.min(rhs.0))
    }

    #[inline]
    pub fn max(self, rhs: Self) -> Self {
        Self(self.0.max(rhs.0))
    }

    #[inline]
    pub fn abs(self) -> Self {
        Self(self.0.abs())
    }

    #[inline]
    pub fn to_array(self) -> [f32; 3] {
        [self.0.x, self.0.y, self.0.z]
    }

    /// Compute the normal of a triangle defined by three vertices.
    /// Result is not normalized.
    #[inline]
    pub fn normal_tri(v1: Vec3, v2: Vec3, v3: Vec3) -> Vec3 {
        let e1 = v2 - v1;
        let e2 = v3 - v1;
        e1.cross(e2)
    }

    /// Compute the area of a triangle defined by three vertices.
    #[inline]
    pub fn area_tri_v3(v1: Vec3, v2: Vec3, v3: Vec3) -> f32 {
        let n = Self::normal_tri(v1, v2, v3);
        n.length() * 0.5
    }
}

impl Deref for Vec3 {
    type Target = glam::Vec3A;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Vec3 {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Default for Vec3 {
    #[inline]
    fn default() -> Self {
        Self::ZERO
    }
}

impl std::fmt::Display for Vec3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.0.x, self.0.y, self.0.z)
    }
}

impl From<glam::Vec3A> for Vec3 {
    #[inline]
    fn from(v: glam::Vec3A) -> Self {
        Self(v)
    }
}

impl From<Vec3> for glam::Vec3A {
    #[inline]
    fn from(v: Vec3) -> Self {
        v.0
    }
}

impl From<glam::Vec3> for Vec3 {
    #[inline]
    fn from(v: glam::Vec3) -> Self {
        Self(glam::Vec3A::from(v))
    }
}

impl From<Vec3> for glam::Vec3 {
    #[inline]
    fn from(v: Vec3) -> Self {
        glam::Vec3::new(v.0.x, v.0.y, v.0.z)
    }
}

impl From<[f32; 3]> for Vec3 {
    #[inline]
    fn from(a: [f32; 3]) -> Self {
        Self(glam::Vec3A::from_array(a))
    }
}

impl From<Vec3> for [f32; 3] {
    #[inline]
    fn from(v: Vec3) -> Self {
        v.to_array()
    }
}

// --- Custom Serde (serialize only x, y, z) ---

impl Serialize for Vec3 {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeTuple;
        let mut tup = serializer.serialize_tuple(3)?;
        tup.serialize_element(&self.0.x)?;
        tup.serialize_element(&self.0.y)?;
        tup.serialize_element(&self.0.z)?;
        tup.end()
    }
}

impl<'de> Deserialize<'de> for Vec3 {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let (x, y, z) = <(f32, f32, f32)>::deserialize(deserializer)?;
        Ok(Self::new(x, y, z))
    }
}

// --- Operators ---

impl Add for Vec3 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for Vec3 {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Sub for Vec3 {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Self(self.0 - rhs.0)
    }
}

impl SubAssign for Vec3 {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl Mul<f32> for Vec3 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: f32) -> Self {
        Self(self.0 * rhs)
    }
}

impl Mul<Vec3> for f32 {
    type Output = Vec3;
    #[inline]
    fn mul(self, rhs: Vec3) -> Vec3 {
        Vec3(self * rhs.0)
    }
}

impl MulAssign<f32> for Vec3 {
    #[inline]
    fn mul_assign(&mut self, rhs: f32) {
        self.0 *= rhs;
    }
}

impl Mul for Vec3 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self {
        Self(self.0 * rhs.0)
    }
}

impl Div<f32> for Vec3 {
    type Output = Self;
    #[inline]
    fn div(self, rhs: f32) -> Self {
        Self(self.0 / rhs)
    }
}

impl DivAssign<f32> for Vec3 {
    #[inline]
    fn div_assign(&mut self, rhs: f32) {
        self.0 /= rhs;
    }
}

impl Neg for Vec3 {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Self(-self.0)
    }
}

// --- Trait impls ---

impl Lerp for Vec3 {
    #[inline]
    fn lerp(self, other: Self, t: f32) -> Self {
        Self(self.0.lerp(other.0, t))
    }
}

impl ApproxEq for Vec3 {
    #[inline]
    fn approx_eq(&self, other: &Self, epsilon: f32) -> bool {
        (self.0.x - other.0.x).abs() <= epsilon
            && (self.0.y - other.0.y).abs() <= epsilon
            && (self.0.z - other.0.z).abs() <= epsilon
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_ops() {
        let a = Vec3::new(1.0, 2.0, 3.0);
        let b = Vec3::new(4.0, 5.0, 6.0);
        assert_eq!(a + b, Vec3::new(5.0, 7.0, 9.0));
        assert_eq!(a - b, Vec3::new(-3.0, -3.0, -3.0));
    }

    #[test]
    fn test_cross() {
        let x = Vec3::X;
        let y = Vec3::Y;
        let z = x.cross(y);
        assert!(z.approx_eq(&Vec3::Z, 1e-6));
    }

    #[test]
    fn test_normal_tri() {
        let v1 = Vec3::new(0.0, 0.0, 0.0);
        let v2 = Vec3::new(1.0, 0.0, 0.0);
        let v3 = Vec3::new(0.0, 1.0, 0.0);
        let n = Vec3::normal_tri(v1, v2, v3);
        assert!(n.approx_eq(&Vec3::new(0.0, 0.0, 1.0), 1e-6));
    }

    #[test]
    fn test_area_tri() {
        let v1 = Vec3::new(0.0, 0.0, 0.0);
        let v2 = Vec3::new(1.0, 0.0, 0.0);
        let v3 = Vec3::new(0.0, 1.0, 0.0);
        let area = Vec3::area_tri_v3(v1, v2, v3);
        assert!((area - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_serde_roundtrip() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        let json = serde_json::to_string(&v).unwrap();
        let v2: Vec3 = serde_json::from_str(&json).unwrap();
        assert!(v.approx_eq(&v2, 1e-6));
    }

    #[test]
    fn test_normalize_or_zero() {
        let v = Vec3::ZERO;
        assert!(v.normalize_or_zero().approx_eq(&Vec3::ZERO, 1e-6));
        let v2 = Vec3::new(0.0, 3.0, 4.0);
        let n = v2.normalize_or_zero();
        assert!((n.length() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_display() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        let s = format!("{v}");
        assert!(s.contains("1"));
        assert!(s.contains("2"));
        assert!(s.contains("3"));
    }

    #[test]
    fn test_neg() {
        let v = Vec3::new(1.0, -2.0, 3.0);
        let n = -v;
        assert!(n.approx_eq(&Vec3::new(-1.0, 2.0, -3.0), 1e-6));
    }

    #[test]
    fn test_from_array_roundtrip() {
        let arr = [1.0_f32, 2.0, 3.0];
        let v: Vec3 = arr.into();
        let arr2: [f32; 3] = v.into();
        assert_eq!(arr, arr2);
    }
}
