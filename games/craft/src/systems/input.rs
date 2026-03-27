use bevy_ecs::prelude::*;
use anvilkit_core::math::Transform;
use anvilkit_ecs::physics::{DeltaTime, Velocity};
use anvilkit_input::prelude::{InputState, KeyCode};
use anvilkit_camera::prelude::CameraController;

use crate::components::FpsCamera;
use crate::config;
use crate::resources::PlayerState;

/// Player movement system: WASD + Space/Shift for vertical movement.
/// Camera rotation is handled by camera_controller_system in anvilkit-camera.
pub fn player_movement_system(
    dt: Res<DeltaTime>,
    input: Res<InputState>,
    mut player: ResMut<PlayerState>,
    mut query: Query<(&CameraController, &mut Transform, &mut Velocity), With<FpsCamera>>,
) {
    let Ok((ctrl, mut transform, mut vel)) = query.get_single_mut() else { return };

    // Movement direction based on camera yaw (no pitch influence)
    let forward = ctrl.forward_xz();
    let right = ctrl.right_xz();

    let mut dir = glam::Vec3::ZERO;
    if input.is_key_pressed(KeyCode::W) { dir += forward; }
    if input.is_key_pressed(KeyCode::S) { dir -= forward; }
    if input.is_key_pressed(KeyCode::A) { dir -= right; }
    if input.is_key_pressed(KeyCode::D) { dir += right; }

    // Sprint detection
    player.sprinting = input.is_key_pressed(KeyCode::LControl)
        && (input.is_key_pressed(KeyCode::W)
            || input.is_key_pressed(KeyCode::S)
            || input.is_key_pressed(KeyCode::A)
            || input.is_key_pressed(KeyCode::D));

    let speed_multiplier = if player.sprinting { config::SPRINT_MULTIPLIER } else { 1.0 };

    if player.flying {
        if input.is_key_pressed(KeyCode::Space) { dir.y += 1.0; }
        if input.is_key_pressed(KeyCode::LShift) { dir.y -= 1.0; }

        if dir.length_squared() > 0.0 {
            dir = dir.normalize();
        }

        let speed = player.move_speed * speed_multiplier * dt.0;
        transform.translation += dir * speed;
    } else {
        // Non-flying: set horizontal velocity, physics system handles collision
        if dir.length_squared() > 0.0 {
            dir = dir.normalize();
        }
        vel.linear.x = dir.x * player.move_speed * speed_multiplier;
        vel.linear.z = dir.z * player.move_speed * speed_multiplier;

        // Jump request (actual jump handled by physics)
        if input.is_key_pressed(KeyCode::Space) {
            player.jump_requested = true;
        }
    }
}
