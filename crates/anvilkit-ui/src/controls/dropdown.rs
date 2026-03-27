//! Dropdown control — select from a list of options.

use bevy_ecs::prelude::*;
use crate::style::*;

/// Dropdown component.
#[derive(Debug, Clone, Component)]
pub struct Dropdown {
    pub options: Vec<String>,
    pub selected_index: Option<usize>,
    pub open: bool,
    pub placeholder: String,
}

impl Dropdown {
    pub fn new(options: Vec<String>) -> Self {
        Self {
            options,
            selected_index: None,
            open: false,
            placeholder: "Select...".to_string(),
        }
    }

    pub fn with_selected(mut self, index: usize) -> Self {
        if index < self.options.len() {
            self.selected_index = Some(index);
        }
        self
    }

    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Get the currently selected option text.
    pub fn selected_text(&self) -> &str {
        self.selected_index
            .and_then(|i| self.options.get(i))
            .map(|s| s.as_str())
            .unwrap_or(&self.placeholder)
    }

    /// Toggle the dropdown open/closed.
    pub fn toggle(&mut self) {
        self.open = !self.open;
    }

    /// Select an option by index and close.
    pub fn select(&mut self, index: usize) {
        if index < self.options.len() {
            self.selected_index = Some(index);
        }
        self.open = false;
    }

    /// Create the header UiNode (shows selected value).
    pub fn header_node(&self) -> UiNode {
        UiNode {
            style: UiStyle {
                padding: [6.0, 10.0, 6.0, 10.0],
                min_width: Val::Px(120.0),
                height: Val::Px(32.0),
                ..Default::default()
            },
            text: Some(UiText::new(format!("{} ▼", self.selected_text()))),
            background_color: [0.25, 0.25, 0.28, 1.0],
            border_color: [0.4, 0.4, 0.4, 1.0],
            border_width: 1.0,
            corner_radius: 3.0,
            ..Default::default()
        }
    }
}

impl Default for Dropdown {
    fn default() -> Self {
        Self::new(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dropdown_select() {
        let mut dd = Dropdown::new(vec!["A".into(), "B".into(), "C".into()]);
        assert_eq!(dd.selected_text(), "Select...");

        dd.select(1);
        assert_eq!(dd.selected_text(), "B");
        assert!(!dd.open);
    }

    #[test]
    fn test_dropdown_toggle() {
        let mut dd = Dropdown::new(vec!["X".into()]);
        assert!(!dd.open);
        dd.toggle();
        assert!(dd.open);
        dd.toggle();
        assert!(!dd.open);
    }

    #[test]
    fn test_dropdown_with_selected() {
        let dd = Dropdown::new(vec!["One".into(), "Two".into()])
            .with_selected(0);
        assert_eq!(dd.selected_text(), "One");
    }
}
