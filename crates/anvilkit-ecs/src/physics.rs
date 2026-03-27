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
    Sphere {
        /// Sphere radius.
        radius: f32,
    },
    /// 长方体（半尺寸）
    Cuboid {
        /// Half-extents along each axis.
        half_extents: Vec3,
    },
    /// 胶囊体
    Capsule {
        /// Half the height of the cylindrical section.
        half_height: f32,
        /// Radius of the capsule hemispheres.
        radius: f32,
    },
    /// 网格碰撞体（顶点 + 三角形索引）
    TriMesh {
        /// Mesh vertex positions.
        vertices: Vec<Vec3>,
        /// Triangle index triples.
        indices: Vec<[u32; 3]>,
    },
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
    /// The type of rigid body (dynamic, fixed, or kinematic).
    pub body_type: RigidBodyType,
    /// Mass of the rigid body in kilograms.
    pub mass: f32,
    /// Multiplier for gravity applied to this body (1.0 = normal gravity).
    pub gravity_scale: f32,
    /// Whether this body acts as a sensor (detects overlaps without physical response).
    pub is_sensor: bool,
    /// Linear velocity damping factor (resistance to linear motion).
    pub linear_damping: f32,
    /// Angular velocity damping factor (resistance to rotational motion).
    pub angular_damping: f32,
}

impl RigidBody {
    /// Creates a new rigid body with the given type and default physical properties.
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

    /// Creates a new dynamic rigid body.
    pub fn dynamic() -> Self { Self::new(RigidBodyType::Dynamic) }
    /// Creates a new fixed rigid body.
    pub fn fixed() -> Self { Self::new(RigidBodyType::Fixed) }
    /// Creates a new kinematic rigid body.
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
    /// The geometric shape of this collider.
    pub shape: ColliderShape,
    /// Surface friction coefficient (0.0 = frictionless, 1.0 = high friction).
    pub friction: f32,
    /// Bounciness coefficient (0.0 = no bounce, 1.0 = perfectly elastic).
    pub restitution: f32,
}

impl Collider {
    /// Creates a sphere collider with the given radius.
    pub fn sphere(radius: f32) -> Self {
        Self { shape: ColliderShape::Sphere { radius }, friction: 0.5, restitution: 0.0 }
    }

    /// Creates a box collider with the given half-extents.
    pub fn cuboid(half_extents: Vec3) -> Self {
        Self { shape: ColliderShape::Cuboid { half_extents }, friction: 0.5, restitution: 0.0 }
    }

    /// Creates a capsule collider with the given half-height and radius.
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
    /// Linear velocity vector (units per second).
    pub linear: Vec3,
    /// Angular velocity vector (radians per second around each axis).
    pub angular: Vec3,
}

impl Velocity {
    /// Creates a velocity with zero linear and angular components.
    pub fn zero() -> Self { Self { linear: Vec3::ZERO, angular: Vec3::ZERO } }
    /// Creates a velocity with the given linear component and zero angular velocity.
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

/// Syncs DeltaTime from the core Time resource.
pub fn update_delta_time_system(
    time: Res<anvilkit_core::time::Time>,
    mut dt: ResMut<DeltaTime>,
) {
    dt.0 = time.delta_seconds();
}

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
        /// rapier RigidBodyHandle → ECS Entity reverse mapping
        pub body_to_entity: HashMap<rp::RigidBodyHandle, Entity>,
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
                body_to_entity: HashMap::new(),
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
                ColliderShape::TriMesh { vertices, .. } => {
                    // Compute a bounding sphere radius from mesh vertices if available
                    let radius = if vertices.is_empty() {
                        log::warn!(
                            "TriMesh collider has no vertices; falling back to unit ball"
                        );
                        1.0
                    } else {
                        let max_r = vertices.iter()
                            .map(|v| v.length())
                            .fold(0.0f32, f32::max);
                        log::warn!(
                            "TriMesh collider not fully supported; falling back to bounding sphere (r={:.3})",
                            max_r
                        );
                        if max_r < f32::EPSILON { 1.0 } else { max_r }
                    };
                    rp::ColliderBuilder::ball(radius)
                }
            }
            .friction(col.friction)
            .restitution(col.restitution)
            .build();

            let c = &mut *ctx;
            c.collider_set.insert_with_parent(collider, body_handle, &mut c.rigid_body_set);
            ctx.entity_to_body.insert(entity, body_handle);
            ctx.body_to_entity.insert(body_handle, entity);
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

    /// 物理射线检测结果
    #[derive(Debug, Clone)]
    pub struct RaycastHit {
        /// The entity that was hit.
        pub entity: Entity,
        /// World-space hit point.
        pub point: Vec3,
        /// Surface normal at the hit point.
        pub normal: Vec3,
        /// Distance from ray origin to the hit point.
        pub distance: f32,
    }

    impl RapierContext {
        /// Cast a ray and return the closest hit.
        pub fn raycast(
            &self,
            origin: Vec3,
            direction: Vec3,
            max_distance: f32,
        ) -> Option<RaycastHit> {
            let ray = rp::Ray::new(
                rp::point![origin.x, origin.y, origin.z],
                rp::vector![direction.x, direction.y, direction.z],
            );

            // Use rapier's query pipeline for efficient raycasting
            let mut best: Option<(Entity, f32, Vec3, Vec3)> = None;

            for (collider_handle, collider) in self.collider_set.iter() {
                if let Some(hit) = collider.shape().cast_ray_and_get_normal(
                    collider.position(),
                    &ray,
                    max_distance,
                    true,
                ) {
                    if best.as_ref().map_or(true, |b| hit.toi < b.1) {
                        // Find the entity for this collider's parent body
                        let entity = collider.parent()
                            .and_then(|bh| self.body_to_entity.get(&bh).copied());
                        if let Some(entity) = entity {
                            let point = ray.point_at(hit.toi);
                            best = Some((
                                entity,
                                hit.toi,
                                Vec3::new(point.x, point.y, point.z),
                                Vec3::new(hit.normal.x, hit.normal.y, hit.normal.z),
                            ));
                        }
                    }
                }
            }

            best.map(|(entity, distance, point, normal)| RaycastHit {
                entity,
                point,
                normal,
                distance,
            })
        }
    }

    /// 从 rapier NarrowPhase 提取碰撞事件到 ECS CollisionEvents 资源
    /// 从 rapier NarrowPhase 提取碰撞事件到 ECS CollisionEvents 资源
    pub fn extract_collision_events_system(
        ctx: Res<RapierContext>,
        mut events: ResMut<CollisionEvents>,
    ) {
        events.clear();
        for pair in ctx.narrow_phase.contact_pairs() {
            if pair.has_any_active_contact {
                let entity_a = ctx.collider_set.get(pair.collider1)
                    .and_then(|c| c.parent())
                    .and_then(|bh| ctx.body_to_entity.get(&bh).copied());
                let entity_b = ctx.collider_set.get(pair.collider2)
                    .and_then(|c| c.parent())
                    .and_then(|bh| ctx.body_to_entity.get(&bh).copied());

                if let (Some(a), Some(b)) = (entity_a, entity_b) {
                    events.push(CollisionEvent { a, b });
                }
            }
        }
    }

    // ==================== Joint Constraints ====================

    /// Fixed joint: locks two bodies together with no relative movement.
    #[derive(Component, Debug, Clone)]
    pub struct FixedJoint {
        /// The other entity to connect to.
        pub target: Entity,
        /// Anchor in this entity's local space.
        pub local_anchor1: Vec3,
        /// Anchor in target entity's local space.
        pub local_anchor2: Vec3,
    }

    impl FixedJoint {
        /// Create a fixed joint connecting to `target` at origin anchors.
        pub fn new(target: Entity) -> Self {
            Self { target, local_anchor1: Vec3::ZERO, local_anchor2: Vec3::ZERO }
        }
        /// Set anchor points.
        pub fn with_anchors(mut self, a1: Vec3, a2: Vec3) -> Self {
            self.local_anchor1 = a1;
            self.local_anchor2 = a2;
            self
        }
    }

    /// Revolute joint: allows rotation around a single axis.
    #[derive(Component, Debug, Clone)]
    pub struct RevoluteJoint {
        pub target: Entity,
        pub local_anchor1: Vec3,
        pub local_anchor2: Vec3,
        /// Rotation axis (normalized).
        pub axis: Vec3,
        /// Optional angle limits (min, max) in radians.
        pub limits: Option<(f32, f32)>,
    }

    impl RevoluteJoint {
        pub fn new(target: Entity, axis: Vec3) -> Self {
            Self {
                target,
                local_anchor1: Vec3::ZERO,
                local_anchor2: Vec3::ZERO,
                axis: axis.normalize(),
                limits: None,
            }
        }
        pub fn with_anchors(mut self, a1: Vec3, a2: Vec3) -> Self {
            self.local_anchor1 = a1;
            self.local_anchor2 = a2;
            self
        }
        pub fn with_limits(mut self, min: f32, max: f32) -> Self {
            self.limits = Some((min, max));
            self
        }
    }

    /// Prismatic joint: allows linear sliding along a single axis.
    #[derive(Component, Debug, Clone)]
    pub struct PrismaticJoint {
        pub target: Entity,
        pub local_anchor1: Vec3,
        pub local_anchor2: Vec3,
        /// Sliding axis (normalized).
        pub axis: Vec3,
        /// Optional distance limits (min, max).
        pub limits: Option<(f32, f32)>,
    }

    impl PrismaticJoint {
        pub fn new(target: Entity, axis: Vec3) -> Self {
            Self {
                target,
                local_anchor1: Vec3::ZERO,
                local_anchor2: Vec3::ZERO,
                axis: axis.normalize(),
                limits: None,
            }
        }
        pub fn with_anchors(mut self, a1: Vec3, a2: Vec3) -> Self {
            self.local_anchor1 = a1;
            self.local_anchor2 = a2;
            self
        }
        pub fn with_limits(mut self, min: f32, max: f32) -> Self {
            self.limits = Some((min, max));
            self
        }
    }

    /// Spherical joint: allows rotation around all axes (ball-and-socket).
    #[derive(Component, Debug, Clone)]
    pub struct SphericalJoint {
        pub target: Entity,
        pub local_anchor1: Vec3,
        pub local_anchor2: Vec3,
    }

    impl SphericalJoint {
        pub fn new(target: Entity) -> Self {
            Self { target, local_anchor1: Vec3::ZERO, local_anchor2: Vec3::ZERO }
        }
        pub fn with_anchors(mut self, a1: Vec3, a2: Vec3) -> Self {
            self.local_anchor1 = a1;
            self.local_anchor2 = a2;
            self
        }
    }

    /// Marker: this entity's joint has been synced to Rapier.
    #[derive(Component)]
    pub struct JointSynced(pub rp::ImpulseJointHandle);

    /// Helper to convert glam Vec3 to rapier nalgebra Point3.
    fn to_rapier_point(v: Vec3) -> nalgebra::Point3<f32> {
        nalgebra::Point3::new(v.x, v.y, v.z)
    }

    /// Helper to convert glam Vec3 to rapier nalgebra UnitVector3.
    fn to_rapier_axis(v: Vec3) -> nalgebra::Unit<nalgebra::Vector3<f32>> {
        nalgebra::Unit::new_normalize(nalgebra::Vector3::new(v.x, v.y, v.z))
    }

    /// Syncs ECS joint components to Rapier's ImpulseJointSet.
    /// Runs after body sync, before physics step.
    pub fn sync_joints_to_rapier_system(
        mut context: ResMut<RapierContext>,
        fixed: Query<(Entity, &FixedJoint), Without<JointSynced>>,
        revolute: Query<(Entity, &RevoluteJoint), Without<JointSynced>>,
        prismatic: Query<(Entity, &PrismaticJoint), Without<JointSynced>>,
        spherical: Query<(Entity, &SphericalJoint), Without<JointSynced>>,
        mut commands: Commands,
    ) {
        // Fixed joints
        for (entity, joint) in fixed.iter() {
            let Some(&body1) = context.entity_to_body.get(&entity) else { continue };
            let Some(&body2) = context.entity_to_body.get(&joint.target) else { continue };
            let rapier_joint = rp::FixedJointBuilder::new()
                .local_anchor1(to_rapier_point(joint.local_anchor1))
                .local_anchor2(to_rapier_point(joint.local_anchor2))
                .build();
            let handle = context.impulse_joint_set.insert(body1, body2, rapier_joint, true);
            commands.entity(entity).insert(JointSynced(handle));
        }

        // Revolute joints
        for (entity, joint) in revolute.iter() {
            let Some(&body1) = context.entity_to_body.get(&entity) else { continue };
            let Some(&body2) = context.entity_to_body.get(&joint.target) else { continue };
            let mut builder = rp::RevoluteJointBuilder::new(to_rapier_axis(joint.axis))
                .local_anchor1(to_rapier_point(joint.local_anchor1))
                .local_anchor2(to_rapier_point(joint.local_anchor2));
            if let Some((min, max)) = joint.limits {
                builder = builder.limits([min, max]);
            }
            let handle = context.impulse_joint_set.insert(body1, body2, builder.build(), true);
            commands.entity(entity).insert(JointSynced(handle));
        }

        // Prismatic joints
        for (entity, joint) in prismatic.iter() {
            let Some(&body1) = context.entity_to_body.get(&entity) else { continue };
            let Some(&body2) = context.entity_to_body.get(&joint.target) else { continue };
            let mut builder = rp::PrismaticJointBuilder::new(to_rapier_axis(joint.axis))
                .local_anchor1(to_rapier_point(joint.local_anchor1))
                .local_anchor2(to_rapier_point(joint.local_anchor2));
            if let Some((min, max)) = joint.limits {
                builder = builder.limits([min, max]);
            }
            let handle = context.impulse_joint_set.insert(body1, body2, builder.build(), true);
            commands.entity(entity).insert(JointSynced(handle));
        }

        // Spherical joints
        for (entity, joint) in spherical.iter() {
            let Some(&body1) = context.entity_to_body.get(&entity) else { continue };
            let Some(&body2) = context.entity_to_body.get(&joint.target) else { continue };
            let rapier_joint = rp::SphericalJointBuilder::new()
                .local_anchor1(to_rapier_point(joint.local_anchor1))
                .local_anchor2(to_rapier_point(joint.local_anchor2))
                .build();
            let handle = context.impulse_joint_set.insert(body1, body2, rapier_joint, true);
            commands.entity(entity).insert(JointSynced(handle));
        }
    }

    /// rapier3d 物理插件
    pub struct RapierPhysicsPlugin;

    impl crate::plugin::Plugin for RapierPhysicsPlugin {
        fn build(&self, app: &mut crate::app::App) {
            app.init_resource::<RapierContext>();
            app.init_resource::<CollisionEvents>();
            app.add_systems(
                AnvilKitSchedule::FixedUpdate,
                (
                    sync_to_rapier_system,
                    sync_joints_to_rapier_system.after(sync_to_rapier_system),
                    step_physics_system.after(sync_joints_to_rapier_system),
                    sync_from_rapier_system.after(step_physics_system),
                    extract_collision_events_system.after(step_physics_system),
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
