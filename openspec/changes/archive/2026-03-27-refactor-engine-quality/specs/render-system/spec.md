## MODIFIED Requirements

### Requirement: Render Surface Management
The system SHALL provide `RenderSurface` managing the wgpu swap chain surface, including format selection, configuration, and frame acquisition.

The system SHALL own an `Arc<Window>` clone to guarantee surface lifetime safety without unsafe code.

The system SHALL automatically reconfigure the surface when a `SurfaceError::Lost` or `SurfaceError::Outdated` error is encountered during frame acquisition.

#### Scenario: Surface configuration
- **WHEN** a render surface is created for a window
- **THEN** a compatible texture format is selected and the surface is configured

#### Scenario: Frame acquisition
- **WHEN** a new frame is requested
- **THEN** a `SurfaceTexture` and `TextureView` are returned for rendering

#### Scenario: Surface recovery after lost
- **WHEN** `get_current_texture()` returns `SurfaceError::Lost` or `SurfaceError::Outdated`
- **THEN** the surface is automatically reconfigured with current dimensions and frame acquisition is retried

#### Scenario: No unsafe lifetime transmutation
- **WHEN** `RenderSurface` is constructed
- **THEN** no `unsafe` code is used for lifetime management; the surface holds an `Arc<Window>` clone

### Requirement: Window Management
The system SHALL provide `WindowConfig` for configuring window properties (title, size, resizable, fullscreen, vsync) using a builder pattern.

The system SHALL provide `RenderApp` implementing winit's event handling for window lifecycle management (create, resize, close, input events).

The `vsync` configuration SHALL control the surface present mode: `true` selects `PresentMode::Fifo`, `false` prefers `PresentMode::Mailbox` with fallback to `Fifo`.

#### Scenario: Window creation
- **WHEN** `WindowConfig::new().with_title("Game").with_size(1280, 720)` is used to create a window
- **THEN** a platform window with the specified title and dimensions is created

#### Scenario: Window resize handling
- **WHEN** the user resizes the window
- **THEN** the render surface is reconfigured to match the new dimensions

#### Scenario: VSync configuration
- **WHEN** `WindowConfig` has `vsync: true`
- **THEN** the surface present mode is set to `PresentMode::Fifo` for vertical sync

### Requirement: Draw Command Execution
The system SHALL support batched draw call execution within a single render pass per pass type (shadow, scene, transparent).

All draw commands of the same pass type SHALL be issued within a single `wgpu::RenderPass`, using pipeline and bind group switching as needed, and submitted via a single `queue.submit()` per pass.

The shadow pass SHALL clear the depth buffer exactly once at the start (`LoadOp::Clear`), then use `LoadOp::Load` for subsequent draw calls within the same pass.

#### Scenario: Batched scene rendering
- **WHEN** 100 draw commands are queued for the scene pass
- **THEN** all 100 draw calls execute within a single render pass and a single command encoder submission

#### Scenario: Shadow pass depth clearing
- **WHEN** 20 shadow-casting objects are rendered
- **THEN** the shadow map depth buffer is cleared once at pass start, and all 20 objects contribute to the final shadow map

#### Scenario: Frame presentation
- **WHEN** a frame with draw commands is submitted and presented
- **THEN** the rendered content is visible in the window

### Requirement: Uniform Buffer Management
The system SHALL provide `DynamicUniformBuffer` for managing per-draw-call uniform data via a single GPU buffer with dynamic offsets.

The system SHALL pre-allocate capacity for a configurable maximum number of draw commands (default: 1024).

All per-draw uniform data (model matrix, normal matrix, material parameters) SHALL be written to the dynamic buffer once per frame, and each draw call SHALL reference its data via a dynamic offset.

The system SHALL fall back to multi-submit when draw commands exceed the pre-allocated capacity.

#### Scenario: Dynamic uniform buffer usage
- **WHEN** 50 draw commands are issued in a frame
- **THEN** 50 uniform data blocks are written contiguously to the dynamic buffer, and each draw call uses a corresponding offset

#### Scenario: Capacity overflow fallback
- **WHEN** draw commands exceed the pre-allocated capacity (e.g., >1024)
- **THEN** the system splits rendering into multiple submits, each within capacity

### Requirement: Image-Based Lighting (IBL)
The system SHALL support environment lighting through:
- HDR equirectangular environment map loading (.hdr format)
- Equirectangular to cubemap conversion (GPU-based)
- Diffuse irradiance map convolution
- Specular prefiltered environment map (split-sum approximation, multiple mip levels)
- BRDF integration LUT (2D lookup texture)

The BRDF LUT SHALL be pre-computed as a binary asset file and loaded at startup, rather than computed on the CPU at runtime.

The ambient lighting term SHALL combine diffuse IBL (irradiance * albedo) and specular IBL (prefiltered env * BRDF LUT).

#### Scenario: Environment reflection
- **WHEN** a metallic sphere (metallic=1.0, roughness=0.0) is rendered with an HDR environment map
- **THEN** the sphere shows mirror-like reflections of the environment

#### Scenario: Diffuse environment lighting
- **WHEN** a dielectric sphere (metallic=0.0) is rendered with an HDR environment map
- **THEN** the sphere is lit by the environment's average color from all directions (irradiance)

#### Scenario: BRDF LUT loading
- **WHEN** the render system initializes
- **THEN** the BRDF LUT is loaded from a pre-computed binary asset in under 1ms, not generated on the CPU

## ADDED Requirements

### Requirement: GPU Buffer Pool
The system SHALL provide a `BufferPool` for reusing GPU vertex/index buffers across frames instead of allocating new buffers every frame.

`BufferPool` SHALL provide `acquire(min_size: u64) -> wgpu::Buffer` that returns an existing buffer of sufficient size or creates a new one.

`BufferPool` SHALL provide `release(buffer: wgpu::Buffer)` to return a buffer for future reuse.

`BufferPool` SHALL enforce a maximum pool size (default: 64 buffers), discarding the smallest buffer when the limit is exceeded.

All subsystem renderers (sprite, particle, UI, line, text) SHALL use the buffer pool instead of per-frame allocation.

#### Scenario: Buffer reuse across frames
- **WHEN** a sprite renderer acquires a buffer in frame N and releases it, then acquires a buffer of equal or smaller size in frame N+1
- **THEN** the same GPU buffer is returned without allocation

#### Scenario: Pool size limit enforcement
- **WHEN** the pool contains 64 buffers and a new buffer is released
- **THEN** the smallest existing buffer is discarded to maintain the 64-buffer limit

### Requirement: GPU Resource Lifecycle
The system SHALL provide `remove_mesh(handle)`, `remove_material(handle)`, and `remove_pipeline(handle)` methods on `RenderAssets` for explicit GPU resource deallocation.

The system SHALL drop the underlying wgpu buffer/texture/pipeline when the last handle to a resource is removed.

#### Scenario: Mesh resource removal
- **WHEN** `render_assets.remove_mesh(handle)` is called
- **THEN** the GPU vertex and index buffers associated with that handle are released

#### Scenario: Dynamic content lifecycle
- **WHEN** a game loads a new level and unloads the previous one
- **THEN** GPU resources from the previous level can be explicitly freed via remove methods

### Requirement: PBR Shader Consistency
The system SHALL provide a single shared set of PBR BRDF functions (distribution_ggx, geometry_smith, fresnel_schlick) used by both standard and skinned PBR shaders.

The BRDF functions SHALL have identical parameter signatures, numerical guards (e.g., denominator epsilon), and clamping behavior across all shader variants.

Shadow map texel size SHALL be passed as a uniform parameter, not hardcoded in shader source.

#### Scenario: Skinned vs non-skinned visual parity
- **WHEN** a non-skinned and a skinned mesh with identical materials are rendered side-by-side
- **THEN** their shading output is visually identical (no BRDF formula differences)

#### Scenario: Shadow map resolution change
- **WHEN** the shadow map resolution is changed from 2048 to 4096
- **THEN** the PCF sampling correctly uses the updated texel size from the uniform, without shader recompilation

### Requirement: Shared Rendering Utilities
The system SHALL provide a public `pack_scene_lights()` function for converting ECS light components into GPU-ready light uniform arrays.

This function SHALL be the single source of truth for light packing, used by all examples, games, and the render plugin.

#### Scenario: Single source of truth
- **WHEN** a game or example needs to pack light data for the scene uniform
- **THEN** it calls `anvilkit_render::pack_scene_lights()` instead of implementing its own version

### Requirement: Render Pipeline Performance Metrics
The system SHALL track per-frame metrics including: number of encoder submissions, number of render passes, total draw calls, and buffer pool utilization.

#### Scenario: Batching verification
- **WHEN** a scene with 100 objects is rendered
- **THEN** `RenderStats` reports 2-4 encoder submissions (shadow + scene + transparent + tonemap), not 100+
