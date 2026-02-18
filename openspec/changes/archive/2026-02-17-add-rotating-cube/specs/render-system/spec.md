## ADDED Requirements

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

## MODIFIED Requirements

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
