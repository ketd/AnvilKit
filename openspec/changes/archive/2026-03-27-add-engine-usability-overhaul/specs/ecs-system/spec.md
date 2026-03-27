## MODIFIED Requirements

### Requirement: Schedule System

The system SHALL provide `AnvilKitSchedule` enum with phases: `Startup`, `Main`, `PreUpdate`, `FixedUpdate`, `Update`, `PostUpdate`, `Cleanup`.

The `FixedUpdate` schedule SHALL run at a configurable fixed timestep (default 1/60 seconds) using a time accumulator. When the accumulated time exceeds the fixed step, the schedule runs one or more times to catch up.

The system SHALL provide `AnvilKitSystemSet` enum for grouping systems by concern: `Input`, `Time`, `Physics`, `GameLogic`, `Transform`, `Render`, `Audio`, `UI`, `Network`, `Debug`.

The system SHALL configure inter-set execution order: `Input` → `Time` → `Physics` → `GameLogic` → `Transform` → `Render` → `Audio` → `UI` → `Network` → `Debug`.

`AnvilKitSchedule` SHALL implement the `ScheduleLabel` trait from `bevy_ecs`.

The system SHALL provide `ScheduleBuilder` for constructing schedules with system sets.

#### Scenario: System ordering by schedule phase
- **WHEN** systems are added to `PreUpdate`, `Update`, and `PostUpdate`
- **THEN** they execute in that order each frame

#### Scenario: System set grouping
- **WHEN** systems are assigned to `AnvilKitSystemSet::Physics`
- **THEN** they can be collectively ordered relative to other system sets

#### Scenario: Fixed update physics
- **WHEN** a physics system is added to `FixedUpdate` and the frame takes 32ms
- **THEN** the physics system runs twice (2 × 16.67ms) to maintain 60Hz simulation

#### Scenario: System set ordering
- **WHEN** an Input system and a Physics system are registered in their respective sets
- **THEN** the Input system always executes before the Physics system within the same schedule phase

## ADDED Requirements

### Requirement: Event System

The system SHALL use Bevy's `Events<T>` system for all engine-level events, providing automatic double-buffering, per-system cursor tracking via `EventReader<T>`, and write access via `EventWriter<T>`.

The system SHALL register the following event types:
- `CollisionEvent` — physics collision notifications
- `NetworkEvent` — network state change notifications

Game code SHALL be able to register custom event types via `app.add_event::<T>()`.

#### Scenario: Collision event lifecycle
- **WHEN** the collision detection system detects a collision between entity A and entity B
- **THEN** it writes a `CollisionEvent` via `EventWriter<CollisionEvent>`, and any system with `EventReader<CollisionEvent>` can read it for up to 2 frames

#### Scenario: Multiple readers
- **WHEN** two independent systems both have `EventReader<CollisionEvent>`
- **THEN** each system sees all events independently (each has its own cursor)

#### Scenario: Custom game events
- **WHEN** `app.add_event::<PlayerDied>()` is called and a system writes `PlayerDied` events
- **THEN** other systems can read them via `EventReader<PlayerDied>`

### Requirement: Game State Machine

The system SHALL provide integration with Bevy's `States` system for managing game state transitions (e.g., Menu, Playing, Paused, GameOver).

The system SHALL support `OnEnter(state)`, `OnExit(state)`, and `OnTransition { from, to }` schedule hooks for state-specific system registration.

State transitions SHALL be requested via `NextState<S>` resource and applied during the `StateTransition` schedule point.

#### Scenario: State transition
- **WHEN** `next_state.set(GameState::Playing)` is called during the Menu state
- **THEN** `OnExit(GameState::Menu)` systems run, then `OnEnter(GameState::Playing)` systems run

#### Scenario: State-conditional systems
- **WHEN** a system is added with `.run_if(in_state(GameState::Playing))`
- **THEN** it only executes when the current state is `Playing`

#### Scenario: Pause/Resume
- **WHEN** the game transitions from Playing → Paused → Playing
- **THEN** OnExit(Playing) runs on pause, OnEnter(Playing) runs on resume

### Requirement: Scene Serialization Extended

The system SHALL support serializing arbitrary ECS components (not only Transform) through a component registration system.

The system SHALL provide `app.register_serializable::<T>()` for registering component types that participate in scene save/load.

Serialization SHALL preserve entity hierarchy (Parent/Children relationships).

#### Scenario: Full scene round-trip
- **WHEN** a scene with entities having Transform, Name, Tag, Visibility, and custom components is saved and loaded
- **THEN** all registered components are restored with their original values

#### Scenario: Hierarchy preservation
- **WHEN** a scene with parent-child relationships is serialized and deserialized
- **THEN** the Parent and Children components are correctly restored

### Requirement: Hierarchy Recursive Despawn

The system SHALL provide `TransformHierarchy::despawn_recursive(commands, entity)` that despawns an entity and all its descendants in the hierarchy.

#### Scenario: Despawn subtree
- **WHEN** `despawn_recursive(commands, parent)` is called on an entity with 3 children (one of which has 2 grandchildren)
- **THEN** the parent, 3 children, and 2 grandchildren (6 entities total) are all despawned

#### Scenario: Despawn leaf
- **WHEN** `despawn_recursive(commands, leaf)` is called on an entity with no children
- **THEN** only the leaf entity is despawned
