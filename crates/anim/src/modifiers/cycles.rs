//! Cycles modifier: repeats or mirrors the F-Curve outside its keyframe range.

use serde::{Deserialize, Serialize};

/// How the cycle repeats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CycleMode {
    /// No cycling.
    None,
    /// Repeat the curve as-is.
    Repeat,
    /// Repeat with the offset accumulated each cycle.
    RepeatOffset,
    /// Mirror (ping-pong) each cycle.
    Mirror,
}

impl Default for CycleMode {
    fn default() -> Self {
        Self::None
    }
}

/// Cycles modifier configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CyclesModifier {
    /// Cycle mode before the first keyframe.
    pub mode_before: CycleMode,
    /// Cycle mode after the last keyframe.
    pub mode_after: CycleMode,
    /// Number of cycles before (0 = infinite).
    pub cycles_before: u32,
    /// Number of cycles after (0 = infinite).
    pub cycles_after: u32,
}

impl Default for CyclesModifier {
    fn default() -> Self {
        Self {
            mode_before: CycleMode::None,
            mode_after: CycleMode::None,
            cycles_before: 0,
            cycles_after: 0,
        }
    }
}

impl CyclesModifier {
    /// Remap a time value that is outside the curve's `[start, end]` range
    /// back into that range according to the cycle mode.
    ///
    /// Returns `(remapped_time, value_offset)` where `value_offset` is
    /// accumulated for `RepeatOffset` mode.
    pub fn remap_time(
        &self,
        time: f32,
        range_start: f32,
        range_end: f32,
        start_value: f32,
        end_value: f32,
    ) -> (f32, f32) {
        let duration = range_end - range_start;
        if duration <= f32::EPSILON {
            return (range_start, 0.0);
        }

        let (mode, max_cycles) = if time < range_start {
            (self.mode_before, self.cycles_before)
        } else if time > range_end {
            (self.mode_after, self.cycles_after)
        } else {
            return (time, 0.0);
        };

        if mode == CycleMode::None {
            return (time, 0.0);
        }

        let offset_from_start = time - range_start;
        let cycle_count = (offset_from_start / duration).floor();
        let cycle_abs = cycle_count.abs() as u32;

        if max_cycles > 0 && cycle_abs >= max_cycles {
            // Exceeded max cycles, clamp.
            return if time < range_start {
                (range_start, 0.0)
            } else {
                (range_end, 0.0)
            };
        }

        let within = offset_from_start - cycle_count * duration;

        match mode {
            CycleMode::Repeat => (range_start + within, 0.0),
            CycleMode::RepeatOffset => {
                let value_delta = end_value - start_value;
                let accumulated = cycle_count * value_delta;
                (range_start + within, accumulated)
            }
            CycleMode::Mirror => {
                let cycle_int = cycle_count.abs() as i32;
                if cycle_int % 2 == 0 {
                    (range_start + within, 0.0)
                } else {
                    (range_end - within, 0.0)
                }
            }
            CycleMode::None => (time, 0.0),
        }
    }
}
