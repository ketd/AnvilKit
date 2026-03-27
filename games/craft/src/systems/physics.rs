use bevy_ecs::prelude::*;
use anvilkit_core::math::Transform;
use anvilkit_ecs::physics::{DeltaTime, Velocity, AabbCollider};
use anvilkit_render::plugin::CameraComponent;

use crate::components::FpsCamera;
use crate::config;
use crate::resources::{PlayerState, VoxelWorld};

/// Check if an AABB (defined by eye position + collider extents) collides with any solid block.
fn collides_aabb(world: &VoxelWorld, eye_pos: glam::Vec3, half_w: f32, height: f32, eye_offset: f32) -> bool {
    let feet_y = eye_pos.y - eye_offset;

    let min_x = (eye_pos.x - half_w).floor() as i32;
    let max_x = (eye_pos.x + half_w).floor() as i32;
    let min_y = feet_y.floor() as i32;
    let max_y = (feet_y + height).floor() as i32;
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
                    let py_max = feet_y + height;
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

/// Player physics system using engine `Velocity` and `AabbCollider` components.
///
/// Handles gravity, jumping, and per-axis voxel collision detection.
pub fn player_physics_system(
    dt: Res<DeltaTime>,
    mut player: ResMut<PlayerState>,
    world: Res<VoxelWorld>,
    mut query: Query<(&mut Transform, &mut Velocity, &AabbCollider, &CameraComponent), With<FpsCamera>>,
) {
    if player.flying {
        return;
    }

    let delta = dt.0;

    for (mut transform, mut vel, collider, _cam) in query.iter_mut() {
        // Derive player dimensions from AabbCollider
        let half_w = collider.half_extents.x;
        let height = collider.half_extents.y * 2.0;
        let eye_offset = height - (height - config::EYE_OFFSET);

        // Gravity (when airborne)
        if !player.on_ground {
            vel.linear.y = (vel.linear.y - config::GRAVITY * delta).max(-config::TERMINAL_VELOCITY);
        }

        // Jump
        if player.on_ground && player.jump_requested {
            vel.linear.y = config::JUMP_VEL;
            player.on_ground = false;
        }
        player.jump_requested = false;

        // Cache vy for landing impact detection
        player.last_vy = vel.linear.y;

        let pos = transform.translation;

        // Safety: push out if embedded in a block
        if collides_aabb(&world, pos, half_w, height, eye_offset) {
            log::warn!(
                "Player embedded at ({:.2}, {:.2}, {:.2})! Pushing up.",
                pos.x, pos.y, pos.z
            );
            for _ in 0..5 {
                transform.translation.y += 1.0;
                if !collides_aabb(&world, transform.translation, half_w, height, eye_offset) {
                    break;
                }
            }
            vel.linear.y = 0.0;
            player.on_ground = false;
            continue;
        }

        // --- Y axis ---
        if vel.linear.y.abs() > 0.0001 {
            let new_y = pos.y + vel.linear.y * delta;
            let test_pos_y = glam::Vec3::new(pos.x, new_y, pos.z);
            if collides_aabb(&world, test_pos_y, half_w, height, eye_offset) {
                if vel.linear.y < 0.0 {
                    let feet_y = new_y - eye_offset;
                    let block_top = feet_y.floor() + 1.0;
                    transform.translation.y = block_top + eye_offset + 0.001;
                    player.on_ground = true;
                }
                vel.linear.y = 0.0;
            } else {
                transform.translation.y = new_y;
                player.on_ground = false;
            }
        } else if player.on_ground {
            let below = glam::Vec3::new(pos.x, pos.y - 0.05, pos.z);
            if !collides_aabb(&world, below, half_w, height, eye_offset) {
                player.on_ground = false;
                vel.linear.y = -config::GRAVITY * delta;
            }
        }

        // --- X axis ---
        let vel_x = vel.linear.x;
        if vel_x.abs() > 0.001 {
            let new_x = transform.translation.x + vel_x * delta;
            let test_pos_x = glam::Vec3::new(new_x, transform.translation.y, transform.translation.z);
            if !collides_aabb(&world, test_pos_x, half_w, height, eye_offset) {
                transform.translation.x = new_x;
            }
        }
        vel.linear.x = 0.0;

        // --- Z axis ---
        let vel_z = vel.linear.z;
        if vel_z.abs() > 0.001 {
            let new_z = transform.translation.z + vel_z * delta;
            let test_pos_z = glam::Vec3::new(transform.translation.x, transform.translation.y, new_z);
            if !collides_aabb(&world, test_pos_z, half_w, height, eye_offset) {
                transform.translation.z = new_z;
            }
        }
        vel.linear.z = 0.0;
    }
}
