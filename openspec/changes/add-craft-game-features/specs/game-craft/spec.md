## ADDED Requirements

### Requirement: Biome-Driven World Generation
The world generation system SHALL use temperature and humidity noise maps to select biomes, and each biome SHALL define surface blocks, fill blocks, vegetation probability, and tree species.

The system SHALL support at least 6 biome types: Plains, Forest, Desert, Tundra, Ocean, and Mountains.

Biome transitions SHALL be smooth, with height and block type interpolation at boundaries.

#### Scenario: Desert biome generation
- **WHEN** the temperature noise is high and humidity noise is low at a given XZ coordinate
- **THEN** the terrain uses Sand surface blocks, Sandstone fill blocks, and spawns Cactus plants instead of grass/flowers

#### Scenario: Biome boundary smoothing
- **WHEN** a column is at the boundary between Forest and Desert biomes
- **THEN** the terrain height and surface block type transition gradually over 8-16 blocks, not abruptly

### Requirement: Ore Generation
The world generation system SHALL place ore veins within stone layers according to configurable Y-range, vein size, and frequency parameters defined in a data table.

The system SHALL support at least 6 ore types: Coal, Iron, Gold, Diamond, Redstone, and Lapis.

#### Scenario: Diamond ore depth restriction
- **WHEN** the world generator places Diamond ore
- **THEN** Diamond ore only appears at Y levels 1-16, not above

#### Scenario: Ore vein clustering
- **WHEN** an ore vein is generated
- **THEN** it forms a connected cluster of 2-8 ore blocks (depending on ore type), not scattered individual blocks

### Requirement: Block Lighting System
The system SHALL store per-block light values as two 4-bit channels: sky light (0-15) and block light (0-15).

Sky light SHALL propagate downward from Y=255 and attenuate horizontally using BFS.

Block light SHALL propagate from light-emitting blocks (e.g., Torch=14, Glowstone=15) using BFS with 1-level attenuation per step.

The renderer SHALL use `max(sky_light * day_factor, block_light) / 15.0` as the light multiplier per vertex.

#### Scenario: Torch placement underground
- **WHEN** the player places a Torch block at position (10, 30, 10) underground
- **THEN** the surrounding area within 14 blocks is illuminated with decreasing brightness, and the torch position has block light level 14

#### Scenario: Sky light obstruction
- **WHEN** an opaque block is placed above a previously lit area
- **THEN** the sky light below that block is recalculated and reduced appropriately

#### Scenario: Light level update budget
- **WHEN** multiple blocks are placed or broken in a single frame
- **THEN** at most 1024 light level updates are processed per frame to maintain performance

### Requirement: Entity and Mob System
The system SHALL support spawning, updating, and despawning mob entities with configurable properties defined in a data table.

Passive mobs SHALL use an Idle/Wander FSM behavior pattern. Hostile mobs SHALL use an Idle/Detect/Chase/Attack FSM pattern.

Mob spawning SHALL be governed by rules based on biome, light level, time of day, and global entity cap.

#### Scenario: Zombie spawn at night
- **WHEN** it is nighttime and a surface block has light level <= 7
- **THEN** a Zombie may spawn on that block if the global mob count is below 128

#### Scenario: Passive mob behavior
- **WHEN** a Pig entity is alive and not attacked
- **THEN** it alternates between standing idle (2-5 seconds) and wandering in a random direction (3-8 seconds)

#### Scenario: Mob despawn at distance
- **WHEN** a mob entity is more than 128 blocks from the nearest player
- **THEN** it is despawned to free resources

### Requirement: Item and Crafting System
The system SHALL maintain an item registry with typed item definitions (tools, weapons, armor, food, materials) loaded from a data table.

The system SHALL support shaped and shapeless crafting recipes matched against a 2x2 (inventory) or 3x3 (workbench) grid.

Tools SHALL have mining speed multipliers, durability, and tier-based mining level restrictions.

#### Scenario: Crafting wooden pickaxe
- **WHEN** the player places 3 Plank items in the top row and 2 Stick items in the center column of a 3x3 crafting grid
- **THEN** the output slot shows 1 Wooden Pickaxe

#### Scenario: Tool durability
- **WHEN** a player uses a Wooden Pickaxe to break a block
- **THEN** the pickaxe's durability decreases by 1, and when durability reaches 0, the pickaxe is destroyed

#### Scenario: Mining level restriction
- **WHEN** a player attempts to mine Diamond Ore with a Stone Pickaxe
- **THEN** the block breaks but drops nothing (Iron Pickaxe or higher required)

### Requirement: Hunger and Food System
The system SHALL track player hunger (0-20) and saturation (0-20) levels.

Physical actions (moving, jumping, mining, attacking) SHALL consume saturation first, then hunger.

Eating food SHALL restore hunger and saturation according to per-item food values.

#### Scenario: Hunger depletion
- **WHEN** the player sprints for an extended period
- **THEN** saturation depletes first, then hunger level decreases

#### Scenario: Starvation damage
- **WHEN** the player's hunger reaches 0
- **THEN** the player takes periodic damage (1 HP every 4 seconds)

#### Scenario: Natural regeneration
- **WHEN** the player's hunger is >= 18
- **THEN** the player slowly regenerates health (0.5 HP every 4 seconds)

### Requirement: World Creation and Selection UI
The main menu SHALL provide a "World Selection" screen listing all saved worlds with last-played time.

The system SHALL provide a "Create World" screen with world name input, seed input, and game mode selection.

#### Scenario: Create new world
- **WHEN** the player enters name "My World", seed "12345", selects Survival mode, and clicks Create
- **THEN** a new save slot is created with the given seed and the game starts in Survival mode

#### Scenario: Load existing world
- **WHEN** the player selects an existing world from the world list
- **THEN** the saved world is loaded with all chunk data, player state, and inventory restored

### Requirement: Settings Persistence and Effect
All settings changes SHALL be persisted to disk and restored on next launch.

Settings values SHALL be propagated to the actual game systems they control (FOV to camera, sensitivity to mouse, view distance to chunk manager).

#### Scenario: FOV setting takes effect
- **WHEN** the player changes FOV from 70 to 90 in the settings menu
- **THEN** the camera projection matrix is immediately updated with the new FOV value

#### Scenario: Settings persistence
- **WHEN** the player changes view distance to 12 and restarts the game
- **THEN** the view distance is still 12 after restart

### Requirement: F3 Debug Overlay
The system SHALL provide a toggleable (F3 key) debug overlay displaying: FPS, player coordinates, facing direction, chunk coordinates, light level at feet, biome name, loaded chunk count, and active entity count.

#### Scenario: Toggle debug overlay
- **WHEN** the player presses F3 during gameplay
- **THEN** a text overlay appears in the top-left showing real-time debug information

### Requirement: Mining Progress Indicator
Block breaking SHALL require holding the attack button for a duration determined by block hardness and tool effectiveness.

The system SHALL display a cracking overlay on the targeted block that progresses through 10 visual stages.

#### Scenario: Mining stone with wooden pickaxe
- **WHEN** the player holds left-click on a Stone block with a Wooden Pickaxe equipped
- **THEN** a crack overlay appears and progresses over ~1.5 seconds until the block breaks

#### Scenario: Instant break in creative mode
- **WHEN** the player is in Creative mode and clicks a block
- **THEN** the block breaks instantly with no mining progress animation

## MODIFIED Requirements

### Requirement: Non-Blocking Initial World Generation
The system SHALL generate initial chunks asynchronously using the existing worker thread pipeline, displaying a loading screen until the minimum playable area is ready.

The system SHALL NOT block the main thread for multi-second synchronous chunk generation at startup.

The loading screen SHALL display a progress indicator showing chunk generation progress.

#### Scenario: Startup experience
- **WHEN** the game launches and loads or creates a world
- **THEN** a loading screen with progress bar is displayed while chunks are generated in the background, and gameplay begins once the initial area (load_radius chunks) is ready

#### Scenario: Progressive loading
- **WHEN** the initial load is in progress
- **THEN** chunks closest to the spawn point are prioritized and become playable first
