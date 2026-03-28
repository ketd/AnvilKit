use bevy_ecs::prelude::*;
use anvilkit_ecs::physics::DeltaTime;
use anvilkit_gameplay::health::{Health, DamageEvent, DeathEvent};
use anvilkit_core::math::Transform;

use crate::components::FpsCamera;
use crate::resources::{PlayerState, VoxelWorld};
use crate::config;
use crate::chunk::CHUNK_SIZE;

/// Fall damage: triggers when landing with velocity exceeding the jump threshold.
pub fn fall_damage_system(
    player: Res<PlayerState>,
    query: Query<Entity, With<FpsCamera>>,
    mut damage_events: EventWriter<DamageEvent>,
) {
    // Just transitioned from airborne to on_ground
    if player.on_ground && !player.was_on_ground {
        let fall_speed = -player.last_vy;
        let threshold = config::JUMP_VEL + 2.0;
        if fall_speed > threshold {
            let damage = (fall_speed - threshold) * 1.0;
            for entity in query.iter() {
                damage_events.send(DamageEvent {
                    target: entity,
                    amount: damage,
                    source: None,
                });
            }
        }
    }
}

/// Drowning: 1 HP damage every 2 seconds when head is submerged in water.
pub fn drowning_system(
    dt: Res<DeltaTime>,
    voxel_world: Res<VoxelWorld>,
    query: Query<(Entity, &Transform), With<FpsCamera>>,
    mut damage_events: EventWriter<DamageEvent>,
    mut timer: Local<f32>,
) {
    for (entity, transform) in query.iter() {
        let head_pos = transform.translation;
        let block = voxel_world.get_block(
            head_pos.x as i32,
            head_pos.y as i32,
            head_pos.z as i32,
        );
        if block.is_water() {
            *timer += dt.0;
            if *timer >= 2.0 {
                *timer -= 2.0;
                damage_events.send(DamageEvent {
                    target: entity,
                    amount: 1.0,
                    source: None,
                });
            }
        } else {
            *timer = 0.0;
        }
    }
}

/// Health regeneration: applies regen_rate * dt each frame for living entities.
pub fn health_regen_system(dt: Res<DeltaTime>, mut query: Query<&mut Health>) {
    for mut hp in &mut query {
        if hp.is_alive() && hp.regen_rate > 0.0 {
            let amount = hp.regen_rate * dt.0;
            hp.heal(amount);
        }
    }
}

/// Death respawn: teleport to spawn point with full health.
pub fn death_respawn_system(
    mut events: EventReader<DeathEvent>,
    mut query: Query<(&mut Transform, &mut Health), With<FpsCamera>>,
    mut player: ResMut<PlayerState>,
) {
    for ev in events.read() {
        if let Ok((mut transform, mut hp)) = query.get_mut(ev.entity) {
            transform.translation = glam::Vec3::new(
                (CHUNK_SIZE as f32) * 3.5,
                50.0,
                (CHUNK_SIZE as f32) * 3.5,
            );
            hp.current = hp.max;
            player.flying = true;
            println!("Player died! Respawning...");
        }
    }
}
