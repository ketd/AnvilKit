use bevy_ecs::prelude::*;
use anvilkit_core::math::Transform;
use anvilkit_ecs::physics::DeltaTime;
use anvilkit_render::plugin::CameraComponent;

use crate::components::FpsCamera;
use crate::resources::{PlayerState, VoxelWorld};

const PLAYER_WIDTH: f32 = 0.6;
const PLAYER_HEIGHT: f32 = 1.8;
const EYE_OFFSET: f32 = 1.6;
const GRAVITY: f32 = 20.0;
const JUMP_VEL: f32 = 8.0;
const TERMINAL_VELOCITY: f32 = 50.0;

/// Check if an AABB (defined by eye position) collides with any solid block.
fn collides_aabb(world: &VoxelWorld, eye_pos: glam::Vec3) -> bool {
    let half_w = PLAYER_WIDTH * 0.5;
    let feet_y = eye_pos.y - EYE_OFFSET;

    let min_x = (eye_pos.x - half_w).floor() as i32;
    let max_x = (eye_pos.x + half_w).floor() as i32;
    let min_y = feet_y.floor() as i32;
    let max_y = (feet_y + PLAYER_HEIGHT).floor() as i32;
    let min_z = (eye_pos.z - half_w).floor() as i32;
    let max_z = (eye_pos.z + half_w).floor() as i32;

    for bx in min_x..=max_x {
        for by in min_y..=max_y {
            for bz in min_z..=max_z {
                let block = world.get_block(bx, by, bz);
                if block.is_obstacle() {
                    let bx_min = bx as f32;
                    let bx_max = (bx + 1) as f32;
                    let by_min = by as f32;
                    let by_max = (by + 1) as f32;
                    let bz_min = bz as f32;
                    let bz_max = (bz + 1) as f32;

                    let px_min = eye_pos.x - half_w;
                    let px_max = eye_pos.x + half_w;
                    let py_min = feet_y;
                    let py_max = feet_y + PLAYER_HEIGHT;
                    let pz_min = eye_pos.z - half_w;
                    let pz_max = eye_pos.z + half_w;

                    if px_max > bx_min && px_min < bx_max
                        && py_max > by_min && py_min < by_max
                        && pz_max > bz_min && pz_min < bz_max
                    {
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// Physics system: gravity, jump, collision detection.
pub fn player_physics_system(
    dt: Res<DeltaTime>,
    mut player: ResMut<PlayerState>,
    world: Res<VoxelWorld>,
    mut query: Query<(&mut Transform, &CameraComponent), With<FpsCamera>>,
) {
    if player.flying {
        return;
    }

    let delta = dt.0;

    // Only apply gravity when airborne
    if !player.on_ground {
        player.velocity.y = (player.velocity.y - GRAVITY * delta).max(-TERMINAL_VELOCITY);
    }

    // Jump
    if player.on_ground && player.jump_requested {
        player.velocity.y = JUMP_VEL;
        player.on_ground = false;
    }
    player.jump_requested = false;

    for (mut transform, _cam) in query.iter_mut() {
        let pos = transform.translation;

        // Safety: push out if embedded in a block
        if collides_aabb(&world, pos) {
            log::warn!(
                "Player embedded at ({:.2}, {:.2}, {:.2})! Pushing up.",
                pos.x, pos.y, pos.z
            );
            transform.translation.y += 1.0;
            player.velocity.y = 0.0;
            player.on_ground = false;
            continue;
        }

        // --- Y axis ---
        if player.velocity.y.abs() > 0.0001 {
            let new_y = pos.y + player.velocity.y * delta;
            let test_pos_y = glam::Vec3::new(pos.x, new_y, pos.z);
            if collides_aabb(&world, test_pos_y) {
                if player.velocity.y < 0.0 {
                    // Landing: snap feet to just above the block top
                    // (epsilon prevents floating-point boundary oscillation)
                    let feet_y = new_y - EYE_OFFSET;
                    let block_top = feet_y.floor() + 1.0;
                    transform.translation.y = block_top + EYE_OFFSET + 0.001;
                    player.on_ground = true;
                }
                // else: hit ceiling
                player.velocity.y = 0.0;
            } else {
                transform.translation.y = new_y;
                player.on_ground = false;
            }
        } else if player.on_ground {
            // Ground check: are we still standing on something?
            let below = glam::Vec3::new(pos.x, pos.y - 0.05, pos.z);
            if !collides_aabb(&world, below) {
                player.on_ground = false;
                // Start falling
                player.velocity.y = -GRAVITY * delta;
            }
        }

        // --- X axis ---
        let vel_x = player.velocity.x;
        if vel_x.abs() > 0.001 {
            let new_x = transform.translation.x + vel_x * delta;
            let test_pos_x = glam::Vec3::new(new_x, transform.translation.y, transform.translation.z);
            if !collides_aabb(&world, test_pos_x) {
                transform.translation.x = new_x;
            }
        }
        player.velocity.x = 0.0;

        // --- Z axis ---
        let vel_z = player.velocity.z;
        if vel_z.abs() > 0.001 {
            let new_z = transform.translation.z + vel_z * delta;
            let test_pos_z = glam::Vec3::new(transform.translation.x, transform.translation.y, new_z);
            if !collides_aabb(&world, test_pos_z) {
                transform.translation.z = new_z;
            }
        }
        player.velocity.z = 0.0;
    }
}
