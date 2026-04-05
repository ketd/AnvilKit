## MODIFIED Requirements

### Requirement: Motion Blur Post-Processing
The motion blur system SHALL store the previous frame's view-projection matrix and use it to compute per-pixel velocity vectors for directional blur.

The system SHALL handle the first frame gracefully by using the current frame's matrix as the previous frame (resulting in zero motion blur).

#### Scenario: Camera rotation blur
- **WHEN** motion blur is enabled and the camera rotates quickly
- **THEN** the rendered image shows directional streaking in the direction of camera movement

#### Scenario: First frame safety
- **WHEN** motion blur is enabled and it is the first frame of rendering
- **THEN** no blur is applied (prev_view_proj equals current view_proj)

### Requirement: Color Grading Post-Processing
The color grading system SHALL read from the HDR source texture and write to a separate intermediate texture, then copy the result back. The system SHALL NOT read and write the same texture in a single render pass.

#### Scenario: Exposure adjustment
- **WHEN** color grading is enabled with exposure = 1.5
- **THEN** the final image is 1.5x brighter without GPU validation errors

### Requirement: Depth of Field Post-Processing
The depth of field system SHALL execute all three passes: Circle of Confusion computation, disk blur at half resolution, and final composite blending focused and blurred regions.

#### Scenario: Near-focus blur
- **WHEN** DOF is enabled with focus_distance = 10.0
- **THEN** objects closer than the near focus plane appear blurred, objects at focus distance are sharp

## ADDED Requirements

### Requirement: Particle System ECS Integration
The engine SHALL provide `particle_emit_system` and `particle_update_system` ECS systems that automatically process `ParticleEmitter` components each frame.

The `ParticleRenderer` SHALL support depth testing (read-only) and texture atlas sampling for particle visuals.

Particles SHALL be sorted by camera distance for correct alpha blending.

#### Scenario: Automatic particle emission
- **WHEN** an entity with `ParticleEmitter { emit_rate: 10.0, .. }` exists
- **THEN** approximately 10 particles are emitted per second without manual API calls

#### Scenario: Particle depth ordering
- **WHEN** particles exist at different distances from the camera
- **THEN** they are rendered back-to-front with correct alpha blending against the scene

### Requirement: Sprite System ECS Integration
The engine SHALL provide a `sprite_collect_system` that queries all `Sprite` + `Transform` entities and builds a `SpriteBatch` for rendering.

#### Scenario: Automatic sprite rendering
- **WHEN** entities with `Sprite` and `Transform` components exist
- **THEN** they are automatically collected and rendered each frame without manual batching
