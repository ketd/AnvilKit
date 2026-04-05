## MODIFIED Requirements

### Requirement: Post-Processing Pipeline
The post-processing pipeline SHALL be split into two tiers:
- **Default tier** (always compiled): Bloom, Tonemapping
- **Advanced tier** (behind `advanced-render` feature): SSAO, Depth of Field, Motion Blur, Color Grading, IBL

The advanced tier modules SHALL be conditionally compiled via `#[cfg(feature = "advanced-render")]`. Settings types for advanced effects SHALL only be available when the feature is enabled.

#### Scenario: Default build post-processing
- **WHEN** a game uses the default feature set
- **THEN** Bloom and Tonemapping are available
- **AND** `BloomSettings` is a public type
- **AND** SSAO/DoF/MotionBlur/ColorGrading settings types do not exist in the public API

#### Scenario: Advanced build post-processing
- **WHEN** a game enables `advanced-render` feature
- **THEN** all post-processing effects are available
- **AND** their settings types are public and configurable
