use bevy_ecs::prelude::*;
use anvilkit_core::math::Transform;
use anvilkit_ecs::physics::DeltaTime;
use anvilkit_input::prelude::{InputState, KeyCode};
use anvilkit_render::plugin::CameraComponent;

use crate::components::FpsCamera;
use crate::resources::{PlayerState, MouseDelta};

/// FPS camera: WASD + Space/Shift for vertical, mouse for look.
pub fn fps_camera_system(
    dt: Res<DeltaTime>,
    input: Res<InputState>,
    mut player: ResMut<PlayerState>,
    mouse_delta: Res<MouseDelta>,
    mut query: Query<(&mut Transform, &CameraComponent), With<FpsCamera>>,
) {
    // Mouse look
    player.yaw += mouse_delta.dx * player.mouse_sensitivity;
    player.pitch = (player.pitch + mouse_delta.dy * player.mouse_sensitivity)
        .clamp(-1.5, 1.5);

    let rotation = glam::Quat::from_rotation_y(player.yaw)
        * glam::Quat::from_rotation_x(player.pitch);

    // Movement: forward/right based on yaw only (no pitch influence on movement)
    let forward = glam::Quat::from_rotation_y(player.yaw) * glam::Vec3::Z;
    let right = glam::Quat::from_rotation_y(player.yaw) * glam::Vec3::X;

    let mut dir = glam::Vec3::ZERO;
    if input.is_key_pressed(KeyCode::W) { dir += forward; }
    if input.is_key_pressed(KeyCode::S) { dir -= forward; }
    if input.is_key_pressed(KeyCode::A) { dir -= right; }
    if input.is_key_pressed(KeyCode::D) { dir += right; }

    if player.flying {
        if input.is_key_pressed(KeyCode::Space) { dir.y += 1.0; }
        if input.is_key_pressed(KeyCode::LShift) { dir.y -= 1.0; }

        if dir.length_squared() > 0.0 {
            dir = dir.normalize();
        }

        let speed = player.move_speed * dt.0;

        for (mut transform, _cam) in query.iter_mut() {
            transform.translation += dir * speed;
            transform.rotation = rotation;
        }
    } else {
        // Non-flying: set horizontal velocity, physics system handles collision
        if dir.length_squared() > 0.0 {
            dir = dir.normalize();
        }
        player.velocity.x = dir.x * player.move_speed;
        player.velocity.z = dir.z * player.move_speed;

        // Jump request (actual jump handled by physics)
        if input.is_key_pressed(KeyCode::Space) {
            player.jump_requested = true;
        }

        for (mut transform, _cam) in query.iter_mut() {
            transform.rotation = rotation;
        }
    }
}
