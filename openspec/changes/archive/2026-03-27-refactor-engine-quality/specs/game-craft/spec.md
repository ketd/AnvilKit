## ADDED Requirements

### Requirement: Frame-Rate Independent Game Logic
The game SHALL update `DeltaTime` resource from actual wall-clock elapsed time (`Instant::elapsed()`) every frame, not use a hardcoded value.

`DeltaTime` SHALL be clamped to the range `[0.001, 0.1]` seconds to prevent physics instability from frame spikes or debugger pauses.

All movement, physics, and time-dependent systems (day/night cycle, animations) SHALL multiply their rates by `DeltaTime` to achieve frame-rate independence.

#### Scenario: 60 FPS gameplay
- **WHEN** the game runs at 60 FPS
- **THEN** `DeltaTime` is approximately 0.0167 seconds and player movement speed matches the intended 5.0 units/second

#### Scenario: 144 FPS gameplay
- **WHEN** the game runs at 144 FPS
- **THEN** player movement speed is the same as at 60 FPS (5.0 units/second), not 2.4x faster

#### Scenario: Frame spike clamping
- **WHEN** a frame takes 500ms (e.g., due to disk I/O)
- **THEN** `DeltaTime` is clamped to 0.1 seconds, preventing the player from teleporting through walls

### Requirement: Seamless Async Chunk Loading
The system SHALL mark newly inserted chunks and their four cardinal neighbors as dirty upon insertion into the world, triggering re-meshing with correct neighbor data.

Async-generated chunks SHALL be re-meshed on the main thread with actual neighbor data after insertion, not meshed on the worker thread with `[None; 4]` neighbors.

#### Scenario: New chunk boundary faces
- **WHEN** a new chunk is inserted adjacent to an existing chunk
- **THEN** both chunks are re-meshed, and the shared boundary faces are correctly culled (hidden internal faces are not rendered)

#### Scenario: New chunk AO continuity
- **WHEN** a new chunk is inserted next to existing terrain
- **THEN** ambient occlusion at the chunk boundary matches the neighboring chunk's AO, with no visible seam

#### Scenario: Chunk insertion performance
- **WHEN** a new chunk is inserted during gameplay
- **THEN** the chunk and up to 4 neighbors are re-meshed within the next 2 frames, not causing visible pop-in delay

### Requirement: Non-Blocking Initial World Generation
The system SHALL generate initial chunks asynchronously using the existing worker thread pipeline, displaying a loading screen until the minimum playable area is ready.

The system SHALL NOT block the main thread for multi-second synchronous chunk generation at startup.

#### Scenario: Startup experience
- **WHEN** the game launches
- **THEN** a loading screen is displayed while chunks are generated in the background, and gameplay begins once the initial area (load_radius chunks) is ready

#### Scenario: Progressive loading
- **WHEN** the initial load is in progress
- **THEN** chunks closest to the spawn point are prioritized and become playable first

### Requirement: Centralized World Seed
The world seed SHALL be stored as a single ECS `Resource` and read from that resource by all systems that need it (world generation, persistence, async workers).

No system SHALL contain a hardcoded seed value.

#### Scenario: Seed consistency
- **WHEN** a world is generated with seed 42 and saved to disk
- **THEN** loading the save file restores the same seed, and newly generated chunks match the original terrain

#### Scenario: Custom seed
- **WHEN** the user configures a custom seed via game settings
- **THEN** all chunk generation uses the configured seed from the resource

### Requirement: Greedy Mesh Optimization
The greedy meshing system SHALL allocate the face mask buffer once per chunk mesh operation and clear/reuse it for each depth slice, instead of allocating a new buffer per slice.

#### Scenario: Mesh memory efficiency
- **WHEN** a chunk is meshed
- **THEN** the total heap allocation for mask buffers is O(1) per chunk (one allocation), not O(depth_slices) allocations

### Requirement: Cross-Chunk Diagonal AO
The system SHALL correctly resolve block lookups at diagonal chunk corners (e.g., x < 0 AND z < 0) by checking both neighbor dimensions and falling back to the diagonal neighbor chunk if available.

#### Scenario: Corner AO accuracy
- **WHEN** AO is calculated for a block at position (0, y, 0) in a chunk
- **THEN** the diagonal neighbor at (-1, y, -1) is resolved from the correct neighboring chunk, not defaulted to Air

### Requirement: Robust Asset Loading
Texture and asset loading SHALL use `Result` return types instead of `.expect()` panics.

The game SHALL display an error message and gracefully degrade (e.g., use a fallback checkerboard texture) when an asset file is missing.

#### Scenario: Missing texture file
- **WHEN** the texture atlas file is not found at the expected path
- **THEN** the game logs an error and uses a procedurally generated fallback texture instead of crashing

### Requirement: Tonemap Shader Correctness
The tonemap shader SHALL not apply manual gamma correction (`pow(c, 1/2.2)`) when the swapchain surface format is sRGB, as the GPU performs the conversion automatically.

The shader SHALL query or receive the surface format and conditionally apply gamma correction only for linear surface formats.

Underwater UV distortion SHALL clamp coordinates to `[0, 1]` to prevent sampling outside the texture bounds.

#### Scenario: sRGB swapchain
- **WHEN** the swapchain format is `Bgra8UnormSrgb`
- **THEN** the tonemap shader outputs linear color values without manual gamma, relying on hardware sRGB conversion

#### Scenario: Linear swapchain
- **WHEN** the swapchain format is `Bgra8Unorm` (linear)
- **THEN** the tonemap shader applies manual `pow(c, 1/2.2)` gamma correction

#### Scenario: Underwater UV clamping
- **WHEN** the underwater distortion effect shifts UV coordinates
- **THEN** the distorted coordinates are clamped to `[0.0, 1.0]` before texture sampling
