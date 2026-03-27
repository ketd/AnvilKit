# render-advanced Specification

## Purpose
TBD - created by archiving change add-engine-v02-features. Update Purpose after archive.
## Requirements
### Requirement: Cascade Shadow Maps
The system SHALL replace the single-level directional shadow map with Cascade Shadow Maps (CSM) for distance-dependent shadow quality.

The system SHALL split the camera frustum into N cascades (default 3), each with its own shadow map rendered from the directional light's perspective.

The shader SHALL select the tightest cascade per fragment and blend between adjacent cascades at boundaries.

#### Scenario: Near shadows are sharp
- **WHEN** geometry is close to the camera (within cascade 0 range)
- **THEN** shadow edges are sharp with minimal aliasing

#### Scenario: Far shadows remain visible
- **WHEN** geometry is far from the camera (cascade 2+)
- **THEN** shadows are still rendered, with proportionally lower resolution

#### Scenario: Cascade transition
- **WHEN** a surface spans the boundary between two cascades
- **THEN** the shadow smoothly blends between cascades without visible seams

### Requirement: GPU Skeletal Animation
The system SHALL provide a runtime skeletal animation pipeline that transforms mesh vertices using joint matrices on the GPU.

The system SHALL compute joint matrices from `Skeleton` + `AnimationClip` data structures (defined in M11a) each frame and upload them as a uniform/storage buffer.

The vertex shader SHALL read joint indices and weights from vertex attributes and apply linear blend skinning.

#### Scenario: Animated character rendering
- **WHEN** an entity has `Skeleton`, `AnimationPlayer`, and `MeshHandle` components
- **THEN** the mesh deforms according to the current animation pose each frame

#### Scenario: Animation blending
- **WHEN** `AnimationPlayer` transitions between two clips
- **THEN** the joint transforms are linearly interpolated over the blend duration

#### Scenario: Animation clip loop
- **WHEN** `AnimationPlayer::looping` is `true` and playback reaches the end
- **THEN** the animation seamlessly wraps to the beginning

### Requirement: GPU Particle System
The system SHALL provide a GPU-accelerated particle system using compute shaders for particle simulation.

The system SHALL support emitter shapes: Point, Sphere, Cone, Box (matching existing `EmitShape` enum from M11c).

Particles SHALL be rendered as camera-facing billboards with alpha blending.

#### Scenario: Particle emission
- **WHEN** a `ParticleEmitter` component is active
- **THEN** particles are spawned at the configured rate and shape

#### Scenario: Particle lifecycle
- **WHEN** a particle's age exceeds its lifetime
- **THEN** it is recycled (returned to the free pool) without allocation

#### Scenario: 10K particle performance
- **WHEN** 10,000 particles are active simultaneously
- **THEN** the system maintains 60 FPS on mid-range hardware (GTX 1060 / M1 equivalent)

