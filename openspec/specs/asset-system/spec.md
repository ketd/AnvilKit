# asset-system Specification

## Purpose
Asset loading infrastructure for AnvilKit, providing glTF model import and CPU-side mesh data management.

**Crate**: `anvilkit-assets` | **Status**: Implemented (glTF mesh loading) | **Dependencies**: `anvilkit-core`, `gltf 1.4`, `glam`
## Requirements
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

### Requirement: Texture Data Extraction
The system SHALL provide a `TextureData` struct containing image width, height, and RGBA pixel data extracted from glTF files.

The system SHALL support both embedded glTF textures (base64/binary) and external image references.

#### Scenario: Embedded texture extraction
- **WHEN** a glTF file contains an embedded base color texture
- **THEN** `TextureData` is returned with correct dimensions and RGBA pixel data

#### Scenario: No texture available
- **WHEN** a glTF material has no base color texture
- **THEN** the material's `base_color_texture` field is `None` and `base_color_factor` provides a fallback color

### Requirement: Material Data Extraction
The system SHALL provide a `MaterialData` struct containing base color texture reference (optional), base color factor `[f32; 4]`, metallic factor `f32`, roughness factor `f32`, normal map texture (optional), normal scale `f32`, occlusion texture (optional), emissive texture (optional), and emissive factor `[f32; 3]`.

#### Scenario: Material with texture
- **WHEN** a glTF primitive has a material with a base color texture
- **THEN** `MaterialData` contains both the texture reference and the color factor

#### Scenario: Material without texture
- **WHEN** a glTF material has only a base color factor (no texture)
- **THEN** `MaterialData.base_color_texture` is `None` and `base_color_factor` contains the RGBA color

#### Scenario: PBR parameters
- **WHEN** a glTF material specifies metallic=0.8 and roughness=0.2
- **THEN** `MaterialData.metallic_factor` is 0.8 and `MaterialData.roughness_factor` is 0.2

### Requirement: HDR Environment Map Loading
The system SHALL provide loading of HDR equirectangular environment maps in Radiance HDR (.hdr) format.

The loader SHALL return floating-point RGB pixel data suitable for GPU texture creation.

#### Scenario: HDR file loading
- **WHEN** `load_hdr_environment("assets/env.hdr")` is called with a valid HDR file
- **THEN** floating-point RGB data with width, height, and pixel values potentially exceeding 1.0 are returned

### Requirement: Complete glTF Material Extraction
The system SHALL extract the full PBR material definition from glTF files including:
- metallic factor and roughness factor
- metallic-roughness texture (if present)
- normal map texture with scale
- occlusion texture
- emissive texture and factor

#### Scenario: Full material extraction
- **WHEN** a glTF file contains a material with metallicRoughness texture and normal map
- **THEN** `MaterialData` contains both textures and all scalar parameters

