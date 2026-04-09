use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::traits::{ApproxEq, Lerp};
use crate::vec::Vec3;

/// Quaternion wrapping `glam::Quat`.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(transparent)]
pub struct Quat(pub glam::Quat);

impl Quat {
    pub const IDENTITY: Self = Self(glam::Quat::IDENTITY);

    #[inline]
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self(glam::Quat::from_xyzw(x, y, z, w))
    }

    #[inline]
    pub fn from_xyzw(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self(glam::Quat::from_xyzw(x, y, z, w))
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
    pub fn w(self) -> f32 {
        self.0.w
    }

    /// Create a quaternion from an axis and angle (radians).
    #[inline]
    pub fn from_axis_angle(axis: Vec3, angle: f32) -> Self {
        Self(glam::Quat::from_axis_angle(glam::Vec3::from(axis), angle))
    }

    /// Create a quaternion from Euler angles (radians) in YXZ order.
    #[inline]
    pub fn from_euler(yaw: f32, pitch: f32, roll: f32) -> Self {
        Self(glam::Quat::from_euler(glam::EulerRot::YXZ, yaw, pitch, roll))
    }

    /// Create a quaternion from Euler angles in a specified order.
    ///
    /// Supports all 6 Euler rotation orders used by Blender:
    /// XYZ, XZY, YXZ, YZX, ZXY, ZYX.
    ///
    /// The three angles correspond to rotations around the axes in the order
    /// specified: `a` is the first axis, `b` the second, `c` the third.
    #[inline]
    pub fn from_euler_order(order: EulerOrder, a: f32, b: f32, c: f32) -> Self {
        let rot = match order {
            EulerOrder::XYZ => glam::EulerRot::XYZ,
            EulerOrder::XZY => glam::EulerRot::XZY,
            EulerOrder::YXZ => glam::EulerRot::YXZ,
            EulerOrder::YZX => glam::EulerRot::YZX,
            EulerOrder::ZXY => glam::EulerRot::ZXY,
            EulerOrder::ZYX => glam::EulerRot::ZYX,
        };
        Self(glam::Quat::from_euler(rot, a, b, c))
    }

    /// Create a quaternion from a rotation matrix.
    #[inline]
    pub fn from_mat4(mat: &crate::mat::Mat4) -> Self {
        Self(glam::Quat::from_mat4(&mat.0))
    }

    /// Normalize the quaternion.
    #[inline]
    pub fn normalize(self) -> Self {
        Self(self.0.normalize())
    }

    /// Conjugate (inverse for unit quaternions).
    #[inline]
    pub fn conjugate(self) -> Self {
        Self(self.0.conjugate())
    }

    /// Inverse of the quaternion.
    #[inline]
    pub fn inverse(self) -> Self {
        Self(self.0.inverse())
    }

    /// Dot product with another quaternion.
    #[inline]
    pub fn dot(self, rhs: Self) -> f32 {
        self.0.dot(rhs.0)
    }

    /// Length / magnitude.
    #[inline]
    pub fn length(self) -> f32 {
        self.0.length()
    }

    /// Spherical linear interpolation.
    #[inline]
    pub fn slerp(self, other: Self, t: f32) -> Self {
        Self(self.0.slerp(other.0, t))
    }

    /// Rotate a Vec3 by this quaternion.
    #[inline]
    pub fn mul_vec3(self, v: Vec3) -> Vec3 {
        Vec3::from(glam::Vec3A::from(self.0.mul_vec3(glam::Vec3::from(v))))
    }

    /// Convert to axis-angle representation. Returns (axis, angle).
    #[inline]
    pub fn to_axis_angle(self) -> (Vec3, f32) {
        let (axis, angle) = self.0.to_axis_angle();
        (Vec3::from(glam::Vec3A::from(axis)), angle)
    }

    /// Convert to Euler angles (YXZ order). Returns (yaw, pitch, roll).
    #[inline]
    pub fn to_euler(self) -> (f32, f32, f32) {
        self.0.to_euler(glam::EulerRot::YXZ)
    }

    /// Convert to Euler angles in a specified order.
    ///
    /// Returns `(a, b, c)` where the angles correspond to rotations around
    /// the axes in the specified order.
    #[inline]
    pub fn to_euler_order(self, order: EulerOrder) -> (f32, f32, f32) {
        let rot = match order {
            EulerOrder::XYZ => glam::EulerRot::XYZ,
            EulerOrder::XZY => glam::EulerRot::XZY,
            EulerOrder::YXZ => glam::EulerRot::YXZ,
            EulerOrder::YZX => glam::EulerRot::YZX,
            EulerOrder::ZXY => glam::EulerRot::ZXY,
            EulerOrder::ZYX => glam::EulerRot::ZYX,
        };
        self.0.to_euler(rot)
    }
}

/// All six Euler rotation orders supported by Blender.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EulerOrder {
    XYZ,
    XZY,
    YXZ,
    YZX,
    ZXY,
    ZYX,
}

impl Default for Quat {
    #[inline]
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl From<glam::Quat> for Quat {
    #[inline]
    fn from(q: glam::Quat) -> Self {
        Self(q)
    }
}

impl From<Quat> for glam::Quat {
    #[inline]
    fn from(q: Quat) -> Self {
        q.0
    }
}

impl std::ops::Mul for Quat {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self {
        Self(self.0 * rhs.0)
    }
}

impl std::ops::Mul<Vec3> for Quat {
    type Output = Vec3;
    #[inline]
    fn mul(self, rhs: Vec3) -> Vec3 {
        self.mul_vec3(rhs)
    }
}

// --- Lerp uses slerp ---

impl Lerp for Quat {
    #[inline]
    fn lerp(self, other: Self, t: f32) -> Self {
        self.slerp(other, t)
    }
}

impl ApproxEq for Quat {
    #[inline]
    fn approx_eq(&self, other: &Self, epsilon: f32) -> bool {
        // Quaternions q and -q represent the same rotation.
        let d = self.dot(*other).abs();
        (d - 1.0).abs() <= epsilon
    }
}

// --- Custom Serde ---

impl Serialize for Quat {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeTuple;
        let mut tup = serializer.serialize_tuple(4)?;
        tup.serialize_element(&self.0.x)?;
        tup.serialize_element(&self.0.y)?;
        tup.serialize_element(&self.0.z)?;
        tup.serialize_element(&self.0.w)?;
        tup.end()
    }
}

impl<'de> Deserialize<'de> for Quat {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let (x, y, z, w) = <(f32, f32, f32, f32)>::deserialize(deserializer)?;
        Ok(Self(glam::Quat::from_xyzw(x, y, z, w)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity() {
        let q = Quat::IDENTITY;
        let v = Vec3::new(1.0, 0.0, 0.0);
        let r = q * v;
        assert!(r.approx_eq(&v, 1e-6));
    }

    #[test]
    fn test_axis_angle_roundtrip() {
        let q = Quat::from_axis_angle(Vec3::Y, std::f32::consts::FRAC_PI_2);
        let (axis, angle) = q.to_axis_angle();
        assert!(axis.approx_eq(&Vec3::Y, 1e-6));
        assert!((angle - std::f32::consts::FRAC_PI_2).abs() < 1e-6);
    }

    #[test]
    fn test_slerp() {
        let a = Quat::IDENTITY;
        let b = Quat::from_axis_angle(Vec3::Y, std::f32::consts::FRAC_PI_2);
        let mid = a.slerp(b, 0.5);
        let expected = Quat::from_axis_angle(Vec3::Y, std::f32::consts::FRAC_PI_4);
        assert!(mid.approx_eq(&expected, 1e-4));
    }

    #[test]
    fn test_serde_roundtrip() {
        let q = Quat::from_axis_angle(Vec3::Z, 1.0);
        let json = serde_json::to_string(&q).unwrap();
        let q2: Quat = serde_json::from_str(&json).unwrap();
        assert!(q.approx_eq(&q2, 1e-6));
    }

    #[test]
    fn test_euler_order_xyz_roundtrip() {
        let q = Quat::from_euler_order(EulerOrder::XYZ, 0.3, 0.5, 0.7);
        let (a, b, c) = q.to_euler_order(EulerOrder::XYZ);
        let q2 = Quat::from_euler_order(EulerOrder::XYZ, a, b, c);
        assert!(q.approx_eq(&q2, 1e-4));
    }

    #[test]
    fn test_euler_order_xzy_roundtrip() {
        let q = Quat::from_euler_order(EulerOrder::XZY, 0.3, 0.5, 0.7);
        let (a, b, c) = q.to_euler_order(EulerOrder::XZY);
        let q2 = Quat::from_euler_order(EulerOrder::XZY, a, b, c);
        assert!(q.approx_eq(&q2, 1e-4));
    }

    #[test]
    fn test_euler_order_yxz_roundtrip() {
        let q = Quat::from_euler_order(EulerOrder::YXZ, 0.3, 0.5, 0.7);
        let (a, b, c) = q.to_euler_order(EulerOrder::YXZ);
        let q2 = Quat::from_euler_order(EulerOrder::YXZ, a, b, c);
        assert!(q.approx_eq(&q2, 1e-4));
    }

    #[test]
    fn test_euler_order_yzx_roundtrip() {
        let q = Quat::from_euler_order(EulerOrder::YZX, 0.3, 0.5, 0.7);
        let (a, b, c) = q.to_euler_order(EulerOrder::YZX);
        let q2 = Quat::from_euler_order(EulerOrder::YZX, a, b, c);
        assert!(q.approx_eq(&q2, 1e-4));
    }

    #[test]
    fn test_euler_order_zxy_roundtrip() {
        let q = Quat::from_euler_order(EulerOrder::ZXY, 0.3, 0.5, 0.7);
        let (a, b, c) = q.to_euler_order(EulerOrder::ZXY);
        let q2 = Quat::from_euler_order(EulerOrder::ZXY, a, b, c);
        assert!(q.approx_eq(&q2, 1e-4));
    }

    #[test]
    fn test_euler_order_zyx_roundtrip() {
        let q = Quat::from_euler_order(EulerOrder::ZYX, 0.3, 0.5, 0.7);
        let (a, b, c) = q.to_euler_order(EulerOrder::ZYX);
        let q2 = Quat::from_euler_order(EulerOrder::ZYX, a, b, c);
        assert!(q.approx_eq(&q2, 1e-4));
    }
}
