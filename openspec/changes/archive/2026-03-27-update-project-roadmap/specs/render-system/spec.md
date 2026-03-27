## REMOVED Requirements
### Requirement: Render Context
**Reason**: RenderContext was a monolithic rendering wrapper that duplicated RenderDevice + RenderSurface functionality. Removed in M6a as part of legacy cleanup — no callers existed.
**Migration**: Use RenderDevice and RenderSurface directly, or the ECS RenderState resource.

## MODIFIED Requirements

### Requirement: ECS Render Plugin
The system SHALL provide `RenderPlugin` implementing the ECS `Plugin` trait to register rendering systems and resources.

The system SHALL provide `CameraComponent` for camera parameters (FOV, near/far planes, active flag, aspect ratio).

The system SHALL register the following ECS resources on build:
- `ActiveCamera` — computed view-projection matrix and camera position
- `DrawCommandList` — per-frame draw commands extracted from entities
- `RenderAssets` — GPU-side mesh and material storage
- `SceneLights` — scene lighting configuration (directional light)

The system SHALL provide a `camera_system` that queries `(CameraComponent, Transform)` to compute `view_proj` and `camera_pos`, using `RenderState.surface_size` for aspect ratio when available.

The system SHALL provide a `render_extract_system` that queries `(MeshHandle, MaterialHandle, Transform, Option<MaterialParams>)` to populate `DrawCommandList` with per-object draw commands including metallic/roughness parameters.

#### Scenario: Plugin registration
- **WHEN** `app.add_plugins(RenderPlugin)` is called
- **THEN** rendering systems (camera_system, render_extract_system) and resources (ActiveCamera, DrawCommandList, RenderAssets, SceneLights) are registered in the ECS world

#### Scenario: Material params extraction
- **WHEN** an entity has `MeshHandle`, `MaterialHandle`, and `Transform` but no `MaterialParams`
- **THEN** `render_extract_system` uses default values (metallic=0.0, roughness=0.5)

### Requirement: Uniform Buffer Management
The system SHALL provide `create_uniform_buffer()` for creating GPU uniform buffers with `UNIFORM | COPY_DST` usage, supporting per-frame updates via `queue.write_buffer()`.

The ECS rendering path SHALL use a 256-byte `PbrSceneUniform` buffer containing model matrix, view-projection matrix, normal matrix, camera position, light direction, light color with intensity, and material parameters.

#### Scenario: PBR uniform update
- **WHEN** a 256-byte PbrSceneUniform is written to the uniform buffer each frame per object
- **THEN** the GPU shader receives the updated transformation, lighting, and material data

### Requirement: Indexed Drawing
The system SHALL support indexed draw calls via `draw_indexed()` when an index buffer is configured.

The ECS rendering path SHALL support both `Uint16` and `Uint32` index formats via `RenderAssets::upload_mesh()` and `RenderAssets::upload_mesh_u32()`.

#### Scenario: Cube with index buffer
- **WHEN** 24 vertices and 36 u16 indices are uploaded via `RenderAssets::upload_mesh()`
- **THEN** `draw_indexed(0..36, 0, 0..1)` renders 12 triangles forming a cube

### Requirement: u32 Index Buffer Support
The system SHALL provide `create_index_buffer_u32()` for creating index buffers with u32 indices.

The system SHALL support both `Uint16` and `Uint32` index formats, selectable via `RenderAssets::upload_mesh()` (u16) or `RenderAssets::upload_mesh_u32()` (u32).

#### Scenario: u32 indexed draw
- **WHEN** `RenderAssets::upload_mesh_u32()` is called with a u32 index buffer
- **THEN** the stored `GpuMesh` uses `IndexFormat::Uint32` for correct index interpretation

### Requirement: Multiple Bind Group Support
The system SHALL support binding multiple bind groups in a single render pass.

Group 0 SHALL be used for per-object scene uniform data (256-byte `PbrSceneUniform` containing model/view_proj/normal_matrix/camera/light/material). Group 1 SHALL be used for material data (textures and samplers).

#### Scenario: Two bind groups
- **WHEN** `set_bind_group(0, scene_group)` and `set_bind_group(1, material_group)` are called in the render pass
- **THEN** the PBR scene uniform is accessible via `@group(0) @binding(0)` and material textures via `@group(1)`

### Requirement: Cook-Torrance PBR BRDF
The system SHALL support physically-based rendering using the Cook-Torrance microfacet BRDF model with:
- GGX/Trowbridge-Reitz Normal Distribution Function (NDF)
- Schlick approximation for Fresnel reflectance
- Smith GGX geometric attenuation (height-correlated)

The BRDF SHALL use metallic-roughness workflow parameters delivered through the ECS pipeline:
- `MaterialParams` component on entities (metallic, roughness)
- `SceneLights` resource for directional light configuration
- `PbrSceneUniform` for GPU data transfer (camera_pos, light_dir, light_color, material_params)

#### Scenario: Metallic vs dielectric
- **WHEN** a metallic sphere (metallic=1.0, roughness=0.3) and a dielectric sphere (metallic=0.0, roughness=0.3) are rendered under the same light
- **THEN** the metallic sphere shows tinted reflections using base color as F0, while the dielectric shows white reflections with F0=0.04

#### Scenario: ECS-driven PBR rendering
- **WHEN** an entity is spawned with `(MeshHandle, MaterialHandle, MaterialParams, Transform)` and a `SceneLights` resource exists
- **THEN** the render pipeline automatically extracts material params and light data into `PbrSceneUniform` for GPU rendering

## ADDED Requirements

### Requirement: PBR Scene Uniform
The system SHALL provide `PbrSceneUniform` as a 256-byte `#[repr(C)]` struct containing:
- `model: [[f32; 4]; 4]` (64 bytes) — per-object model matrix
- `view_proj: [[f32; 4]; 4]` (64 bytes) — camera view-projection matrix
- `normal_matrix: [[f32; 4]; 4]` (64 bytes) — inverse-transpose of model matrix
- `camera_pos: [f32; 4]` (16 bytes) — camera world position
- `light_dir: [f32; 4]` (16 bytes) — directional light direction
- `light_color: [f32; 4]` (16 bytes) — light color RGB + intensity in W
- `material_params: [f32; 4]` (16 bytes) — metallic, roughness, reserved, reserved

The struct SHALL implement `bytemuck::Pod + bytemuck::Zeroable` and provide a `Default` implementation.

#### Scenario: Uniform size validation
- **WHEN** `std::mem::size_of::<PbrSceneUniform>()` is queried
- **THEN** the result is exactly 256 bytes

### Requirement: Scene Lighting Resource
The system SHALL provide `DirectionalLight` with direction (`Vec3`), color (`Vec3`), and intensity (`f32`) fields.

The system SHALL provide `SceneLights` as an ECS `Resource` holding a `DirectionalLight`, registered by `RenderPlugin` with sensible defaults (warm white light, direction [-0.5, -0.8, 0.3], intensity 5.0).

#### Scenario: Default scene lighting
- **WHEN** `SceneLights::default()` is created
- **THEN** a single directional light with non-zero intensity and normalized direction is provided

### Requirement: Material Parameters Component
The system SHALL provide `MaterialParams` as an ECS `Component` with `metallic: f32` and `roughness: f32` fields.

When an entity lacks `MaterialParams`, the render extract system SHALL use defaults (metallic=0.0, roughness=0.5).

#### Scenario: Entity with explicit material params
- **WHEN** an entity has `MaterialParams { metallic: 0.8, roughness: 0.2 }`
- **THEN** the draw command carries metallic=0.8 and roughness=0.2 to the GPU uniform

#### Scenario: Entity without material params
- **WHEN** an entity has `MeshHandle` and `MaterialHandle` but no `MaterialParams`
- **THEN** the draw command uses metallic=0.0 and roughness=0.5
