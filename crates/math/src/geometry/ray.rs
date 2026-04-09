use serde::{Deserialize, Serialize};

use crate::geometry::{Aabb, Plane};
use crate::traits::ApproxEq;
use crate::vec::Vec3;

/// A ray defined by an origin point and direction.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl Ray {
    #[inline]
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self { origin, direction }
    }

    /// Get a point along the ray at parameter `t`.
    #[inline]
    pub fn point_at(&self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }

    /// Intersect the ray with a plane.
    /// Returns `Some(t)` if the ray hits the plane, `None` if parallel.
    pub fn intersect_plane(&self, plane: &Plane) -> Option<f32> {
        let denom = plane.normal.dot(self.direction);
        if denom.abs() < 1e-8 {
            return None;
        }
        let t = -(plane.normal.dot(self.origin) + plane.distance) / denom;
        if t >= 0.0 { Some(t) } else { None }
    }

    /// Intersect the ray with an axis-aligned bounding box.
    /// Returns `Some((t_min, t_max))` if the ray hits, `None` if it misses.
    pub fn intersect_aabb(&self, aabb: &Aabb) -> Option<(f32, f32)> {
        let inv_dir = Vec3::new(
            1.0 / self.direction.x(),
            1.0 / self.direction.y(),
            1.0 / self.direction.z(),
        );

        let t1 = (aabb.min.x() - self.origin.x()) * inv_dir.x();
        let t2 = (aabb.max.x() - self.origin.x()) * inv_dir.x();
        let t3 = (aabb.min.y() - self.origin.y()) * inv_dir.y();
        let t4 = (aabb.max.y() - self.origin.y()) * inv_dir.y();
        let t5 = (aabb.min.z() - self.origin.z()) * inv_dir.z();
        let t6 = (aabb.max.z() - self.origin.z()) * inv_dir.z();

        let tmin = t1.min(t2).max(t3.min(t4)).max(t5.min(t6));
        let tmax = t1.max(t2).min(t3.max(t4)).min(t5.max(t6));

        if tmax < 0.0 || tmin > tmax {
            None
        } else {
            Some((tmin.max(0.0), tmax))
        }
    }
}

impl ApproxEq for Ray {
    #[inline]
    fn approx_eq(&self, other: &Self, epsilon: f32) -> bool {
        self.origin.approx_eq(&other.origin, epsilon)
            && self.direction.approx_eq(&other.direction, epsilon)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_at() {
        let ray = Ray::new(Vec3::ZERO, Vec3::X);
        let p = ray.point_at(5.0);
        assert!(p.approx_eq(&Vec3::new(5.0, 0.0, 0.0), 1e-6));
    }

    #[test]
    fn test_plane_intersection() {
        let ray = Ray::new(Vec3::new(0.0, 0.0, -5.0), Vec3::Z);
        let plane = Plane::new(Vec3::Z, 0.0);
        let t = ray.intersect_plane(&plane).unwrap();
        assert!((t - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_aabb_intersection() {
        let ray = Ray::new(Vec3::new(0.0, 0.0, -5.0), Vec3::Z);
        let aabb = Aabb::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
        let hit = ray.intersect_aabb(&aabb);
        assert!(hit.is_some());
        let (tmin, tmax) = hit.unwrap();
        assert!((tmin - 4.0).abs() < 1e-6);
        assert!((tmax - 6.0).abs() < 1e-6);
    }
}
