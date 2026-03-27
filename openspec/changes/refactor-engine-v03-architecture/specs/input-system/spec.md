## ADDED Requirements

### Requirement: Input Plugin
The system SHALL provide `InputPlugin` that initializes `InputState` and `ActionMap` as ECS resources and registers `action_map_update_system` to update action states each frame.

Games SHALL NOT need to manually insert `InputState` — the plugin handles initialization.

#### Scenario: Plugin initialization
- **WHEN** `InputPlugin` is added to the app
- **THEN** `InputState` and `ActionMap` resources are available to all ECS systems

### Requirement: ActionMap Game Integration
The system SHALL provide a method to apply key binding overrides from `Settings.input.action_overrides` to `ActionMap`.

Games SHALL define their input actions via `ActionMap` instead of hardcoding `KeyCode` checks.

#### Scenario: Rebindable movement
- **WHEN** a game defines `action_map.bind("move_forward", KeyBinding::Key(KeyCode::W))` and the player rebinds it to KeyCode::Z via settings
- **THEN** `action_map.is_active("move_forward")` returns true when Z is pressed

### Requirement: Gamepad Event Forwarding
The system SHALL forward gamepad button and axis events to `GamepadState` at runtime, not just define the data structures.

#### Scenario: Gamepad axis input
- **WHEN** the left stick is pushed forward on a connected gamepad
- **THEN** `gamepad_state.axis(GamepadAxis::LeftStickY)` returns a positive value
