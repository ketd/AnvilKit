## ADDED Requirements

### Requirement: Camera Controller System
The system SHALL provide `CameraController` component with configurable camera modes: `FreeCamera`, `FirstPerson`, and `ThirdPerson { distance, offset }`.

The system SHALL provide `camera_controller_system` that updates camera transform based on the active mode and input state.

The system SHALL respect the user-configured base FOV on `CameraComponent` and apply FOV adjustments (e.g., sprint zoom) as offsets from that base, not from a hardcoded value.

#### Scenario: Custom FOV preservation
- **WHEN** a camera is configured with `fov = 90.0` and the controller applies a +10 FOV offset
- **THEN** the resulting FOV is 100.0, not 80.0 (i.e., the base is read from the component, not hardcoded to 70)

#### Scenario: Free camera movement
- **WHEN** the camera mode is `FreeCamera` and WASD keys are pressed
- **THEN** the camera moves in the direction relative to its current orientation

#### Scenario: Third-person follow
- **WHEN** the camera mode is `ThirdPerson { distance: 5.0, offset: (0, 2, 0) }` and a target entity exists
- **THEN** the camera orbits at 5.0 units distance with a 2-unit vertical offset

### Requirement: Third-Person Look-At Correctness
The third-person camera look-at calculation SHALL use a right-handed coordinate system with the following axis derivation:
1. `forward = normalize(target - camera_position)`
2. `right = normalize(forward × world_up)`
3. `up = normalize(right × forward)`

The system SHALL handle the degenerate case where `forward` is parallel to `world_up` (looking straight up/down) by falling back to a stable alternative up vector.

#### Scenario: Standard look-at orientation
- **WHEN** the camera looks at a target horizontally (no pitch)
- **THEN** the camera's up vector aligns with world Y-up and the right vector is perpendicular to both forward and up

#### Scenario: Near-vertical look direction
- **WHEN** the camera looks nearly straight down (forward ≈ -Y)
- **THEN** the look-at calculation does not produce NaN or flipped orientation; a fallback up vector (e.g., Z) is used

### Requirement: Camera System Testing
The camera system SHALL have unit tests covering FOV calculations, mode switching, and look-at matrix correctness.

#### Scenario: FOV offset calculation
- **WHEN** base FOV is 60.0 and sprint offset is +15.0
- **THEN** `compute_fov()` returns 75.0

#### Scenario: Look-at matrix validity
- **WHEN** a look-at matrix is computed for any non-degenerate camera/target pair
- **THEN** the resulting matrix is orthonormal (determinant ≈ 1.0, columns are unit vectors)

#### Scenario: Mode transition
- **WHEN** the camera switches from `FirstPerson` to `ThirdPerson`
- **THEN** the camera smoothly interpolates to the new position over a configurable transition duration
