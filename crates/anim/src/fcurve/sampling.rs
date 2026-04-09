//! Bulk sampling utilities for F-Curves.

use crate::fcurve::evaluate::evaluate_fcurve;
use crate::fcurve::FCurve;
use crate::AnimResult;

/// Sample an F-Curve at regular intervals over a frame range.
///
/// Returns a `Vec` of `(time, value)` pairs sampled at every `step` frames
/// from `start` to `end` (inclusive of `start`, exclusive of `end` unless
/// it falls exactly on a step boundary).
pub fn sample_fcurve_range(
    curve: &FCurve,
    start: f32,
    end: f32,
    step: f32,
) -> AnimResult<Vec<(f32, f32)>> {
    if step <= 0.0 || start >= end {
        return Ok(Vec::new());
    }

    let capacity = ((end - start) / step).ceil() as usize + 1;
    let mut samples = Vec::with_capacity(capacity);
    let mut t = start;

    while t <= end + f32::EPSILON {
        let value = evaluate_fcurve(curve, t)?;
        samples.push((t, value));
        t += step;
    }

    Ok(samples)
}
