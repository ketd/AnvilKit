use bevy_ecs::prelude::*;
use glam::Vec3;
use anvilkit_core::math::Transform;
use anvilkit_core::math::Velocity;
use anvilkit_core::time::DeltaTime;
use anvilkit_input::prelude::{InputState, MouseButton};
use anvilkit_render::renderer::raycast::{screen_to_ray, ray_plane_intersection};
use anvilkit_render::renderer::draw::ActiveCamera;

use crate::components::CueBall;
use crate::resources::{BilliardConfig, GameState, GamePhase, ShotState};

/// Mouse aiming system: converts mouse position to aim direction on the table.
pub fn aim_input_system(
    input: Res<InputState>,
    cam: Res<ActiveCamera>,
    config: Res<BilliardConfig>,
    game_state: Res<GameState>,
    mut shot: ResMut<ShotState>,
    win_size: Option<Res<WindowSize>>,
    cue_query: Query<&Transform, With<CueBall>>,
) {
    if game_state.phase != GamePhase::Aiming && game_state.phase != GamePhase::PowerCharging {
        return;
    }

    let Ok(cue_transform) = cue_query.get_single() else { return };
    let cue_pos = cue_transform.translation;

    let mouse_pos = input.mouse_position();
    let win = win_size.map(|w| glam::Vec2::new(w.width, w.height))
        .unwrap_or(glam::Vec2::new(1280.0, 720.0));
    let window_size = win;

    let (origin, dir) = screen_to_ray(mouse_pos, window_size, &cam.view_proj);

    // Intersect with the table plane (y = ball_radius)
    if let Some(hit) = ray_plane_intersection(origin, dir, config.ball_radius) {
        // Check if hit is within table bounds
        let hw = config.table_half_width;
        let hd = config.table_half_depth;
        if hit.x.abs() <= hw && hit.z.abs() <= hd {
            shot.aim_point = hit;
            let aim_dir = hit - cue_pos;
            let aim_xz = Vec3::new(aim_dir.x, 0.0, aim_dir.z);
            if aim_xz.length_squared() > 0.001 {
                shot.aim_direction = aim_xz.normalize();
                shot.aim_valid = true;
            } else {
                shot.aim_valid = false;
            }
        } else {
            shot.aim_valid = false;
        }
    } else {
        shot.aim_valid = false;
    }
}

/// Shot execution: mouse press starts charging, release fires.
pub fn shot_execution_system(
    input: Res<InputState>,
    dt: Res<DeltaTime>,
    config: Res<BilliardConfig>,
    mut game_state: ResMut<GameState>,
    mut shot: ResMut<ShotState>,
    mut cue_query: Query<&mut Velocity, With<CueBall>>,
) {
    match game_state.phase {
        GamePhase::Aiming => {
            // Start charging on mouse press
            if input.is_mouse_just_pressed(MouseButton::Left) && shot.aim_valid {
                game_state.phase = GamePhase::PowerCharging;
                shot.charge_time = 0.0;
                shot.power = 0.0;
            }
        }
        GamePhase::PowerCharging => {
            if input.is_mouse_pressed(MouseButton::Left) {
                // Accumulate power
                shot.charge_time += dt.0;
                shot.power = (shot.charge_time / 2.0).min(1.0);
            } else {
                // Released: fire the shot
                if let Ok(mut vel) = cue_query.get_single_mut() {
                    let speed = shot.power * config.max_shot_power;
                    vel.linear = shot.aim_direction * speed;
                }
                shot.power = 0.0;
                shot.charge_time = 0.0;
                game_state.phase = GamePhase::BallsMoving;
                game_state.potted_this_turn.clear();
                game_state.is_scratch = false;
            }
        }
        _ => {}
    }
}

// WindowSize is now provided by anvilkit_app::WindowSize (inserted by the runner).
pub use anvilkit_app::WindowSize;
