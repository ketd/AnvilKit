use crate::resources::{GameState, GamePhase, ShotState};

/// Format the HUD text for the current game state.
pub fn format_status_text(game_state: &GameState, shot: &ShotState) -> String {
    match game_state.phase {
        GamePhase::Aiming => {
            format!(
                "Player {} | P1:{} P2:{} | Aim and click to charge",
                game_state.current_player + 1,
                game_state.player_scores[0],
                game_state.player_scores[1],
            )
        }
        GamePhase::PowerCharging => {
            let pct = (shot.power * 100.0) as u32;
            format!(
                "Player {} | Power: {}% | Release to shoot",
                game_state.current_player + 1,
                pct,
            )
        }
        GamePhase::BallsMoving => {
            format!(
                "Player {} | P1:{} P2:{} | Balls moving...",
                game_state.current_player + 1,
                game_state.player_scores[0],
                game_state.player_scores[1],
            )
        }
        GamePhase::BallsStopped => {
            format!(
                "Player {} | P1:{} P2:{}",
                game_state.current_player + 1,
                game_state.player_scores[0],
                game_state.player_scores[1],
            )
        }
        GamePhase::GameOver => {
            if let Some(winner) = game_state.winner {
                format!("GAME OVER - Player {} wins! P1:{} P2:{}",
                    winner + 1,
                    game_state.player_scores[0],
                    game_state.player_scores[1],
                )
            } else {
                "GAME OVER".to_string()
            }
        }
    }
}
