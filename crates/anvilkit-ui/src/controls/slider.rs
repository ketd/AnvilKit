//! Slider control — drag a handle to select a value in a range.

use bevy_ecs::prelude::*;
use crate::style::*;

/// Slider component.
#[derive(Debug, Clone, Component)]
pub struct Slider {
    pub value: f32,
    pub min: f32,
    pub max: f32,
    pub step: f32,
    pub dragging: bool,
}

impl Slider {
    pub fn new(min: f32, max: f32) -> Self {
        Self { value: min, min, max, step: 0.0, dragging: false }
    }

    pub fn with_value(mut self, value: f32) -> Self {
        self.value = value.clamp(self.min, self.max);
        self
    }

    pub fn with_step(mut self, step: f32) -> Self {
        self.step = step;
        self
    }

    /// Normalized position [0.0, 1.0].
    pub fn normalized(&self) -> f32 {
        if (self.max - self.min).abs() < f32::EPSILON { return 0.0; }
        ((self.value - self.min) / (self.max - self.min)).clamp(0.0, 1.0)
    }

    /// Set value from normalized position.
    pub fn set_normalized(&mut self, t: f32) {
        let mut v = self.min + t.clamp(0.0, 1.0) * (self.max - self.min);
        if self.step > 0.0 {
            v = (v / self.step).round() * self.step;
        }
        self.value = v.clamp(self.min, self.max);
    }

    /// Create the track UiNode.
    pub fn track_node(&self) -> UiNode {
        UiNode {
            style: UiStyle {
                width: Val::Percent(100.0),
                height: Val::Px(20.0),
                ..Default::default()
            },
            background_color: [0.2, 0.2, 0.2, 1.0],
            corner_radius: 4.0,
            ..Default::default()
        }
    }
}

/// System that updates slider value from mouse drag.
pub fn slider_system(
    mut query: Query<(Entity, &mut Slider, &UiNode)>,
    events: Res<crate::events::UiEvents>,
) {
    for (entity, mut slider, _node) in &mut query {
        if events.was_clicked(entity) {
            slider.dragging = true;
        }
        // When dragging, we'd update from mouse position relative to track rect.
        // The full implementation requires cursor position from the app layer.
        // For now, clicking toggles between min/max as a placeholder.
        if slider.dragging && events.was_clicked(entity) {
            let mid = (slider.min + slider.max) * 0.5;
            if slider.value < mid {
                slider.value = slider.max;
            } else {
                slider.value = slider.min;
            }
            slider.dragging = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slider_normalized() {
        let s = Slider::new(0.0, 100.0).with_value(50.0);
        assert!((s.normalized() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_slider_set_normalized() {
        let mut s = Slider::new(0.0, 100.0);
        s.set_normalized(0.75);
        assert!((s.value - 75.0).abs() < 0.01);
    }

    #[test]
    fn test_slider_step() {
        let mut s = Slider::new(0.0, 100.0).with_step(10.0);
        s.set_normalized(0.73);
        assert!((s.value - 70.0).abs() < 0.01);
    }
}
