use bevy_ecs::prelude::*;
use anvilkit_camera::controller::{CameraController, CameraMode};
use anvilkit_camera::effects::CameraEffects;
use anvilkit_render::renderer::draw::ActiveCamera;
use crate::resources::PlayerState;

/// Drives camera effects (landing shake, sprint FOV) and updates
/// third-person camera target each frame. Runs before `camera_controller_system`.
pub fn camera_effects_system(
    active_cam: Option<Res<ActiveCamera>>,
    mut player: ResMut<PlayerState>,
    mut query: Query<(&mut CameraController, &mut CameraEffects)>,
) {
    let cam_pos = active_cam
        .as_ref()
        .map(|c| c.camera_pos)
        .unwrap_or(glam::Vec3::ZERO);

    for (mut ctrl, mut fx) in query.iter_mut() {
        // Update third-person camera target
        if let CameraMode::ThirdPerson { ref mut target, .. } = &mut ctrl.mode {
            *target = cam_pos;
        }

        // Landing shake
        if player.on_ground && !player.was_on_ground {
            let impact = (-player.velocity.y * 0.02).clamp(0.0, 0.3);
            if impact > 0.02 {
                fx.add_shake(impact);
            }
        }

        // Sprint FOV
        fx.fov_target = if player.sprinting { 10.0 } else { 0.0 };
    }

    // Update was_on_ground tracker
    player.was_on_ground = player.on_ground;
}
