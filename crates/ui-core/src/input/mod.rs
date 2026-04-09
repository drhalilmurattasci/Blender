//! Input state tracking: mouse position, button state, drag state.

/// Current mouse / pointer state.
///
/// Tracks both instantaneous (per-frame) presses and continuous hold state.
/// Widgets should generally use `left_pressed()` (single-fire on the frame
/// the button went down) rather than `left_held()` (true every frame while
/// held), to avoid retriggering on every frame.
#[derive(Debug, Clone)]
pub struct InputState {
    /// Current mouse X in window pixels.
    pub mouse_x: f64,
    /// Current mouse Y in window pixels.
    pub mouse_y: f64,
    /// Whether each mouse button is currently held.
    pub buttons: [bool; 5],
    /// Whether each mouse button was pressed this frame (went from up to down).
    pub buttons_pressed: [bool; 5],
    /// Whether each mouse button was released this frame (went from down to up).
    pub buttons_released: [bool; 5],
    /// Previous frame's button state (for edge detection).
    buttons_prev: [bool; 5],
    /// Active drag, if any.
    pub drag: Option<DragState>,
    /// Whether an event was consumed this frame.
    pub consumed: bool,
}

impl InputState {
    /// Create a new default input state.
    pub fn new() -> Self {
        Self {
            mouse_x: 0.0,
            mouse_y: 0.0,
            buttons: [false; 5],
            buttons_pressed: [false; 5],
            buttons_released: [false; 5],
            buttons_prev: [false; 5],
            drag: None,
            consumed: false,
        }
    }

    /// Call at the start of each frame to update edge-detection state.
    pub fn begin_frame(&mut self) {
        for i in 0..5 {
            self.buttons_pressed[i] = self.buttons[i] && !self.buttons_prev[i];
            self.buttons_released[i] = !self.buttons[i] && self.buttons_prev[i];
        }
        self.consumed = false;
    }

    /// Call at the end of each frame to snapshot current state for next frame.
    pub fn end_frame(&mut self) {
        self.buttons_prev = self.buttons;
    }

    /// Returns `true` if the left mouse button is held.
    pub fn left_held(&self) -> bool {
        self.buttons[0]
    }

    /// Returns `true` if the left mouse button was pressed this frame.
    pub fn left_pressed(&self) -> bool {
        self.buttons_pressed[0]
    }

    /// Returns `true` if the left mouse button was released this frame.
    pub fn left_released(&self) -> bool {
        self.buttons_released[0]
    }

    /// Returns `true` if the right mouse button is held.
    pub fn right_held(&self) -> bool {
        self.buttons[1]
    }

    /// Returns `true` if the right mouse button was pressed this frame.
    pub fn right_pressed(&self) -> bool {
        self.buttons_pressed[1]
    }

    /// Returns `true` if the middle mouse button is held.
    pub fn middle_held(&self) -> bool {
        self.buttons[2]
    }

    /// Returns `true` if the middle mouse button was pressed this frame.
    pub fn middle_pressed(&self) -> bool {
        self.buttons_pressed[2]
    }
}

impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}

/// State of an ongoing drag gesture.
#[derive(Debug, Clone)]
pub struct DragState {
    /// Starting position (x, y).
    pub start: [f64; 2],
    /// Current position (x, y).
    pub current: [f64; 2],
    /// Which mouse button initiated the drag.
    pub button_index: usize,
}

impl DragState {
    /// Displacement vector from start to current.
    pub fn delta(&self) -> [f64; 2] {
        [
            self.current[0] - self.start[0],
            self.current[1] - self.start[1],
        ]
    }

    /// Euclidean distance from start to current.
    pub fn distance(&self) -> f64 {
        let [dx, dy] = self.delta();
        (dx * dx + dy * dy).sqrt()
    }
}
