use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use crate::traits::{ApproxEq, Lerp};

/// 4D vector wrapping `glam::Vec4`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[repr(C)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

// SAFETY: Vec4 is repr(C) with only f32 fields.
unsafe impl Zeroable for Vec4 {}
unsafe impl Pod for Vec4 {}

impl Vec4 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0, z: 0.0, w: 0.0 };
    pub const ONE: Self = Self { x: 1.0, y: 1.0, z: 1.0, w: 1.0 };
    pub const X: Self = Self { x: 1.0, y: 0.0, z: 0.0, w: 0.0 };
    pub const Y: Self = Self { x: 0.0, y: 1.0, z: 0.0, w: 0.0 };
    pub const Z: Self = Self { x: 0.0, y: 0.0, z: 1.0, w: 0.0 };
    pub const W: Self = Self { x: 0.0, y: 0.0, z: 0.0, w: 1.0 };

    #[inline]
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    #[inline]
    pub const fn splat(v: f32) -> Self {
        Self { x: v, y: v, z: v, w: v }
    }

    #[inline]
    pub fn length(self) -> f32 {
        self.to_glam().length()
    }

    #[inline]
    pub fn length_squared(self) -> f32 {
        self.to_glam().length_squared()
    }

    #[inline]
    pub fn normalize(self) -> Self {
        Self::from_glam(self.to_glam().normalize())
    }

    #[inline]
    pub fn dot(self, rhs: Self) -> f32 {
        self.to_glam().dot(rhs.to_glam())
    }

    #[inline]
    pub fn min(self, rhs: Self) -> Self {
        Self::from_glam(self.to_glam().min(rhs.to_glam()))
    }

    #[inline]
    pub fn max(self, rhs: Self) -> Self {
        Self::from_glam(self.to_glam().max(rhs.to_glam()))
    }

    #[inline]
    pub fn abs(self) -> Self {
        Self::from_glam(self.to_glam().abs())
    }

    #[inline]
    pub(crate) fn to_glam(self) -> glam::Vec4 {
        glam::Vec4::new(self.x, self.y, self.z, self.w)
    }

    #[inline]
    pub(crate) fn from_glam(v: glam::Vec4) -> Self {
        Self { x: v.x, y: v.y, z: v.z, w: v.w }
    }
}

impl Default for Vec4 {
    #[inline]
    fn default() -> Self {
        Self::ZERO
    }
}

impl fmt::Display for Vec4 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {}, {}, {})", self.x, self.y, self.z, self.w)
    }
}

impl From<glam::Vec4> for Vec4 {
    #[inline]
    fn from(v: glam::Vec4) -> Self {
        Self::from_glam(v)
    }
}

impl From<Vec4> for glam::Vec4 {
    #[inline]
    fn from(v: Vec4) -> Self {
        v.to_glam()
    }
}

impl From<[f32; 4]> for Vec4 {
    #[inline]
    fn from(a: [f32; 4]) -> Self {
        Self { x: a[0], y: a[1], z: a[2], w: a[3] }
    }
}

impl From<Vec4> for [f32; 4] {
    #[inline]
    fn from(v: Vec4) -> Self {
        [v.x, v.y, v.z, v.w]
    }
}

// --- Operators ---

impl Add for Vec4 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self { x: self.x + rhs.x, y: self.y + rhs.y, z: self.z + rhs.z, w: self.w + rhs.w }
    }
}

impl AddAssign for Vec4 {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
        self.w += rhs.w;
    }
}

impl Sub for Vec4 {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Self { x: self.x - rhs.x, y: self.y - rhs.y, z: self.z - rhs.z, w: self.w - rhs.w }
    }
}

impl SubAssign for Vec4 {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
        self.w -= rhs.w;
    }
}

impl Mul<f32> for Vec4 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: f32) -> Self {
        Self { x: self.x * rhs, y: self.y * rhs, z: self.z * rhs, w: self.w * rhs }
    }
}

impl Mul<Vec4> for f32 {
    type Output = Vec4;
    #[inline]
    fn mul(self, rhs: Vec4) -> Vec4 {
        Vec4 { x: self * rhs.x, y: self * rhs.y, z: self * rhs.z, w: self * rhs.w }
    }
}

impl MulAssign<f32> for Vec4 {
    #[inline]
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
        self.w *= rhs;
    }
}

impl Mul for Vec4 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self {
        Self { x: self.x * rhs.x, y: self.y * rhs.y, z: self.z * rhs.z, w: self.w * rhs.w }
    }
}

impl Div<f32> for Vec4 {
    type Output = Self;
    #[inline]
    fn div(self, rhs: f32) -> Self {
        Self { x: self.x / rhs, y: self.y / rhs, z: self.z / rhs, w: self.w / rhs }
    }
}

impl DivAssign<f32> for Vec4 {
    #[inline]
    fn div_assign(&mut self, rhs: f32) {
        self.x /= rhs;
        self.y /= rhs;
        self.z /= rhs;
        self.w /= rhs;
    }
}

impl Neg for Vec4 {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Self { x: -self.x, y: -self.y, z: -self.z, w: -self.w }
    }
}

// --- Trait impls ---

impl Lerp for Vec4 {
    #[inline]
    fn lerp(self, other: Self, t: f32) -> Self {
        Self {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
            z: self.z + (other.z - self.z) * t,
            w: self.w + (other.w - self.w) * t,
        }
    }
}

impl ApproxEq for Vec4 {
    #[inline]
    fn approx_eq(&self, other: &Self, epsilon: f32) -> bool {
        (self.x - other.x).abs() <= epsilon
            && (self.y - other.y).abs() <= epsilon
            && (self.z - other.z).abs() <= epsilon
            && (self.w - other.w).abs() <= epsilon
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_ops() {
        let a = Vec4::new(1.0, 2.0, 3.0, 4.0);
        let b = Vec4::new(5.0, 6.0, 7.0, 8.0);
        assert_eq!(a + b, Vec4::new(6.0, 8.0, 10.0, 12.0));
    }

    #[test]
    fn test_pod() {
        let v = Vec4::new(1.0, 2.0, 3.0, 4.0);
        let bytes = bytemuck::bytes_of(&v);
        assert_eq!(bytes.len(), 16);
    }

    #[test]
    fn test_lerp() {
        let a = Vec4::ZERO;
        let b = Vec4::new(10.0, 20.0, 30.0, 40.0);
        let mid = a.lerp(b, 0.5);
        assert!(mid.approx_eq(&Vec4::new(5.0, 10.0, 15.0, 20.0), 1e-6));
    }

    #[test]
    fn test_display() {
        let v = Vec4::new(1.0, 2.0, 3.0, 4.0);
        let s = format!("{v}");
        assert!(s.contains("1"));
        assert!(s.contains("4"));
    }

    #[test]
    fn test_default() {
        assert_eq!(Vec4::default(), Vec4::ZERO);
    }
}
