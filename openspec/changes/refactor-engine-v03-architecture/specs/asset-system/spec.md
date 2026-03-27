## ADDED Requirements

### Requirement: Integrated Asset Pipeline
The system SHALL integrate `AssetServer`, `AssetCache`, and `DependencyGraph` into a unified pipeline where:
- On load, the content hash is computed and checked against `AssetCache` before performing I/O
- When a scene asset is loaded, its sub-asset dependencies (textures, materials) are registered in `DependencyGraph`
- On unload, `DependencyGraph::remove_and_cascade()` transitively unloads orphaned sub-assets
- `DependencyGraph` SHALL use `AssetId` (not bare `u64`) for type safety

#### Scenario: Cache hit avoids I/O
- **WHEN** an asset with content hash H was previously loaded and cached
- **THEN** `AssetServer::load()` returns the cached data without reading from disk

#### Scenario: Cascade unload
- **WHEN** a glTF scene is unloaded and its textures have no other referencing scenes
- **THEN** the textures are automatically unloaded via dependency cascade

### Requirement: Asset Type Registration
The system SHALL provide a method to register parsed asset values back into the server, connecting raw byte loading with typed storage.

`AssetServer` SHALL provide `insert_parsed<T>(id: AssetId, value: T)` that stores the parsed asset in the appropriate `AssetStorage<T>`.

#### Scenario: Parse and register
- **WHEN** raw bytes are loaded, parsed into `MeshData`, and registered via `insert_parsed`
- **THEN** `asset_storage.get(id)` returns the parsed `MeshData`
