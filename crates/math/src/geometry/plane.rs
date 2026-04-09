use serde::{Deserialize, Serialize};

use crate::traits::ApproxEq;
use crate::vec::Vec3;

/// A plane defined by a normal vector and a distance from the origin.
/// The plane equation is: dot(normal, point) + distance = 0.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Plane {
    pub normal: Vec3,
    pub distance: f32,
}

impl Plane {
    /// Create a new plane from a normal and distance.
    #[inline]
    pub fn new(normal: Vec3, distance: f32) -> Self {
        Self { normal, distance }
    }

    /// Create a plane from a normal and a point on the plane.
    #[inline]
    pub fn from_point_normal(point: Vec3, normal: Vec3) -> Self {
        let d = -normal.dot(point);
        Self { normal, distance: d }
    }

    /// Create a plane from three points (counter-clockwise winding).
    #[inline]
    pub fn from_points(a: Vec3, b: Vec3, c: Vec3) -> Self {
        let normal = (b - a).cross(c - a).normalize();
        Self::from_point_normal(a, normal)
    }

    /// Signed distance from the plane to a point.
    /// Positive means the point is on the side the normal points to.
    #[inline]
    pub fn signed_distance(&self, point: Vec3) -> f32 {
        self.normal.dot(point) + self.distance
    }

    /// Project a point onto the plane.
    #[inline]
    pub fn project_point(&self, point: Vec3) -> Vec3 {
        point - self.normal * self.signed_distance(point)
    }

    /// Normalize the plane (make the normal unit length).
    #[inline]
    pub fn normalize(&self) -> Self {
        let len = self.normal.length();
        if len < 1e-10 {
            return *self;
        }
        Self {
            normal: self.normal / len,
            distance: self.distance / len,
        }
    }
}

impl ApproxEq for Plane {
    #[inline]
    fn approx_eq(&self, other: &Self, epsilon: f32) -> bool {
        self.normal.approx_eq(&other.normal, epsilon)
            && (self.distance - other.distance).abs() <= epsilon
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signed_distance() {
        let plane = Plane::new(Vec3::Y, 0.0);
        assert!((plane.signed_distance(Vec3::new(0.0, 5.0, 0.0)) - 5.0).abs() < 1e-6);
        assert!((plane.signed_distance(Vec3::new(0.0, -3.0, 0.0)) + 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_project_point() {
        let plane = Plane::new(Vec3::Y, 0.0);
        let p = plane.project_point(Vec3::new(3.0, 5.0, 7.0));
        assert!(p.approx_eq(&Vec3::new(3.0, 0.0, 7.0), 1e-6));
    }

    #[test]
    fn test_from_points() {
        let a = Vec3::new(0.0, 0.0, 0.0);
        let b = Vec3::new(1.0, 0.0, 0.0);
        let c = Vec3::new(0.0, 0.0, 1.0);
        let plane = Plane::from_points(a, b, c);
        // Normal should point in -Y direction for this winding
        assert!((plane.signed_distance(a)).abs() < 1e-6);
    }
}
