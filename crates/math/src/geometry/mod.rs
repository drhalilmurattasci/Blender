mod aabb;
mod plane;
mod ray;
mod transform;

pub use aabb::Aabb;
pub use plane::Plane;
pub use ray::Ray;
pub use transform::Transform;

use crate::vec::Vec3;

// --- Blender-inspired geometry functions ---

/// Compute barycentric-like interpolation weights for a point relative to a polygon.
///
/// Adapted from Blender's `interp_weights_poly_v3`. Given a point `co` and a
/// polygon defined by `verts`, returns a weight for each vertex such that
/// the weighted sum of the vertices approximates `co`.
///
/// Uses the mean value coordinates algorithm.
pub fn interp_weights_poly_v3(co: Vec3, verts: &[Vec3]) -> Vec<f32> {
    let n = verts.len();
    if n == 0 {
        return vec![];
    }
    if n == 1 {
        return vec![1.0];
    }
    if n == 2 {
        // Simple linear interpolation along the edge.
        let edge = verts[1] - verts[0];
        let len_sq = edge.length_squared();
        if len_sq < 1e-12 {
            return vec![0.5, 0.5];
        }
        let t = (co - verts[0]).dot(edge) / len_sq;
        let t = t.clamp(0.0, 1.0);
        return vec![1.0 - t, t];
    }

    // Mean value coordinates
    let mut weights = vec![0.0_f32; n];
    let mut diffs: Vec<Vec3> = verts.iter().map(|v| *v - co).collect();
    let dists: Vec<f32> = diffs.iter().map(|d| d.length()).collect();

    // Check if point is on a vertex
    for i in 0..n {
        if dists[i] < 1e-10 {
            weights[i] = 1.0;
            return weights;
        }
    }

    // Normalize direction vectors
    for i in 0..n {
        diffs[i] = diffs[i] / dists[i];
    }

    for i in 0..n {
        let j = (i + 1) % n;

        let cos_angle = diffs[i].dot(diffs[j]).clamp(-1.0, 1.0);

        // Check for edge-on-edge case: if the point lies exactly on the
        // edge between verts[i] and verts[j], the angle between the
        // direction vectors is PI (cos_angle ~ -1). In that case the
        // mean-value formula degenerates, so we fall back to linear
        // interpolation along that edge. This matches Blender's behaviour
        // for concave polygons and edge-coincident points.
        if cos_angle < -1.0 + 1e-6 {
            // Point is on edge i--j. Return edge-interpolated weights.
            let mut edge_weights = vec![0.0_f32; n];
            let total = dists[i] + dists[j];
            if total < 1e-10 {
                edge_weights[i] = 0.5;
                edge_weights[j] = 0.5;
            } else {
                // Closer to i => more weight on i's opposite vertex (j)
                // ... actually: weight is inversely proportional to distance.
                edge_weights[i] = dists[j] / total;
                edge_weights[j] = dists[i] / total;
            }
            return edge_weights;
        }

        let angle = cos_angle.acos();
        if angle.abs() < 1e-10 {
            continue;
        }

        let tan_half = (angle * 0.5).tan();

        if dists[i] > 1e-10 {
            weights[i] += tan_half / dists[i];
        }
        if dists[j] > 1e-10 {
            weights[j] += tan_half / dists[j];
        }
    }

    // Normalize
    let total_weight: f32 = weights.iter().sum();
    if total_weight > 1e-10 {
        for w in &mut weights {
            *w /= total_weight;
        }
    }

    weights
}

/// Find the closest point on a triangle to a given point.
///
/// Adapted from Blender's `closest_on_tri_to_point_v3`.
/// Returns the closest point and the barycentric coordinates (u, v, w).
pub fn closest_point_on_tri(p: Vec3, a: Vec3, b: Vec3, c: Vec3) -> (Vec3, f32, f32, f32) {
    let ab = b - a;
    let ac = c - a;
    let ap = p - a;

    let d1 = ab.dot(ap);
    let d2 = ac.dot(ap);
    if d1 <= 0.0 && d2 <= 0.0 {
        return (a, 1.0, 0.0, 0.0);
    }

    let bp = p - b;
    let d3 = ab.dot(bp);
    let d4 = ac.dot(bp);
    if d3 >= 0.0 && d4 <= d3 {
        return (b, 0.0, 1.0, 0.0);
    }

    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let ab_squared = d1 - d3;
        if ab_squared == 0.0 {
            return (a, 1.0, 0.0, 0.0);
        }
        let v = d1 / ab_squared;
        return (a + ab * v, 1.0 - v, v, 0.0);
    }

    let cp = p - c;
    let d5 = ab.dot(cp);
    let d6 = ac.dot(cp);
    if d6 >= 0.0 && d5 <= d6 {
        return (c, 0.0, 0.0, 1.0);
    }

    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let ac_squared = d2 - d6;
        if ac_squared == 0.0 {
            return (a, 1.0, 0.0, 0.0);
        }
        let w = d2 / ac_squared;
        return (a + ac * w, 1.0 - w, 0.0, w);
    }

    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let bc_squared = (d4 - d3) + (d5 - d6);
        if bc_squared == 0.0 {
            return (b, 0.0, 1.0, 0.0);
        }
        let w = (d4 - d3) / bc_squared;
        return (b + (c - b) * w, 0.0, 1.0 - w, w);
    }

    let denom = 1.0 / (va + vb + vc);
    let v = vb * denom;
    let w = vc * denom;
    let closest = a + ab * v + ac * w;
    (closest, 1.0 - v - w, v, w)
}

/// Project a point onto an infinite line defined by two endpoints.
///
/// Adapted from Blender's `closest_to_line_v3`. Returns the parametric
/// factor `lambda` along the line (`l1` at 0, `l2` at 1) and the closest
/// point.
pub fn closest_to_line_v3(p: Vec3, l1: Vec3, l2: Vec3) -> (Vec3, f32) {
    let u = l2 - l1;
    let h = p - l1;
    let denom = u.dot(u);
    if denom < 1e-35 {
        return (l1, 0.0);
    }
    let lambda = u.dot(h) / denom;
    let closest = l1 + u * lambda;
    (closest, lambda)
}

/// Project a point onto a line segment, clamping to endpoints.
///
/// Adapted from Blender's `closest_to_line_segment_v3`. Returns the
/// closest point and the clamped parametric factor in \[0, 1\].
pub fn closest_to_line_segment_v3(p: Vec3, l1: Vec3, l2: Vec3) -> (Vec3, f32) {
    let (cp, lambda) = closest_to_line_v3(p, l1, l2);
    if lambda <= 0.0 {
        (l1, 0.0)
    } else if lambda >= 1.0 {
        (l2, 1.0)
    } else {
        (cp, lambda)
    }
}

/// Squared distance from a point to a line segment.
///
/// Adapted from Blender's `dist_squared_to_line_segment_v3`.
pub fn dist_squared_to_line_segment_v3(p: Vec3, l1: Vec3, l2: Vec3) -> f32 {
    let (closest, _) = closest_to_line_segment_v3(p, l1, l2);
    (closest - p).length_squared()
}

/// Ray-triangle intersection using the Moeller-Trumbore algorithm.
///
/// Adapted from Blender's `isect_ray_tri_v3`. Returns `Some((t, u, v))`
/// where `t` is the distance along the ray direction and `(u, v)` are
/// barycentric coordinates. Returns `None` on miss.
pub fn isect_ray_tri_v3(
    ray_origin: Vec3,
    ray_direction: Vec3,
    v0: Vec3,
    v1: Vec3,
    v2: Vec3,
) -> Option<(f32, f32, f32)> {
    const EPSILON: f32 = 1e-8;

    let e1 = v1 - v0;
    let e2 = v2 - v0;
    let pvec = ray_direction.cross(e2);
    let det = e1.dot(pvec);

    if det > -EPSILON && det < EPSILON {
        return None;
    }

    let inv_det = 1.0 / det;
    let tvec = ray_origin - v0;
    let u = tvec.dot(pvec) * inv_det;
    if u < 0.0 || u > 1.0 {
        return None;
    }

    let qvec = tvec.cross(e1);
    let v = ray_direction.dot(qvec) * inv_det;
    if v < 0.0 || (u + v) > 1.0 {
        return None;
    }

    let t = e2.dot(qvec) * inv_det;
    if t < 0.0 {
        return None;
    }

    Some((t, u, v))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::ApproxEq;

    #[test]
    fn test_interp_weights_triangle() {
        let verts = [
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        ];
        let center = Vec3::new(1.0 / 3.0, 1.0 / 3.0, 0.0);
        let w = interp_weights_poly_v3(center, &verts);
        assert_eq!(w.len(), 3);
        let sum: f32 = w.iter().sum();
        assert!((sum - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_closest_point_on_tri_vertex() {
        let a = Vec3::new(0.0, 0.0, 0.0);
        let b = Vec3::new(1.0, 0.0, 0.0);
        let c = Vec3::new(0.0, 1.0, 0.0);
        let p = Vec3::new(-1.0, -1.0, 0.0);
        let (closest, u, _v, _w) = closest_point_on_tri(p, a, b, c);
        assert!(closest.approx_eq(&a, 1e-6));
        assert!((u - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_closest_point_on_tri_inside() {
        let a = Vec3::new(0.0, 0.0, 0.0);
        let b = Vec3::new(2.0, 0.0, 0.0);
        let c = Vec3::new(0.0, 2.0, 0.0);
        let p = Vec3::new(0.5, 0.5, 1.0);
        let (closest, _u, _v, _w) = closest_point_on_tri(p, a, b, c);
        // z should be 0 since triangle is on z=0 plane
        assert!((closest.z() - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_closest_point_on_tri_degenerate() {
        // Degenerate triangle (all same point)
        let a = Vec3::new(1.0, 1.0, 1.0);
        let (closest, _, _, _) = closest_point_on_tri(Vec3::ZERO, a, a, a);
        assert!(closest.approx_eq(&a, 1e-6));
    }

    #[test]
    fn test_closest_to_line_v3() {
        let l1 = Vec3::new(0.0, 0.0, 0.0);
        let l2 = Vec3::new(10.0, 0.0, 0.0);
        let p = Vec3::new(5.0, 3.0, 0.0);
        let (closest, lambda) = closest_to_line_v3(p, l1, l2);
        assert!(closest.approx_eq(&Vec3::new(5.0, 0.0, 0.0), 1e-6));
        assert!((lambda - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_closest_to_line_segment_clamped() {
        let l1 = Vec3::new(0.0, 0.0, 0.0);
        let l2 = Vec3::new(10.0, 0.0, 0.0);
        let p = Vec3::new(-5.0, 3.0, 0.0);
        let (closest, lambda) = closest_to_line_segment_v3(p, l1, l2);
        assert!(closest.approx_eq(&l1, 1e-6));
        assert!((lambda - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_dist_squared_to_line_segment() {
        let l1 = Vec3::new(0.0, 0.0, 0.0);
        let l2 = Vec3::new(10.0, 0.0, 0.0);
        let p = Vec3::new(5.0, 3.0, 0.0);
        let d2 = dist_squared_to_line_segment_v3(p, l1, l2);
        assert!((d2 - 9.0).abs() < 1e-6);
    }

    #[test]
    fn test_isect_ray_tri_hit() {
        let origin = Vec3::new(0.25, 0.25, -1.0);
        let dir = Vec3::Z;
        let v0 = Vec3::new(0.0, 0.0, 0.0);
        let v1 = Vec3::new(1.0, 0.0, 0.0);
        let v2 = Vec3::new(0.0, 1.0, 0.0);
        let result = isect_ray_tri_v3(origin, dir, v0, v1, v2);
        assert!(result.is_some());
        let (t, u, v) = result.unwrap();
        assert!((t - 1.0).abs() < 1e-6);
        assert!((u - 0.25).abs() < 1e-6);
        assert!((v - 0.25).abs() < 1e-6);
    }

    #[test]
    fn test_isect_ray_tri_miss() {
        let origin = Vec3::new(2.0, 2.0, -1.0);
        let dir = Vec3::Z;
        let v0 = Vec3::new(0.0, 0.0, 0.0);
        let v1 = Vec3::new(1.0, 0.0, 0.0);
        let v2 = Vec3::new(0.0, 1.0, 0.0);
        assert!(isect_ray_tri_v3(origin, dir, v0, v1, v2).is_none());
    }

    #[test]
    fn test_interp_weights_point_on_edge() {
        // Point lies exactly on the edge between v0 and v1 of a square.
        let verts = [
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(1.0, 1.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        ];
        let on_edge = Vec3::new(0.5, 0.0, 0.0);
        let w = interp_weights_poly_v3(on_edge, &verts);
        assert_eq!(w.len(), 4);
        let sum: f32 = w.iter().sum();
        assert!((sum - 1.0).abs() < 1e-4, "weights should sum to 1, got {sum}");
        // The two edge vertices should share the weight.
        assert!(w[0] > 0.1, "v0 should have significant weight");
        assert!(w[1] > 0.1, "v1 should have significant weight");
    }

    #[test]
    fn test_interp_weights_point_on_vertex() {
        let verts = [
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.5, 1.0, 0.0),
        ];
        let on_vert = Vec3::new(0.0, 0.0, 0.0);
        let w = interp_weights_poly_v3(on_vert, &verts);
        assert_eq!(w.len(), 3);
        assert!((w[0] - 1.0).abs() < 1e-6);
        assert!(w[1].abs() < 1e-6);
        assert!(w[2].abs() < 1e-6);
    }

    #[test]
    fn test_interp_weights_concave_quad() {
        // A concave (bowtie-like) polygon -- weights should still sum to 1
        // and not produce NaN.
        let verts = [
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(2.0, 1.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(2.0, -1.0, 0.0),
        ];
        let center = Vec3::new(1.0, 0.1, 0.0);
        let w = interp_weights_poly_v3(center, &verts);
        assert_eq!(w.len(), 4);
        let sum: f32 = w.iter().sum();
        assert!((sum - 1.0).abs() < 1e-4, "weights should sum to 1, got {sum}");
        assert!(w.iter().all(|x| !x.is_nan()), "no NaN weights");
    }

    #[test]
    fn test_closest_point_on_tri_all_voronoi_regions() {
        let a = Vec3::new(0.0, 0.0, 0.0);
        let b = Vec3::new(4.0, 0.0, 0.0);
        let c = Vec3::new(0.0, 4.0, 0.0);

        // Region: vertex A
        let (cp, u, _v, _w) = closest_point_on_tri(Vec3::new(-1.0, -1.0, 0.0), a, b, c);
        assert!(cp.approx_eq(&a, 1e-5));
        assert!((u - 1.0).abs() < 1e-5);

        // Region: vertex B
        let (cp, _, v, _) = closest_point_on_tri(Vec3::new(5.0, -1.0, 0.0), a, b, c);
        assert!(cp.approx_eq(&b, 1e-5));
        assert!((v - 1.0).abs() < 1e-5);

        // Region: vertex C
        let (cp, _, _, w) = closest_point_on_tri(Vec3::new(-1.0, 5.0, 0.0), a, b, c);
        assert!(cp.approx_eq(&c, 1e-5));
        assert!((w - 1.0).abs() < 1e-5);

        // Region: edge AB
        let (cp, _, _, _) = closest_point_on_tri(Vec3::new(2.0, -1.0, 0.0), a, b, c);
        assert!((cp.y() - 0.0).abs() < 1e-5);
        assert!(cp.x() > 0.0 && cp.x() < 4.0);

        // Region: edge AC
        let (cp, _, _, _) = closest_point_on_tri(Vec3::new(-1.0, 2.0, 0.0), a, b, c);
        assert!((cp.x() - 0.0).abs() < 1e-5);
        assert!(cp.y() > 0.0 && cp.y() < 4.0);

        // Region: edge BC
        let (cp, _, _, _) = closest_point_on_tri(Vec3::new(3.0, 3.0, 0.0), a, b, c);
        // Should be on the hypotenuse (x + y = 4)
        assert!(((cp.x() + cp.y()) - 4.0).abs() < 1e-4);

        // Region: interior (point above triangle)
        let (cp, u, v, w) = closest_point_on_tri(Vec3::new(1.0, 1.0, 5.0), a, b, c);
        assert!((cp.z() - 0.0).abs() < 1e-5);
        assert!(u > 0.0 && v > 0.0 && w > 0.0);
        assert!(((u + v + w) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_closest_to_line_v3_degenerate() {
        // Degenerate line (both endpoints the same).
        let p = Vec3::new(5.0, 5.0, 5.0);
        let l = Vec3::new(1.0, 1.0, 1.0);
        let (closest, lambda) = closest_to_line_v3(p, l, l);
        assert!(closest.approx_eq(&l, 1e-6));
        assert!((lambda - 0.0).abs() < 1e-6);
    }
}
