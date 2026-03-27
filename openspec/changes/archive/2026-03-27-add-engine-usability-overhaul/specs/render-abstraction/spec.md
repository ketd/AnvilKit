## ADDED Requirements

### Requirement: Standard Material

The system SHALL provide a `StandardMaterial` ECS component that encapsulates PBR material parameters (base_color, metallic, roughness, normal_scale, emissive, textures) and automatically creates the corresponding GPU pipeline and bind group on first use.

The system SHALL cache pipelines by (vertex_format, blend_mode, cull_mode) key and bind groups by (material_id, texture_set) key to avoid redundant GPU resource creation.

When `StandardMaterial` parameters change, the system SHALL mark the bind group as dirty and rebuild it on the next frame without recreating the pipeline.

#### Scenario: Automatic pipeline creation
- **WHEN** an entity with `StandardMaterial` and `MeshHandle` is rendered for the first time
- **THEN** the SceneRenderer automatically creates a PBR pipeline matching the material's configuration and caches it for reuse

#### Scenario: Material parameter update
- **WHEN** `StandardMaterial.roughness` is changed from 0.3 to 0.8 at runtime
- **THEN** the bind group is rebuilt on the next frame with the new roughness value, without pipeline recreation

#### Scenario: Texture-less material
- **WHEN** a `StandardMaterial` is created with only color factors (no textures)
- **THEN** the system uses 1x1 fallback textures (white for base color, default normal, etc.)

### Requirement: Mesh Handle Component

The system SHALL provide a `MeshHandle` ECS component that references a mesh uploaded to the GPU via `RenderAssets`.

Entities with both `MeshHandle` and `StandardMaterial` (and a `Transform`) SHALL be automatically collected into `DrawCommandList` by the render extract system.

#### Scenario: Automatic render extraction
- **WHEN** an entity has `MeshHandle`, `StandardMaterial`, and `Transform` components
- **THEN** it is automatically included in the draw command list each frame without user intervention

#### Scenario: Missing material
- **WHEN** an entity has `MeshHandle` and `Transform` but no `StandardMaterial`
- **THEN** the entity is skipped during render extraction with no error

### Requirement: Scene Renderer

The system SHALL provide a `SceneRenderer` that orchestrates the full multi-pass rendering pipeline:
1. Shadow pass (CSM cascades)
2. HDR scene pass (forward PBR)
3. Post-processing chain (SSAO → DOF → Motion Blur → Bloom → Color Grading)
4. Tonemap pass (HDR → sRGB swapchain)

The `SceneRenderer` SHALL automatically handle window resize by recreating all size-dependent GPU resources (depth textures, HDR targets, MSAA targets, post-processing textures, tonemap bind groups).

The `SceneRenderer` SHALL automatically manage the uniform buffer, writing all draw command uniforms in a single batch before rendering.

#### Scenario: Automatic resize
- **WHEN** the window is resized from 1280x720 to 1920x1080
- **THEN** all render targets, depth textures, and post-processing resources are automatically recreated at the new resolution

#### Scenario: Zero-boilerplate rendering
- **WHEN** the user spawns entities with MeshHandle + StandardMaterial + Transform and calls `app.run()`
- **THEN** the SceneRenderer automatically renders them with PBR lighting, shadows, and tone mapping

#### Scenario: Post-processing toggle
- **WHEN** `PostProcessSettings.ssao` is set to `Some(SsaoSettings::default())`
- **THEN** SSAO is integrated into the rendering pipeline on the next frame

### Requirement: Post-Process Settings

The system SHALL provide a `PostProcessSettings` ECS resource with optional settings for each effect:
- `ssao: Option<SsaoSettings>`
- `dof: Option<DofSettings>`
- `motion_blur: Option<MotionBlurSettings>`
- `bloom: Option<BloomSettings>`
- `color_grading: Option<ColorGradingSettings>`

Setting a field to `None` SHALL disable that effect. Setting it to `Some(settings)` SHALL enable it with the given parameters.

#### Scenario: All effects disabled
- **WHEN** `PostProcessSettings` has all fields set to `None`
- **THEN** the renderer skips all post-processing passes and goes directly to tonemap

#### Scenario: Selective effects
- **WHEN** only `bloom` and `ssao` are enabled
- **THEN** only SSAO and Bloom passes execute (in that order), other effects are skipped

### Requirement: Default Plugins

The system SHALL provide a `DefaultPlugins` plugin group that registers:
- `AnvilKitEcsPlugin` (ECS core + schedules + transform)
- `RenderPlugin` (GPU device + window + render systems)
- `AutoInputPlugin` (automatic winit → InputState forwarding)
- `AutoDeltaTimePlugin` (automatic frame timing → DeltaTime/Time update)
- `AudioPlugin` (audio engine initialization)

#### Scenario: Minimal application
- **WHEN** `App::new().add_plugins(DefaultPlugins).add_systems(Update, setup).run()` is called
- **THEN** a window opens with a running event loop, input handling, and audio ready

#### Scenario: Custom window config
- **WHEN** `DefaultPlugins` is created with `DefaultPlugins::new().with_window(WindowConfig::new().with_title("My Game"))`
- **THEN** the window uses the custom configuration

### Requirement: Auto Input Plugin

The system SHALL provide an `AutoInputPlugin` that automatically forwards winit keyboard and mouse events to the `InputState` ECS resource, eliminating manual event forwarding in user code.

The plugin SHALL call `InputState::end_frame()` at the start of each frame to clear per-frame state.

#### Scenario: Automatic key press
- **WHEN** the user presses the W key on the keyboard
- **THEN** `InputState::is_key_pressed(KeyCode::W)` returns true in the same frame's Update systems

#### Scenario: Mouse delta
- **WHEN** the user moves the mouse
- **THEN** `InputState::mouse_delta()` returns the accumulated movement since the last frame

### Requirement: Auto Delta Time Plugin

The system SHALL provide an `AutoDeltaTimePlugin` that automatically updates the `Time` and `DeltaTime` ECS resources each frame using `std::time::Instant`.

Delta time SHALL be clamped to `[0.001, 0.1]` seconds to prevent physics explosion on long frames.

#### Scenario: Frame timing
- **WHEN** 16.6ms passes between two frames
- **THEN** `DeltaTime.0` is approximately 0.0166 and `Time.delta_seconds()` returns the same value

#### Scenario: Large delta clamping
- **WHEN** the application is paused for 2 seconds (e.g., debugger breakpoint)
- **THEN** `DeltaTime.0` is clamped to 0.1 seconds, not 2.0

### Requirement: Mesh Data Convenience Methods

The system SHALL provide `MeshData::to_pbr_vertices() -> Vec<PbrVertex>` that converts mesh data (positions, normals, texcoords, tangents) into the GPU-ready `PbrVertex` format in a single call.

#### Scenario: glTF to GPU vertex conversion
- **WHEN** `scene.mesh.to_pbr_vertices()` is called on a loaded glTF mesh
- **THEN** a `Vec<PbrVertex>` is returned with correctly mapped position, normal, texcoord, and tangent fields
