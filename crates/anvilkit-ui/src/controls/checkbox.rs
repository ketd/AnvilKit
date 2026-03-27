//! Checkbox control — toggles a boolean value on click.

use bevy_ecs::prelude::*;
use crate::style::*;

/// Checkbox component — attached alongside a UiNode.
#[derive(Debug, Clone, Component)]
pub struct Checkbox {
    pub checked: bool,
    pub label: String,
}

impl Checkbox {
    pub fn new(label: impl Into<String>) -> Self {
        Self { checked: false, label: label.into() }
    }

    pub fn checked(mut self, value: bool) -> Self {
        self.checked = value;
        self
    }

    /// Create the UiNode for this checkbox.
    pub fn node(&self) -> UiNode {
        let indicator = if self.checked { "[x]" } else { "[ ]" };
        let text = format!("{} {}", indicator, self.label);
        UiNode {
            style: UiStyle {
                padding: [4.0, 8.0, 4.0, 8.0],
                ..Default::default()
            },
            text: Some(UiText::new(text).with_font_size(16.0)),
            background_color: [0.2, 0.2, 0.2, 0.8],
            corner_radius: 3.0,
            ..Default::default()
        }
    }

    /// Toggle the checked state. Returns the new value.
    pub fn toggle(&mut self) -> bool {
        self.checked = !self.checked;
        self.checked
    }
}

/// Event emitted when a checkbox value changes.
#[derive(Debug, Clone, Event)]
pub struct UiChangeEvent {
    pub entity: Entity,
    pub value: bool,
}

/// System that handles checkbox clicks.
pub fn checkbox_system(
    mut query: Query<(Entity, &mut Checkbox)>,
    events: Res<crate::events::UiEvents>,
    mut change_events: EventWriter<UiChangeEvent>,
) {
    for (entity, mut cb) in &mut query {
        if events.was_clicked(entity) {
            let new_val = cb.toggle();
            change_events.send(UiChangeEvent { entity, value: new_val });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checkbox_toggle() {
        let mut cb = Checkbox::new("Test");
        assert!(!cb.checked);
        assert!(cb.toggle());
        assert!(cb.checked);
        assert!(!cb.toggle());
    }

    #[test]
    fn test_checkbox_node() {
        let cb = Checkbox::new("Enable").checked(true);
        let node = cb.node();
        assert!(node.text.unwrap().content.contains("[x]"));
    }
}
