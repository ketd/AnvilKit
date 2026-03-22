## ADDED Requirements

### Requirement: Save Game Management
The system SHALL provide a `SaveManager` resource for managing multiple save slots with metadata.

Each save slot SHALL contain: slot index/name, timestamp, play time, game version, optional thumbnail (PNG bytes), and arbitrary game-specific metadata (`HashMap<String, serde_json::Value>`).

The system SHALL support operations: `save(slot, world)`, `load(slot) -> World`, `list_saves() -> Vec<SaveSlotInfo>`, `delete(slot)`.

Save files SHALL be stored in a platform-appropriate directory (`dirs::data_dir()` / AppName / saves /).

#### Scenario: Save to named slot
- **WHEN** `save_manager.save("quick", &world)` is called
- **THEN** the ECS world snapshot + metadata is written to `saves/quick.sav`

#### Scenario: List available saves
- **WHEN** `save_manager.list_saves()` is called
- **THEN** all save slots are returned with their metadata (timestamp, play time, game version) without loading the full world data

#### Scenario: Auto-save
- **WHEN** `SaveManagerConfig::auto_save_interval` is set to `Duration::from_secs(300)`
- **THEN** the world is automatically saved to the `_autosave` slot every 5 minutes

#### Scenario: Save version mismatch
- **WHEN** a save file was created with game version 0.1 and the current version is 0.2
- **THEN** the system attempts migration via registered `SaveMigration` callbacks, or returns an error if no migration path exists

### Requirement: Player Settings Persistence
The system SHALL provide a `Settings` resource backed by a configuration file (RON or TOML format).

Settings SHALL include typed sections: `GraphicsSettings`, `AudioSettings`, `InputSettings`, with game-extensible custom sections.

Settings SHALL be loaded at startup from `config/settings.ron` and saved when modified via `settings.save()`.

#### Scenario: Graphics settings persistence
- **WHEN** the player changes resolution to 1920x1080 and calls `settings.save()`
- **THEN** the settings file is updated and the new resolution is applied on next launch

#### Scenario: Key rebinding persistence
- **WHEN** the player rebinds "Jump" from Space to F
- **THEN** the `InputSettings::action_overrides` map is updated and persisted

#### Scenario: Default settings fallback
- **WHEN** no settings file exists (first launch)
- **THEN** `Settings::default()` is used and a settings file is created on first save

#### Scenario: Custom game settings
- **WHEN** a game registers a custom settings section via `settings.register_section::<MyGameSettings>()`
- **THEN** the section is serialized/deserialized alongside engine settings

### Requirement: Structured World Storage
The system SHALL provide a key-value storage backend for large, structured game data that doesn't fit in ECS scene snapshots (e.g., voxel chunks, procedural terrain, inventory databases).

The storage SHALL support `WorldStorage::open(path)` returning a handle with `get(key) -> Option<Vec<u8>>`, `put(key, &[u8])`, `delete(key)`, `keys_with_prefix(prefix) -> Vec<String>`.

The default backend SHALL use a single-file embedded database (SQLite via `rusqlite`, or a custom append-only format) for atomic writes and crash safety.

#### Scenario: Chunk data storage
- **WHEN** `storage.put("chunk/3/-2", &chunk_bytes)` is called
- **THEN** the chunk data is durably written and retrievable via `storage.get("chunk/3/-2")`

#### Scenario: Prefix enumeration
- **WHEN** `storage.keys_with_prefix("chunk/")` is called
- **THEN** all stored chunk keys are returned without loading their values

#### Scenario: Atomic batch write
- **WHEN** `storage.batch(|tx| { tx.put(...); tx.put(...); })` is called
- **THEN** either all writes succeed or none are applied (transactional)

#### Scenario: Crash safety
- **WHEN** the game crashes mid-write
- **THEN** the storage is in a consistent state on next launch (no partial writes)

### Requirement: Asset Cache
The system SHALL provide a persistent on-disk cache for compiled assets (shader bytecode, compressed textures, generated mipmaps).

Cache keys SHALL be derived from the source file content hash + pipeline configuration hash.

#### Scenario: Shader cache hit
- **WHEN** a WGSL shader is loaded and its content hash matches a cached entry
- **THEN** the pre-compiled pipeline is loaded from cache, skipping shader compilation

#### Scenario: Cache invalidation
- **WHEN** a source asset file is modified (content hash changes)
- **THEN** the cached entry is invalidated and the asset is recompiled

#### Scenario: Cache size management
- **WHEN** the cache exceeds `AssetCacheConfig::max_size_mb` (default 512 MB)
- **THEN** least-recently-used entries are evicted to bring the cache under the limit
