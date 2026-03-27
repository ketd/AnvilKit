//! UI data model types: style, text, nodes.

use bevy_ecs::prelude::*;

/// Flexbox layout direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexDirection {
    Row,
    RowReverse,
    Column,
    ColumnReverse,
}

/// Flexbox alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Align {
    Start,
    Center,
    End,
    Stretch,
    SpaceBetween,
    SpaceAround,
}

/// Size value — pixels, percentage, or auto.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Val {
    Auto,
    Px(f32),
    Percent(f32),
}

impl Default for Val {
    fn default() -> Self { Val::Auto }
}

/// UI layout style (Flexbox properties).
#[derive(Debug, Clone)]
pub struct UiStyle {
    pub flex_direction: FlexDirection,
    pub justify_content: Align,
    pub align_items: Align,
    pub width: Val,
    pub height: Val,
    pub min_width: Val,
    pub min_height: Val,
    pub max_width: Val,
    pub max_height: Val,
    pub padding: [f32; 4],
    pub margin: [f32; 4],
    pub gap: f32,
    pub flex_grow: f32,
}

impl Default for UiStyle {
    fn default() -> Self {
        Self {
            flex_direction: FlexDirection::Row,
            justify_content: Align::Start,
            align_items: Align::Stretch,
            width: Val::Auto,
            height: Val::Auto,
            min_width: Val::Auto,
            min_height: Val::Auto,
            max_width: Val::Auto,
            max_height: Val::Auto,
            padding: [0.0; 4],
            margin: [0.0; 4],
            gap: 0.0,
            flex_grow: 0.0,
        }
    }
}

/// Text content and font configuration.
#[derive(Debug, Clone)]
pub struct UiText {
    pub content: String,
    pub font_size: f32,
    pub color: [f32; 4],
    pub font_family: String,
}

impl UiText {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            font_size: 16.0,
            color: [1.0, 1.0, 1.0, 1.0],
            font_family: "default".to_string(),
        }
    }

    pub fn with_font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    pub fn with_color(mut self, color: [f32; 4]) -> Self {
        self.color = color;
        self
    }
}

impl Default for UiText {
    fn default() -> Self {
        Self::new("")
    }
}

/// UI interaction state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub enum UiInteraction {
    None,
    Hovered,
    Pressed,
    Focused,
}

impl Default for UiInteraction {
    fn default() -> Self { Self::None }
}

/// A UI element — the fundamental building block.
#[derive(Debug, Clone, Component)]
pub struct UiNode {
    pub style: UiStyle,
    pub text: Option<UiText>,
    pub background_color: [f32; 4],
    pub border_color: [f32; 4],
    pub border_width: f32,
    pub corner_radius: f32,
    pub visible: bool,
    pub computed_rect: [f32; 4],
}

impl Default for UiNode {
    fn default() -> Self {
        Self {
            style: UiStyle::default(),
            text: None,
            background_color: [0.0, 0.0, 0.0, 0.0],
            border_color: [1.0, 1.0, 1.0, 0.0],
            border_width: 0.0,
            corner_radius: 0.0,
            visible: true,
            computed_rect: [0.0, 0.0, 0.0, 0.0],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_val_default() {
        assert_eq!(Val::default(), Val::Auto);
    }

    #[test]
    fn test_ui_text_builder() {
        let text = UiText::new("Hello")
            .with_font_size(24.0)
            .with_color([1.0, 0.0, 0.0, 1.0]);
        assert_eq!(text.content, "Hello");
        assert_eq!(text.font_size, 24.0);
        assert_eq!(text.color, [1.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_ui_node_default() {
        let node = UiNode::default();
        assert!(node.visible);
        assert!(node.text.is_none());
        assert_eq!(node.border_width, 0.0);
    }

    #[test]
    fn test_ui_interaction_default() {
        assert_eq!(UiInteraction::default(), UiInteraction::None);
    }
}
