use bevy_ecs::prelude::*;

/// 碰撞事件
///
/// 通过 `EventWriter<CollisionEvent>` 发送，`EventReader<CollisionEvent>` 接收。
/// 事件自动双缓冲，存活 2 帧后由引擎清除。
#[derive(Debug, Clone, Copy, Event)]
pub struct CollisionEvent {
    /// First entity involved in the collision.
    pub a: Entity,
    /// Second entity involved in the collision.
    pub b: Entity,
}

/// 碰撞事件列表资源（已废弃）
#[deprecated(note = "使用 EventReader<CollisionEvent> 替代")]
pub struct CollisionEvents {
    /// List of collision events detected this frame.
    pub events: Vec<CollisionEvent>,
}

#[allow(deprecated)]
impl Resource for CollisionEvents {}

#[allow(deprecated)]
impl Default for CollisionEvents {
    fn default() -> Self { Self { events: Vec::new() } }
}

#[allow(deprecated)]
impl CollisionEvents {
    /// Removes all collision events from the list.
    pub fn clear(&mut self) { self.events.clear(); }
    /// Adds a collision event to the list.
    pub fn push(&mut self, event: CollisionEvent) { self.events.push(event); }
    /// Returns an iterator over all collision events.
    pub fn iter(&self) -> impl Iterator<Item = &CollisionEvent> { self.events.iter() }
    /// Returns true if there are no collision events.
    pub fn is_empty(&self) -> bool { self.events.is_empty() }
}

#[cfg(test)]
mod tests {
    #[allow(deprecated)]
    use super::*;

    #[test]
    fn test_collision_events() {
        let mut events = CollisionEvents::default();
        assert!(events.is_empty());
        events.push(CollisionEvent { a: Entity::PLACEHOLDER, b: Entity::PLACEHOLDER });
        assert_eq!(events.events.len(), 1);
        events.clear();
        assert!(events.is_empty());
    }
}
