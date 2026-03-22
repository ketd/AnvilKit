## ADDED Requirements

### Requirement: Physics Engine Runtime
The system SHALL integrate `rapier3d` (and optionally `rapier2d`) as the physics simulation backend, gated behind `physics-3d` and `physics-2d` Cargo feature flags.

The system SHALL provide a `PhysicsPlugin` that registers a `PhysicsWorld` resource and a `physics_step_system` running in `AnvilKitSchedule::PostUpdate`.

#### Scenario: Rigid body simulation
- **WHEN** entities have `RigidBody(Dynamic)` + `Collider` + `Transform` components
- **THEN** the physics system simulates gravity, forces, and collisions, updating `Transform` each frame

#### Scenario: Static colliders
- **WHEN** an entity has `RigidBody(Static)` + `Collider`
- **THEN** it acts as immovable geometry that dynamic bodies collide against

#### Scenario: Physics disabled
- **WHEN** the `physics-3d` feature flag is not enabled
- **THEN** the `PhysicsPlugin` is a no-op and rapier is not compiled

### Requirement: Collision Detection and Events
The system SHALL provide collision event reporting via a `CollisionEvents` resource.

The system SHALL support collision filtering via `CollisionGroups` component (membership + filter bitmasks).

#### Scenario: Collision callback
- **WHEN** two dynamic bodies overlap
- **THEN** a `CollisionEvent::Started(entity_a, entity_b)` is emitted

#### Scenario: Collision groups filtering
- **WHEN** two bodies have non-overlapping collision group filters
- **THEN** no collision is detected or reported between them

### Requirement: Physics Raycasting
The system SHALL provide `PhysicsWorld::raycast(origin, direction, max_distance)` returning the closest hit (entity, point, normal, distance).

#### Scenario: Raycast hit
- **WHEN** a ray is cast and intersects a collider
- **THEN** the hit entity, world-space point, surface normal, and distance are returned

#### Scenario: Raycast miss
- **WHEN** a ray is cast and intersects no colliders within max_distance
- **THEN** `None` is returned

### Requirement: Constraints and Joints
The system SHALL provide joint components: `FixedJoint`, `RevoluteJoint`, `PrismaticJoint`, `SphericalJoint`.

#### Scenario: Revolute joint
- **WHEN** two bodies are connected by a `RevoluteJoint`
- **THEN** they rotate freely around the joint axis but cannot translate apart
