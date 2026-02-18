## ADDED Requirements

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

## MODIFIED Requirements

### Requirement: Render Pipeline Builder
The system SHALL provide `RenderPipelineBuilder` with a fluent API for constructing wgpu render pipelines, including shader module creation from WGSL source.

The system SHALL provide `BasicRenderPipeline` wrapping a configured wgpu pipeline.

The builder SHALL support configuring vertex buffer layouts via `with_vertex_layouts()`.

#### Scenario: Pipeline creation
- **WHEN** a pipeline is built with vertex shader, fragment shader, and vertex format
- **THEN** a valid `RenderPipeline` is created and ready for use in render passes

#### Scenario: Pipeline with vertex layout
- **WHEN** `with_vertex_layouts(&[ColorVertex::layout()])` is called on the builder
- **THEN** the resulting pipeline accepts vertex buffers matching the specified layout
