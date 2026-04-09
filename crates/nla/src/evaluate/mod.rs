//! NLA evaluation: traverse the NLA stack and produce final animation values.

use crate::blend;
use crate::track::NlaStack;
use crate::NlaBlendMode;

/// Result of evaluating the NLA stack for a single channel.
#[derive(Debug, Clone)]
pub struct NlaEvalResult {
    /// Channel path.
    pub path: String,
    /// Channel array index.
    pub index: u32,
    /// The final blended value.
    pub value: f32,
}

/// Evaluate the NLA stack at a given frame for scalar channels.
///
/// `stack`: the NLA stack to evaluate.
/// `frame`: the current scene time in frames.
/// `action_lookup`: a callback that evaluates an action at a given frame
///   and returns `(path, index, value)` tuples for each channel.
/// `rest_values`: a callback that returns the rest value for a given (path, index).
pub fn evaluate_nla_stack(
    stack: &NlaStack,
    frame: f32,
    action_lookup: &dyn Fn(&str, f32) -> Vec<(String, u32, f32)>,
    rest_values: &dyn Fn(&str, u32) -> f32,
) -> Vec<NlaEvalResult> {
    let has_solo = stack.has_solo();
    let mut channel_values: Vec<ChannelAccum> = Vec::new();

    // Evaluate tracks from bottom to top.
    for track in &stack.tracks {
        if !track.is_evaluable() {
            continue;
        }
        if has_solo && !track.solo {
            continue;
        }

        // Find active strips at this frame.
        let active_strips = track.active_strips_at(frame);

        for strip in &active_strips {
            let action_frame = strip.map_frame(frame);
            let blend_factor = strip.blend_factor(frame);

            if blend_factor <= f32::EPSILON {
                continue;
            }

            // Evaluate the action at the mapped frame.
            let action_values = action_lookup(&strip.action_name, action_frame);

            for (path, index, value) in action_values {
                let rest = rest_values(&path, index);
                accumulate_channel(
                    &mut channel_values,
                    &path,
                    index,
                    value,
                    blend_factor,
                    strip.blend_mode,
                    rest,
                );
            }
        }
    }

    channel_values
        .into_iter()
        .map(|ch| NlaEvalResult {
            path: ch.path,
            index: ch.index,
            value: ch.value,
        })
        .collect()
}

struct ChannelAccum {
    path: String,
    index: u32,
    value: f32,
}

fn accumulate_channel(
    channels: &mut Vec<ChannelAccum>,
    path: &str,
    index: u32,
    strip_value: f32,
    influence: f32,
    mode: NlaBlendMode,
    rest_value: f32,
) {
    if let Some(existing) = channels.iter_mut().find(|c| c.path == path && c.index == index) {
        existing.value = blend::blend_value(existing.value, strip_value, influence, mode, rest_value);
    } else {
        // First contribution to this channel.
        let value = blend::blend_value(rest_value, strip_value, influence, mode, rest_value);
        channels.push(ChannelAccum {
            path: path.to_string(),
            index,
            value,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strip::NlaStrip;
    use crate::track::{NlaStack, NlaTrack};
    use crate::NlaExtrapolation;

    fn simple_action_lookup(action_name: &str, frame: f32) -> Vec<(String, u32, f32)> {
        match action_name {
            "walk" => vec![("location".to_string(), 0, frame * 0.1)],
            "run" => vec![("location".to_string(), 0, frame * 0.3)],
            _ => vec![],
        }
    }

    fn rest_value(_path: &str, _index: u32) -> f32 {
        0.0
    }

    #[test]
    fn empty_stack_returns_empty() {
        let stack = NlaStack::new();
        let results = evaluate_nla_stack(&stack, 5.0, &simple_action_lookup, &rest_value);
        assert!(results.is_empty());
    }

    #[test]
    fn single_track_single_strip() {
        let mut stack = NlaStack::new();
        let mut track = NlaTrack::new("Track 1");
        let mut strip = NlaStrip::new("Walk", "walk", 0.0, 20.0);
        strip.action_start = 0.0;
        strip.action_end = 20.0;
        strip.extrapolation = NlaExtrapolation::Nothing;
        track.add_strip(strip);
        stack.push_track(track);

        let results = evaluate_nla_stack(&stack, 10.0, &simple_action_lookup, &rest_value);
        assert_eq!(results.len(), 1);
        assert!((results[0].value - 1.0).abs() < 1e-4, "10.0 * 0.1 = 1.0, got {}", results[0].value);
    }

    #[test]
    fn muted_track_skipped() {
        let mut stack = NlaStack::new();
        let mut track = NlaTrack::new("Muted");
        track.muted = true;
        let mut strip = NlaStrip::new("Walk", "walk", 0.0, 20.0);
        strip.action_start = 0.0;
        strip.action_end = 20.0;
        strip.extrapolation = NlaExtrapolation::Nothing;
        track.add_strip(strip);
        stack.push_track(track);

        let results = evaluate_nla_stack(&stack, 10.0, &simple_action_lookup, &rest_value);
        assert!(results.is_empty());
    }

    #[test]
    fn solo_track_only() {
        let mut stack = NlaStack::new();

        // Track 1: not solo.
        let mut track1 = NlaTrack::new("Track 1");
        let mut strip1 = NlaStrip::new("Walk", "walk", 0.0, 20.0);
        strip1.action_start = 0.0;
        strip1.action_end = 20.0;
        strip1.extrapolation = NlaExtrapolation::Nothing;
        track1.add_strip(strip1);
        stack.push_track(track1);

        // Track 2: solo.
        let mut track2 = NlaTrack::new("Track 2");
        track2.solo = true;
        let mut strip2 = NlaStrip::new("Run", "run", 0.0, 20.0);
        strip2.action_start = 0.0;
        strip2.action_end = 20.0;
        strip2.extrapolation = NlaExtrapolation::Nothing;
        track2.add_strip(strip2);
        stack.push_track(track2);

        let results = evaluate_nla_stack(&stack, 10.0, &simple_action_lookup, &rest_value);
        assert_eq!(results.len(), 1);
        // Should be run (0.3 * 10 = 3.0), not walk.
        assert!((results[0].value - 3.0).abs() < 1e-4, "solo should pick run, got {}", results[0].value);
    }

    #[test]
    fn strip_influence_less_than_one() {
        let mut stack = NlaStack::new();
        let mut track = NlaTrack::new("Track 1");
        let mut strip = NlaStrip::new("Walk", "walk", 0.0, 20.0);
        strip.action_start = 0.0;
        strip.action_end = 20.0;
        strip.influence = 0.5;
        strip.extrapolation = NlaExtrapolation::Nothing;
        track.add_strip(strip);
        stack.push_track(track);

        let results = evaluate_nla_stack(&stack, 10.0, &simple_action_lookup, &rest_value);
        assert_eq!(results.len(), 1);
        // Replace blend: rest * (1 - 0.5) + 1.0 * 0.5 = 0.5
        assert!((results[0].value - 0.5).abs() < 1e-4, "expected 0.5, got {}", results[0].value);
    }
}
