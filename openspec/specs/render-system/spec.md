# render-system Specification

## Purpose
Cross-platform rendering infrastructure for AnvilKit, built on wgpu 0.19 and winit 0.30.

**Crate**: `anvilkit-render` | **Status**: Implemented and verified (22 unit tests + 83 doc tests, zero errors) | **Dependencies**: `anvilkit-core`, `anvilkit-ecs`, `wgpu 0.19`, `winit 0.30`, `bevy_ecs 0.14`, `pollster`
## Requirements
### Requirement: Window Management
The system SHALL provide `WindowConfig` for configuring window properties (title, size, resizable, fullscreen) using a builder pattern.

The system SHALL provide `RenderApp` implementing winit's event handling for window lifecycle management (create, resize, close, input events).

#### Scenario: Window creation
- **WHEN** `WindowConfig::new().with_title("Game").with_size(1280, 720)` is used to create a window
- **THEN** a platform window with the specified title and dimensions is created

#### Scenario: Window resize handling
- **WHEN** the user resizes the window
- **THEN** the render surface is reconfigured to match the new dimensions

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

#### Scenario: Surface configuration
- **WHEN** a render surface is created for a window
- **THEN** a compatible texture format is selected and the surface is configured

#### Scenario: Frame acquisition
- **WHEN** a new frame is requested
- **THEN** a `SurfaceTexture` and `TextureView` are returned for rendering

### Requirement: Render Context
The system SHALL provide `RenderContext` as a unified high-level rendering interface combining device and surface management.

#### Scenario: Begin and end frame
- **WHEN** a frame render cycle begins
- **THEN** the context acquires a surface texture, creates a command encoder, and provides a render pass

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

The system SHALL provide rendering components:
- `RenderComponent` — marks an entity for rendering (visible flag, layer)
- `CameraComponent` — camera parameters (FOV, near/far planes, active flag)
- `MeshComponent` — mesh geometry reference (mesh_id, vertex/index counts)
- `MaterialComponent` — material properties (base color, metallic, roughness)

The system SHALL provide `RenderConfig` as a global resource and `RenderSystemSet` for system ordering.

#### Scenario: Plugin registration
- **WHEN** `app.add_plugins(RenderPlugin)` is called
- **THEN** rendering systems and resources are registered in the ECS world

#### Scenario: Render component query
- **WHEN** a system queries for `(Entity, &RenderComponent, &CameraComponent)`
- **THEN** only entities with both components are returned

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
The system SHALL support issuing draw calls within a render pass, including pipeline binding, vertex buffer binding, and draw commands.

#### Scenario: Triangle rendering
- **WHEN** a pipeline is bound, a vertex buffer with 3 vertices is set, and `draw(0..3, 0..1)` is called
- **THEN** a triangle is rendered to the current frame's texture

#### Scenario: Frame presentation
- **WHEN** a frame with draw commands is submitted and presented
- **THEN** the rendered content is visible in the window

### Requirement: Uniform Buffer Management
The system SHALL provide `create_uniform_buffer()` for creating GPU uniform buffers with `UNIFORM | COPY_DST` usage, supporting per-frame updates via `queue.write_buffer()`.

#### Scenario: MVP matrix uniform
- **WHEN** a 64-byte MVP matrix is written to a uniform buffer each frame
- **THEN** the GPU shader receives the updated transformation matrix

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

The system SHALL support both indexed and non-indexed rendering in the same `RenderApp` depending on configuration.

#### Scenario: Cube with index buffer
- **WHEN** 24 vertices and 36 indices are configured via `set_pipeline_3d()`
- **THEN** `draw_indexed(0..36, 0, 0..1)` renders 12 triangles forming a cube

### Requirement: Mesh Vertex Format
The system SHALL provide a `MeshVertex` type with position (`[f32; 3]`), normal (`[f32; 3]`), and texture coordinate (`[f32; 2]`) attributes, totaling 32 bytes stride.

`MeshVertex` SHALL implement the `Vertex` trait providing a `VertexBufferLayout` with three attributes at shader locations 0, 1, 2.

#### Scenario: MeshVertex layout
- **WHEN** `MeshVertex::layout()` is called
- **THEN** the returned layout has stride 32, with Float32x3 at offset 0 (location 0), Float32x3 at offset 12 (location 1), and Float32x2 at offset 24 (location 2)

### Requirement: u32 Index Buffer Support
The system SHALL provide `create_index_buffer_u32()` for creating index buffers with u32 indices.

The system SHALL support both `Uint16` and `Uint32` index formats in the render pass, selectable via `set_pipeline_3d_u32()`.

#### Scenario: u32 indexed draw
- **WHEN** `set_pipeline_3d_u32()` is called with a u32 index buffer and `draw_indexed` is issued
- **THEN** the render pass uses `IndexFormat::Uint32` for correct index interpretation

#### Scenario: Backward compatibility
- **WHEN** `set_pipeline_3d()` is called (without u32 suffix)
- **THEN** `IndexFormat::Uint16` is used, preserving existing behavior

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

Group 0 SHALL be used for per-frame uniforms (MVP matrix). Group 1 SHALL be used for material data (textures and samplers).

#### Scenario: Two bind groups
- **WHEN** `set_bind_group(0, mvp_group)` and `set_bind_group(1, material_group)` are called in the render pass
- **THEN** both groups are accessible in the shader via `@group(0)` and `@group(1)`

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
The system SHALL demonstrate physically-based rendering using the Cook-Torrance microfacet BRDF model with:
- GGX/Trowbridge-Reitz Normal Distribution Function (NDF)
- Schlick approximation for Fresnel reflectance
- Smith GGX geometric attenuation (height-correlated)

The BRDF SHALL use metallic-roughness workflow parameters from glTF materials.

#### Scenario: Metallic vs dielectric
- **WHEN** a metallic sphere (metallic=1.0, roughness=0.3) and a dielectric sphere (metallic=0.0, roughness=0.3) are rendered under the same light
- **THEN** the metallic sphere shows tinted reflections using base color as F0, while the dielectric shows white reflections with F0=0.04

#### Scenario: Roughness variation
- **WHEN** surfaces with roughness from 0.0 to 1.0 are rendered
- **THEN** roughness=0.0 produces sharp specular highlights and roughness=1.0 produces wide diffuse-like reflections

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

The ambient lighting term SHALL combine diffuse IBL (irradiance * albedo) and specular IBL (prefiltered env * BRDF LUT).

#### Scenario: Environment reflection
- **WHEN** a metallic sphere (metallic=1.0, roughness=0.0) is rendered with an HDR environment map
- **THEN** the sphere shows mirror-like reflections of the environment

#### Scenario: Diffuse environment lighting
- **WHEN** a dielectric sphere (metallic=0.0) is rendered with an HDR environment map
- **THEN** the sphere is lit by the environment's average color from all directions (irradiance)

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

