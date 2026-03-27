## ADDED Requirements

### Requirement: Camera Plugin
The system SHALL provide `CameraPlugin` that registers `camera_controller_system` in the `AnvilKitSchedule::Update` schedule.

`CameraPlugin` SHALL be included in `DefaultPlugins`.

Games SHALL NOT need to manually import and register the camera controller system.

#### Scenario: Automatic camera system
- **WHEN** `DefaultPlugins` is added to an app and an entity has `CameraController` + `Transform`
- **THEN** the camera controller system runs automatically each frame

### Requirement: Orbit Camera Mode
The system SHALL provide `CameraMode::Orbit { target: Vec3, distance: f32, min_distance: f32, max_distance: f32 }` for orbiting around a target point based on mouse drag.

The orbit camera SHALL support: mouse drag to rotate, scroll wheel to zoom, and configurable distance limits.

#### Scenario: Orbit rotation
- **WHEN** the user drags the mouse while in Orbit mode
- **THEN** the camera orbits around the target point, maintaining constant distance

#### Scenario: Orbit zoom
- **WHEN** the user scrolls the mouse wheel in Orbit mode
- **THEN** the camera distance from the target changes within the configured limits

### Requirement: Deprecated Code Removal
The deprecated `MouseDelta` struct SHALL be removed entirely, as `InputState::mouse_delta()` provides the same functionality.

#### Scenario: Clean API
- **WHEN** a developer browses `anvilkit-camera` public types
- **THEN** no deprecated types are visible
