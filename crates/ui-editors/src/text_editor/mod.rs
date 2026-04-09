//! Text editor: built-in code / script editor.

/// Text editor state.
pub struct TextEditor {
    /// Name of the currently open text data-block.
    pub text_name: Option<String>,
    /// Cursor line (0-based).
    pub cursor_line: usize,
    /// Cursor column (0-based).
    pub cursor_column: usize,
    /// Selection anchor (line, column). `None` if no selection.
    pub selection_anchor: Option<(usize, usize)>,
    /// Scroll offset (line).
    pub scroll_line: usize,
    /// Whether to show line numbers.
    pub show_line_numbers: bool,
    /// Whether to show syntax highlighting.
    pub syntax_highlight: bool,
    /// Whether word-wrap is enabled.
    pub word_wrap: bool,
    /// Font size in pixels.
    pub font_size: f32,
    /// Tab width in spaces.
    pub tab_width: u8,
}

impl TextEditor {
    pub fn new() -> Self {
        Self {
            text_name: None,
            cursor_line: 0,
            cursor_column: 0,
            selection_anchor: None,
            scroll_line: 0,
            show_line_numbers: true,
            syntax_highlight: true,
            word_wrap: false,
            font_size: 13.0,
            tab_width: 4,
        }
    }

    /// Move the cursor to a specific position.
    pub fn set_cursor(&mut self, line: usize, column: usize) {
        self.cursor_line = line;
        self.cursor_column = column;
        self.selection_anchor = None;
    }

    /// Begin a selection from the current cursor position.
    pub fn begin_selection(&mut self) {
        self.selection_anchor = Some((self.cursor_line, self.cursor_column));
    }

    /// Whether there is an active selection.
    pub fn has_selection(&self) -> bool {
        self.selection_anchor.is_some()
    }
}

impl Default for TextEditor {
    fn default() -> Self {
        Self::new()
    }
}
