//! Focus management — Tab navigation between UI elements.

use bevy_ecs::prelude::*;
use crate::style::UiInteraction;

/// Tracks the currently focused UI entity.
#[derive(Resource, Default)]
pub struct UiFocus {
    /// The currently focused entity, if any.
    pub focused: Option<Entity>,
    /// Ordered list of focusable entities (set by the layout system).
    pub focusable_order: Vec<Entity>,
}

impl UiFocus {
    /// Move focus to the next focusable entity (Tab).
    pub fn focus_next(&mut self) {
        if self.focusable_order.is_empty() {
            self.focused = None;
            return;
        }
        let idx = self.focused
            .and_then(|f| self.focusable_order.iter().position(|e| *e == f))
            .map(|i| (i + 1) % self.focusable_order.len())
            .unwrap_or(0);
        self.focused = Some(self.focusable_order[idx]);
    }

    /// Move focus to the previous focusable entity (Shift+Tab).
    pub fn focus_prev(&mut self) {
        if self.focusable_order.is_empty() {
            self.focused = None;
            return;
        }
        let len = self.focusable_order.len();
        let idx = self.focused
            .and_then(|f| self.focusable_order.iter().position(|e| *e == f))
            .map(|i| if i == 0 { len - 1 } else { i - 1 })
            .unwrap_or(0);
        self.focused = Some(self.focusable_order[idx]);
    }

    /// Set focus to a specific entity.
    pub fn set_focus(&mut self, entity: Entity) {
        self.focused = Some(entity);
    }

    /// Clear focus.
    pub fn clear_focus(&mut self) {
        self.focused = None;
    }
}

/// System that updates UiInteraction components based on focus state.
pub fn focus_interaction_system(
    focus: Res<UiFocus>,
    mut query: Query<(Entity, &mut UiInteraction)>,
) {
    for (entity, mut interaction) in &mut query {
        if focus.focused == Some(entity) {
            if *interaction != UiInteraction::Pressed {
                *interaction = UiInteraction::Focused;
            }
        } else if *interaction == UiInteraction::Focused {
            *interaction = UiInteraction::None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_focus_next() {
        let mut world = bevy_ecs::world::World::new();
        let e1 = world.spawn_empty().id();
        let e2 = world.spawn_empty().id();
        let e3 = world.spawn_empty().id();

        let mut focus = UiFocus {
            focused: None,
            focusable_order: vec![e1, e2, e3],
        };

        focus.focus_next();
        assert_eq!(focus.focused, Some(e1));
        focus.focus_next();
        assert_eq!(focus.focused, Some(e2));
        focus.focus_next();
        assert_eq!(focus.focused, Some(e3));
        focus.focus_next();
        assert_eq!(focus.focused, Some(e1)); // wraps
    }

    #[test]
    fn test_focus_prev() {
        let mut world = bevy_ecs::world::World::new();
        let e1 = world.spawn_empty().id();
        let e2 = world.spawn_empty().id();

        let mut focus = UiFocus {
            focused: Some(e1),
            focusable_order: vec![e1, e2],
        };

        focus.focus_prev();
        assert_eq!(focus.focused, Some(e2)); // wraps backward
    }
}
