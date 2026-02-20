//! # 物理组件
//!
//! 定义 ECS 物理组件类型，用于与 rapier 物理引擎集成。
//! 组件定义不依赖 rapier，实际物理模拟由 PhysicsPlugin 提供。
//!
//! ## 使用示例
//!
//! ```rust
//! use anvilkit_ecs::physics::{RigidBody, RigidBodyType, Collider, ColliderShape, Velocity};
//! use glam::Vec3;
//!
//! let body = RigidBody::new(RigidBodyType::Dynamic);
//! assert_eq!(body.body_type, RigidBodyType::Dynamic);
//!
//! let collider = Collider::sphere(0.5);
//! let velocity = Velocity::linear(Vec3::new(1.0, 0.0, 0.0));
//! ```

use bevy_ecs::prelude::*;
use glam::Vec3;

/// 刚体类型
///
/// # 示例
///
/// ```rust
/// use anvilkit_ecs::physics::RigidBodyType;
///
/// let dynamic = RigidBodyType::Dynamic;
/// let fixed = RigidBodyType::Fixed;
/// assert_ne!(dynamic, fixed);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RigidBodyType {
    /// 动态刚体（受力和重力影响）
    Dynamic,
    /// 固定刚体（不移动）
    Fixed,
    /// 运动学刚体（手动控制位置，不受力影响）
    Kinematic,
}

/// 碰撞体形状
///
/// # 示例
///
/// ```rust
/// use anvilkit_ecs::physics::ColliderShape;
///
/// let sphere = ColliderShape::Sphere { radius: 1.0 };
/// let cube = ColliderShape::Cuboid { half_extents: glam::Vec3::ONE };
/// ```
#[derive(Debug, Clone)]
pub enum ColliderShape {
    /// 球体
    Sphere { radius: f32 },
    /// 长方体（半尺寸）
    Cuboid { half_extents: Vec3 },
    /// 胶囊体
    Capsule { half_height: f32, radius: f32 },
    /// 网格碰撞体（顶点 + 三角形索引）
    TriMesh { vertices: Vec<Vec3>, indices: Vec<[u32; 3]> },
}

/// 刚体组件
///
/// 附加到实体上表示该实体参与物理模拟。
///
/// # 示例
///
/// ```rust
/// use anvilkit_ecs::physics::{RigidBody, RigidBodyType};
///
/// let body = RigidBody::new(RigidBodyType::Dynamic);
/// assert!(!body.is_sensor);
/// assert_eq!(body.mass, 1.0);
/// ```
#[derive(Debug, Clone, Component)]
pub struct RigidBody {
    pub body_type: RigidBodyType,
    pub mass: f32,
    pub gravity_scale: f32,
    pub is_sensor: bool,
    pub linear_damping: f32,
    pub angular_damping: f32,
}

impl RigidBody {
    pub fn new(body_type: RigidBodyType) -> Self {
        Self {
            body_type,
            mass: 1.0,
            gravity_scale: 1.0,
            is_sensor: false,
            linear_damping: 0.0,
            angular_damping: 0.05,
        }
    }

    pub fn dynamic() -> Self { Self::new(RigidBodyType::Dynamic) }
    pub fn fixed() -> Self { Self::new(RigidBodyType::Fixed) }
    pub fn kinematic() -> Self { Self::new(RigidBodyType::Kinematic) }
}

/// 碰撞体组件
///
/// 定义实体的碰撞形状和物理材质。
///
/// # 示例
///
/// ```rust
/// use anvilkit_ecs::physics::Collider;
///
/// let sphere = Collider::sphere(0.5);
/// assert_eq!(sphere.friction, 0.5);
/// assert_eq!(sphere.restitution, 0.0);
/// ```
#[derive(Debug, Clone, Component)]
pub struct Collider {
    pub shape: ColliderShape,
    pub friction: f32,
    pub restitution: f32,
}

impl Collider {
    pub fn sphere(radius: f32) -> Self {
        Self { shape: ColliderShape::Sphere { radius }, friction: 0.5, restitution: 0.0 }
    }

    pub fn cuboid(half_extents: Vec3) -> Self {
        Self { shape: ColliderShape::Cuboid { half_extents }, friction: 0.5, restitution: 0.0 }
    }

    pub fn capsule(half_height: f32, radius: f32) -> Self {
        Self { shape: ColliderShape::Capsule { half_height, radius }, friction: 0.5, restitution: 0.0 }
    }
}

/// 速度组件
///
/// # 示例
///
/// ```rust
/// use anvilkit_ecs::physics::Velocity;
/// use glam::Vec3;
///
/// let vel = Velocity::linear(Vec3::X);
/// assert_eq!(vel.linear, Vec3::X);
/// assert_eq!(vel.angular, Vec3::ZERO);
/// ```
#[derive(Debug, Clone, Copy, Component)]
pub struct Velocity {
    pub linear: Vec3,
    pub angular: Vec3,
}

impl Velocity {
    pub fn zero() -> Self { Self { linear: Vec3::ZERO, angular: Vec3::ZERO } }
    pub fn linear(linear: Vec3) -> Self { Self { linear, angular: Vec3::ZERO } }
}

impl Default for Velocity {
    fn default() -> Self { Self::zero() }
}

// ---------------------------------------------------------------------------
//  Physics runtime (自写 AABB 碰撞 + 速度积分)
// ---------------------------------------------------------------------------

use crate::schedule::AnvilKitSchedule;
use anvilkit_core::math::Transform;

/// 帧时间资源
#[derive(Resource)]
pub struct DeltaTime(pub f32);

impl Default for DeltaTime {
    fn default() -> Self { Self(1.0 / 60.0) }
}

/// 碰撞事件
#[derive(Debug, Clone, Copy)]
pub struct CollisionEvent {
    pub a: Entity,
    pub b: Entity,
}

/// 碰撞事件列表资源
#[derive(Resource, Default)]
pub struct CollisionEvents {
    pub events: Vec<CollisionEvent>,
}

impl CollisionEvents {
    pub fn clear(&mut self) { self.events.clear(); }
    pub fn push(&mut self, event: CollisionEvent) { self.events.push(event); }
    pub fn iter(&self) -> impl Iterator<Item = &CollisionEvent> { self.events.iter() }
    pub fn is_empty(&self) -> bool { self.events.is_empty() }
}

/// 简单 AABB 碰撞体组件（局部空间）
#[derive(Debug, Clone, Copy, Component)]
pub struct AabbCollider {
    pub half_extents: Vec3,
}

impl AabbCollider {
    pub fn new(half_extents: Vec3) -> Self { Self { half_extents } }
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
    mut events: ResMut<CollisionEvents>,
) {
    events.clear();

    let entities: Vec<_> = query.iter().collect();
    for i in 0..entities.len() {
        for j in (i + 1)..entities.len() {
            let (ea, ta, ca) = &entities[i];
            let (eb, tb, cb) = &entities[j];

            // World-space AABB
            let a_min = ta.translation - ca.half_extents;
            let a_max = ta.translation + ca.half_extents;
            let b_min = tb.translation - cb.half_extents;
            let b_max = tb.translation + cb.half_extents;

            if a_min.x <= b_max.x && a_max.x >= b_min.x
                && a_min.y <= b_max.y && a_max.y >= b_min.y
                && a_min.z <= b_max.z && a_max.z >= b_min.z
            {
                events.push(CollisionEvent { a: *ea, b: *eb });
            }
        }
    }
}

/// 物理插件（自写 AABB + 速度积分）
pub struct PhysicsPlugin;

impl crate::plugin::Plugin for PhysicsPlugin {
    fn build(&self, app: &mut crate::app::App) {
        app.init_resource::<DeltaTime>();
        app.init_resource::<CollisionEvents>();
        app.add_systems(
            AnvilKitSchedule::Update,
            (
                velocity_integration_system,
                collision_detection_system.after(velocity_integration_system),
            ),
        );
    }
}

// ---------------------------------------------------------------------------
//  rapier3d physics integration (feature-gated)
// ---------------------------------------------------------------------------

#[cfg(feature = "rapier")]
pub mod rapier_integration {
    use super::*;
    use rapier3d::prelude as rp;
    use rapier3d::prelude::nalgebra;
    use std::collections::HashMap;

    /// rapier3d 物理世界资源
    #[derive(Resource)]
    pub struct RapierContext {
        pub gravity: rp::Vector<f32>,
        pub integration_parameters: rp::IntegrationParameters,
        pub physics_pipeline: rp::PhysicsPipeline,
        pub island_manager: rp::IslandManager,
        pub broad_phase: rp::BroadPhase,
        pub narrow_phase: rp::NarrowPhase,
        pub rigid_body_set: rp::RigidBodySet,
        pub collider_set: rp::ColliderSet,
        pub impulse_joint_set: rp::ImpulseJointSet,
        pub multibody_joint_set: rp::MultibodyJointSet,
        pub ccd_solver: rp::CCDSolver,
        /// ECS Entity → rapier RigidBodyHandle mapping
        pub entity_to_body: HashMap<Entity, rp::RigidBodyHandle>,
    }

    impl Default for RapierContext {
        fn default() -> Self {
            Self {
                gravity: rp::vector![0.0, -9.81, 0.0],
                integration_parameters: rp::IntegrationParameters::default(),
                physics_pipeline: rp::PhysicsPipeline::new(),
                island_manager: rp::IslandManager::new(),
                broad_phase: rp::BroadPhase::new(),
                narrow_phase: rp::NarrowPhase::new(),
                rigid_body_set: rp::RigidBodySet::new(),
                collider_set: rp::ColliderSet::new(),
                impulse_joint_set: rp::ImpulseJointSet::new(),
                multibody_joint_set: rp::MultibodyJointSet::new(),
                ccd_solver: rp::CCDSolver::new(),
                entity_to_body: HashMap::new(),
            }
        }
    }

    /// ECS RigidBody/Collider → rapier bodies 同步
    pub fn sync_to_rapier_system(
        query: Query<(Entity, &RigidBody, &Collider, &Transform), Without<RapierSynced>>,
        mut ctx: ResMut<RapierContext>,
        mut commands: Commands,
    ) {
        for (entity, rb, col, transform) in query.iter() {
            let body_type = match rb.body_type {
                RigidBodyType::Dynamic => rp::RigidBodyType::Dynamic,
                RigidBodyType::Fixed => rp::RigidBodyType::Fixed,
                RigidBodyType::Kinematic => rp::RigidBodyType::KinematicPositionBased,
            };

            let body = rp::RigidBodyBuilder::new(body_type)
                .translation(rp::vector![
                    transform.translation.x,
                    transform.translation.y,
                    transform.translation.z
                ])
                .gravity_scale(rb.gravity_scale)
                .linear_damping(rb.linear_damping)
                .angular_damping(rb.angular_damping)
                .build();

            let body_handle = ctx.rigid_body_set.insert(body);

            let collider = match &col.shape {
                ColliderShape::Sphere { radius } => {
                    rp::ColliderBuilder::ball(*radius)
                }
                ColliderShape::Cuboid { half_extents } => {
                    rp::ColliderBuilder::cuboid(half_extents.x, half_extents.y, half_extents.z)
                }
                ColliderShape::Capsule { half_height, radius } => {
                    rp::ColliderBuilder::capsule_y(*half_height, *radius)
                }
                ColliderShape::TriMesh { .. } => {
                    // Fallback to unit ball for trimesh (full trimesh conversion is complex)
                    rp::ColliderBuilder::ball(1.0)
                }
            }
            .friction(col.friction)
            .restitution(col.restitution)
            .build();

            let c = &mut *ctx;
            c.collider_set.insert_with_parent(collider, body_handle, &mut c.rigid_body_set);
            ctx.entity_to_body.insert(entity, body_handle);
            commands.entity(entity).insert(RapierSynced);
        }
    }

    /// 执行物理模拟步骤
    pub fn step_physics_system(mut ctx: ResMut<RapierContext>) {
        let c = &mut *ctx;
        c.physics_pipeline.step(
            &c.gravity,
            &c.integration_parameters,
            &mut c.island_manager,
            &mut c.broad_phase,
            &mut c.narrow_phase,
            &mut c.rigid_body_set,
            &mut c.collider_set,
            &mut c.impulse_joint_set,
            &mut c.multibody_joint_set,
            &mut c.ccd_solver,
            None,
            &(),
            &(),
        );
    }

    /// rapier positions → ECS Transform 同步
    pub fn sync_from_rapier_system(
        mut query: Query<(Entity, &mut Transform), With<RapierSynced>>,
        ctx: Res<RapierContext>,
    ) {
        for (entity, mut transform) in query.iter_mut() {
            if let Some(&handle) = ctx.entity_to_body.get(&entity) {
                if let Some(body) = ctx.rigid_body_set.get(handle) {
                    let pos = body.translation();
                    let rot = body.rotation();
                    transform.translation = Vec3::new(pos.x, pos.y, pos.z);
                    transform.rotation = glam::Quat::from_xyzw(rot.i, rot.j, rot.k, rot.w);
                }
            }
        }
    }

    /// 标记已同步到 rapier 的实体
    #[derive(Component)]
    pub struct RapierSynced;

    /// rapier3d 物理插件
    pub struct RapierPhysicsPlugin;

    impl crate::plugin::Plugin for RapierPhysicsPlugin {
        fn build(&self, app: &mut crate::app::App) {
            app.init_resource::<RapierContext>();
            app.add_systems(
                AnvilKitSchedule::Update,
                (
                    sync_to_rapier_system,
                    step_physics_system.after(sync_to_rapier_system),
                    sync_from_rapier_system.after(step_physics_system),
                ),
            );
        }
    }
}

#[cfg(feature = "rapier")]
pub use rapier_integration::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rigid_body() {
        let body = RigidBody::dynamic();
        assert_eq!(body.body_type, RigidBodyType::Dynamic);
        assert_eq!(body.mass, 1.0);
    }

    #[test]
    fn test_collider() {
        let c = Collider::sphere(1.0);
        assert_eq!(c.friction, 0.5);
        match c.shape {
            ColliderShape::Sphere { radius } => assert_eq!(radius, 1.0),
            _ => panic!("wrong shape"),
        }
    }

    #[test]
    fn test_velocity() {
        let v = Velocity::linear(Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(v.linear, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(v.angular, Vec3::ZERO);
    }

    #[test]
    fn test_delta_time_default() {
        let dt = DeltaTime::default();
        assert!((dt.0 - 1.0 / 60.0).abs() < 0.001);
    }

    #[test]
    fn test_collision_events() {
        let mut events = CollisionEvents::default();
        assert!(events.is_empty());
        events.push(CollisionEvent { a: Entity::PLACEHOLDER, b: Entity::PLACEHOLDER });
        assert_eq!(events.events.len(), 1);
        events.clear();
        assert!(events.is_empty());
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
        app.add_systems(AnvilKitSchedule::Update, velocity_integration_system);

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
        app.init_resource::<CollisionEvents>();
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

        let events = app.world.get_resource::<CollisionEvents>().unwrap();
        // Entities 0 and 1 collide; entity 2 is too far from both
        assert_eq!(events.events.len(), 1);
    }
}
