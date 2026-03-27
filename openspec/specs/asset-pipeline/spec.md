# asset-pipeline Specification

## Purpose
TBD - created by archiving change add-engine-v02-features. Update Purpose after archive.
## Requirements
### Requirement: Async Asset Loading
The system SHALL provide asynchronous asset loading via a background thread pool, preventing frame hitches when loading large assets.

`AssetServer::load_async(path)` SHALL return an `AssetHandle<T>` immediately, with `LoadState` transitioning from `Loading` → `Loaded` or `Failed`.

#### Scenario: Non-blocking load
- **WHEN** `asset_server.load_async("models/character.glb")` is called
- **THEN** the main thread continues rendering while the asset loads in the background

#### Scenario: Load completion callback
- **WHEN** an asset finishes loading
- **THEN** the `LoadState` resource for that handle transitions to `Loaded` and the asset is available for rendering

#### Scenario: Load failure
- **WHEN** an asset file does not exist or is corrupted
- **THEN** `LoadState` transitions to `Failed` with an error message, and a fallback placeholder is used

### Requirement: Asset Hot Reload
The system SHALL provide optional file watching (`notify` crate) that detects changes to asset source files and automatically reloads them.

Hot reload SHALL be enabled via `AssetServerConfig::hot_reload(true)` and disabled in release builds by default.

#### Scenario: Shader hot reload
- **WHEN** a `.wgsl` shader file is modified on disk while the game is running
- **THEN** the affected render pipeline is rebuilt with the new shader within 1 second

#### Scenario: Texture hot reload
- **WHEN** a `.png` texture file is modified on disk
- **THEN** the GPU texture is updated without restarting the game

#### Scenario: Hot reload disabled in release
- **WHEN** the game is compiled with `--release` and `hot_reload` is not explicitly enabled
- **THEN** no file watchers are created and the feature has zero overhead

### Requirement: Asset Dependencies and Deduplication
The system SHALL track asset dependencies (e.g., a glTF scene depends on textures) and deduplicate shared assets.

#### Scenario: Shared texture deduplication
- **WHEN** two meshes reference the same texture file
- **THEN** only one GPU texture is created, and both meshes share the same `AssetHandle`

#### Scenario: Cascading unload
- **WHEN** a scene asset is unloaded and its textures have no other references
- **THEN** the dependent textures are also unloaded from GPU memory

