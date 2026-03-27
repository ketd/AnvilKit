//! Window size ECS resource.

use bevy_ecs::prelude::*;

/// Window dimensions in physical pixels, automatically updated on resize.
#[derive(Debug, Clone, Resource)]
pub struct WindowSize {
    /// Window width in physical pixels.
    pub width: f32,
    /// Window height in physical pixels.
    pub height: f32,
}

impl WindowSize {
    /// Create a new WindowSize.
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    /// Aspect ratio (width / height).
    pub fn aspect_ratio(&self) -> f32 {
        if self.height > 0.0 {
            self.width / self.height
        } else {
            1.0
        }
    }

    /// Return as (width, height) tuple.
    pub fn as_tuple(&self) -> (f32, f32) {
        (self.width, self.height)
    }
}

impl Default for WindowSize {
    fn default() -> Self {
        Self {
            width: 1280.0,
            height: 720.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aspect_ratio() {
        let ws = WindowSize::new(1920.0, 1080.0);
        assert!((ws.aspect_ratio() - 16.0 / 9.0).abs() < 0.01);
    }

    #[test]
    fn test_zero_height() {
        let ws = WindowSize::new(100.0, 0.0);
        assert_eq!(ws.aspect_ratio(), 1.0);
    }
}
