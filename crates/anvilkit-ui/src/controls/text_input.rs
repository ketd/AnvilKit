//! TextInput control — single-line text editing with cursor.

use bevy_ecs::prelude::*;
use crate::style::*;

/// Text input component.
#[derive(Debug, Clone, Component)]
pub struct TextInput {
    pub text: String,
    pub placeholder: String,
    pub cursor_pos: usize,
    pub selection_start: Option<usize>,
    pub max_length: usize,
    pub active: bool,
}

impl TextInput {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            placeholder: String::new(),
            cursor_pos: 0,
            selection_start: None,
            max_length: 256,
            active: false,
        }
    }

    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self.cursor_pos = self.text.len();
        self
    }

    pub fn with_max_length(mut self, max: usize) -> Self {
        self.max_length = max;
        self
    }

    /// Insert a character at cursor position.
    pub fn insert_char(&mut self, ch: char) {
        if self.text.len() >= self.max_length { return; }
        self.delete_selection();
        let byte_pos = self.cursor_byte_pos();
        self.text.insert(byte_pos, ch);
        self.cursor_pos += 1;
    }

    /// Delete the character before cursor (Backspace).
    pub fn backspace(&mut self) {
        if self.selection_start.is_some() {
            self.delete_selection();
            return;
        }
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
            let byte_pos = self.cursor_byte_pos();
            if byte_pos < self.text.len() {
                self.text.remove(byte_pos);
            }
        }
    }

    /// Delete the character at cursor (Delete).
    pub fn delete_forward(&mut self) {
        if self.selection_start.is_some() {
            self.delete_selection();
            return;
        }
        let byte_pos = self.cursor_byte_pos();
        if byte_pos < self.text.len() {
            self.text.remove(byte_pos);
        }
    }

    /// Move cursor left.
    pub fn move_left(&mut self) {
        self.selection_start = None;
        if self.cursor_pos > 0 { self.cursor_pos -= 1; }
    }

    /// Move cursor right.
    pub fn move_right(&mut self) {
        self.selection_start = None;
        if self.cursor_pos < self.text.chars().count() { self.cursor_pos += 1; }
    }

    /// Select all text.
    pub fn select_all(&mut self) {
        self.selection_start = Some(0);
        self.cursor_pos = self.text.chars().count();
    }

    /// Get the display text (shows placeholder if empty).
    pub fn display_text(&self) -> &str {
        if self.text.is_empty() { &self.placeholder } else { &self.text }
    }

    fn cursor_byte_pos(&self) -> usize {
        self.text.char_indices()
            .nth(self.cursor_pos)
            .map(|(i, _)| i)
            .unwrap_or(self.text.len())
    }

    fn delete_selection(&mut self) {
        let Some(sel_start) = self.selection_start.take() else { return };
        let (start, end) = if sel_start < self.cursor_pos {
            (sel_start, self.cursor_pos)
        } else {
            (self.cursor_pos, sel_start)
        };
        let byte_start = self.text.char_indices().nth(start).map(|(i, _)| i).unwrap_or(self.text.len());
        let byte_end = self.text.char_indices().nth(end).map(|(i, _)| i).unwrap_or(self.text.len());
        self.text.drain(byte_start..byte_end);
        self.cursor_pos = start;
    }

    /// Create the UiNode for this text input.
    pub fn node(&self) -> UiNode {
        let display = self.display_text().to_string();
        let color = if self.text.is_empty() {
            [0.5, 0.5, 0.5, 1.0]
        } else {
            [1.0, 1.0, 1.0, 1.0]
        };
        UiNode {
            style: UiStyle {
                padding: [6.0, 10.0, 6.0, 10.0],
                min_width: Val::Px(120.0),
                height: Val::Px(32.0),
                ..Default::default()
            },
            text: Some(UiText::new(display).with_color(color)),
            background_color: [0.1, 0.1, 0.1, 1.0],
            border_color: if self.active { [0.3, 0.6, 1.0, 1.0] } else { [0.4, 0.4, 0.4, 1.0] },
            border_width: 1.0,
            corner_radius: 3.0,
            ..Default::default()
        }
    }
}

impl Default for TextInput {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_backspace() {
        let mut ti = TextInput::new();
        ti.insert_char('H');
        ti.insert_char('i');
        assert_eq!(ti.text, "Hi");
        assert_eq!(ti.cursor_pos, 2);

        ti.backspace();
        assert_eq!(ti.text, "H");
        assert_eq!(ti.cursor_pos, 1);
    }

    #[test]
    fn test_move_cursor() {
        let mut ti = TextInput::new().with_text("abc");
        assert_eq!(ti.cursor_pos, 3);
        ti.move_left();
        assert_eq!(ti.cursor_pos, 2);
        ti.move_right();
        assert_eq!(ti.cursor_pos, 3);
    }

    #[test]
    fn test_select_all_delete() {
        let mut ti = TextInput::new().with_text("hello");
        ti.select_all();
        ti.backspace();
        assert_eq!(ti.text, "");
    }

    #[test]
    fn test_placeholder() {
        let ti = TextInput::new().with_placeholder("Type here...");
        assert_eq!(ti.display_text(), "Type here...");
    }

    #[test]
    fn test_max_length() {
        let mut ti = TextInput::new().with_max_length(3);
        ti.insert_char('a');
        ti.insert_char('b');
        ti.insert_char('c');
        ti.insert_char('d'); // should be rejected
        assert_eq!(ti.text, "abc");
    }
}
