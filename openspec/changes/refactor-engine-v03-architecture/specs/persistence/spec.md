## MODIFIED Requirements

### Requirement: Save and Load
The system SHALL provide `SaveManager` as an ECS `Resource` (when `bevy_ecs` feature is enabled) for managing save slots.

`SaveManager` SHALL derive `Resource` when the `bevy_ecs` feature is active, allowing it to be inserted directly into the ECS world.

`AutoSaveConfig` and `AutoSaveState` SHALL also derive `Resource` under the `bevy_ecs` feature.

The system SHALL provide `PersistencePlugin` that registers `SaveManager`, `AutoSaveConfig`, `AutoSaveState` as resources and adds an `auto_save_system` to the Update schedule.

#### Scenario: SaveManager as ECS resource
- **WHEN** `PersistencePlugin` is added to the app with a configured `SaveManager`
- **THEN** ECS systems can access `SaveManager` via `Res<SaveManager>` or `ResMut<SaveManager>`

#### Scenario: Auto-save via ECS system
- **WHEN** the auto-save interval elapses
- **THEN** the `auto_save_system` triggers a save to the rotating auto-save slot

## ADDED Requirements

### Requirement: Persistence Error Category
The error system SHALL provide an `AnvilKitError::Persistence` variant with optional `path` field, distinguishing persistence failures from generic errors.

Convenience constructors `AnvilKitError::persistence(msg)` and `AnvilKitError::persistence_with_path(msg, path)` SHALL be provided.

All persistence module functions SHALL use this error variant instead of `AnvilKitError::generic()`.

#### Scenario: Error category distinction
- **WHEN** a save file write fails
- **THEN** the returned error has `ErrorCategory::Persistence` and can be matched separately from IO or Generic errors

### Requirement: Settings Engine Integration
The system SHALL provide a `SettingsApplyPlugin` (or integration layer) that syncs `Settings.graphics` to `BloomSettings`/`SsaoSettings` and `Settings.audio` to `AudioBus` each frame.

#### Scenario: Graphics settings apply
- **WHEN** `Settings.graphics.bloom` is set to false
- **THEN** `BloomSettings.enabled` is set to false on the next frame

### Requirement: Scene Serialization Registry
`SerializableRegistry` SHALL be functional: `SceneSerializer::save` and `SceneSerializer::load` SHALL consult the registry to serialize/deserialize custom component types into the `custom_data` field.

The registry SHALL store serialize/deserialize function pointers (or trait objects) in addition to TypeId.

#### Scenario: Custom component serialization
- **WHEN** `Health` is registered via `app.register_serializable::<Health>("Health")` and a scene is saved
- **THEN** entities with `Health` components have their health data serialized into `custom_data["Health"]`
