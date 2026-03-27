## ADDED Requirements

### Requirement: Frame Performance Profiler
The system SHALL provide a `Profiler` resource that tracks per-frame GPU and CPU timing.

The profiler SHALL measure: total frame time, ECS update time, render pass times (per-pass breakdown), draw call count, triangle count, GPU memory usage.

#### Scenario: Profiler overlay
- **WHEN** `ProfilerSettings::overlay_enabled` is `true`
- **THEN** a real-time performance overlay is rendered showing FPS, frame time graph, and per-pass GPU times

#### Scenario: Profiler data export
- **WHEN** `profiler.export_csv(path)` is called
- **THEN** frame timing data is written to a CSV file for offline analysis

### Requirement: Debug Rendering Modes
The system SHALL provide toggleable debug visualization modes beyond the existing `DebugMode` enum (M12a).

New modes SHALL include: `ColliderWireframe` (physics shapes), `NavMeshOverlay` (navigation mesh), `BoundingBoxes` (AABB wireframes), `LightVolumes` (light influence spheres).

#### Scenario: Collider wireframe
- **WHEN** `DebugMode::ColliderWireframe` is active
- **THEN** all physics colliders are rendered as colored wireframes on top of the scene

#### Scenario: Multiple debug modes
- **WHEN** multiple debug modes are enabled simultaneously (e.g., `Normals` + `BoundingBoxes`)
- **THEN** both visualizations are composited together

### Requirement: Console and Command System
The system SHALL provide an in-game debug console activated by a configurable key (default: backtick/tilde).

The console SHALL support registering named commands with argument parsing.

#### Scenario: Toggle physics debug
- **WHEN** the user types `debug physics` in the console
- **THEN** physics debug visualization is toggled

#### Scenario: Set time scale
- **WHEN** the user types `timescale 0.5`
- **THEN** the game's `DeltaTime` is scaled to 50% speed
