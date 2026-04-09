//! Console editor: interactive Python console for scripting.

/// A single line in the console history.
#[derive(Debug, Clone)]
pub struct ConsoleLine {
    /// The text content.
    pub text: String,
    /// Whether this is user input (`true`) or output/error (`false`).
    pub is_input: bool,
}

/// Console editor state.
pub struct ConsoleEditor {
    /// History of console lines.
    pub history: Vec<ConsoleLine>,
    /// Current input line being typed.
    pub input: String,
    /// Command history for up/down arrow recall.
    pub command_history: Vec<String>,
    /// Index into command_history for recall navigation.
    pub history_index: Option<usize>,
    /// Scroll position (lines from the bottom).
    pub scroll_offset: usize,
    /// Maximum number of history lines to retain.
    pub max_history: usize,
    /// Console prompt string.
    pub prompt: String,
}

impl ConsoleEditor {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            input: String::new(),
            command_history: Vec::new(),
            history_index: None,
            scroll_offset: 0,
            max_history: 10_000,
            prompt: ">>> ".to_string(),
        }
    }

    /// Submit the current input line.
    pub fn submit(&mut self) -> String {
        let line = std::mem::take(&mut self.input);
        if !line.is_empty() {
            self.command_history.push(line.clone());
        }
        self.history.push(ConsoleLine {
            text: line.clone(),
            is_input: true,
        });
        self.history_index = None;
        self.scroll_offset = 0;

        // Trim history.
        while self.history.len() > self.max_history {
            self.history.remove(0);
        }

        line
    }

    /// Append output text to the console.
    pub fn append_output(&mut self, text: impl Into<String>) {
        self.history.push(ConsoleLine {
            text: text.into(),
            is_input: false,
        });
    }

    /// Navigate command history upwards.
    pub fn history_up(&mut self) {
        if self.command_history.is_empty() {
            return;
        }
        let idx = match self.history_index {
            Some(i) if i > 0 => i - 1,
            Some(i) => i,
            None => self.command_history.len() - 1,
        };
        self.history_index = Some(idx);
        self.input = self.command_history[idx].clone();
    }

    /// Navigate command history downwards.
    pub fn history_down(&mut self) {
        if let Some(idx) = self.history_index {
            if idx + 1 < self.command_history.len() {
                self.history_index = Some(idx + 1);
                self.input = self.command_history[idx + 1].clone();
            } else {
                self.history_index = None;
                self.input.clear();
            }
        }
    }

    /// Clear the console output.
    pub fn clear(&mut self) {
        self.history.clear();
        self.scroll_offset = 0;
    }
}

impl Default for ConsoleEditor {
    fn default() -> Self {
        Self::new()
    }
}
