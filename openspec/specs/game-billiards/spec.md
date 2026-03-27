# game-billiards Specification

## Purpose
TBD - created by archiving change refactor-engine-quality. Update Purpose after archive.
## Requirements
### Requirement: Ball State Consistency
The system SHALL maintain consistent ball tracking state throughout all game events (potting, scratches, resets).

When a cue ball scratch occurs (cue ball potted), the system SHALL:
1. Reset the cue ball position and velocity
2. Restore `BallTracker.on_table[0]` to `true`

#### Scenario: Scratch recovery
- **WHEN** the cue ball is potted (scratch event)
- **THEN** the cue ball is respawned at the head spot with zero velocity and `on_table[0]` is set to `true`

#### Scenario: Ball tracker accuracy after scratch
- **WHEN** a scratch occurs and gameplay continues
- **THEN** `BallTracker.on_table[0]` reflects the cue ball's actual presence on the table (`true`)

### Requirement: Correct MSAA Resolve
All intermediate render passes in the multi-pass rendering pipeline SHALL resolve the MSAA color attachment to the HDR target texture.

The system SHALL NOT skip MSAA resolve for intermediate passes, as this causes only the last draw command's fragments to survive.

#### Scenario: Multi-object scene rendering
- **WHEN** 16 balls and a table are rendered in the scene pass
- **THEN** all objects are visible in the final resolved HDR target, not just the last-drawn object

#### Scenario: MSAA resolve consistency
- **WHEN** every draw command's render pass completes
- **THEN** `resolve_target` is set to the HDR texture view and `store_op` is `StoreOp::Discard` for the MSAA attachment

### Requirement: Dead Code Removal
The game SHALL NOT contain unused component definitions, unused variables, or no-op code.

Specifically:
- `Cushion` and `Pocket` component structs SHALL be removed if cushion/pocket logic is implemented via `BilliardConfig` hardcoded geometry
- `let _ = entity;` no-op statements SHALL be removed

#### Scenario: Clean component definitions
- **WHEN** the billiards component module is reviewed
- **THEN** every defined component is spawned as an entity or used in at least one query

### Requirement: Resource Consistency
The game SHALL read configuration from the ECS `Resource<BilliardConfig>` that was inserted during setup, not create new `BilliardConfig::default()` instances.

#### Scenario: Config modification respected
- **WHEN** `BilliardConfig` is modified after insertion (e.g., changing table size)
- **THEN** scene initialization reads the modified config, not a fresh default

### Requirement: Frame-Rate Independent Physics
The billiards physics simulation (rolling friction, collision response) SHALL produce consistent results regardless of frame rate.

The friction decay model SHALL use a fixed-timestep or frame-rate-compensated formula rather than a per-frame exponential decay that varies with frame rate.

#### Scenario: Consistent ball deceleration
- **WHEN** a ball is struck with the same force at 30 FPS and 60 FPS
- **THEN** the ball travels the same total distance before stopping (within 5% tolerance)

