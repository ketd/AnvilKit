use bevy_ecs::prelude::*;
use glam::Vec3;
use anvilkit_core::math::Transform;
use anvilkit_ecs::physics::Velocity;

use crate::components::{CueBall, NumberedBall};
use crate::resources::{BilliardConfig, GameState, GamePhase, BallTracker};

/// Check if all balls have stopped moving, transition game phase.
pub fn game_logic_system(
    config: Res<BilliardConfig>,
    mut game_state: ResMut<GameState>,
    mut tracker: ResMut<BallTracker>,
    mut cue_query: Query<(&mut Transform, &mut Velocity), With<CueBall>>,
    ball_query: Query<&Velocity, (With<NumberedBall>, Without<CueBall>)>,
) {
    if game_state.phase != GamePhase::BallsMoving {
        return;
    }

    // Check if any ball is still moving
    let min_vel_sq = config.min_velocity * config.min_velocity;
    let mut any_moving = false;

    if let Ok((_, vel)) = cue_query.get_single() {
        if vel.linear.length_squared() > min_vel_sq {
            any_moving = true;
        }
    }
    if !any_moving {
        for vel in &ball_query {
            if vel.linear.length_squared() > min_vel_sq {
                any_moving = true;
                break;
            }
        }
    }

    if any_moving {
        return;
    }

    // All balls stopped — process turn results

    // Score potted balls
    let player_idx = game_state.current_player as usize;
    let potted: Vec<u8> = game_state.potted_this_turn.clone();
    for ball_num in &potted {
        if *ball_num >= 1 && *ball_num <= 15 {
            game_state.player_scores[player_idx] += 1;
        }
    }

    // Check for 8-ball potted → game over
    if !tracker.on_table[8] {
        // The player who potted the 8-ball wins (simplified rules)
        game_state.winner = Some(game_state.current_player);
        game_state.phase = GamePhase::GameOver;
        return;
    }

    // Handle scratch: reset cue ball to starting position
    if game_state.is_scratch {
        if let Ok((mut t, mut v)) = cue_query.get_single_mut() {
            t.translation = Vec3::new(0.0, config.ball_radius, -config.table_half_depth * 0.5);
            v.linear = Vec3::ZERO;
        }
        // Restore cue ball tracking state
        tracker.on_table[0] = true;
    }

    // Switch player if no balls potted (or scratch)
    if game_state.potted_this_turn.is_empty() || game_state.is_scratch {
        game_state.current_player = 1 - game_state.current_player;
    }

    game_state.phase = GamePhase::Aiming;
}

/// Check if all numbered balls are potted → game over.
pub fn check_game_over_system(
    tracker: Res<BallTracker>,
    mut game_state: ResMut<GameState>,
) {
    if game_state.phase == GamePhase::GameOver {
        return;
    }

    // Count remaining balls (excluding cue ball at index 0)
    let remaining = tracker.on_table[1..].iter().filter(|&&on| on).count();
    if remaining == 0 {
        // All balls potted, current player wins
        game_state.winner = Some(game_state.current_player);
        game_state.phase = GamePhase::GameOver;
    }
}
