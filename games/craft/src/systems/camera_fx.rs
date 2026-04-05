use bevy_ecs::prelude::*;
use anvilkit_camera::controller::{CameraController, CameraMode};
use anvilkit_camera::effects::CameraEffects;
use anvilkit_camera::orbit::OrbitState;
use anvilkit_render::renderer::draw::ActiveCamera;
use crate::config;
use crate::resources::PlayerState;

/// Drives camera effects (landing shake, sprint FOV) and updates
/// third-person camera target each frame.
pub fn camera_effects_system(
    active_cam: Option<Res<ActiveCamera>>,
    mut player: ResMut<PlayerState>,
    mut query: Query<(&CameraController, &mut CameraEffects, Option<&mut OrbitState>)>,
) {
    let cam_pos = active_cam
        .as_ref()
        .map(|c| c.camera_pos)
        .unwrap_or(glam::Vec3::ZERO);

    for (ctrl, mut fx, orbit) in query.iter_mut() {
        // Update third-person camera target via OrbitState
        if ctrl.mode == CameraMode::ThirdPerson {
            if let Some(mut orbit) = orbit {
                orbit.target = cam_pos;
            }
        }

        // Landing shake — only trigger on significant falls (not from ground check wobble)
        if player.on_ground && !player.was_on_ground {
            let fall_speed = -player.last_vy;
            // Require substantial fall velocity before shaking (above jump threshold)
            if fall_speed > config::JUMP_VEL {
                let impact = ((fall_speed - config::JUMP_VEL) * 0.02).clamp(0.0, 0.3);
                fx.add_shake(impact);
            }
        }

        // Sprint FOV
        fx.fov_target = if player.sprinting { 10.0 } else { 0.0 };
    }

    // Update was_on_ground tracker (must happen after fall_damage_system reads it)
    player.was_on_ground = player.on_ground;
}
