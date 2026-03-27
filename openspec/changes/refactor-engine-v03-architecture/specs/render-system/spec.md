## MODIFIED Requirements

### Requirement: Render Surface Management
The system SHALL provide `RenderSurface` managing the wgpu swap chain surface, including format selection, configuration, and frame acquisition.

The system SHALL own an `Arc<Window>` clone to guarantee surface lifetime safety without unsafe code.

The system SHALL automatically reconfigure the surface when a `SurfaceError::Lost` or `SurfaceError::Outdated` error is encountered during frame acquisition.

The system SHALL expose window size as a `WindowSize` ECS resource, automatically updated on resize events, for use by game systems (raycasting, UI layout, etc.).

#### Scenario: Surface configuration
- **WHEN** a render surface is created for a window
- **THEN** a compatible texture format is selected and the surface is configured

#### Scenario: WindowSize resource
- **WHEN** the window is resized to 1920x1080
- **THEN** the `WindowSize` resource reflects `(1920, 1080)` on the next frame

## ADDED Requirements

### Requirement: Shared Vertex Buffer Utility
The system SHALL provide a `CachedBuffer` utility struct that manages a grow-only GPU buffer, replacing the duplicated cached vertex buffer pattern across UiRenderer, TextRenderer, SpriteRenderer, LineRenderer, and ParticleRenderer.

`CachedBuffer` SHALL reallocate only when the required size exceeds the current capacity, and SHALL use `queue.write_buffer()` for data updates.

#### Scenario: Buffer reuse
- **WHEN** a renderer writes 1000 vertices (frame N) then 800 vertices (frame N+1)
- **THEN** the same buffer is reused without reallocation

#### Scenario: Buffer growth
- **WHEN** a renderer writes 2000 vertices but the buffer capacity is 1000
- **THEN** a new buffer of capacity >= 2000 is allocated and the old buffer is dropped

### Requirement: Shared Projection Uniform
The system SHALL provide a single `ProjectionUniform` struct (64 bytes, `[[f32; 4]; 4]`) used by all 2D renderers (sprite, text, UI, line, particle) instead of per-renderer duplicate definitions.

#### Scenario: Uniform type sharing
- **WHEN** UiRenderer and TextRenderer both need an orthographic projection uniform
- **THEN** both use the same `ProjectionUniform` type from the shared `renderer::common` module

### Requirement: Consolidated Debug Rendering
The system SHALL merge `debug.rs` (DebugMode, RenderStats, DebugOverlay) and `debug_renderer.rs` (DebugRenderer) into a single `debug` module.

`LineRenderer` SHALL be removed; all line rendering SHALL use `DebugRenderer` which already supports lines, boxes, spheres, and points.

Dead `DebugOverlay` flags (`show_wireframe`, `show_bounds`, `show_lights`, `show_skeleton`) and unimplemented `DebugMode` variants (Normals, Metallic, Roughness, etc.) SHALL be removed.

#### Scenario: Line rendering via DebugRenderer
- **WHEN** a game needs to draw 3D wireframe lines (crosshair, block highlight)
- **THEN** it uses `DebugRenderer::draw_line()` instead of the removed `LineRenderer`

### Requirement: Component Location Cleanup
The system SHALL move `Aabb` component and `raycast` functions (screen_to_ray, ray_plane_intersection, ray_sphere_intersection) from the render crate to `anvilkit-core::math`, as they are pure math utilities with no GPU dependency.

The render crate SHALL NOT initialize `InputState` or `DeltaTime` in `RenderPlugin::build()`. These resources SHALL be initialized by their respective plugins (InputPlugin, app runner).

#### Scenario: Aabb in core
- **WHEN** the physics system needs an Aabb for spatial queries
- **THEN** it imports `anvilkit_core::math::Aabb` without depending on the render crate

### Requirement: Render File Organization
The system SHALL split `window/events.rs` (1414 lines) into focused modules:
- `window/events.rs` — winit ApplicationHandler only
- `renderer/lighting.rs` — pack_lights, compute_cascade_matrices, compute_light_space_matrix
- `renderer/gpu_init.rs` — inject_render_state_to_ecs (pipeline/buffer/texture creation)
- `renderer/render_loop.rs` — render_ecs (shadow + scene + post-process orchestration)

#### Scenario: File responsibility
- **WHEN** a developer needs to modify the shadow pass
- **THEN** they edit `renderer/render_loop.rs`, not a 1414-line event handler file
