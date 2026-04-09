use serde::{Deserialize, Serialize};

use crate::mat::Mat4;
use crate::quat::Quat;
use crate::traits::{ApproxEq, Lerp};
use crate::vec::Vec3;

/// A 3D transform consisting of position, rotation, and scale.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Transform {
    pub const IDENTITY: Self = Self {
        position: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    };

    /// Create a new transform.
    #[inline]
    pub fn new(position: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Self { position, rotation, scale }
    }

    /// Create a transform from just a translation.
    #[inline]
    pub fn from_translation(position: Vec3) -> Self {
        Self {
            position,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }

    /// Create a transform from just a rotation.
    #[inline]
    pub fn from_rotation(rotation: Quat) -> Self {
        Self {
            position: Vec3::ZERO,
            rotation,
            scale: Vec3::ONE,
        }
    }

    /// Convert to a 4x4 matrix.
    #[inline]
    pub fn to_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }

    /// Create a transform from a 4x4 matrix.
    /// Returns `None` if the matrix cannot be decomposed.
    pub fn from_matrix(mat: &Mat4) -> Option<Self> {
        let (scale, rotation, position) = mat.to_scale_rotation_translation()?;
        Some(Self { position, rotation, scale })
    }

    /// Transform a point.
    #[inline]
    pub fn transform_point(&self, point: Vec3) -> Vec3 {
        self.position + self.rotation * (self.scale * point)
    }

    /// Transform a direction vector (ignores translation).
    #[inline]
    pub fn transform_vector(&self, vector: Vec3) -> Vec3 {
        self.rotation * (self.scale * vector)
    }

    /// Get the inverse transform.
    pub fn inverse(&self) -> Self {
        let inv_rotation = self.rotation.inverse();
        let inv_scale = Vec3::new(
            1.0 / self.scale.x(),
            1.0 / self.scale.y(),
            1.0 / self.scale.z(),
        );
        let inv_position = inv_rotation * (inv_scale * -self.position);
        Self {
            position: inv_position,
            rotation: inv_rotation,
            scale: inv_scale,
        }
    }

    /// Combine two transforms: apply `self` then `other`.
    #[inline]
    pub fn then(&self, other: &Transform) -> Transform {
        Transform {
            position: other.transform_point(self.position),
            rotation: other.rotation * self.rotation,
            scale: other.scale * self.scale,
        }
    }
}

impl Default for Transform {
    #[inline]
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Lerp for Transform {
    #[inline]
    fn lerp(self, other: Self, t: f32) -> Self {
        Self {
            position: self.position.lerp(other.position, t),
            rotation: self.rotation.lerp(other.rotation, t),
            scale: self.scale.lerp(other.scale, t),
        }
    }
}

impl ApproxEq for Transform {
    #[inline]
    fn approx_eq(&self, other: &Self, epsilon: f32) -> bool {
        self.position.approx_eq(&other.position, epsilon)
            && self.rotation.approx_eq(&other.rotation, epsilon)
            && self.scale.approx_eq(&other.scale, epsilon)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity() {
        let t = Transform::IDENTITY;
        let p = Vec3::new(1.0, 2.0, 3.0);
        assert!(t.transform_point(p).approx_eq(&p, 1e-6));
    }

    #[test]
    fn test_matrix_roundtrip() {
        let t = Transform::new(
            Vec3::new(1.0, 2.0, 3.0),
            Quat::from_axis_angle(Vec3::Y, std::f32::consts::FRAC_PI_4),
            Vec3::new(2.0, 2.0, 2.0),
        );
        let m = t.to_matrix();
        let t2 = Transform::from_matrix(&m).unwrap();
        assert!(t.approx_eq(&t2, 1e-4));
    }

    #[test]
    fn test_matrix_roundtrip_negative_scale() {
        let t = Transform::new(
            Vec3::new(5.0, 6.0, 7.0),
            Quat::from_axis_angle(Vec3::Z, 0.5),
            Vec3::new(-1.0, 2.0, 3.0),
        );
        let m = t.to_matrix();
        let t2 = Transform::from_matrix(&m).unwrap();
        // The scale should have a negative x component.
        assert!(t2.scale.x() < 0.0, "expected negative x scale");
        // Verify that the recomposed matrix matches.
        let m2 = t2.to_matrix();
        assert!(m.approx_eq(&m2, 1e-3));
    }
}
