//! NLA tracks: containers for NLA strips.

use crate::strip::NlaStrip;
use serde::{Deserialize, Serialize};

/// An NLA track: a horizontal lane containing non-overlapping NLA strips.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NlaTrack {
    /// Display name of the track.
    pub name: String,
    /// Strips in this track, sorted by start frame.
    pub strips: Vec<NlaStrip>,
    /// Whether the track is muted.
    pub muted: bool,
    /// Whether the track is locked (not editable).
    pub locked: bool,
    /// Whether the track contributes to the final animation.
    pub active: bool,
    /// Whether this track is solo (only solo tracks are evaluated).
    pub solo: bool,
}

impl NlaTrack {
    /// Create a new empty NLA track.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            strips: Vec::new(),
            muted: false,
            locked: false,
            active: true,
            solo: false,
        }
    }

    /// Add a strip, maintaining sort order by start frame.
    pub fn add_strip(&mut self, strip: NlaStrip) {
        let pos = self
            .strips
            .binary_search_by(|s| s.start.partial_cmp(&strip.start).unwrap_or(std::cmp::Ordering::Equal));
        let idx = match pos {
            Ok(i) | Err(i) => i,
        };
        self.strips.insert(idx, strip);
    }

    /// Find all strips that are active at the given frame, accounting for extrapolation.
    pub fn active_strips_at(&self, frame: f32) -> Vec<&NlaStrip> {
        self.strips
            .iter()
            .filter(|s| !s.muted && s.is_active_at(frame))
            .collect()
    }

    /// Whether this track should be evaluated.
    #[inline]
    pub fn is_evaluable(&self) -> bool {
        self.active && !self.muted
    }

    /// Compute the time range covered by all strips in this track.
    pub fn time_range(&self) -> Option<(f32, f32)> {
        if self.strips.is_empty() {
            return None;
        }
        let start = self.strips.first().unwrap().start;
        let end = self.strips.iter().map(|s| s.end).fold(f32::MIN, f32::max);
        Some((start, end))
    }
}

/// The NLA stack: all tracks for a single animated data block.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NlaStack {
    /// Tracks from bottom (lowest priority) to top (highest priority).
    pub tracks: Vec<NlaTrack>,
    /// The "tweak mode" active strip (being edited live).
    pub active_strip: Option<ActiveStripRef>,
}

/// Reference to the currently-edited strip in tweak mode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveStripRef {
    /// Track index.
    pub track_index: usize,
    /// Strip index within the track.
    pub strip_index: usize,
}

impl NlaStack {
    pub fn new() -> Self {
        Self {
            tracks: Vec::new(),
            active_strip: None,
        }
    }

    /// Whether any track has the solo flag set.
    pub fn has_solo(&self) -> bool {
        self.tracks.iter().any(|t| t.solo)
    }

    /// Add a new track to the top of the stack.
    pub fn push_track(&mut self, track: NlaTrack) {
        self.tracks.push(track);
    }
}

impl Default for NlaStack {
    fn default() -> Self {
        Self::new()
    }
}
