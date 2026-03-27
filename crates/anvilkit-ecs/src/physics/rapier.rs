#[cfg(feature = "rapier")]
pub mod rapier_integration {
    use super::super::components::*;
    use super::super::events::*;
    use crate::schedule::AnvilKitSchedule;
    use anvilkit_core::math::Transform;
    use bevy_ecs::prelude::*;
    use glam::Vec3;
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
