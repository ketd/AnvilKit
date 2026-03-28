use bevy_ecs::prelude::*;
use anvilkit_core::math::Transform;
use anvilkit_ecs::physics::{DeltaTime, Velocity};
use anvilkit_input::prelude::ActionMap;
use anvilkit_camera::prelude::CameraController;

use crate::components::FpsCamera;
use crate::config;
use crate::resources::{PlayerState, SelectedBlock};
use crate::render::filters::ActiveFilter;

/// Player movement system: reads ActionMap for WASD + Space/Shift + Sprint.
/// Camera rotation is handled by camera_controller_system in anvilkit-camera.
pub fn player_movement_system(
    dt: Res<DeltaTime>,
    actions: Res<ActionMap>,
    mut player: ResMut<PlayerState>,
    mut query: Query<(&CameraController, &mut Transform, &mut Velocity), With<FpsCamera>>,
) {
    let Ok((ctrl, mut transform, mut vel)) = query.get_single_mut() else { return };

    // Movement direction based on camera yaw (no pitch influence)
    let forward = ctrl.forward_xz();
    let right = ctrl.right_xz();

    let mut dir = glam::Vec3::ZERO;
    if actions.is_action_active("move_forward")  { dir += forward; }
    if actions.is_action_active("move_backward") { dir -= forward; }
    if actions.is_action_active("move_left")     { dir -= right; }
    if actions.is_action_active("move_right")    { dir += right; }

    // Sprint detection
    let moving = actions.is_action_active("move_forward")
        || actions.is_action_active("move_backward")
        || actions.is_action_active("move_left")
        || actions.is_action_active("move_right");
    player.sprinting = actions.is_action_active("sprint") && moving;

    let speed_multiplier = if player.sprinting { config::SPRINT_MULTIPLIER } else { 1.0 };

    if player.flying {
        if actions.is_action_active("jump")    { dir.y += 1.0; }
        if actions.is_action_active("descend") { dir.y -= 1.0; }

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
        if actions.is_action_active("jump") {
            player.jump_requested = true;
        }
    }
}

/// Hotbar slot selection via ActionMap (replaces digit key handling in on_window_event).
pub fn hotbar_selection_system(
    actions: Res<ActionMap>,
    mut selected: ResMut<SelectedBlock>,
) {
    for i in 0..9 {
        let action = format!("slot_{}", i + 1);
        if actions.is_action_just_pressed(&action) {
            selected.index = i;
            selected.block_type = config::BLOCK_PALETTE[i];
        }
    }
}

/// Toggle actions: flying mode and post-processing filter cycle.
pub fn toggle_actions_system(
    actions: Res<ActionMap>,
    mut player: ResMut<PlayerState>,
    mut filter: Option<ResMut<ActiveFilter>>,
) {
    if actions.is_action_just_pressed("toggle_flying") {
        player.flying = !player.flying;
    }
    if actions.is_action_just_pressed("cycle_filter") {
        if let Some(ref mut af) = filter {
            af.filter = af.filter.cycle();
        }
    }
}
