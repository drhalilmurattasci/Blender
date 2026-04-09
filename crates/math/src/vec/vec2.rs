use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use crate::traits::{ApproxEq, Lerp};

/// 2D vector wrapping `glam::Vec2`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[repr(C)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

// SAFETY: Vec2 is repr(C) with only f32 fields.
unsafe impl Zeroable for Vec2 {}
unsafe impl Pod for Vec2 {}

impl Vec2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
    pub const ONE: Self = Self { x: 1.0, y: 1.0 };
    pub const X: Self = Self { x: 1.0, y: 0.0 };
    pub const Y: Self = Self { x: 0.0, y: 1.0 };

    #[inline]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    #[inline]
    pub const fn splat(v: f32) -> Self {
        Self { x: v, y: v }
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
    pub fn distance(self, rhs: Self) -> f32 {
        self.to_glam().distance(rhs.to_glam())
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
    pub(crate) fn to_glam(self) -> glam::Vec2 {
        glam::Vec2::new(self.x, self.y)
    }

    #[inline]
    pub(crate) fn from_glam(v: glam::Vec2) -> Self {
        Self { x: v.x, y: v.y }
    }
}

impl From<glam::Vec2> for Vec2 {
    #[inline]
    fn from(v: glam::Vec2) -> Self {
        Self::from_glam(v)
    }
}

impl From<Vec2> for glam::Vec2 {
    #[inline]
    fn from(v: Vec2) -> Self {
        v.to_glam()
    }
}

impl From<[f32; 2]> for Vec2 {
    #[inline]
    fn from(a: [f32; 2]) -> Self {
        Self { x: a[0], y: a[1] }
    }
}

impl From<Vec2> for [f32; 2] {
    #[inline]
    fn from(v: Vec2) -> Self {
        [v.x, v.y]
    }
}

impl Default for Vec2 {
    #[inline]
    fn default() -> Self {
        Self::ZERO
    }
}

impl fmt::Display for Vec2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

// --- Operators ---

impl Add for Vec2 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self { x: self.x + rhs.x, y: self.y + rhs.y }
    }
}

impl AddAssign for Vec2 {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Sub for Vec2 {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Self { x: self.x - rhs.x, y: self.y - rhs.y }
    }
}

impl SubAssign for Vec2 {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl Mul<f32> for Vec2 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: f32) -> Self {
        Self { x: self.x * rhs, y: self.y * rhs }
    }
}

impl Mul<Vec2> for f32 {
    type Output = Vec2;
    #[inline]
    fn mul(self, rhs: Vec2) -> Vec2 {
        Vec2 { x: self * rhs.x, y: self * rhs.y }
    }
}

impl MulAssign<f32> for Vec2 {
    #[inline]
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl Mul for Vec2 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self {
        Self { x: self.x * rhs.x, y: self.y * rhs.y }
    }
}

impl Div<f32> for Vec2 {
    type Output = Self;
    #[inline]
    fn div(self, rhs: f32) -> Self {
        Self { x: self.x / rhs, y: self.y / rhs }
    }
}

impl DivAssign<f32> for Vec2 {
    #[inline]
    fn div_assign(&mut self, rhs: f32) {
        self.x /= rhs;
        self.y /= rhs;
    }
}

impl Neg for Vec2 {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Self { x: -self.x, y: -self.y }
    }
}

// --- Trait impls ---

impl Lerp for Vec2 {
    #[inline]
    fn lerp(self, other: Self, t: f32) -> Self {
        Self {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
        }
    }
}

impl ApproxEq for Vec2 {
    #[inline]
    fn approx_eq(&self, other: &Self, epsilon: f32) -> bool {
        (self.x - other.x).abs() <= epsilon && (self.y - other.y).abs() <= epsilon
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_ops() {
        let a = Vec2::new(1.0, 2.0);
        let b = Vec2::new(3.0, 4.0);
        assert_eq!(a + b, Vec2::new(4.0, 6.0));
        assert_eq!(a - b, Vec2::new(-2.0, -2.0));
        assert_eq!(a * 2.0, Vec2::new(2.0, 4.0));
    }

    #[test]
    fn test_lerp() {
        let a = Vec2::ZERO;
        let b = Vec2::new(10.0, 20.0);
        let mid = a.lerp(b, 0.5);
        assert!(mid.approx_eq(&Vec2::new(5.0, 10.0), 1e-6));
    }

    #[test]
    fn test_pod() {
        let v = Vec2::new(1.0, 2.0);
        let bytes = bytemuck::bytes_of(&v);
        assert_eq!(bytes.len(), 8);
    }

    #[test]
    fn test_normalize() {
        let v = Vec2::new(3.0, 4.0);
        let n = v.normalize();
        assert!((n.length() - 1.0).abs() < 1e-6);
        assert!((n.x - 0.6).abs() < 1e-6);
        assert!((n.y - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_distance() {
        let a = Vec2::new(0.0, 0.0);
        let b = Vec2::new(3.0, 4.0);
        assert!((a.distance(b) - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_min_max_abs() {
        let a = Vec2::new(-3.0, 5.0);
        let b = Vec2::new(2.0, -1.0);
        assert_eq!(a.min(b), Vec2::new(-3.0, -1.0));
        assert_eq!(a.max(b), Vec2::new(2.0, 5.0));
        assert_eq!(a.abs(), Vec2::new(3.0, 5.0));
    }

    #[test]
    fn test_neg() {
        let v = Vec2::new(1.0, -2.0);
        assert_eq!(-v, Vec2::new(-1.0, 2.0));
    }

    #[test]
    fn test_display() {
        let v = Vec2::new(1.0, 2.0);
        let s = format!("{v}");
        assert!(s.contains("1"));
        assert!(s.contains("2"));
    }

    #[test]
    fn test_default() {
        assert_eq!(Vec2::default(), Vec2::ZERO);
    }
}
