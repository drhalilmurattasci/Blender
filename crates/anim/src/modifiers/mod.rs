//! F-Curve modifiers: post-processing applied to evaluated curve values.

mod cycles;
mod envelope;
mod noise;

pub use cycles::{CycleMode, CyclesModifier};
pub use envelope::{EnvelopeControlPoint, EnvelopeModifier};
pub use noise::NoiseModifier;

use crate::FrameTime;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

/// An F-Curve modifier that transforms the evaluated value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FcurveModifier {
    /// Cycles (repeat / mirror) the curve outside its range.
    Cycles(CyclesModifier),
    /// Adds procedural noise to the value.
    Noise(NoiseModifier),
    /// Reshapes the curve with an envelope.
    Envelope(EnvelopeModifier),
    /// Limits the output value to a range.
    Limits {
        min_x: f32,
        max_x: f32,
        min_y: f32,
        max_y: f32,
        use_min_x: bool,
        use_max_x: bool,
        use_min_y: bool,
        use_max_y: bool,
    },
    /// Steps the output to discrete values.
    Stepped {
        step_size: f32,
        offset: f32,
        use_start: bool,
        use_end: bool,
        start_frame: f32,
        end_frame: f32,
    },
}

/// Apply a stack of modifiers to a base value at the given time.
pub fn apply_modifiers(modifiers: &SmallVec<[FcurveModifier; 2]>, time: FrameTime, mut value: f32) -> f32 {
    for modifier in modifiers {
        value = apply_single(modifier, time, value);
    }
    value
}

fn apply_single(modifier: &FcurveModifier, time: FrameTime, value: f32) -> f32 {
    match modifier {
        FcurveModifier::Noise(noise) => noise.apply(time, value),
        FcurveModifier::Envelope(env) => env.apply(time, value),
        FcurveModifier::Limits {
            min_y,
            max_y,
            use_min_y,
            use_max_y,
            ..
        } => {
            let mut v = value;
            if *use_min_y {
                v = v.max(*min_y);
            }
            if *use_max_y {
                v = v.min(*max_y);
            }
            v
        }
        FcurveModifier::Stepped {
            step_size,
            offset,
            use_start,
            use_end,
            start_frame,
            end_frame,
        } => {
            if *use_start && time < *start_frame {
                return value;
            }
            if *use_end && time > *end_frame {
                return value;
            }
            if *step_size > f32::EPSILON {
                ((value - offset) / step_size).floor() * step_size + offset
            } else {
                value
            }
        }
        FcurveModifier::Cycles(_cycles) => {
            // Cycles modifier affects time remapping, handled at a higher level.
            // Here we pass through; actual cycle evaluation is done in fcurve/evaluate.
            value
        }
    }
}
