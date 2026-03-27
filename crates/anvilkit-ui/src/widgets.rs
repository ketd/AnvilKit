//! Widget factory methods for common UI elements.

use crate::style::*;

/// Widget factory.
pub struct Widget;

impl Widget {
    /// Create a button with text.
    pub fn button(label: &str) -> UiNode {
        UiNode {
            style: UiStyle {
                padding: [8.0, 16.0, 8.0, 16.0],
                ..Default::default()
            },
            text: Some(UiText::new(label).with_font_size(16.0)),
            background_color: [0.3, 0.3, 0.3, 1.0],
            border_color: [0.6, 0.6, 0.6, 1.0],
            border_width: 1.0,
            corner_radius: 4.0,
            ..Default::default()
        }
    }

    /// Create a text label.
    pub fn label(text: &str) -> UiNode {
        UiNode {
            text: Some(UiText::new(text)),
            ..Default::default()
        }
    }

    /// Create a panel (container with background).
    pub fn panel() -> UiNode {
        UiNode {
            style: UiStyle {
                flex_direction: FlexDirection::Column,
                padding: [8.0, 8.0, 8.0, 8.0],
                gap: 4.0,
                ..Default::default()
            },
            background_color: [0.15, 0.15, 0.15, 0.9],
            corner_radius: 6.0,
            ..Default::default()
        }
    }

    /// Create a horizontal row container.
    pub fn row() -> UiNode {
        UiNode {
            style: UiStyle {
                flex_direction: FlexDirection::Row,
                gap: 4.0,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// Create a vertical column container.
    pub fn column() -> UiNode {
        UiNode {
            style: UiStyle {
                flex_direction: FlexDirection::Column,
                gap: 4.0,
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_button() {
        let btn = Widget::button("Click me");
        assert!(btn.text.is_some());
        assert_eq!(btn.text.as_ref().unwrap().content, "Click me");
        assert!(btn.border_width > 0.0);
    }

    #[test]
    fn test_panel() {
        let p = Widget::panel();
        assert_eq!(p.style.flex_direction, FlexDirection::Column);
        assert!(p.background_color[3] > 0.0);
    }

    #[test]
    fn test_row_column() {
        let r = Widget::row();
        assert_eq!(r.style.flex_direction, FlexDirection::Row);
        let c = Widget::column();
        assert_eq!(c.style.flex_direction, FlexDirection::Column);
    }
}
