//! Timeline editor: frame-based playback control and keyframe display.

/// Playback state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
}

impl Default for PlaybackState {
    fn default() -> Self {
        Self::Stopped
    }
}

/// Timeline editor state.
pub struct TimelineEditor {
    /// Current frame (may be fractional during playback).
    pub frame_current: f32,
    /// Start of the visible range.
    pub frame_start: i32,
    /// End of the visible range.
    pub frame_end: i32,
    /// Playback start frame.
    pub playback_start: i32,
    /// Playback end frame.
    pub playback_end: i32,
    /// Playback state.
    pub playback_state: PlaybackState,
    /// Whether to loop playback.
    pub loop_playback: bool,
    /// Whether to sync playback to audio.
    pub sync_audio: bool,
    /// Zoom level (pixels per frame).
    pub pixels_per_frame: f32,
    /// Scroll offset in frames.
    pub scroll_offset: f32,
    /// Whether to show keyframes for the active object.
    pub show_keyframes: bool,
    /// Whether to show markers.
    pub show_markers: bool,
}

impl TimelineEditor {
    pub fn new() -> Self {
        Self {
            frame_current: 1.0,
            frame_start: 1,
            frame_end: 250,
            playback_start: 1,
            playback_end: 250,
            playback_state: PlaybackState::default(),
            loop_playback: true,
            sync_audio: false,
            pixels_per_frame: 8.0,
            scroll_offset: 0.0,
            show_keyframes: true,
            show_markers: true,
        }
    }

    /// Advance the frame by one step. Returns the new frame.
    pub fn step_forward(&mut self) -> f32 {
        self.frame_current += 1.0;
        if self.loop_playback && self.frame_current > self.playback_end as f32 {
            self.frame_current = self.playback_start as f32;
        }
        self.frame_current
    }

    /// Go back one frame.
    pub fn step_backward(&mut self) -> f32 {
        self.frame_current -= 1.0;
        if self.loop_playback && self.frame_current < self.playback_start as f32 {
            self.frame_current = self.playback_end as f32;
        }
        self.frame_current
    }

    /// Jump to a specific frame.
    pub fn set_frame(&mut self, frame: f32) {
        self.frame_current = frame;
    }

    /// Toggle playback.
    pub fn toggle_playback(&mut self) {
        self.playback_state = match self.playback_state {
            PlaybackState::Playing => PlaybackState::Stopped,
            _ => PlaybackState::Playing,
        };
    }
}

impl Default for TimelineEditor {
    fn default() -> Self {
        Self::new()
    }
}
