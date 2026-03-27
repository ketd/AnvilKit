//! UI theme — default colors, fonts, spacing.

use bevy_ecs::prelude::*;

/// Global UI theme resource.
#[derive(Debug, Clone, Resource)]
pub struct UiTheme {
    /// Default background color for panels.
    pub panel_bg: [f32; 4],
    /// Default button background color.
    pub button_bg: [f32; 4],
    /// Button background when hovered.
    pub button_hover_bg: [f32; 4],
    /// Button background when pressed.
    pub button_press_bg: [f32; 4],
    /// Default text color.
    pub text_color: [f32; 4],
    /// Muted/secondary text color.
    pub text_muted: [f32; 4],
    /// Default border color.
    pub border_color: [f32; 4],
    /// Accent color for focused/active elements.
    pub accent_color: [f32; 4],
    /// Default font size in pixels.
    pub font_size: f32,
    /// Default padding in pixels [top, right, bottom, left].
    pub padding: [f32; 4],
    /// Default gap between children.
    pub gap: f32,
    /// Default corner radius.
    pub corner_radius: f32,
}

impl Default for UiTheme {
    fn default() -> Self {
        Self {
            panel_bg: [0.12, 0.12, 0.14, 0.95],
            button_bg: [0.25, 0.25, 0.28, 1.0],
            button_hover_bg: [0.35, 0.35, 0.38, 1.0],
            button_press_bg: [0.18, 0.18, 0.20, 1.0],
            text_color: [0.92, 0.92, 0.92, 1.0],
            text_muted: [0.6, 0.6, 0.6, 1.0],
            border_color: [0.4, 0.4, 0.42, 1.0],
            accent_color: [0.3, 0.6, 1.0, 1.0],
            font_size: 16.0,
            padding: [6.0, 12.0, 6.0, 12.0],
            gap: 4.0,
            corner_radius: 4.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_default() {
        let theme = UiTheme::default();
        assert!(theme.font_size > 0.0);
        assert!(theme.panel_bg[3] > 0.0);
    }
}
