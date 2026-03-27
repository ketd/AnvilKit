## ADDED Requirements

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
