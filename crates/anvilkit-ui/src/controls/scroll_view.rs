//! ScrollView control — scrollable container.

use bevy_ecs::prelude::*;
use crate::style::*;

/// Scroll view component.
#[derive(Debug, Clone, Component)]
pub struct ScrollView {
    /// Current scroll offset in pixels (positive = scrolled down).
    pub scroll_y: f32,
    /// Total content height (set by layout).
    pub content_height: f32,
    /// Visible viewport height (set by layout).
    pub viewport_height: f32,
    /// Scroll speed multiplier.
    pub scroll_speed: f32,
}

impl ScrollView {
    pub fn new() -> Self {
        Self {
            scroll_y: 0.0,
            content_height: 0.0,
            viewport_height: 0.0,
            scroll_speed: 40.0,
        }
    }

    /// Maximum scroll offset.
    pub fn max_scroll(&self) -> f32 {
        (self.content_height - self.viewport_height).max(0.0)
    }

    /// Scroll by a delta (positive = scroll down).
    pub fn scroll_by(&mut self, delta: f32) {
        self.scroll_y = (self.scroll_y + delta).clamp(0.0, self.max_scroll());
    }

    /// Scroll to top.
    pub fn scroll_to_top(&mut self) {
        self.scroll_y = 0.0;
    }

    /// Scroll to bottom.
    pub fn scroll_to_bottom(&mut self) {
        self.scroll_y = self.max_scroll();
    }

    /// Whether the view can scroll further down.
    pub fn can_scroll_down(&self) -> bool {
        self.scroll_y < self.max_scroll()
    }

    /// Whether the view can scroll further up.
    pub fn can_scroll_up(&self) -> bool {
        self.scroll_y > 0.0
    }

    /// Create the container UiNode.
    pub fn node(&self, width: Val, height: Val) -> UiNode {
        UiNode {
            style: UiStyle {
                flex_direction: FlexDirection::Column,
                width,
                height,
                ..Default::default()
            },
            background_color: [0.1, 0.1, 0.1, 0.5],
            ..Default::default()
        }
    }
}

impl Default for ScrollView {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scroll_clamping() {
        let mut sv = ScrollView {
            content_height: 500.0,
            viewport_height: 200.0,
            ..ScrollView::new()
        };
        assert_eq!(sv.max_scroll(), 300.0);

        sv.scroll_by(100.0);
        assert_eq!(sv.scroll_y, 100.0);

        sv.scroll_by(500.0); // clamped
        assert_eq!(sv.scroll_y, 300.0);

        sv.scroll_by(-400.0); // clamped at 0
        assert_eq!(sv.scroll_y, 0.0);
    }

    #[test]
    fn test_scroll_to_extremes() {
        let mut sv = ScrollView {
            content_height: 1000.0,
            viewport_height: 200.0,
            ..ScrollView::new()
        };
        sv.scroll_to_bottom();
        assert_eq!(sv.scroll_y, 800.0);
        sv.scroll_to_top();
        assert_eq!(sv.scroll_y, 0.0);
    }
}
