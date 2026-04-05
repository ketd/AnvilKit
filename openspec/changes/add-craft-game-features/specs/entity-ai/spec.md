## ADDED Requirements

### Requirement: Mob Entity Framework
The system SHALL support mob entities as ECS entities with Transform, Velocity, Health, MobType, AiState, and AABB collider components. Mob properties SHALL be defined in a data table (`mobs.ron`).

#### Scenario: Mob spawn
- **WHEN** a Zombie is spawned at position (100, 65, 200)
- **THEN** an ECS entity is created with Health(20), MobType::Zombie, AiState::Idle, and a bounding box collider

### Requirement: Passive Mob AI
Passive mobs SHALL follow an Idle/Wander/Flee finite state machine. They wander randomly when idle and flee from the player when attacked.

#### Scenario: Passive wander
- **WHEN** a Pig is in Idle state for 3 seconds
- **THEN** it transitions to Wander state and moves in a random direction for 3-8 seconds

#### Scenario: Passive flee
- **WHEN** a Cow takes damage from the player
- **THEN** it transitions to Flee state and moves away from the player for 5 seconds

### Requirement: Hostile Mob AI
Hostile mobs SHALL follow an Idle/Detect/Chase/Attack finite state machine. They detect the player within a configurable range, chase using grid-based A* pathfinding, and attack when in melee range.

#### Scenario: Player detection
- **WHEN** a Zombie is within 16 blocks of the player and has line-of-sight
- **THEN** it transitions from Idle to Chase state

#### Scenario: Melee attack
- **WHEN** a Zombie in Chase state reaches within 1.5 blocks of the player
- **THEN** it transitions to Attack state and deals 3 damage per hit every 1 second

### Requirement: Mob Spawn Rules
Mob spawning SHALL be governed by configurable rules: biome whitelist, light level range, time-of-day restriction, Y-level range, and per-chunk density cap.

#### Scenario: Hostile spawn conditions
- **WHEN** it is nighttime and a surface block has light level <= 7 in a Plains biome
- **THEN** hostile mobs (Zombie, Skeleton, Spider, Creeper) may spawn on valid surfaces

#### Scenario: Global mob cap
- **WHEN** 128 mob entities are already alive
- **THEN** no new mobs are spawned until existing mobs are despawned or killed

### Requirement: Item Drop Entities
When mobs die or blocks are broken, the system SHALL spawn item drop entities that float, rotate, and can be picked up by the player.

#### Scenario: Mob death drops
- **WHEN** a Pig dies
- **THEN** 1-3 Raw Porkchop ItemDrop entities are spawned at its position with upward velocity scatter

#### Scenario: Player pickup
- **WHEN** the player moves within 1.5 blocks of an ItemDrop
- **THEN** the item is added to the player's inventory and the drop entity is despawned
