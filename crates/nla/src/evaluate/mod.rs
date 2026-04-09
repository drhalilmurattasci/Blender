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
