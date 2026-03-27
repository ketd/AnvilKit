# render-system Specification

## Purpose
Cross-platform rendering infrastructure for AnvilKit, built on wgpu 0.19 and winit 0.30.

**Crate**: `anvilkit-render` | **Status**: Implemented and verified (22 unit tests + 83 doc tests, zero errors) | **Dependencies**: `anvilkit-core`, `anvilkit-ecs`, `wgpu 0.19`, `winit 0.30`, `bevy_ecs 0.14`, `pollster`
## Requirements
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

### Requirement: GPU Device Management
The system SHALL provide `RenderDevice` wrapping wgpu `Instance`, `Adapter`, `Device`, and `Queue`.

The system SHALL support automatic GPU adapter selection with fallback to software rendering.

#### Scenario: Device initialization
- **WHEN** `RenderDevice` is created
- **THEN** a compatible GPU adapter is selected and a device/queue pair is obtained

#### Scenario: No GPU available
- **WHEN** no compatible GPU adapter is found
- **THEN** the system returns an appropriate error with a clear message

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

### Requirement: Render Pipeline Builder
The system SHALL provide `RenderPipelineBuilder` with a fluent API for constructing wgpu render pipelines, including shader module creation from WGSL source.

The system SHALL provide `BasicRenderPipeline` wrapping a configured wgpu pipeline.

The builder SHALL support configuring vertex buffer layouts via `with_vertex_layouts()`.

The builder SHALL support configuring depth testing via `with_depth_format()` and bind group layouts via `with_bind_group_layouts()`.

#### Scenario: Pipeline creation
- **WHEN** a pipeline is built with vertex shader, fragment shader, and vertex format
- **THEN** a valid `RenderPipeline` is created and ready for use in render passes

#### Scenario: Pipeline with vertex layout
- **WHEN** `with_vertex_layouts(&[ColorVertex::layout()])` is called on the builder
- **THEN** the resulting pipeline accepts vertex buffers matching the specified layout

#### Scenario: Pipeline with depth and uniforms
- **WHEN** `with_depth_format(DEPTH_FORMAT)` and `with_bind_group_layouts(vec![layout])` are called
- **THEN** the resulting pipeline enables depth testing and accepts the specified bind group

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

### Requirement: Window Configuration Comprehensive Testing
窗口配置系统 SHALL 对所有 builder 方法和边界值进行测试验证。

#### Scenario: Builder 方法链完整性
- **WHEN** WindowConfig 的所有 builder 方法被依次调用
- **THEN** 最终配置反映所有设置值

#### Scenario: 无效窗口尺寸
- **WHEN** 窗口尺寸设为 0x0 或负值
- **THEN** 使用默认尺寸或返回错误

### Requirement: Render Device Initialization Testing
渲染设备初始化 SHALL 对创建流程和错误路径进行测试验证。

#### Scenario: 实例创建验证
- **WHEN** wgpu Instance 被创建
- **THEN** 支持的后端列表非空

#### Scenario: 格式选择逻辑
- **WHEN** 查询首选表面格式
- **THEN** 返回 sRGB 格式或平台默认格式

### Requirement: Render Pipeline Builder Testing
渲染管线构建器 SHALL 对 builder 模式和默认值进行测试验证。

#### Scenario: 默认管线配置
- **WHEN** 使用默认参数构建 RenderPipelineBuilder
- **THEN** 所有必填字段有合理默认值

#### Scenario: 自定义管线配置
- **WHEN** 通过 builder 设置自定义顶点格式和着色器
- **THEN** 构建的管线描述符反映自定义设置

### Requirement: Render Plugin ECS Integration Testing
RenderPlugin SHALL 对 ECS 集成流程进行测试验证。

#### Scenario: 插件注册系统
- **WHEN** RenderPlugin 被添加到 App
- **THEN** 必要的系统和资源被注册到对应的 Schedule

#### Scenario: 插件配置传递
- **WHEN** RenderPlugin 使用自定义 RenderConfig 创建
- **THEN** 配置值在插件 build 时被正确应用

### Requirement: Vertex Buffer Management
The system SHALL provide a `Vertex` trait for defining vertex data types with GPU-compatible memory layout.

The system SHALL provide a `ColorVertex` type with position (`[f32; 3]`) and color (`[f32; 3]`) attributes.

The system SHALL provide `create_vertex_buffer()` and `create_index_buffer()` functions for uploading geometry data to the GPU.

#### Scenario: Vertex buffer creation
- **WHEN** `create_vertex_buffer(device, &vertices)` is called with a slice of `ColorVertex` data
- **THEN** a wgpu `Buffer` is returned containing the vertex data in GPU memory

#### Scenario: Custom vertex type
- **WHEN** a type implements `Vertex` + `bytemuck::Pod` + `bytemuck::Zeroable`
- **THEN** it can be used with `create_vertex_buffer()` and provides its own `VertexBufferLayout`

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

### Requirement: Depth Testing
The system SHALL provide `DEPTH_FORMAT` constant and `create_depth_texture()` for depth buffer management.

The system SHALL support depth stencil attachment in render passes when a depth texture view is configured.

The depth texture SHALL be recreated automatically when the window is resized.

#### Scenario: Depth buffer creation
- **WHEN** `create_depth_texture(device, width, height, label)` is called
- **THEN** a `Depth32Float` texture and view are returned for use as depth attachment

#### Scenario: Face occlusion
- **WHEN** a 3D object's back faces are behind front faces from the camera's perspective
- **THEN** the depth test correctly occludes back faces

### Requirement: Indexed Drawing
The system SHALL support indexed draw calls via `draw_indexed()` when an index buffer is configured.

The ECS rendering path SHALL support both `Uint16` and `Uint32` index formats via `RenderAssets::upload_mesh()` and `RenderAssets::upload_mesh_u32()`.

#### Scenario: Cube with index buffer
- **WHEN** 24 vertices and 36 u16 indices are uploaded via `RenderAssets::upload_mesh()`
- **THEN** `draw_indexed(0..36, 0, 0..1)` renders 12 triangles forming a cube

### Requirement: Mesh Vertex Format
The system SHALL provide a `MeshVertex` type with position (`[f32; 3]`), normal (`[f32; 3]`), and texture coordinate (`[f32; 2]`) attributes, totaling 32 bytes stride.

`MeshVertex` SHALL implement the `Vertex` trait providing a `VertexBufferLayout` with three attributes at shader locations 0, 1, 2.

#### Scenario: MeshVertex layout
- **WHEN** `MeshVertex::layout()` is called
- **THEN** the returned layout has stride 32, with Float32x3 at offset 0 (location 0), Float32x3 at offset 12 (location 1), and Float32x2 at offset 24 (location 2)

### Requirement: u32 Index Buffer Support
The system SHALL provide `create_index_buffer_u32()` for creating index buffers with u32 indices.

The system SHALL support both `Uint16` and `Uint32` index formats, selectable via `RenderAssets::upload_mesh()` (u16) or `RenderAssets::upload_mesh_u32()` (u32).

#### Scenario: u32 indexed draw
- **WHEN** `RenderAssets::upload_mesh_u32()` is called with a u32 index buffer
- **THEN** the stored `GpuMesh` uses `IndexFormat::Uint32` for correct index interpretation

### Requirement: GPU Texture Creation
The system SHALL provide `create_texture()` for uploading RGBA image data to a GPU texture with an associated `TextureView`.

The system SHALL provide `create_sampler()` for creating texture samplers with configurable filtering modes.

#### Scenario: Texture upload
- **WHEN** `create_texture(device, queue, width, height, rgba_data, label)` is called
- **THEN** a wgpu `Texture` and `TextureView` are returned with the image data uploaded

#### Scenario: Sampler creation
- **WHEN** `create_sampler(device, label)` is called
- **THEN** a linear-filtering `Sampler` suitable for base color textures is returned

### Requirement: Multiple Bind Group Support
The system SHALL support binding multiple bind groups in a single render pass.

Group 0 SHALL be used for per-object scene uniform data (256-byte `PbrSceneUniform` containing model/view_proj/normal_matrix/camera/light/material). Group 1 SHALL be used for material data (textures and samplers).

#### Scenario: Two bind groups
- **WHEN** `set_bind_group(0, scene_group)` and `set_bind_group(1, material_group)` are called in the render pass
- **THEN** the PBR scene uniform is accessible via `@group(0) @binding(0)` and material textures via `@group(1)`

### Requirement: Blinn-Phong Lighting
The system SHALL demonstrate Blinn-Phong lighting capability through example shaders that combine ambient, diffuse (Lambert), and specular (Blinn-Phong) components.

The lighting model SHALL operate in world space using separated model/view/projection matrices and a proper normal matrix (inverse transpose of model matrix).

#### Scenario: Lit textured model
- **WHEN** a textured model is rendered with a directional light
- **THEN** the model shows ambient illumination on shadowed faces, diffuse shading proportional to surface-to-light angle, and specular highlights near the reflection direction

#### Scenario: Normal matrix correctness
- **WHEN** a model has non-uniform scaling
- **THEN** normals are correctly transformed using the inverse-transpose of the model matrix, preventing shading distortion

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

### Requirement: HDR Rendering Pipeline
The system SHALL support high dynamic range rendering through:
- Rgba16Float offscreen render target for scene rendering
- Full-screen post-processing pass for tone mapping
- ACES Filmic tone mapping operator
- Linear-space rendering with sRGB output conversion

The system SHALL support multi-pass rendering (scene pass → post-process pass) in `RenderApp`.

#### Scenario: HDR to LDR conversion
- **WHEN** a scene with bright specular highlights (values > 1.0) is rendered
- **THEN** the tone mapping compresses the dynamic range to [0,1] without clipping, preserving highlight detail

#### Scenario: Multi-pass rendering
- **WHEN** the scene pass renders to an offscreen HDR target
- **THEN** the post-process pass reads the HDR texture and outputs tone-mapped results to the swap chain

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

### Requirement: Normal Mapping
The system SHALL support tangent-space normal mapping through:
- Extended vertex format with tangent attribute `[f32; 4]` (xyz = tangent direction, w = bitangent sign)
- TBN (Tangent-Bitangent-Normal) matrix construction in vertex shader
- Normal map sampling and world-space normal perturbation in fragment shader

The system SHALL extract tangent data from glTF files or compute it using MikkTSpace algorithm.

#### Scenario: Surface detail
- **WHEN** a flat surface with a brick normal map is rendered
- **THEN** the surface appears to have 3D brick geometry despite being geometrically flat

#### Scenario: Tangent extraction
- **WHEN** a glTF file contains TANGENT attributes
- **THEN** they are loaded directly; otherwise tangents are computed from positions, normals, and UVs

### Requirement: Complete Material System
The system SHALL support the full glTF PBR material model including:
- Base color texture + factor
- Metallic-roughness texture + factors
- Normal map with configurable scale
- Ambient occlusion map
- Emissive texture + factor

The system SHALL support multiple simultaneous light sources (directional lights + point lights).

#### Scenario: Full material rendering
- **WHEN** a glTF model with all PBR textures (baseColor, metallicRoughness, normal, AO, emissive) is loaded
- **THEN** all texture channels contribute to the final rendered appearance

#### Scenario: Multiple lights
- **WHEN** a scene contains both directional and point lights
- **THEN** each light contributes independently to the final illumination using the PBR BRDF

### Requirement: Post-Processing Pipeline Integration

The system SHALL integrate all implemented post-processing effects (SSAO, DOF, Motion Blur, Color Grading) into the main `render_ecs()` rendering loop, controlled by the `PostProcessSettings` resource.

The post-processing chain SHALL execute in fixed order: SSAO → DOF → Motion Blur → Bloom → Color Grading → Tonemap.

Each effect SHALL be independently enable/disable via its corresponding `Option<Settings>` field.

#### Scenario: SSAO integration
- **WHEN** `PostProcessSettings.ssao` is `Some(SsaoSettings { quality: High, .. })`
- **THEN** the SSAO pass executes after the scene pass and the AO factor is applied during tonemapping

#### Scenario: Full pipeline
- **WHEN** all five post-processing effects are enabled
- **THEN** they execute in order (SSAO → DOF → Motion Blur → Bloom → Color Grading) before the final tonemap pass

### Requirement: Mipmap Generation

The system SHALL automatically generate mipmaps for textures created via `create_texture()` and `create_texture_linear()`.

Mipmap generation SHALL use a blit-chain approach (downscaling each mip level from the previous one using linear filtering).

The sampler SHALL use `FilterMode::Linear` for `mipmap_filter` when mipmaps are available.

#### Scenario: Texture with mipmaps
- **WHEN** a 1024x1024 texture is created
- **THEN** a full mip chain (1024 → 512 → 256 → ... → 1) is generated and the texture has `mip_level_count = floor(log2(max(w,h))) + 1`

#### Scenario: Oblique surface rendering
- **WHEN** a textured surface is viewed at a steep angle
- **THEN** the appropriate mip level is sampled, eliminating aliasing artifacts

### Requirement: CSM Camera FOV Correctness

The cascade shadow map system SHALL use the actual `CameraComponent.fov` value when computing cascade frustum splits, not a hardcoded value.

The CSM system SHALL use the same coordinate system handedness (left-handed) as the main camera projection.

#### Scenario: Wide FOV camera
- **WHEN** the camera FOV is set to 90 degrees
- **THEN** the CSM cascade frusta match the 90-degree camera frustum, producing correct shadow coverage

#### Scenario: Coordinate system consistency
- **WHEN** the main camera uses `perspective_lh` projection
- **THEN** the CSM cascade matrices also use left-handed projection, eliminating shadow coordinate mismatch

### Requirement: Point Light and Spot Light Shadows

The system SHALL support shadow mapping for point lights (using cubemap shadow maps) and spot lights (using 2D perspective shadow maps).

Point light shadows SHALL use a 6-face cubemap with one depth-only render pass per face.

Spot light shadows SHALL use a single 2D depth texture with a perspective projection matching the spot cone angle.

#### Scenario: Point light shadow
- **WHEN** a point light is positioned inside a room
- **THEN** objects cast shadows in all directions from the light source

#### Scenario: Spot light shadow
- **WHEN** a spot light with a 45-degree cone angle illuminates a scene
- **THEN** shadows are cast only within the light's cone of influence

### Requirement: Configurable Backface Culling

The system SHALL support per-material backface culling configuration via the material system.

The default cull mode SHALL be `Back` for opaque materials and `None` for double-sided materials (as indicated by glTF `doubleSided` property).

#### Scenario: Single-sided mesh
- **WHEN** a closed mesh (e.g., sphere) is rendered with default cull mode
- **THEN** back faces are culled, improving rendering performance

#### Scenario: Double-sided material
- **WHEN** a glTF material has `doubleSided: true`
- **THEN** `cull_mode: None` is used, rendering both sides of each triangle

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

