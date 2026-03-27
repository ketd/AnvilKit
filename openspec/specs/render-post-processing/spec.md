# render-post-processing Specification

## Purpose
TBD - created by archiving change add-engine-v02-features. Update Purpose after archive.
## Requirements
### Requirement: Bloom Post-Processing
The system SHALL provide a configurable Bloom post-processing pass integrated into the HDR render pipeline.

The Bloom pipeline SHALL consist of: brightness threshold extraction → progressive downsample (4 mip levels) → Gaussian blur per level → progressive upsample with additive blending → composite with scene.

The system SHALL expose `BloomSettings` resource with configurable threshold (default 1.0), intensity (default 0.3), and mip count (default 4).

#### Scenario: Bloom applied to bright surfaces
- **WHEN** a fragment's HDR luminance exceeds the bloom threshold
- **THEN** it contributes to the bloom texture, creating a soft glow visible in the final composited image

#### Scenario: Bloom disabled
- **WHEN** `BloomSettings::enabled` is set to `false`
- **THEN** the bloom passes are skipped entirely with zero GPU cost

#### Scenario: Bloom intensity adjustment
- **WHEN** `BloomSettings::intensity` is changed at runtime
- **THEN** the bloom contribution scales proportionally without recompiling shaders

### Requirement: Screen-Space Ambient Occlusion (SSAO)
The system SHALL provide SSAO as an optional post-processing pass using depth and normal buffers.

The system SHALL implement hemisphere kernel sampling (16-64 samples) with a noise texture for sample rotation.

SSAO SHALL render at half resolution by default and use bilateral blur to upsample, preserving edges.

#### Scenario: SSAO enhances scene depth
- **WHEN** SSAO is enabled and the scene contains geometry with concavities
- **THEN** darkened contact shadows appear at corners, crevices, and surface intersections

#### Scenario: SSAO performance scaling
- **WHEN** `SsaoSettings::quality` is set to `Low` / `Medium` / `High`
- **THEN** the sample count adjusts (16 / 32 / 64) to trade quality for performance

#### Scenario: SSAO without normal buffer
- **WHEN** the render pipeline does not provide a normal G-buffer
- **THEN** SSAO reconstructs normals from the depth buffer using cross-product of screen-space derivatives

### Requirement: Advanced Post-Processing Pipeline
The system SHALL provide a composable post-processing stack where effects are applied in a configurable order.

The stack SHALL support at minimum: Bloom, SSAO, Tone Mapping (existing), and a slot for user-defined fullscreen passes.

#### Scenario: Post-processing order
- **WHEN** the frame is rendered
- **THEN** post-processing effects are applied in order: SSAO → Scene composite → Bloom → Tone mapping → Output

#### Scenario: Custom fullscreen pass
- **WHEN** a user registers a custom `PostProcessPass` with a shader and bind group
- **THEN** it is inserted into the post-processing stack at the specified position

