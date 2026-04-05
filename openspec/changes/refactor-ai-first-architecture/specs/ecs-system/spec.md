## REMOVED Requirements

### Requirement: Custom Application Framework
**Reason**: `anvilkit_ecs::App` reimplements 95% of `bevy_app::App` (add_plugins, add_systems, insert_resource, add_event, FixedUpdate accumulator, AppExit). Replaced by direct `bevy_app::App` dependency.
**Migration**: Replace `use anvilkit_ecs::app::App` with `use bevy_app::App`. Replace `AnvilKitSchedule::Update` with `bevy_app::Update`. Replace `anvilkit_ecs::app::DeltaTime` with `bevy_time::Time`.

### Requirement: Custom Schedule Labels
**Reason**: `AnvilKitSchedule` (Startup/Main/PreUpdate/FixedUpdate/Update/PostUpdate/Cleanup) duplicates Bevy's built-in schedule labels.
**Migration**: Use `bevy_app::{Startup, PreUpdate, Update, PostUpdate, FixedUpdate}` directly.

### Requirement: Custom AppExit
**Reason**: `anvilkit_ecs::app::AppExit` duplicates `bevy_app::AppExit`.
**Migration**: Use `bevy_app::AppExit`.

## MODIFIED Requirements

### Requirement: ECS Plugin System
The engine SHALL use `bevy_app::App` as the primary application container. Plugins SHALL implement `bevy_app::Plugin` instead of the custom `anvilkit_ecs::plugin::Plugin` trait. The `AnvilKitApp<G>` runner SHALL wrap `bevy_app::App` and provide the `GameCallbacks` lifecycle on top.

#### Scenario: Game initialization with bevy_app
- **WHEN** a game creates `bevy_app::App::new()` and adds `DefaultPlugins`
- **THEN** all engine systems are registered via Bevy's plugin mechanism
- **AND** the app is passed to `AnvilKitApp::run(config, app, game)` for the event loop

### Requirement: Serializable Component Registry
The engine SHALL provide a `register_serializable::<T>(name)` method as an extension trait on `bevy_app::App`. This is the only custom addition to the Bevy App API.

#### Scenario: Registering a serializable component
- **WHEN** `app.register_serializable::<Health>("Health")` is called
- **THEN** the type is added to `SerializableRegistry` for scene serialization
