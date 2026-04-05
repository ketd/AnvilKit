use bevy_ecs::prelude::*;
use anvilkit_core::math::Transform;
use anvilkit_core::time::DeltaTime;
use anvilkit_core::math::Velocity;
use anvilkit_render::transform::AabbCollider;
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
        // Clear physics_pos while flying so the first walk frame reads from transform
        player.physics_pos = None;
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

        // Use physics-authoritative position to avoid head bob feedback
        let pos = player.physics_pos.unwrap_or(transform.translation);

        // Void fall safety: teleport to spawn if fallen below world
        if pos.y < -64.0 {
            transform.translation = glam::Vec3::new(
                (crate::chunk::CHUNK_SIZE as f32) * 3.5,
                50.0,
                (crate::chunk::CHUNK_SIZE as f32) * 3.5,
            );
            vel.linear.y = 0.0;
            player.on_ground = false;
            player.physics_pos = Some(transform.translation);
            log::warn!("Player fell into void, teleporting to spawn");
            continue;
        }

        // Sync transform from physics_pos (undo any head bob offset from previous frame)
        transform.translation = pos;

        // Safety: push out if embedded in a block
        if collides_aabb(&world, pos, half_w, height, eye_offset) {
            log::warn!(
                "Player embedded at ({:.2}, {:.2}, {:.2})! Pushing up.",
                pos.x, pos.y, pos.z
            );
            for _ in 0..10 {
                transform.translation.y += 1.0;
                if !collides_aabb(&world, transform.translation, half_w, height, eye_offset) {
                    break;
                }
            }
            vel.linear.y = 0.0;
            player.on_ground = false;
            player.physics_pos = Some(transform.translation);
            continue;
        }

        // --- Y axis ---
        if vel.linear.y.abs() > 0.0001 {
            let new_y = pos.y + vel.linear.y * delta;
            let test_pos_y = glam::Vec3::new(pos.x, new_y, pos.z);
            if collides_aabb(&world, test_pos_y, half_w, height, eye_offset) {
                if vel.linear.y < 0.0 {
                    // Landing: snap feet to the top of the block we hit.
                    // Must use new_y (the fallen position) to find the correct block.
                    let feet_y = new_y - eye_offset;
                    let block_top = feet_y.floor() + 1.0;
                    // Tiny epsilon (0.0005) keeps feet strictly above the integer boundary
                    // to avoid float-precision collide-with-supporting-block issues.
                    transform.translation.y = block_top + eye_offset + 0.0005;
                    player.on_ground = true;
                }
                vel.linear.y = 0.0;
            } else {
                transform.translation.y = new_y;
                player.on_ground = false;
            }
        } else if player.on_ground {
            // Stable ground check: query blocks directly below the player's footprint.
            // This replaces the old AABB-probe approach which caused per-frame oscillation
            // at block boundaries (probe shifts the AABB into/out of collision each frame).
            let feet_y = pos.y - eye_offset;
            let support_y = (feet_y - 0.01).floor() as i32;

            // Shrink footprint slightly (0.01 inset) to avoid edge-case support from
            // blocks the player is barely touching.
            let min_bx = (pos.x - half_w + 0.01).floor() as i32;
            let max_bx = (pos.x + half_w - 0.01).floor() as i32;
            let min_bz = (pos.z - half_w + 0.01).floor() as i32;
            let max_bz = (pos.z + half_w - 0.01).floor() as i32;

            let mut supported = false;
            'support: for bx in min_bx..=max_bx {
                for bz in min_bz..=max_bz {
                    if world.get_block(bx, support_y, bz).is_obstacle() {
                        supported = true;
                        break 'support;
                    }
                }
            }

            if !supported {
                player.on_ground = false;
                // Don't apply a gravity impulse here — the normal gravity code at the
                // top of the loop handles it on the next frame. This avoids the
                // "kick → snap → kick" oscillation the old code had.
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

        // Store physics-authoritative position (before head bob/shake is applied in PostUpdate)
        player.physics_pos = Some(transform.translation);
    }
}
