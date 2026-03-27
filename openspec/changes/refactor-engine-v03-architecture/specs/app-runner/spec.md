## ADDED Requirements

### Requirement: Application Runner
The system SHALL provide `AnvilKitApp` as a complete game shell that manages the winit event loop, input forwarding, frame timing, ECS updates, and window lifecycle.

`AnvilKitApp` SHALL accept a `GameConfig` with window configuration, plugin list, and optional callbacks (init, post_update, render).

Games SHALL be able to start with `AnvilKitApp::run(config)` without implementing `ApplicationHandler` or manually managing input/timing/resize.

The runner SHALL execute the frame lifecycle in this order: input forwarding → DeltaTime update → `app.update()` (ECS systems) → post_update callback → render → `input.end_frame()`.

#### Scenario: Minimal game setup
- **WHEN** a game calls `AnvilKitApp::run(GameConfig::new().with_title("My Game").with_plugins(DefaultPlugins))`
- **THEN** a window opens, the event loop runs, and ECS systems are ticked each frame

#### Scenario: Custom post-update logic
- **WHEN** a game registers a `post_update` callback via `GameConfig`
- **THEN** the callback is invoked each frame after ECS update and before rendering

#### Scenario: Automatic resize handling
- **WHEN** the user resizes the window
- **THEN** the runner automatically reconfigures the render surface, depth texture, and HDR targets without game code intervention

### Requirement: Frame Lifecycle Management
The system SHALL provide `DeltaTime` as a core ECS resource updated by the app runner each frame, computed from `Instant::elapsed()` and clamped to `[0.001, 0.1]` seconds.

`DeltaTime` SHALL be defined in the ECS crate's `app` module (not in `physics`), with a re-export at the original `physics::DeltaTime` path for backward compatibility.

#### Scenario: DeltaTime accuracy
- **WHEN** a frame takes 16.67ms
- **THEN** `DeltaTime.0` is approximately 0.01667

#### Scenario: DeltaTime clamping
- **WHEN** a frame takes 500ms (e.g., due to system stall)
- **THEN** `DeltaTime.0` is clamped to 0.1 to prevent physics explosions

### Requirement: Input Forwarding
The system SHALL automatically forward winit keyboard, mouse, cursor, scroll, and device events to the ECS `InputState` resource without game code intervention.

The system SHALL call `InputState::end_frame()` at the end of each frame to reset "just pressed" / "just released" states.

#### Scenario: Keyboard input
- **WHEN** a key is pressed during the frame
- **THEN** `input.is_key_pressed(key)` returns true in systems running that frame

#### Scenario: Mouse delta
- **WHEN** `DeviceEvent::MouseMotion` is received
- **THEN** `input.mouse_delta()` reflects the accumulated movement for that frame
