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
The system SHALL provide a `MeshData` struct containing `positions: Vec<Vec3>`, `normals: Vec<Vec3>`, `texcoords: Vec<Vec2>`, `tangents: Vec<Vec4>`, and `indices: Vec<u32>`.

`MeshData` SHALL provide `vertex_count()` and `index_count()` accessors.

`MeshData` SHALL validate on construction that `positions.len() == normals.len() == texcoords.len()`, returning an error if lengths are inconsistent.

#### Scenario: Mesh data integrity
- **WHEN** `MeshData` is constructed from a glTF file
- **THEN** `positions.len() == normals.len() == texcoords.len()` and all indices are within `0..vertex_count()`

#### Scenario: Inconsistent array lengths
- **WHEN** `MeshData` is constructed with `positions.len() != normals.len()`
- **THEN** the construction returns an error describing the length mismatch

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

### Requirement: Asset Memory Cache

The `AssetServer` SHALL maintain an in-memory cache of loaded assets, keyed by `AssetId`. Subsequent requests for the same asset path SHALL return the cached data without re-reading the file.

The cache SHALL support explicit invalidation via `AssetServer::reload(asset_id)`.

#### Scenario: Cache hit
- **WHEN** `load_async("models/tree.glb")` is called twice
- **THEN** the file is read from disk only once; the second call returns the cached handle immediately

#### Scenario: Cache invalidation
- **WHEN** `AssetServer::reload(tree_id)` is called
- **THEN** the cached data for `tree_id` is discarded and the file is re-read from disk

### Requirement: Hot Reload Integration

The `AssetServer` SHALL integrate with `FileWatcher` to automatically reload assets when their source files change on disk.

The integration SHALL maintain a reverse mapping from file path to `AssetId` for change notification routing.

Hot reload SHALL be enabled when the `hot-reload` feature flag is active.

#### Scenario: Texture hot reload
- **WHEN** a texture file `textures/wall.png` is modified on disk while the game is running
- **THEN** the AssetServer detects the change, reloads the texture, and the next frame renders with the updated texture

#### Scenario: Feature gate
- **WHEN** the `hot-reload` feature flag is not enabled
- **THEN** no file watcher is created and no hot-reload processing occurs

### Requirement: glTF Animation Extraction

The system SHALL provide `load_gltf_animations(path) -> Result<Vec<(Skeleton, Vec<AnimationClip>)>>` that extracts skeleton hierarchy and animation clips from glTF files.

The loader SHALL extract joint nodes, inverse bind matrices, and skin data into `Skeleton` structures.

The loader SHALL extract animation channels (translation, rotation, scale) with keyframes (Step, Linear, CubicSpline interpolation) into `AnimationClip` structures.

#### Scenario: Skinned character
- **WHEN** `load_gltf_animations("character.glb")` is called on a file with 1 skeleton and 3 animations (idle, walk, run)
- **THEN** a `Skeleton` with the joint hierarchy and 3 `AnimationClip` instances are returned

#### Scenario: No animations
- **WHEN** `load_gltf_animations("static_mesh.glb")` is called on a file with no animations
- **THEN** an empty vector is returned (no error)

### Requirement: Standalone Texture Loading

The system SHALL provide `load_texture(path) -> Result<TextureData>` that loads PNG and JPEG image files directly into `TextureData` without requiring a glTF container.

The loader SHALL convert all input formats to RGBA8 for GPU upload consistency.

#### Scenario: PNG texture loading
- **WHEN** `load_texture("textures/grass.png")` is called with a valid PNG file
- **THEN** a `TextureData` with correct width, height, and RGBA pixel data is returned

#### Scenario: JPEG texture loading
- **WHEN** `load_texture("textures/sky.jpg")` is called with a valid JPEG file
- **THEN** the RGB data is converted to RGBA (alpha = 255) and returned as `TextureData`

### Requirement: Automatic Asset Unloading

The system SHALL automatically remove assets from `AssetStorage` when all `AssetHandle<T>` references to an asset are dropped (reference count reaches zero).

The system SHALL provide a `process_unloads()` method (called alongside `process_completed()`) that checks for zero-reference assets and removes them.

#### Scenario: Handle drop cleanup
- **WHEN** all `AssetHandle<Mesh>` clones for a mesh are dropped
- **THEN** the mesh data is removed from `AssetStorage` on the next `process_unloads()` call

#### Scenario: Shared handle
- **WHEN** an `AssetHandle` is cloned to 3 systems and 2 drop their clones
- **THEN** the asset remains in storage because 1 handle still exists

### Requirement: Background Asset Parsing

The system SHALL perform asset parsing (glTF deserialization, PNG/JPEG decoding) in worker threads, not on the main thread.

The `load_async` pipeline SHALL: (1) dispatch file I/O to worker, (2) parse format in worker, (3) return parsed data via channel to main thread.

#### Scenario: glTF parsing off main thread
- **WHEN** `load_async("heavy_scene.glb")` is called for a 50MB glTF file
- **THEN** both file reading and glTF parsing happen in a worker thread; the main thread receives a ready-to-use `MeshData` + `MaterialData`

### Requirement: Skeletal Animation TRS Composition
The system SHALL compute bone matrices by accumulating Translation, Rotation, and Scale channels independently per joint, then composing them in standard TRS order: `T × R × S`.

When a joint has both Translation and Rotation channels animated, the system SHALL NOT left-multiply each channel's matrix onto the local transform sequentially. Instead, it SHALL collect the final T, R, S values and compose a single `Mat4::from_scale_rotation_translation(s, r, t)`.

#### Scenario: Joint with translation and rotation
- **WHEN** a joint is animated with Translation(2, 0, 0) and Rotation(45° around Y)
- **THEN** the resulting local transform is equivalent to `Mat4::from_translation(2,0,0) * Mat4::from_rotation_y(45°)`, i.e., translate first then rotate in world space

#### Scenario: Joint with all three channels
- **WHEN** a joint has Translation, Rotation, and Scale channels
- **THEN** the composed matrix equals `T × R × S` matching the glTF specification

### Requirement: Cubic Spline Interpolation
The system SHALL implement glTF cubic spline interpolation for animation keyframes, using the cubic Hermite spline formula with in-tangent and out-tangent data.

The interpolation formula SHALL be: `p(t) = (2t³ - 3t² + 1)v₀ + (t³ - 2t² + t)b₀ + (-2t³ + 3t²)v₁ + (t³ - t²)a₁`

where `v₀`, `v₁` are keyframe values, `b₀` is the out-tangent of keyframe 0 scaled by delta time, and `a₁` is the in-tangent of keyframe 1 scaled by delta time.

#### Scenario: Cubic spline vs linear
- **WHEN** an animation channel uses `CubicSpline` interpolation
- **THEN** the sampled value follows a smooth cubic curve between keyframes, not a linear segment

#### Scenario: Tangent influence
- **WHEN** an animation has keyframes with non-zero tangent values
- **THEN** the interpolated curve overshoots or undershoots keyframe values according to the tangent directions

### Requirement: Texture Format Handling
The system SHALL support converting the following glTF texture formats to RGBA8: R8G8B8A8, R8G8B8, R8G8, R8, R16G16B16A16.

For unsupported formats, the system SHALL return a descriptive error instead of silently returning `None`.

#### Scenario: R8G8B8 to RGBA8 conversion
- **WHEN** a glTF texture uses R8G8B8 format (3 channels, no alpha)
- **THEN** the system converts it to RGBA8 with alpha set to 255

#### Scenario: R16G16B16A16 conversion
- **WHEN** a glTF texture uses 16-bit per channel format
- **THEN** the system downsamples to 8-bit RGBA by dividing values by 257 (65535/255)

#### Scenario: Unsupported format error
- **WHEN** a glTF texture uses an unrecognized format
- **THEN** the system returns an error with the format name, not a silent `None`

