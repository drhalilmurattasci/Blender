//! NLA strips: time-mapped references to animation actions.

use crate::{NlaBlendMode, NlaExtrapolation};
use serde::{Deserialize, Serialize};

/// An NLA strip: a time-ranged reference to an animation action
/// with time mapping, blending, and influence controls.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NlaStrip {
    /// Display name of the strip.
    pub name: String,
    /// Name of the action this strip references.
    pub action_name: String,
    /// Start frame of the strip on the timeline.
    pub start: f32,
    /// End frame of the strip on the timeline.
    pub end: f32,
    /// Start frame within the action.
    pub action_start: f32,
    /// End frame within the action.
    pub action_end: f32,
    /// Blend mode.
    pub blend_mode: NlaBlendMode,
    /// Influence factor [0, 1].
    pub influence: f32,
    /// Whether the strip is muted.
    pub muted: bool,
    /// Whether the action is reversed.
    pub reversed: bool,
    /// Time scale factor (1.0 = normal speed).
    pub scale: f32,
    /// Number of times the action repeats within the strip.
    pub repeat: f32,
    /// Blend in duration (frames).
    pub blend_in: f32,
    /// Blend out duration (frames).
    pub blend_out: f32,
    /// Extrapolation mode.
    pub extrapolation: NlaExtrapolation,
    /// Whether to use auto-blending.
    pub use_auto_blend: bool,
    /// Strip type.
    pub strip_type: StripType,
}

/// Type of NLA strip.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StripType {
    /// Regular action clip.
    Clip,
    /// Transition between two strips.
    Transition,
    /// Meta strip (container for other strips).
    Meta,
    /// Sound strip.
    Sound,
}

impl Default for StripType {
    fn default() -> Self {
        Self::Clip
    }
}

impl NlaStrip {
    /// Create a new NLA strip referencing an action.
    pub fn new(name: impl Into<String>, action_name: impl Into<String>, start: f32, end: f32) -> Self {
        Self {
            name: name.into(),
            action_name: action_name.into(),
            start,
            end,
            action_start: start,
            action_end: end,
            blend_mode: NlaBlendMode::Replace,
            influence: 1.0,
            muted: false,
            reversed: false,
            scale: 1.0,
            repeat: 1.0,
            blend_in: 0.0,
            blend_out: 0.0,
            extrapolation: NlaExtrapolation::Hold,
            use_auto_blend: false,
            strip_type: StripType::Clip,
        }
    }

    /// Duration of the strip on the timeline.
    #[inline]
    pub fn duration(&self) -> f32 {
        self.end - self.start
    }

    /// Whether a given frame falls within the strip's range (not accounting for extrapolation).
    #[inline]
    pub fn contains_frame(&self, frame: f32) -> bool {
        frame >= self.start && frame <= self.end
    }

    /// Whether the strip contributes a value at the given frame,
    /// accounting for NLA extrapolation modes (Hold, HoldForward, Nothing).
    #[inline]
    pub fn is_active_at(&self, frame: f32) -> bool {
        if self.contains_frame(frame) {
            return true;
        }
        match self.extrapolation {
            NlaExtrapolation::Hold => true, // holds both before and after
            NlaExtrapolation::HoldForward => frame > self.end,
            NlaExtrapolation::Nothing => false,
        }
    }

    /// Map a timeline frame to the corresponding frame within the action.
    ///
    /// Matches Blender's `nlastrip_get_frame` for action clips.
    ///
    /// Blender's approach: the strip occupies `[start, end]` on the timeline.
    /// The action plays `repeat` times within that range, scaled by `scale`.
    /// One repeat in strip-time occupies `strip_duration / repeat` frames.
    /// One repeat in action-time spans `action_length` frames.
    /// So: `action_time = fmod(strip_time, strip_duration / repeat) * (action_length / (strip_duration / repeat))`
    ///
    /// Equivalently (Blender's actual formula):
    /// `action_start + fmod(strip_time, action_length * scale) / scale`
    /// where `repeat` is accounted for because the strip_duration = action_length * scale * repeat.
    #[inline]
    pub fn map_frame(&self, frame: f32) -> f32 {
        let strip_duration = self.duration();
        if strip_duration <= f32::EPSILON || self.scale.abs() < f32::EPSILON {
            return self.action_start;
        }

        let action_length = self.action_end - self.action_start;
        if action_length.abs() < f32::EPSILON {
            return self.action_start;
        }

        // For extrapolation (Hold): clamp to strip boundaries so the action
        // evaluates at its start or end frame.
        let clamped_frame = frame.clamp(self.start, self.end);

        // Position within the strip.
        let strip_time = clamped_frame - self.start;

        // In Blender, one cycle of the action in strip-time = action_length * scale.
        // The strip plays `repeat` cycles total, so strip_duration = action_length * scale * repeat.
        // We modulo by ONE cycle's strip-time length to handle repeats.
        let one_cycle_strip_time = action_length * self.scale;

        // Modulo to handle repeats, then divide by scale to get action-local time.
        let cycle_time = if one_cycle_strip_time.abs() > f32::EPSILON {
            let rem = strip_time % one_cycle_strip_time;
            // Ensure positive modulo for negative strip_time.
            if rem < 0.0 { rem + one_cycle_strip_time } else { rem }
        } else {
            0.0
        };

        let action_time = if self.scale.abs() > f32::EPSILON {
            cycle_time / self.scale
        } else {
            0.0
        };

        let action_time = if self.reversed {
            action_length - action_time
        } else {
            action_time
        };

        self.action_start + action_time
    }

    /// Compute the blend factor at a given frame, accounting for blend-in/out
    /// and extrapolation.
    ///
    /// When the frame is outside the strip range (extrapolation), the influence
    /// is returned without blend-in/out ramps, matching Blender's behavior
    /// where held values use full influence.
    #[inline]
    pub fn blend_factor(&self, frame: f32) -> f32 {
        // Outside strip range: for Hold/HoldForward extrapolation, use full influence.
        if frame < self.start || frame > self.end {
            return self.influence.clamp(0.0, 1.0);
        }

        let mut factor = self.influence;

        // Blend in.
        if self.blend_in > f32::EPSILON {
            let blend_end = self.start + self.blend_in;
            if frame < blend_end {
                let t = ((frame - self.start) / self.blend_in).clamp(0.0, 1.0);
                factor *= t;
            }
        }

        // Blend out.
        if self.blend_out > f32::EPSILON {
            let blend_start = self.end - self.blend_out;
            if frame > blend_start {
                let t = ((self.end - frame) / self.blend_out).clamp(0.0, 1.0);
                factor *= t;
            }
        }

        factor.clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn basic_strip() -> NlaStrip {
        let mut s = NlaStrip::new("test", "action", 10.0, 30.0);
        s.action_start = 0.0;
        s.action_end = 20.0;
        s
    }

    #[test]
    fn map_frame_at_start() {
        let s = basic_strip();
        let f = s.map_frame(10.0);
        assert!((f - 0.0).abs() < 1e-4, "expected action_start, got {f}");
    }

    #[test]
    fn map_frame_at_end() {
        let s = basic_strip();
        let f = s.map_frame(30.0);
        // At the exact end of a single cycle, modulo wraps to 0 (start of action).
        // This matches Blender's behavior where fmod(duration, duration) == 0.
        assert!((f - 0.0).abs() < 1e-4, "expected action_start at cycle boundary, got {f}");
    }

    #[test]
    fn map_frame_near_end() {
        let s = basic_strip();
        // Just before the end should map near the action end.
        let f = s.map_frame(29.99);
        assert!((f - 19.99).abs() < 0.1, "expected near action_end, got {f}");
    }

    #[test]
    fn map_frame_midpoint() {
        let s = basic_strip();
        let f = s.map_frame(20.0);
        assert!((f - 10.0).abs() < 1e-4, "expected 10.0, got {f}");
    }

    #[test]
    fn map_frame_reversed() {
        let mut s = basic_strip();
        s.reversed = true;
        let f = s.map_frame(10.0);
        assert!((f - 20.0).abs() < 1e-4, "reversed start should map to action_end, got {f}");
    }

    #[test]
    fn map_frame_zero_duration() {
        let s = NlaStrip::new("test", "action", 5.0, 5.0);
        let f = s.map_frame(5.0);
        // Should not panic.
        assert!(f.is_finite(), "zero duration map_frame produced {f}");
    }

    #[test]
    fn map_frame_zero_scale() {
        let mut s = basic_strip();
        s.scale = 0.0;
        let f = s.map_frame(20.0);
        assert!(f.is_finite(), "zero scale map_frame produced {f}");
    }

    #[test]
    fn blend_factor_within_range() {
        let s = basic_strip();
        let f = s.blend_factor(20.0);
        assert!((f - 1.0).abs() < 1e-6);
    }

    #[test]
    fn blend_factor_with_blend_in() {
        let mut s = basic_strip();
        s.blend_in = 4.0;
        // At start, factor should be 0.
        let f0 = s.blend_factor(10.0);
        assert!(f0 < 1e-6, "blend_in at start: {f0}");
        // At midpoint of blend-in, should be 0.5.
        let f_mid = s.blend_factor(12.0);
        assert!((f_mid - 0.5).abs() < 1e-4, "blend_in midpoint: {f_mid}");
    }

    #[test]
    fn blend_factor_with_blend_out() {
        let mut s = basic_strip();
        s.blend_out = 4.0;
        // At end, factor should be 0.
        let f0 = s.blend_factor(30.0);
        assert!(f0 < 1e-6, "blend_out at end: {f0}");
    }

    #[test]
    fn blend_factor_outside_range_extrapolation() {
        let s = basic_strip();
        // Before strip, Hold extrapolation returns full influence.
        let f = s.blend_factor(0.0);
        assert!((f - 1.0).abs() < 1e-6);
    }

    #[test]
    fn is_active_at_with_nothing_extrapolation() {
        let mut s = basic_strip();
        s.extrapolation = NlaExtrapolation::Nothing;
        assert!(!s.is_active_at(5.0));
        assert!(!s.is_active_at(35.0));
        assert!(s.is_active_at(20.0));
    }

    #[test]
    fn is_active_at_with_hold_forward() {
        let mut s = basic_strip();
        s.extrapolation = NlaExtrapolation::HoldForward;
        assert!(!s.is_active_at(5.0)); // Before: not active.
        assert!(s.is_active_at(35.0)); // After: active.
    }
}
