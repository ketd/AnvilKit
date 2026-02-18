## ADDED Requirements

### Requirement: glTF Mesh Loading
The system SHALL provide a `load_gltf_mesh()` function that loads the first mesh primitive from a glTF/GLB file and returns CPU-side mesh data.

The function SHALL extract vertex positions (required), normals (required), texture coordinates (optional, defaults to zero), and indices (required, as u32).

The function SHALL return `AnvilKitError::Asset` on failure with a descriptive message.

#### Scenario: Load valid GLB file
- **WHEN** `load_gltf_mesh("assets/suzanne.glb")` is called with a valid GLB file containing positions, normals, and indices
- **THEN** a `MeshData` is returned with correct vertex counts and index data

#### Scenario: Missing normals
- **WHEN** a glTF file lacks normal attributes
- **THEN** `AnvilKitError::Asset` is returned with a message indicating missing normals

#### Scenario: Missing texture coordinates
- **WHEN** a glTF file lacks texture coordinates
- **THEN** `MeshData.texcoords` is filled with `Vec2::ZERO` for each vertex

### Requirement: CPU-Side Mesh Data
The system SHALL provide a `MeshData` struct containing `positions: Vec<Vec3>`, `normals: Vec<Vec3>`, `texcoords: Vec<Vec2>`, and `indices: Vec<u32>`.

`MeshData` SHALL provide `vertex_count()` and `index_count()` accessors.

#### Scenario: Mesh data integrity
- **WHEN** `MeshData` is constructed from a glTF file
- **THEN** `positions.len() == normals.len() == texcoords.len()` and all indices are within `0..vertex_count()`
