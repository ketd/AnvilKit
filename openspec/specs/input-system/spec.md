# input-system Specification

## Purpose
TBD - created by archiving change add-engine-usability-overhaul. Update Purpose after archive.
## Requirements
### Requirement: Gamepad Support

The system SHALL provide `GamepadState` ECS resource tracking connected gamepads and their input state.

The system SHALL provide `GamepadButton` enum covering standard gamepad buttons (South/East/West/North, DPad, Shoulders, Triggers, Thumbsticks, Start, Select).

The system SHALL provide `GamepadAxis` enum covering analog axes (LeftStickX, LeftStickY, RightStickX, RightStickY, LeftTrigger, RightTrigger).

The system SHALL map winit gamepad events to `GamepadState` automatically via the `AutoInputPlugin`.

#### Scenario: Gamepad button press
- **WHEN** the user presses the South button (A on Xbox, Cross on PlayStation) on gamepad 0
- **THEN** `GamepadState::is_button_pressed(0, GamepadButton::South)` returns true

#### Scenario: Gamepad axis reading
- **WHEN** the user pushes the left stick to the right on gamepad 0
- **THEN** `GamepadState::axis_value(0, GamepadAxis::LeftStickX)` returns a value close to 1.0

#### Scenario: Gamepad connection
- **WHEN** a gamepad is connected while the game is running
- **THEN** `GamepadState::connected_gamepads()` includes the new gamepad's ID

### Requirement: Axis-Based Input

The system SHALL provide an `InputAxis` type representing a continuous-value input in the range `[-1.0, 1.0]` (for directional axes) or `[0.0, 1.0]` (for triggers).

The `ActionMap` SHALL support binding `InputAxis` values to named actions, in addition to existing binary key/button bindings.

#### Scenario: Analog movement
- **WHEN** an action "move_horizontal" is bound to `GamepadAxis::LeftStickX`
- **THEN** `ActionMap::axis_value("move_horizontal")` returns the stick's current position as a float

#### Scenario: Keyboard as axis
- **WHEN** an action "move_horizontal" is bound to `KeyCode::A` (negative) and `KeyCode::D` (positive)
- **THEN** pressing D returns 1.0, pressing A returns -1.0, pressing both returns 0.0, pressing neither returns 0.0

### Requirement: ActionMap Performance

The `ActionMap` SHALL use an interned `ActionId` (u32 index) instead of `String` for action lookup keys, eliminating per-frame heap allocation.

The system SHALL provide `ActionMap::register_action(name) -> ActionId` for creating action IDs and `ActionMap::axis_value(id)` / `ActionMap::is_active(id)` for querying by ID.

String-based lookup SHALL remain available as a convenience method that internally resolves to `ActionId`.

#### Scenario: Zero-allocation lookup
- **WHEN** `ActionMap::is_active(move_action_id)` is called with a pre-registered `ActionId`
- **THEN** the lookup is a direct array index with no string hashing or heap allocation

#### Scenario: String convenience
- **WHEN** `ActionMap::is_active_by_name("move_forward")` is called
- **THEN** the string is resolved to an `ActionId` via a HashMap lookup (one-time cost) and the result is returned

