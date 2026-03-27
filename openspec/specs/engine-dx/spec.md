# engine-dx Specification

## Purpose
TBD - created by archiving change add-engine-usability-overhaul. Update Purpose after archive.
## Requirements
### Requirement: Minimal Hello World

The system SHALL enable a fully functional 3D PBR rendering application in 30 lines of code or fewer, using `DefaultPlugins`, `StandardMaterial`, and `MeshHandle`.

The minimal example SHALL demonstrate: window creation, camera setup, mesh rendering with PBR material, directional lighting, and automatic event loop.

#### Scenario: Hello Cube
- **WHEN** the following code is compiled and run:
  ```rust
  fn main() {
      App::new()
          .add_plugins(DefaultPlugins)
          .add_systems(Startup, setup)
          .run();
  }
  fn setup(world: &mut World, assets: &RenderAssets) {
      // spawn camera
      // spawn light
      // spawn cube with StandardMaterial
  }
  ```
- **THEN** a window opens showing a lit, shaded 3D cube with PBR material

#### Scenario: No manual wgpu code
- **WHEN** the user creates a basic 3D scene using StandardMaterial + MeshHandle
- **THEN** no BindGroupLayout, BindGroup, RenderPipeline, or CommandEncoder code is required in user code

### Requirement: Example Deduplication

The system SHALL provide a shared `DemoApp` scaffold (or equivalent) that encapsulates the common boilerplate across demo examples: scene initialization, render loop, window resize handling, and frame capture.

Each demo example SHALL only contain the code unique to that demo (light configuration, camera animation, specific scene setup).

#### Scenario: Demo with shared scaffold
- **WHEN** a new demo is created using the shared scaffold
- **THEN** it requires fewer than 80 lines of demo-specific code (vs. the current ~500 lines)

#### Scenario: Frame capture support
- **WHEN** the demo scaffold is used with `--capture-dir` and `--capture-frames` CLI flags
- **THEN** frame capture works automatically without per-demo implementation

### Requirement: Unified Import Path

All examples and games SHALL import engine types through the `anvilkit` umbrella crate's `prelude` module, not through individual crate paths.

The `anvilkit::prelude` SHALL re-export all commonly used types from all sub-crates.

#### Scenario: Single import
- **WHEN** a user writes `use anvilkit::prelude::*;`
- **THEN** all core types (App, Plugin, Transform, StandardMaterial, MeshHandle, InputState, KeyCode, etc.) are available

#### Scenario: No deep imports
- **WHEN** a user needs `PbrVertex`, `RenderAssets`, or `SceneLights`
- **THEN** these are available via `anvilkit::render::prelude::*` (one level deep, not `anvilkit_render::renderer::assets::RenderAssets`)

### Requirement: Dead Dependency Cleanup

The workspace SHALL not include dependencies that have zero usage in any crate or example.

#### Scenario: rapier2d removal
- **WHEN** the workspace Cargo.toml is inspected
- **THEN** `rapier2d` is not listed (no 2D physics usage exists)

#### Scenario: kira removal
- **WHEN** the workspace Cargo.toml is inspected
- **THEN** `kira` is not listed (audio system uses rodio exclusively)

#### Scenario: egui as dev dependency
- **WHEN** `egui`, `egui-wgpu`, `egui-winit` are used
- **THEN** they are listed under `[dev-dependencies]` or behind a `dev` feature flag, not as default dependencies

