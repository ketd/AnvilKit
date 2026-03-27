## MODIFIED Requirements

### Requirement: Application Framework

The system SHALL provide an `App` container that manages the ECS `World`, system scheduling, and the main application loop.

`App` SHALL support adding plugins (`add_plugins`), inserting systems (`add_systems`), inserting resources (`insert_resource`, `init_resource`), and running the main loop (`run`) or single updates (`update`).

`App::add_plugins` SHALL check `Plugin::is_unique()` and skip registration if a unique plugin of the same type is already registered, logging a warning.

`App::update` SHALL log schedule execution errors via `log::error!()` instead of silently discarding them.

`App` SHALL own the `DeltaTime` resource definition (moved from `physics` module). The `physics` module SHALL re-export `DeltaTime` for backward compatibility.

The system SHALL provide `AppExit` as a resource to control graceful application shutdown.

#### Scenario: Basic application lifecycle
- **WHEN** `App::new()` is created, plugins and systems are added, and `run()` is called
- **THEN** the application executes startup systems once, then runs update systems in a loop until exit

#### Scenario: DeltaTime import compatibility
- **WHEN** existing code imports `anvilkit_ecs::physics::DeltaTime`
- **THEN** the import continues to work via re-export from the `app` module

## ADDED Requirements

### Requirement: Physics Module Organization
The system SHALL organize physics code into a module directory with clear separation: `physics/components.rs` (RigidBody, Collider, Velocity, ColliderShape), `physics/aabb.rs` (AabbCollider, AABB collision detection), `physics/rapier.rs` (Rapier3D integration, joint constraints), `physics/events.rs` (CollisionEvent via EventWriter).

The deprecated `CollisionEvents` resource SHALL be removed. The Rapier integration SHALL use `EventWriter<CollisionEvent>` instead.

#### Scenario: Rapier collision events via EventWriter
- **WHEN** two Rapier bodies collide during a physics step
- **THEN** a `CollisionEvent { a, b }` is sent via `EventWriter` and readable via `EventReader<CollisionEvent>` in game systems

### Requirement: State Transition Events
The `state_transition_system` SHALL emit `StateTransitionEvent<S>` when the game state changes, enabling systems to react to state transitions.

#### Scenario: State change notification
- **WHEN** `NextGameState` is set to `GameState::Playing` while current state is `GameState::Menu`
- **THEN** a `StateTransitionEvent { from: Menu, to: Playing }` is emitted

### Requirement: Dead Code Removal
The system SHALL remove the following deprecated/dead items:
- `SystemUtils::timed_system` (no-op pass-through)
- `SystemCombinator::chain` and `SystemCombinator::parallel` (no-op pass-throughs)
- `NetworkEvents` deprecated resource and `network_events_cleanup_system`
- `parent_child_sync_system` (defined but never registered)
- `PluginGroup<T>` (only used in tests)
- `MAX_DELTA_SECONDS` (defined but never referenced)
- `audio.rs` component stubs with no backing systems

#### Scenario: Clean public API
- **WHEN** a developer browses the `anvilkit-ecs` public API
- **THEN** no deprecated or non-functional items are visible
