//! Envelope modifier: reshapes F-Curve values through control-point envelopes.

use serde::{Deserialize, Serialize};

/// A control point in the envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvelopeControlPoint {
    /// Frame time of this control point.
    pub time: f32,
    /// Minimum envelope value at this point.
    pub min: f32,
    /// Maximum envelope value at this point.
    pub max: f32,
}

/// Envelope modifier configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvelopeModifier {
    /// Sorted list of envelope control points.
    pub control_points: Vec<EnvelopeControlPoint>,
    /// Reference value (center line of the envelope).
    pub reference_value: f32,
    /// Default minimum distance from reference when no control points apply.
    pub default_min: f32,
    /// Default maximum distance from reference when no control points apply.
    pub default_max: f32,
}

impl Default for EnvelopeModifier {
    fn default() -> Self {
        Self {
            control_points: Vec::new(),
            reference_value: 0.0,
            default_min: -1.0,
            default_max: 1.0,
        }
    }
}

impl EnvelopeModifier {
    /// Apply the envelope to a base value at the given time.
    ///
    /// The envelope scales the value relative to `reference_value` so that
    /// it fits between the interpolated min/max at this time.
    pub fn apply(&self, time: f32, value: f32) -> f32 {
        let (env_min, env_max) = self.evaluate_bounds(time);
        let reference = self.reference_value;
        let diff = value - reference;

        if diff >= 0.0 {
            reference + diff * (env_max - reference).max(0.0)
        } else {
            reference + diff * (reference - env_min).max(0.0)
        }
    }

    /// Evaluate the min/max bounds at the given time by interpolating control points.
    fn evaluate_bounds(&self, time: f32) -> (f32, f32) {
        let cps = &self.control_points;

        if cps.is_empty() {
            return (self.default_min, self.default_max);
        }

        if time <= cps[0].time {
            return (cps[0].min, cps[0].max);
        }

        if time >= cps[cps.len() - 1].time {
            let last = &cps[cps.len() - 1];
            return (last.min, last.max);
        }

        // Find surrounding control points.
        for i in 0..cps.len() - 1 {
            if time >= cps[i].time && time <= cps[i + 1].time {
                let t = if (cps[i + 1].time - cps[i].time).abs() > f32::EPSILON {
                    (time - cps[i].time) / (cps[i + 1].time - cps[i].time)
                } else {
                    0.0
                };
                let min_val = cps[i].min + (cps[i + 1].min - cps[i].min) * t;
                let max_val = cps[i].max + (cps[i + 1].max - cps[i].max) * t;
                return (min_val, max_val);
            }
        }

        (self.default_min, self.default_max)
    }
}
