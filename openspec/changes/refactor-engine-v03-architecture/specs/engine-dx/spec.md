## MODIFIED Requirements

### Requirement: Default Plugin Bundle
The system SHALL provide `DefaultPlugins` that registers all standard engine subsystems: `AnvilKitEcsPlugin`, `RenderPlugin`, `AudioPlugin`, `InputPlugin`, `CameraPlugin`, `AutoDeltaTimePlugin`.

Games using `DefaultPlugins` SHALL NOT need to manually insert `InputState`, `DeltaTime`, or register camera/audio systems.

The facade crate SHALL re-export all commonly-used types in its prelude, including audio components (`AudioSource`, `AudioListener`), persistence types (`SaveManager`, `WorldStorage`), and camera types.

#### Scenario: Complete game setup
- **WHEN** a game creates `App::new()` and adds `DefaultPlugins`
- **THEN** all standard resources (InputState, DeltaTime, ActiveCamera, SceneLights) are available without additional plugin registration

#### Scenario: Facade prelude sufficiency
- **WHEN** a game imports `use anvilkit::prelude::*`
- **THEN** all commonly-used engine types are available without importing individual sub-crates
