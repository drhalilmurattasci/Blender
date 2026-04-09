//! Bezier curve evaluation for F-Curve segments.
//!
//! Matches Blender's `fcurve.cc` implementation:
//! - `correct_bezpart` clamps handle X coordinates to prevent temporal looping.
//! - `findzero` + `solve_cubic` (Cardano's formula) finds the `t` parameter
//!   for a given evaluation time on the X bezier curve.
//! - `berekeny` evaluates the Y bezier curve at the found `t`.

use crate::keyframe::Keyframe;

/// Small negative threshold for root validity (matches Blender's `SMALL`).
const SMALL: f64 = -1.0e-10;
/// Upper bound for valid roots (slight overshoot for numerical stability).
const ROOT_UPPER: f64 = 1.000001;

/// Evaluate a cubic Bezier segment between two keyframes at the given time.
///
/// Uses Blender's analytical cubic solver (`findzero` / `solve_cubic` via
/// Cardano's formula) to find the parameter `t` such that `bezier_x(t) == time`,
/// then evaluates `bezier_y(t)` for the value.
#[inline]
pub fn evaluate_bezier(k1: &Keyframe, k2: &Keyframe, time: f32) -> f32 {
    let x0 = k1.time;
    let y0 = k1.value;
    let mut x1 = k1.handle_right.x;
    let mut y1 = k1.handle_right.y;
    let mut x2 = k2.handle_left.x;
    let mut y2 = k2.handle_left.y;
    let x3 = k2.time;
    let y3 = k2.value;

    // Correct bezier handles to prevent temporal looping (Blender's correct_bezpart).
    correct_bezpart(x0, &mut x1, &mut x2, x3, y0, &mut y1, &mut y2, y3);

    // Find parameter t for given time using Cardano's cubic solver.
    let roots = findzero(time as f64, x0 as f64, x1 as f64, x2 as f64, x3 as f64);

    // Pick the best valid root in [0, 1].
    let t = pick_root(&roots, time as f64, x0 as f64, x1 as f64, x2 as f64, x3 as f64);

    // Evaluate y at the found parameter t (Blender's berekeny).
    berekeny(y0 as f64, y1 as f64, y2 as f64, y3 as f64, t) as f32
}

/// Clamp bezier handle X coordinates to prevent the curve from looping back in time.
///
/// Matches Blender's `BKE_fcurve_correct_bezpart`. Ensures the right handle of
/// `k1` and the left handle of `k2` don't extend beyond the keyframe span.
#[allow(clippy::too_many_arguments)]
fn correct_bezpart(
    v1x: f32, v2x: &mut f32, v3x: &mut f32, v4x: f32,
    v1y: f32, v2y: &mut f32, v3y: &mut f32, v4y: f32,
) {
    // Matches Blender's BKE_fcurve_correct_bezpart exactly.
    let h1x = v1x - *v2x;
    let h1y = v1y - *v2y;
    let h2x = v4x - *v3x;
    let h2y = v4y - *v3y;

    let len = v4x - v1x;

    // If the keyframes are at the same time (degenerate segment), nothing to fix.
    if len <= f32::EPSILON {
        return;
    }

    let len1 = h1x.abs();
    let len2 = h2x.abs();

    // Blender: if both handles have zero X extent, nothing to fix.
    if (len1 + len2) == 0.0 {
        return;
    }

    // If handle 1 extends beyond the span, scale it down.
    if len1 > len {
        let fac = len / len1;
        *v2x = v1x - fac * h1x;
        *v2y = v1y - fac * h1y;
    }

    // If handle 2 extends beyond the span, scale it down.
    if len2 > len {
        let fac = len / len2;
        *v3x = v4x - fac * h2x;
        *v3y = v4y - fac * h2y;
    }
}

/// Find the roots of `bezier_x(t) - x == 0` using Cardano's cubic formula.
///
/// Matches Blender's `findzero` + `solve_cubic`.
/// Returns up to 3 roots.
fn findzero(x: f64, q0: f64, q1: f64, q2: f64, q3: f64) -> Vec<f64> {
    let c0 = q0 - x;
    let c1 = 3.0 * (q1 - q0);
    let c2 = 3.0 * (q0 - 2.0 * q1 + q2);
    let c3 = q3 - q0 + 3.0 * (q1 - q2);

    solve_cubic(c0, c1, c2, c3)
}

/// Solve `c3*t^3 + c2*t^2 + c1*t + c0 = 0` for t in the range [SMALL, ROOT_UPPER].
///
/// Matches Blender's `solve_cubic` using Cardano's formula with complex discriminant analysis.
fn solve_cubic(c0: f64, c1: f64, c2: f64, c3: f64) -> Vec<f64> {
    let mut roots = Vec::with_capacity(3);

    if c3.abs() < 1e-12 {
        // Degenerate: quadratic or lower.
        if c2.abs() < 1e-12 {
            // Linear.
            if c1.abs() > 1e-12 {
                let r = -c0 / c1;
                if r >= SMALL && r <= ROOT_UPPER {
                    roots.push(r);
                }
            }
            return roots;
        }
        // Quadratic: c2*t^2 + c1*t + c0 = 0
        let disc = c1 * c1 - 4.0 * c2 * c0;
        if disc < 0.0 {
            return roots;
        }
        let sq = disc.sqrt();
        let r1 = (-c1 + sq) / (2.0 * c2);
        let r2 = (-c1 - sq) / (2.0 * c2);
        if r1 >= SMALL && r1 <= ROOT_UPPER {
            roots.push(r1);
        }
        if r2 >= SMALL && r2 <= ROOT_UPPER {
            roots.push(r2);
        }
        return roots;
    }

    // Normalize: t^3 + a*t^2 + b*t + c = 0
    let a = c2 / c3;
    let b = c1 / c3;
    let c = c0 / c3;

    let a2 = a * a;
    let q = (3.0 * b - a2) / 9.0;
    let r = (2.0 * a * a2 - 9.0 * a * b + 27.0 * c) / 54.0;

    let q3 = q * q * q;
    let disc = q3 + r * r;

    let a_third = a / 3.0;

    if disc > 0.0 {
        // One real root.
        let s = disc.sqrt();
        let u = cbrt(-r + s);
        let v = cbrt(-r - s);
        let root = u + v - a_third;
        if root >= SMALL && root <= ROOT_UPPER {
            roots.push(root);
        }
    } else if disc.abs() < 1e-12 {
        // Three real roots, at least two equal.
        let u = cbrt(-r);
        let r1 = 2.0 * u - a_third;
        let r2 = -u - a_third;
        if r1 >= SMALL && r1 <= ROOT_UPPER {
            roots.push(r1);
        }
        if r2 >= SMALL && r2 <= ROOT_UPPER {
            roots.push(r2);
        }
    } else {
        // Three distinct real roots (disc < 0).
        let qq = (-q).sqrt();
        // Clamp argument to [-1, 1] to avoid NaN from floating-point overshoot.
        let cos_arg = (r / ((-q3).sqrt())).clamp(-1.0, 1.0);
        let theta = cos_arg.acos();
        let r1 = 2.0 * qq * (theta / 3.0).cos() - a_third;
        let r2 = 2.0 * qq * ((theta + 2.0 * std::f64::consts::PI) / 3.0).cos() - a_third;
        let r3 = 2.0 * qq * ((theta + 4.0 * std::f64::consts::PI) / 3.0).cos() - a_third;
        if r1 >= SMALL && r1 <= ROOT_UPPER {
            roots.push(r1);
        }
        if r2 >= SMALL && r2 <= ROOT_UPPER {
            roots.push(r2);
        }
        if r3 >= SMALL && r3 <= ROOT_UPPER {
            roots.push(r3);
        }
    }

    roots
}

/// Cube root that handles negative values.
fn cbrt(x: f64) -> f64 {
    if x >= 0.0 {
        x.cbrt()
    } else {
        -((-x).cbrt())
    }
}

/// Pick the best root from the set of valid roots.
/// If multiple valid roots exist, pick the one whose bezier_x(t) is closest to `time`.
fn pick_root(roots: &[f64], time: f64, x0: f64, x1: f64, x2: f64, x3: f64) -> f64 {
    if roots.is_empty() {
        // Fallback: linear interpolation parameter.
        if (x3 - x0).abs() < 1e-12 {
            return 0.5;
        }
        return ((time - x0) / (x3 - x0)).clamp(0.0, 1.0);
    }

    let mut best_t = roots[0].clamp(0.0, 1.0);
    let mut best_err = f64::MAX;

    for &root in roots {
        let t = root.clamp(0.0, 1.0);
        let x_at_t = berekeny(x0, x1, x2, x3, t);
        let err = (x_at_t - time).abs();
        if err < best_err {
            best_err = err;
            best_t = t;
        }
    }

    best_t
}

/// Evaluate a cubic bezier at parameter `t` using Blender's coefficient expansion.
///
/// Matches Blender's `berekeny`: `c0 + t*c1 + t^2*c2 + t^3*c3` where:
/// - `c0 = f1`
/// - `c1 = 3*(f2 - f1)`
/// - `c2 = 3*(f1 - 2*f2 + f3)`
/// - `c3 = f4 - f1 + 3*(f2 - f3)`
#[inline]
fn berekeny(f1: f64, f2: f64, f3: f64, f4: f64, t: f64) -> f64 {
    let c0 = f1;
    let c1 = 3.0 * (f2 - f1);
    let c2 = 3.0 * (f1 - 2.0 * f2 + f3);
    let c3 = f4 - f1 + 3.0 * (f2 - f3);
    c0 + t * c1 + t * t * c2 + t * t * t * c3
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keyframe::{Keyframe, KeyframeHandle};

    fn make_bezier_pair(t1: f32, v1: f32, rh: (f32, f32), t2: f32, v2: f32, lh: (f32, f32)) -> (Keyframe, Keyframe) {
        let mut k1 = Keyframe::new(t1, v1);
        k1.handle_right = KeyframeHandle::new(rh.0, rh.1);
        let mut k2 = Keyframe::new(t2, v2);
        k2.handle_left = KeyframeHandle::new(lh.0, lh.1);
        (k1, k2)
    }

    #[test]
    fn bezier_monotone_evaluation() {
        // Evaluate across the range and check all values are finite and within bounds.
        let (k1, k2) = make_bezier_pair(0.0, 0.0, (3.33, 1.67), 10.0, 5.0, (6.67, 3.33));
        for i in 1..10 {
            let t = i as f32;
            let v = evaluate_bezier(&k1, &k2, t);
            assert!(v.is_finite(), "non-finite at t={t}: {v}");
            assert!(v >= -0.5 && v <= 5.5, "value out of expected range at t={t}: {v}");
        }
    }

    #[test]
    fn bezier_midpoint_linear_handles() {
        // With handles forming a straight line, midpoint should be close to linear interp.
        let (k1, k2) = make_bezier_pair(0.0, 0.0, (3.33, 1.67), 10.0, 5.0, (6.67, 3.33));
        let v_mid = evaluate_bezier(&k1, &k2, 5.0);
        assert!((v_mid - 2.5).abs() < 0.2, "midpoint value: {v_mid}");
    }

    #[test]
    fn bezier_same_frame_keyframes() {
        // Degenerate case: keyframes at the same time.
        let (k1, k2) = make_bezier_pair(5.0, 1.0, (5.0, 1.0), 5.0, 3.0, (5.0, 3.0));
        let v = evaluate_bezier(&k1, &k2, 5.0);
        // Should not panic or produce NaN.
        assert!(v.is_finite(), "same-frame bezier produced non-finite: {v}");
    }

    #[test]
    fn bezier_no_nan_with_extreme_handles() {
        // Handles far outside the range.
        let (k1, k2) = make_bezier_pair(0.0, 0.0, (100.0, 50.0), 10.0, 5.0, (-90.0, -45.0));
        let v = evaluate_bezier(&k1, &k2, 5.0);
        assert!(v.is_finite(), "extreme handles produced non-finite: {v}");
    }

    #[test]
    fn cubic_solver_degenerate_linear() {
        // c3 = 0, c2 = 0 => linear root
        let roots = solve_cubic(2.0, -4.0, 0.0, 0.0);
        assert_eq!(roots.len(), 1);
        assert!((roots[0] - 0.5).abs() < 1e-9);
    }

    #[test]
    fn cubic_solver_degenerate_quadratic() {
        // c3 = 0 => quadratic
        let roots = solve_cubic(0.0, -3.0, 2.0, 0.0);
        assert!(!roots.is_empty());
    }

    #[test]
    fn cubic_solver_three_real_roots() {
        // t(t-0.5)(t-1) = t^3 - 1.5t^2 + 0.5t
        let roots = solve_cubic(0.0, 0.5, -1.5, 1.0);
        assert!(roots.len() >= 2, "expected multiple roots, got {}", roots.len());
    }

    #[test]
    fn berekeny_endpoints() {
        let v0 = berekeny(0.0, 1.0, 2.0, 3.0, 0.0);
        let v1 = berekeny(0.0, 1.0, 2.0, 3.0, 1.0);
        assert!((v0 - 0.0).abs() < 1e-12);
        assert!((v1 - 3.0).abs() < 1e-12);
    }
}
