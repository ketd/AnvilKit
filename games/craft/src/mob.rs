//! Mob/entity system — spawning, AI state machines, physics, drops.
//!
//! Passive mobs: Idle ↔ Wander (+ Flee on damage)
//! Hostile mobs: Idle → Detect → Chase → Attack (+ despawn at distance)

use bevy_ecs::prelude::*;
use glam::Vec3;
use anvilkit_core::math::Transform;
use anvilkit_core::time::DeltaTime;
use anvilkit_core::math::Velocity;
use anvilkit_render::transform::AabbCollider;
use anvilkit_gameplay::health::{Health, DamageEvent, DeathEvent};
use anvilkit_gameplay::inventory::ItemStack;

use crate::block::BlockType;
use crate::resources::VoxelWorld;

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

/// Mob type identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub enum MobType {
    // Passive
    Pig,
    Cow,
    Sheep,
    Chicken,
    // Hostile
    Zombie,
    Skeleton,
    Spider,
    Creeper,
}

impl MobType {
    pub fn is_hostile(self) -> bool {
        matches!(self, Self::Zombie | Self::Skeleton | Self::Spider | Self::Creeper)
    }

    pub fn max_health(self) -> f32 {
        match self {
            Self::Pig => 10.0,
            Self::Cow => 10.0,
            Self::Sheep => 8.0,
            Self::Chicken => 4.0,
            Self::Zombie => 20.0,
            Self::Skeleton => 20.0,
            Self::Spider => 16.0,
            Self::Creeper => 20.0,
        }
    }

    pub fn move_speed(self) -> f32 {
        match self {
            Self::Pig | Self::Cow | Self::Sheep => 2.0,
            Self::Chicken => 2.5,
            Self::Zombie => 2.3,
            Self::Skeleton => 2.5,
            Self::Spider => 3.5,
            Self::Creeper => 2.8,
        }
    }

    pub fn attack_damage(self) -> f32 {
        match self {
            Self::Zombie => 3.0,
            Self::Skeleton => 3.0,
            Self::Spider => 2.0,
            Self::Creeper => 0.0, // explodes instead
            _ => 0.0,
        }
    }

    pub fn detection_range(self) -> f32 {
        match self {
            Self::Zombie | Self::Skeleton | Self::Creeper => 16.0,
            Self::Spider => 12.0,
            _ => 0.0,
        }
    }

    /// Items dropped on death: (item_id, min_count, max_count).
    pub fn drops(self) -> &'static [(u32, u32, u32)] {
        match self {
            Self::Pig => &[(1, 1, 3)],     // placeholder item_id=1 (raw porkchop)
            Self::Cow => &[(1, 1, 3)],     // raw beef
            Self::Sheep => &[(1, 1, 2)],   // wool
            Self::Chicken => &[(1, 1, 2)], // raw chicken
            _ => &[],
        }
    }
}

/// AI finite state machine state.
#[derive(Debug, Clone, Copy, PartialEq, Component)]
pub enum AiState {
    Idle { timer: u32 },
    Wander { dir: [i8; 2], timer: u32 },
    Flee { from_x: f32, from_z: f32, timer: u32 },
    Detect,
    Chase,
    Attack { cooldown: u32 },
}

impl Default for AiState {
    fn default() -> Self {
        Self::Idle { timer: 60 }
    }
}

/// Marker: mob entity (for queries).
#[derive(Component)]
pub struct Mob;

/// Mob spawn cooldown timer (global resource).
#[derive(Resource)]
pub struct MobSpawnTimer {
    pub ticks: u32,
}

impl Default for MobSpawnTimer {
    fn default() -> Self {
        Self { ticks: 0 }
    }
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const MAX_MOBS: usize = 128;
const SPAWN_INTERVAL: u32 = 200; // ticks between spawn attempts
const DESPAWN_DISTANCE: f32 = 128.0;
const ATTACK_RANGE: f32 = 1.5;
const ATTACK_COOLDOWN: u32 = 20; // ticks

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Passive AI: Idle ↔ Wander, Flee on damage.
pub fn passive_ai_system(
    _dt: Res<DeltaTime>,
    mut query: Query<(&MobType, &mut AiState, &Transform, &mut Velocity), With<Mob>>,
) {
    for (mob_type, mut ai, transform, mut vel) in &mut query {
        if mob_type.is_hostile() { continue; }

        match *ai {
            AiState::Idle { timer } => {
                vel.linear = Vec3::ZERO;
                if timer == 0 {
                    // Pseudo-random wander direction from position hash
                    let hash = (transform.translation.x as i32).wrapping_mul(73856093)
                        ^ (transform.translation.z as i32).wrapping_mul(19349663);
                    let dx = ((hash % 3) - 1) as i8;
                    let dz = (((hash >> 8) % 3) - 1) as i8;
                    *ai = AiState::Wander { dir: [dx, dz], timer: 120 };
                } else {
                    *ai = AiState::Idle { timer: timer.saturating_sub(1) };
                }
            }
            AiState::Wander { dir, timer } => {
                let speed = mob_type.move_speed();
                vel.linear = Vec3::new(dir[0] as f32 * speed, vel.linear.y, dir[1] as f32 * speed);
                if timer == 0 {
                    *ai = AiState::Idle { timer: 90 };
                } else {
                    *ai = AiState::Wander { dir, timer: timer.saturating_sub(1) };
                }
            }
            AiState::Flee { from_x, from_z, timer } => {
                let from = Vec3::new(from_x, transform.translation.y, from_z);
                let away = (transform.translation - from).normalize_or_zero();
                let speed = mob_type.move_speed() * 1.5;
                vel.linear = Vec3::new(away.x * speed, vel.linear.y, away.z * speed);
                if timer == 0 {
                    *ai = AiState::Wander { dir: [0, 0], timer: 60 };
                } else {
                    *ai = AiState::Flee { from_x, from_z, timer: timer.saturating_sub(1) };
                }
            }
            _ => {}
        }
    }
}

/// Hostile AI: Idle → Detect → Chase → Attack.
pub fn hostile_ai_system(
    player_query: Query<&Transform, (With<crate::components::FpsCamera>, Without<Mob>)>,
    mut mob_query: Query<(&MobType, &mut AiState, &Transform, &mut Velocity), With<Mob>>,
    mut damage_events: EventWriter<DamageEvent>,
    player_entities: Query<Entity, (With<crate::components::FpsCamera>, Without<Mob>)>,
) {
    let player_pos = match player_query.iter().next() {
        Some(t) => t.translation,
        None => return,
    };
    let player_entity = match player_entities.iter().next() {
        Some(e) => e,
        None => return,
    };

    for (mob_type, mut ai, transform, mut vel) in &mut mob_query {
        if !mob_type.is_hostile() { continue; }
        let pos = transform.translation;
        let dist = (pos - player_pos).length();
        let range = mob_type.detection_range();

        match *ai {
            AiState::Idle { timer } => {
                vel.linear = Vec3::ZERO;
                if dist < range {
                    *ai = AiState::Chase;
                } else if timer == 0 {
                    *ai = AiState::Idle { timer: 60 };
                } else {
                    *ai = AiState::Idle { timer: timer.saturating_sub(1) };
                }
            }
            AiState::Chase { .. } => {
                if dist < ATTACK_RANGE {
                    *ai = AiState::Attack { cooldown: ATTACK_COOLDOWN };
                } else if dist > range * 2.0 {
                    *ai = AiState::Idle { timer: 60 };
                } else {
                    let dir = (player_pos - pos).normalize_or_zero();
                    let speed = mob_type.move_speed();
                    vel.linear = Vec3::new(dir.x * speed, vel.linear.y, dir.z * speed);
                    *ai = AiState::Chase;
                }
            }
            AiState::Attack { cooldown } => {
                vel.linear = Vec3::ZERO;
                if cooldown == 0 {
                    if dist < ATTACK_RANGE * 1.5 {
                        damage_events.send(DamageEvent {
                            target: player_entity,
                            amount: mob_type.attack_damage(),
                            source: None,
                        });
                        *ai = AiState::Attack { cooldown: ATTACK_COOLDOWN };
                    } else {
                        *ai = AiState::Chase;
                    }
                } else {
                    *ai = AiState::Attack { cooldown: cooldown.saturating_sub(1) };
                }
            }
            _ => {
                // Detect state → start chasing
                if dist < range {
                    *ai = AiState::Chase;
                } else {
                    *ai = AiState::Idle { timer: 60 };
                }
            }
        }
    }
}

/// Mob gravity + ground collision (simplified).
pub fn mob_physics_system(
    dt: Res<DeltaTime>,
    world: Res<VoxelWorld>,
    mut query: Query<(&mut Transform, &mut Velocity, &AabbCollider), With<Mob>>,
) {
    let gravity = crate::config::GRAVITY;
    for (mut transform, mut vel, _collider) in &mut query {
        // Apply gravity
        vel.linear.y -= gravity * dt.0;
        vel.linear.y = vel.linear.y.max(-crate::config::TERMINAL_VELOCITY);

        // Integrate position
        let new_pos = transform.translation + vel.linear * dt.0;

        // Simple ground check
        let feet_y = new_pos.y - 0.5;
        let block_below = world.get_block(new_pos.x as i32, feet_y as i32, new_pos.z as i32);
        if block_below.is_obstacle() && vel.linear.y < 0.0 {
            transform.translation.x = new_pos.x;
            transform.translation.y = feet_y.ceil() + 0.5;
            transform.translation.z = new_pos.z;
            vel.linear.y = 0.0;
        } else {
            transform.translation = new_pos;
        }
    }
}

/// Despawn mobs too far from the player.
pub fn mob_despawn_system(
    mut commands: Commands,
    player_query: Query<&Transform, (With<crate::components::FpsCamera>, Without<Mob>)>,
    mob_query: Query<(Entity, &Transform), With<Mob>>,
) {
    let player_pos = match player_query.iter().next() {
        Some(t) => t.translation,
        None => return,
    };
    for (entity, transform) in &mob_query {
        let dist = (transform.translation - player_pos).length();
        if dist > DESPAWN_DISTANCE {
            commands.entity(entity).despawn();
        }
    }
}

/// Mob spawning system — periodic spawn attempts around the player.
pub fn mob_spawn_system(
    mut commands: Commands,
    mut timer: ResMut<MobSpawnTimer>,
    player_query: Query<&Transform, (With<crate::components::FpsCamera>, Without<Mob>)>,
    mob_query: Query<&Mob>,
    world: Res<VoxelWorld>,
    day_night: Res<crate::resources::DayNightCycle>,
) {
    timer.ticks += 1;
    if timer.ticks < SPAWN_INTERVAL { return; }
    timer.ticks = 0;

    if mob_query.iter().count() >= MAX_MOBS { return; }

    let player_pos = match player_query.iter().next() {
        Some(t) => t.translation,
        None => return,
    };

    let is_night = day_night.light_dir().y < 0.0;

    // Try to spawn a mob at a random offset from the player
    let hash = (player_pos.x as i32).wrapping_mul(73856093)
        ^ (player_pos.z as i32).wrapping_mul(19349663)
        ^ (timer.ticks as i32).wrapping_mul(83492791);
    let dx = ((hash % 32) - 16) as f32;
    let dz = (((hash >> 8) % 32) - 16) as f32;
    let spawn_x = player_pos.x + dx;
    let spawn_z = player_pos.z + dz;

    // Find ground height
    let mut spawn_y = None;
    for y in (1..120).rev() {
        let block = world.get_block(spawn_x as i32, y, spawn_z as i32);
        let above = world.get_block(spawn_x as i32, y + 1, spawn_z as i32);
        if block.is_obstacle() && above == BlockType::Air {
            spawn_y = Some(y as f32 + 1.5);
            break;
        }
    }
    let spawn_y = match spawn_y {
        Some(y) => y,
        None => return,
    };

    // Check light level for hostile/passive selection
    let light = world.get_light(spawn_x as i32, spawn_y as i32, spawn_z as i32);
    let sky_light = light >> 4;

    let mob_type = if is_night && sky_light <= 7 {
        // Hostile mob at night in dark areas
        match (hash >> 16) % 4 {
            0 => MobType::Zombie,
            1 => MobType::Skeleton,
            2 => MobType::Spider,
            _ => MobType::Creeper,
        }
    } else if sky_light >= 9 && !is_night {
        // Passive mob during day in lit areas
        match (hash >> 16) % 4 {
            0 => MobType::Pig,
            1 => MobType::Cow,
            2 => MobType::Sheep,
            _ => MobType::Chicken,
        }
    } else {
        return; // conditions not met
    };

    let pos = Vec3::new(spawn_x, spawn_y, spawn_z);
    commands.spawn((
        Mob,
        mob_type,
        AiState::default(),
        Transform::from_translation(pos),
        Velocity::zero(),
        AabbCollider { half_extents: Vec3::new(0.4, 0.5, 0.4) },
        Health::new(mob_type.max_health()),
    ));
}

/// Handle mob death → despawn + spawn item drops.
pub fn mob_death_system(
    mut commands: Commands,
    mut death_events: EventReader<DeathEvent>,
    mob_query: Query<(&MobType, &Transform), With<Mob>>,
) {
    for ev in death_events.read() {
        if let Ok((mob_type, transform)) = mob_query.get(ev.entity) {
            let pos = transform.translation;
            let drops = mob_type.drops();
            for &(item_id, min_count, max_count) in drops {
                // Simple "random" count from position hash
                let hash = (pos.x as u32).wrapping_mul(73856093) ^ (pos.z as u32).wrapping_mul(19349663);
                let count = min_count + (hash % (max_count - min_count + 1));
                commands.spawn((
                    ItemDropEntity,
                    DropItem { item_id, count },
                    Transform::from_translation(pos + Vec3::Y * 0.5),
                    Velocity::linear(Vec3::new(0.0, 3.0, 0.0)),
                    DropLifetime(300), // 5 seconds at 60fps
                ));
            }
            commands.entity(ev.entity).despawn();
        }
    }
}

// ---------------------------------------------------------------------------
// Item drop components
// ---------------------------------------------------------------------------

/// Marker for dropped item entities.
#[derive(Component)]
pub struct ItemDropEntity;

/// The item contained in a drop entity (Component wrapper for item data).
#[derive(Debug, Clone, Component)]
pub struct DropItem {
    pub item_id: u32,
    pub count: u32,
}

/// Lifetime counter (ticks). Despawns when reaching zero.
#[derive(Component)]
pub struct DropLifetime(pub u32);

/// Item drop physics + lifetime system.
pub fn item_drop_system(
    mut commands: Commands,
    dt: Res<DeltaTime>,
    world: Res<VoxelWorld>,
    mut query: Query<(Entity, &mut Transform, &mut Velocity, &mut DropLifetime), With<ItemDropEntity>>,
) {
    let gravity = crate::config::GRAVITY;
    for (entity, mut transform, mut vel, mut lifetime) in &mut query {
        if lifetime.0 == 0 {
            commands.entity(entity).despawn();
            continue;
        }
        lifetime.0 -= 1;

        vel.linear.y -= gravity * dt.0;
        let new_pos = transform.translation + vel.linear * dt.0;

        let block_below = world.get_block(new_pos.x as i32, (new_pos.y - 0.2) as i32, new_pos.z as i32);
        if block_below.is_obstacle() && vel.linear.y < 0.0 {
            transform.translation.x = new_pos.x;
            transform.translation.y = (new_pos.y - 0.2).ceil() + 0.2;
            transform.translation.z = new_pos.z;
            vel.linear = Vec3::ZERO;
        } else {
            transform.translation = new_pos;
        }
    }
}

/// Player picks up nearby item drops.
pub fn item_pickup_system(
    mut commands: Commands,
    player_query: Query<&Transform, (With<crate::components::FpsCamera>, Without<ItemDropEntity>)>,
    drop_query: Query<(Entity, &Transform, &DropItem), With<ItemDropEntity>>,
    mut inventory: Query<&mut anvilkit_gameplay::inventory::SlotInventory, With<crate::components::FpsCamera>>,
) {
    let player_pos = match player_query.iter().next() {
        Some(t) => t.translation,
        None => return,
    };

    for (entity, transform, drop_item) in &drop_query {
        let dist = (transform.translation - player_pos).length();
        if dist < 1.5 {
            if let Some(mut inv) = inventory.iter_mut().next() {
                use anvilkit_gameplay::inventory::Inventory;
                let stack = ItemStack::new(drop_item.item_id, drop_item.count);
                let remainder = inv.add_item(stack, 64);
                if remainder.is_none() {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Passive mob flee-on-damage trigger
// ---------------------------------------------------------------------------

/// When a passive mob takes damage, switch to Flee state.
pub fn mob_flee_on_damage_system(
    mut damage_events: EventReader<DamageEvent>,
    mut query: Query<(&MobType, &mut AiState, &Transform), With<Mob>>,
    player_query: Query<&Transform, (With<crate::components::FpsCamera>, Without<Mob>)>,
) {
    let player_pos = player_query.iter().next().map(|t| t.translation);
    for ev in damage_events.read() {
        if let Ok((mob_type, mut ai, _transform)) = query.get_mut(ev.target) {
            if !mob_type.is_hostile() {
                let p = player_pos.unwrap_or(Vec3::ZERO);
                *ai = AiState::Flee { from_x: p.x, from_z: p.z, timer: 100 };
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Simple grid-based A* pathfinding (XZ plane)
// ---------------------------------------------------------------------------

/// Find a path from start to goal on the voxel grid (XZ plane, Y = walk surface).
/// Returns a list of (x, z) waypoints, or None if no path found.
/// Max search area: 32x32 blocks around start.
pub fn grid_astar(
    world: &VoxelWorld,
    start: (i32, i32, i32), // (x, y, z)
    goal: (i32, i32),       // (x, z)
    max_steps: usize,
) -> Option<Vec<(i32, i32)>> {
    use std::collections::{BinaryHeap, HashMap};

    let (sx, sy, sz) = start;
    let (gx, gz) = goal;

    #[derive(Eq, PartialEq)]
    struct Node { cost: u32, x: i32, z: i32 }
    impl Ord for Node {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering { other.cost.cmp(&self.cost) }
    }
    impl PartialOrd for Node {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> { Some(self.cmp(other)) }
    }

    let mut open = BinaryHeap::new();
    let mut came_from: HashMap<(i32, i32), (i32, i32)> = HashMap::new();
    let mut g_score: HashMap<(i32, i32), u32> = HashMap::new();

    let heuristic = |x: i32, z: i32| (x - gx).unsigned_abs() + (z - gz).unsigned_abs();

    open.push(Node { cost: heuristic(sx, sz), x: sx, z: sz });
    g_score.insert((sx, sz), 0);

    let mut steps = 0;
    while let Some(Node { x, z, .. }) = open.pop() {
        if x == gx && z == gz {
            // Reconstruct path
            let mut path = vec![(gx, gz)];
            let mut cur = (gx, gz);
            while let Some(&prev) = came_from.get(&cur) {
                path.push(prev);
                cur = prev;
                if cur == (sx, sz) { break; }
            }
            path.reverse();
            return Some(path);
        }

        steps += 1;
        if steps > max_steps { return None; }

        let g = g_score[&(x, z)];
        let neighbors = [(x+1, z), (x-1, z), (x, z+1), (x, z-1)];

        for (nx, nz) in neighbors {
            // Check walkability: need solid block below and air at foot+head level
            let walk_y = sy;
            // Check if we can step up 1 block
            let block_at = world.get_block(nx, walk_y, nz);
            let block_above = world.get_block(nx, walk_y + 1, nz);
            let block_below = world.get_block(nx, walk_y - 1, nz);

            let walkable = if !block_at.is_obstacle() && block_below.is_obstacle() {
                true // flat walk
            } else if block_at.is_obstacle() && !block_above.is_obstacle()
                && !world.get_block(nx, walk_y + 2, nz).is_obstacle() {
                true // step up 1
            } else if !block_at.is_obstacle() && !block_below.is_obstacle()
                && world.get_block(nx, walk_y - 2, nz).is_obstacle() {
                true // step down 1
            } else {
                false
            };

            if !walkable { continue; }

            let ng = g + 1;
            if ng < *g_score.get(&(nx, nz)).unwrap_or(&u32::MAX) {
                g_score.insert((nx, nz), ng);
                came_from.insert((nx, nz), (x, z));
                let f = ng + heuristic(nx, nz);
                open.push(Node { cost: f, x: nx, z: nz });
            }
        }
    }

    None // no path found
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::world::World;
    use bevy_ecs::schedule::Schedule;

    #[test]
    fn test_mob_type_properties() {
        assert!(MobType::Zombie.is_hostile());
        assert!(!MobType::Pig.is_hostile());
        assert_eq!(MobType::Zombie.max_health(), 20.0);
        assert_eq!(MobType::Pig.max_health(), 10.0);
        assert!(MobType::Zombie.attack_damage() > 0.0);
        assert_eq!(MobType::Pig.attack_damage(), 0.0);
    }

    #[test]
    fn test_ai_state_default() {
        let state = AiState::default();
        assert!(matches!(state, AiState::Idle { timer: 60 }));
    }

    #[test]
    fn test_passive_ai_transitions() {
        let mut world = World::new();
        world.insert_resource(DeltaTime(1.0 / 60.0));

        let entity = world.spawn((
            Mob,
            MobType::Pig,
            AiState::Idle { timer: 0 }, // will transition to Wander
            Transform::from_translation(Vec3::new(10.0, 50.0, 10.0)),
            Velocity::zero(),
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(passive_ai_system);
        schedule.run(&mut world);

        let ai = world.get::<AiState>(entity).unwrap();
        assert!(matches!(ai, AiState::Wander { .. }), "Expected Wander after Idle timer=0, got {:?}", ai);
    }

    #[test]
    fn test_grid_astar_straight_line() {
        // Build a flat world: stone at y=49, air above
        let mut world = VoxelWorld::default();
        let mut chunk = crate::chunk::ChunkData::new();
        for x in 0..crate::chunk::CHUNK_SIZE {
            for z in 0..crate::chunk::CHUNK_SIZE {
                chunk.set(x, 49, z, crate::block::BlockType::Stone);
            }
        }
        let light = crate::lighting::LightMap::new();
        world.chunks.insert((0, 0), chunk);
        world.light_maps.insert((0, 0), light);

        let path = grid_astar(&world, (5, 50, 5), (10, 5), 500);
        assert!(path.is_some(), "Should find path on flat terrain");
        let path = path.unwrap();
        assert_eq!(*path.last().unwrap(), (10, 5));
    }

    #[test]
    fn test_mob_death_despawns() {
        let mut world = World::new();
        world.init_resource::<Events<DeathEvent>>();
        world.insert_resource(DeltaTime(1.0 / 60.0));

        let entity = world.spawn((
            Mob,
            MobType::Pig,
            AiState::default(),
            Transform::from_translation(Vec3::new(10.0, 50.0, 10.0)),
            Velocity::zero(),
            Health::new(10.0),
        )).id();

        world.resource_mut::<Events<DeathEvent>>().send(DeathEvent { entity });

        let mut schedule = Schedule::default();
        schedule.add_systems(mob_death_system);
        schedule.run(&mut world);

        assert!(world.get_entity(entity).is_err(), "Mob should be despawned after death");
    }
}
