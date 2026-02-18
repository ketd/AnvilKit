## ADDED Requirements

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
