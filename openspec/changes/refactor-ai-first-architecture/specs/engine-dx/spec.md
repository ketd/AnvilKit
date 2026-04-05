## ADDED Requirements

### Requirement: Structured Engine Errors
All engine error types SHALL include structured fields: `code` (static identifier), `message` (human-readable), `hint` (agent-readable guidance), `suggested_fix` (optional code patch), and `context` (key-value metadata). Errors SHALL implement `serde::Serialize` for JSON output.

#### Scenario: Asset load failure with structured error
- **WHEN** an asset fails to load due to a missing file
- **THEN** the error includes `code: "ASSET_NOT_FOUND"`, `hint: "Check that the file path is relative to the assets/ directory"`, and `context: {"path": "textures/missing.png"}`

#### Scenario: Pipeline creation failure with fix hint
- **WHEN** a render pipeline fails to compile a shader
- **THEN** the error includes `hint: "Verify WGSL syntax at the reported line"` and `suggested_fix` containing the corrected shader snippet if determinable

### Requirement: Self-Describing API Types
All public engine Component and Resource types SHALL implement the `Describe` trait, providing machine-readable schema information (type name, field names/types/defaults/constraints, usage example). A `#[derive(Describe)]` macro SHALL be provided for automatic implementation.

#### Scenario: Agent discovers resource configuration
- **WHEN** an agent calls `BloomSettings::schema()`
- **THEN** it receives a `ComponentSchema` with fields like `{name: "threshold", type: "f32", default: "1.0", range: "0.0..5.0", description: "HDR brightness threshold for bloom extraction"}`

#### Scenario: Derive macro generates schema
- **WHEN** a type has `#[derive(Describe)]` with `#[describe(hint = "...")]` attributes
- **THEN** `schema()` returns all annotated metadata without manual implementation

## MODIFIED Requirements

### Requirement: Developer Experience — API Simplicity
The engine SHALL minimize API surface area for default builds. Advanced rendering features (SSAO, DoF, Motion Blur, Color Grading, IBL) SHALL be behind an `advanced-render` Cargo feature flag. The default feature set SHALL include: basic PBR, sprites, text rendering, debug lines, shadow mapping, and bloom.

#### Scenario: Default build excludes advanced effects
- **WHEN** a game depends on `anvilkit` without feature flags
- **THEN** SSAO, DoF, Motion Blur, Color Grading, and IBL modules are not compiled
- **AND** the public API does not expose their configuration types

#### Scenario: Opting into advanced effects
- **WHEN** a game adds `anvilkit = { features = ["advanced-render"] }`
- **THEN** all advanced post-processing effects are available

## REMOVED Requirements

### Requirement: CLI Tooling
**Reason**: `anvilkit-cli` template scaffolding is premature without external users. Suspended, not deleted.
**Migration**: None — tool is unused by games. Will resume in Phase 3.
