//! F-Curve evaluation: compute value at a given time.

use crate::fcurve::FCurve;
use crate::interpolation::{self, InterpolationMode};
use crate::keyframe::EasingMode;
use crate::modifiers::apply_modifiers;
use crate::{AnimResult, Extrapolation, FrameTime};

/// Evaluate an F-Curve at the given frame time.
///
/// 1. Finds the surrounding keyframe pair.
/// 2. Interpolates according to the keyframe's interpolation mode.
/// 3. Applies extrapolation if outside keyframe range.
/// 4. Applies modifier stack on top.
pub fn evaluate_fcurve(curve: &FCurve, time: FrameTime) -> AnimResult<f32> {
    if curve.muted {
        return Ok(0.0);
    }

    let base_value = evaluate_base(curve, time);
    let modified = apply_modifiers(&curve.modifiers, time, base_value);
    Ok(modified)
}

/// Evaluate the base curve value (keyframes + extrapolation) without modifiers.
///
/// In Blender, the cycles modifier remaps time *before* curve evaluation
/// (it's a "time modifier"). We apply cycles time remapping here, before
/// looking up keyframes, matching Blender's two-pass modifier approach:
/// 1. Time modifiers adjust the evaluation time.
/// 2. The curve is evaluated at the remapped time.
/// 3. Value modifiers adjust the result.
fn evaluate_base(curve: &FCurve, time: FrameTime) -> f32 {
    let kfs = &curve.keyframes;

    if kfs.is_empty() {
        return 0.0;
    }

    if kfs.len() == 1 {
        return kfs[0].value;
    }

    let first = &kfs[0];
    let last = &kfs[kfs.len() - 1];

    // Apply cycles modifier time remapping BEFORE evaluation (Blender's time modifier pass).
    let (eval_time, value_offset) = apply_cycles_time_remap(curve, time);

    // Before first keyframe (after cycle remap).
    if eval_time <= first.time {
        return extrapolate_before(curve, eval_time) + value_offset;
    }

    // After last keyframe (after cycle remap).
    if eval_time >= last.time {
        return extrapolate_after(curve, eval_time) + value_offset;
    }

    // Binary search for the segment.
    let idx = match kfs.binary_search_by(|k| {
        k.time.partial_cmp(&eval_time).unwrap_or(std::cmp::Ordering::Equal)
    }) {
        Ok(i) => return kfs[i].value + value_offset,
        Err(i) => i - 1,
    };

    let k1 = &kfs[idx];
    let k2 = &kfs[idx + 1];

    interpolate_segment(k1, k2, eval_time) + value_offset
}

/// Apply cycles modifier time remapping if present.
/// Returns `(remapped_time, value_offset)`.
fn apply_cycles_time_remap(curve: &FCurve, time: FrameTime) -> (FrameTime, f32) {
    use crate::modifiers::FcurveModifier;

    for modifier in &curve.modifiers {
        if let FcurveModifier::Cycles(cycles) = modifier {
            if let (Some(first), Some(last)) = (curve.keyframes.first(), curve.keyframes.last()) {
                return cycles.remap_time(
                    time,
                    first.time,
                    last.time,
                    first.value,
                    last.value,
                );
            }
        }
    }
    (time, 0.0)
}

/// Interpolate between two keyframes.
fn interpolate_segment(
    k1: &crate::keyframe::Keyframe,
    k2: &crate::keyframe::Keyframe,
    time: FrameTime,
) -> f32 {
    match k1.interpolation {
        InterpolationMode::Constant => k1.value,
        InterpolationMode::Linear => {
            let t = interpolation::time_factor(time, k1.time, k2.time);
            interpolation::lerp(k1.value, k2.value, t)
        }
        InterpolationMode::Bezier => interpolation::evaluate_bezier(k1, k2, time),
        // All easing modes.
        mode => {
            let t = interpolation::time_factor(time, k1.time, k2.time);
            let eased = apply_easing(mode, k1.easing, t);
            interpolation::lerp(k1.value, k2.value, eased)
        }
    }
}

fn apply_easing(mode: InterpolationMode, easing: EasingMode, t: f32) -> f32 {
    match easing {
        EasingMode::EaseIn => interpolation::ease_in(mode, t),
        EasingMode::EaseOut => interpolation::ease_out(mode, t),
        EasingMode::EaseInOut => interpolation::ease_in_out(mode, t),
        EasingMode::Auto => interpolation::ease_in_out(mode, t),
    }
}

/// Extrapolate before the first keyframe.
///
/// Matches Blender's `fcurve_eval_keyframes_extrapolate`:
/// - Constant: returns the first keyframe's value.
/// - Linear: uses the right handle gradient of the first keyframe (for Bezier),
///   or the slope to the second keyframe (for Linear interpolation mode).
fn extrapolate_before(curve: &FCurve, time: FrameTime) -> f32 {
    let first = &curve.keyframes[0];
    match curve.extrapolation_before {
        Extrapolation::Constant => first.value,
        Extrapolation::Linear => {
            // Blender uses the handle gradient when the first keyframe uses Bezier
            // interpolation, otherwise uses the slope to the next keyframe.
            let slope = if first.interpolation == InterpolationMode::Bezier {
                // Use the right handle of the first keyframe.
                let dx = first.handle_right.x - first.time;
                let dy = first.handle_right.y - first.value;
                if dx.abs() > f32::EPSILON {
                    dy / dx
                } else {
                    0.0
                }
            } else if curve.keyframes.len() >= 2 {
                let second = &curve.keyframes[1];
                if (second.time - first.time).abs() > f32::EPSILON {
                    (second.value - first.value) / (second.time - first.time)
                } else {
                    0.0
                }
            } else {
                0.0
            };
            first.value + slope * (time - first.time)
        }
        Extrapolation::MakesCyclic => first.value,
    }
}

/// Extrapolate after the last keyframe.
///
/// Matches Blender's `fcurve_eval_keyframes_extrapolate`:
/// - Constant: returns the last keyframe's value.
/// - Linear: uses the left handle gradient of the last keyframe (for Bezier),
///   or the slope from the second-to-last keyframe (for Linear interpolation mode).
///
/// Blender checks `endpoint->ipo` (the last keyframe's own interpolation field).
/// Even though our interpolation field semantically means "to the next keyframe",
/// Blender stores and checks the endpoint's own ipo, so we do the same.
fn extrapolate_after(curve: &FCurve, time: FrameTime) -> f32 {
    let last = &curve.keyframes[curve.keyframes.len() - 1];
    match curve.extrapolation_after {
        Extrapolation::Constant => last.value,
        Extrapolation::Linear => {
            let n = curve.keyframes.len();
            // Blender uses the endpoint's own ipo field to decide slope method.
            let slope = if last.interpolation == InterpolationMode::Bezier {
                // Use the left handle of the last keyframe.
                let dx = last.time - last.handle_left.x;
                let dy = last.value - last.handle_left.y;
                if dx.abs() > f32::EPSILON {
                    dy / dx
                } else {
                    0.0
                }
            } else if n >= 2 {
                let prev = &curve.keyframes[n - 2];
                if (last.time - prev.time).abs() > f32::EPSILON {
                    (last.value - prev.value) / (last.time - prev.time)
                } else {
                    0.0
                }
            } else {
                0.0
            };
            last.value + slope * (time - last.time)
        }
        Extrapolation::MakesCyclic => last.value,
    }
}
