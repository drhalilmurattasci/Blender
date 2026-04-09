use serde::{Deserialize, Serialize};

use crate::traits::ApproxEq;
use crate::vec::Vec3;

/// Axis-aligned bounding box.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb {
    /// Create a new AABB from min and max corners.
    #[inline]
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    /// Create an AABB that contains nothing (inverted bounds).
    #[inline]
    pub fn empty() -> Self {
        Self {
            min: Vec3::splat(f32::INFINITY),
            max: Vec3::splat(f32::NEG_INFINITY),
        }
    }

    /// Create an AABB from a center and half-extents.
    #[inline]
    pub fn from_center_half_extents(center: Vec3, half_extents: Vec3) -> Self {
        Self {
            min: center - half_extents,
            max: center + half_extents,
        }
    }

    /// Get the center of the AABB.
    #[inline]
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    /// Get the half-extents (half-size) of the AABB.
    #[inline]
    pub fn half_extents(&self) -> Vec3 {
        (self.max - self.min) * 0.5
    }

    /// Get the size of the AABB.
    #[inline]
    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }

    /// Check if a point is inside the AABB.
    #[inline]
    pub fn contains(&self, point: Vec3) -> bool {
        point.x() >= self.min.x()
            && point.x() <= self.max.x()
            && point.y() >= self.min.y()
            && point.y() <= self.max.y()
            && point.z() >= self.min.z()
            && point.z() <= self.max.z()
    }

    /// Check if this AABB intersects another.
    #[inline]
    pub fn intersects(&self, other: &Aabb) -> bool {
        self.min.x() <= other.max.x()
            && self.max.x() >= other.min.x()
            && self.min.y() <= other.max.y()
            && self.max.y() >= other.min.y()
            && self.min.z() <= other.max.z()
            && self.max.z() >= other.min.z()
    }

    /// Expand the AABB to include a point.
    #[inline]
    pub fn expand(&self, point: Vec3) -> Self {
        Self {
            min: self.min.min(point),
            max: self.max.max(point),
        }
    }

    /// Merge two AABBs into one that contains both.
    #[inline]
    pub fn merge(&self, other: &Aabb) -> Self {
        Self {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }

    /// Ray intersection test. Returns `Some((t_min, t_max))` or `None`.
    pub fn ray_intersect(&self, origin: Vec3, direction: Vec3) -> Option<(f32, f32)> {
        let inv_dir = Vec3::new(
            1.0 / direction.x(),
            1.0 / direction.y(),
            1.0 / direction.z(),
        );

        let t1 = (self.min.x() - origin.x()) * inv_dir.x();
        let t2 = (self.max.x() - origin.x()) * inv_dir.x();
        let t3 = (self.min.y() - origin.y()) * inv_dir.y();
        let t4 = (self.max.y() - origin.y()) * inv_dir.y();
        let t5 = (self.min.z() - origin.z()) * inv_dir.z();
        let t6 = (self.max.z() - origin.z()) * inv_dir.z();

        let tmin = t1.min(t2).max(t3.min(t4)).max(t5.min(t6));
        let tmax = t1.max(t2).min(t3.max(t4)).min(t5.max(t6));

        if tmax < 0.0 || tmin > tmax {
            None
        } else {
            Some((tmin.max(0.0), tmax))
        }
    }
}

impl Default for Aabb {
    #[inline]
    fn default() -> Self {
        Self::empty()
    }
}

impl ApproxEq for Aabb {
    #[inline]
    fn approx_eq(&self, other: &Self, epsilon: f32) -> bool {
        self.min.approx_eq(&other.min, epsilon) && self.max.approx_eq(&other.max, epsilon)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains() {
        let aabb = Aabb::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
        assert!(aabb.contains(Vec3::ZERO));
        assert!(!aabb.contains(Vec3::new(2.0, 0.0, 0.0)));
    }

    #[test]
    fn test_intersects() {
        let a = Aabb::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
        let b = Aabb::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(2.0, 2.0, 2.0));
        assert!(a.intersects(&b));
        let c = Aabb::new(Vec3::new(5.0, 5.0, 5.0), Vec3::new(6.0, 6.0, 6.0));
        assert!(!a.intersects(&c));
    }

    #[test]
    fn test_expand() {
        let aabb = Aabb::empty().expand(Vec3::ZERO).expand(Vec3::ONE);
        assert!(aabb.min.approx_eq(&Vec3::ZERO, 1e-6));
        assert!(aabb.max.approx_eq(&Vec3::ONE, 1e-6));
    }
}
