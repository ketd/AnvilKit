//! UI event system — hit testing and interaction processing.

use bevy_ecs::prelude::*;
use crate::style::UiNode;

/// UI event kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiEventKind {
    HoverEnter,
    HoverLeave,
    Click,
}

/// A UI event targeting a specific entity.
#[derive(Debug, Clone)]
pub struct UiEvent {
    pub entity: Entity,
    pub kind: UiEventKind,
}

/// UI events resource — collects events each frame.
#[derive(Resource, Default)]
pub struct UiEvents {
    pub events: Vec<UiEvent>,
    pub hovered: Option<Entity>,
}

impl UiEvents {
    pub fn clear(&mut self) {
        self.events.clear();
    }

    pub fn iter(&self) -> impl Iterator<Item = &UiEvent> {
        self.events.iter()
    }

    pub fn was_clicked(&self, entity: Entity) -> bool {
        self.events.iter().any(|e| e.entity == entity && e.kind == UiEventKind::Click)
    }

    pub fn is_hovered(&self, entity: Entity) -> bool {
        self.hovered == Some(entity)
    }
}

/// Test if a point is inside a node's computed rect.
pub fn ui_hit_test(
    nodes: &[(Entity, &UiNode)],
    point: glam::Vec2,
) -> Option<Entity> {
    // Iterate in reverse for front-to-back ordering (last drawn = on top)
    for (entity, node) in nodes.iter().rev() {
        if !node.visible { continue; }
        let [x, y, w, h] = node.computed_rect;
        if point.x >= x && point.x <= x + w && point.y >= y && point.y <= y + h {
            return Some(*entity);
        }
    }
    None
}

/// Process mouse interactions and emit events.
pub fn process_ui_interactions(
    nodes: &[(Entity, &UiNode)],
    mouse_pos: glam::Vec2,
    mouse_pressed: bool,
    events: &mut UiEvents,
) {
    events.events.clear();

    let hit = ui_hit_test(nodes, mouse_pos);

    // Hover transitions
    let prev_hovered = events.hovered;
    if hit != prev_hovered {
        if let Some(prev) = prev_hovered {
            events.events.push(UiEvent { entity: prev, kind: UiEventKind::HoverLeave });
        }
        if let Some(current) = hit {
            events.events.push(UiEvent { entity: current, kind: UiEventKind::HoverEnter });
        }
    }
    events.hovered = hit;

    // Click
    if mouse_pressed {
        if let Some(entity) = hit {
            events.events.push(UiEvent { entity, kind: UiEventKind::Click });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hit_test() {
        let mut world = bevy_ecs::world::World::new();
        let e1 = world.spawn_empty().id();
        let n1 = UiNode {
            computed_rect: [10.0, 10.0, 100.0, 50.0],
            visible: true,
            ..Default::default()
        };

        let nodes = vec![(e1, &n1)];
        assert_eq!(ui_hit_test(&nodes, glam::Vec2::new(50.0, 30.0)), Some(e1));
        assert_eq!(ui_hit_test(&nodes, glam::Vec2::new(0.0, 0.0)), None);
    }

    #[test]
    fn test_process_interactions() {
        let mut world = bevy_ecs::world::World::new();
        let e1 = world.spawn_empty().id();
        let n1 = UiNode {
            computed_rect: [0.0, 0.0, 100.0, 50.0],
            visible: true,
            ..Default::default()
        };

        let mut events = UiEvents::default();
        process_ui_interactions(&[(e1, &n1)], glam::Vec2::new(50.0, 25.0), true, &mut events);
        assert!(events.was_clicked(e1));
        assert!(events.is_hovered(e1));
    }
}
