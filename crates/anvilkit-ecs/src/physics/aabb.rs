use bevy_ecs::prelude::*;
use glam::Vec3;

use crate::schedule::AnvilKitSchedule;
use anvilkit_core::math::Transform;

use super::components::Velocity;
use super::events::CollisionEvent;

// Re-export DeltaTime from app module (canonical location)
pub use crate::app::DeltaTime;

/// Syncs DeltaTime from the core Time resource.
pub fn update_delta_time_system(
    time: Res<anvilkit_core::time::Time>,
    mut dt: ResMut<DeltaTime>,
) {
    dt.0 = time.delta_seconds();
}

/// 简单 AABB 碰撞体组件（局部空间）
#[derive(Debug, Clone, Copy, Component)]
pub struct AabbCollider {
    /// Half-extents of the axis-aligned bounding box along each axis.
    pub half_extents: Vec3,
}

impl AabbCollider {
    /// Creates an AABB collider with the given half-extents.
    pub fn new(half_extents: Vec3) -> Self { Self { half_extents } }
    /// Creates a cubic AABB collider with uniform half-extent.
    pub fn cube(half: f32) -> Self { Self { half_extents: Vec3::splat(half) } }
}

/// 速度积分系统：transform.translation += velocity.linear * dt
pub fn velocity_integration_system(
    dt: Res<DeltaTime>,
    mut query: Query<(&mut Transform, &Velocity)>,
) {
    for (mut transform, velocity) in &mut query {
        transform.translation += velocity.linear * dt.0;
    }
}

/// N² AABB 碰撞检测系统
pub fn collision_detection_system(
    query: Query<(Entity, &Transform, &AabbCollider)>,
    mut events: EventWriter<CollisionEvent>,
) {
    let entities: Vec<_> = query.iter().collect();
    for i in 0..entities.len() {
        for j in (i + 1)..entities.len() {
            let (ea, ta, ca) = &entities[i];
            let (eb, tb, cb) = &entities[j];

            // World-space AABB (note: rotation is not handled, only translation + scale)
            let a_min = ta.translation - ca.half_extents * ta.scale;
            let a_max = ta.translation + ca.half_extents * ta.scale;
            let b_min = tb.translation - cb.half_extents * tb.scale;
            let b_max = tb.translation + cb.half_extents * tb.scale;

            if a_min.x <= b_max.x && a_max.x >= b_min.x
                && a_min.y <= b_max.y && a_max.y >= b_min.y
                && a_min.z <= b_max.z && a_max.z >= b_min.z
            {
                events.send(CollisionEvent { a: *ea, b: *eb });
            }
        }
    }
}

/// 物理插件（自写 AABB + 速度积分）
pub struct PhysicsPlugin;

impl crate::plugin::Plugin for PhysicsPlugin {
    fn build(&self, app: &mut crate::app::App) {
        app.init_resource::<DeltaTime>();
        app.add_event::<CollisionEvent>();
        app.add_systems(
            AnvilKitSchedule::FixedUpdate,
            (
                update_delta_time_system,
                velocity_integration_system.after(update_delta_time_system),
                collision_detection_system.after(velocity_integration_system),
            ),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delta_time_default() {
        let dt = DeltaTime::default();
        assert!((dt.0 - 1.0 / 60.0).abs() < 0.001);
    }

    #[test]
    fn test_aabb_collider() {
        let c = AabbCollider::cube(0.5);
        assert_eq!(c.half_extents, Vec3::splat(0.5));
    }

    #[test]
    fn test_velocity_integration() {
        use crate::prelude::*;

        let mut app = App::new();
        app.init_resource::<DeltaTime>();
        app.add_systems(AnvilKitSchedule::FixedUpdate, velocity_integration_system);

        let entity = app.world.spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            Velocity::linear(Vec3::new(10.0, 0.0, 0.0)),
        )).id();

        app.update();

        let t = app.world.get::<Transform>(entity).unwrap();
        let expected_x = 10.0 * (1.0 / 60.0);
        assert!((t.translation.x - expected_x).abs() < 0.001);
    }

    #[test]
    fn test_collision_detection() {
        use crate::prelude::*;

        let mut app = App::new();
        app.add_event::<CollisionEvent>();
        // 使用 Update 调度测试碰撞逻辑（FixedUpdate 需要 Time 有非零 delta）
        app.add_systems(AnvilKitSchedule::Update, collision_detection_system);

        // Two overlapping entities
        app.world.spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            AabbCollider::cube(1.0),
        ));
        app.world.spawn((
            Transform::from_xyz(0.5, 0.0, 0.0),
            AabbCollider::cube(1.0),
        ));
        // One far away
        app.world.spawn((
            Transform::from_xyz(100.0, 0.0, 0.0),
            AabbCollider::cube(1.0),
        ));

        app.update();

        let events = app.world.resource::<Events<CollisionEvent>>();
        let mut reader = events.get_reader();
        let count = reader.read(events).count();
        // Entities 0 and 1 collide; entity 2 is too far from both
        assert_eq!(count, 1);
    }
}
