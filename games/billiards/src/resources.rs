use bevy_ecs::prelude::*;
use glam::Vec3;

/// Game configuration constants.
#[derive(Resource, Clone)]
pub struct BilliardConfig {
    pub ball_radius: f32,
    pub table_half_width: f32,
    pub table_half_depth: f32,
    pub cushion_restitution: f32,
    pub ball_restitution: f32,
    pub rolling_friction: f32,
    pub min_velocity: f32,
    pub max_shot_power: f32,
    pub pocket_radius: f32,
    pub pocket_positions: Vec<Vec3>,
}

impl Default for BilliardConfig {
    fn default() -> Self {
        // 2:1 ratio table, sized so 15-ball rack fits comfortably
        let hw = 5.0f32;  // half-width (X)
        let hd = 2.5f32;  // half-depth (Z)
        Self {
            ball_radius: 0.2,
            table_half_width: hw,
            table_half_depth: hd,
            cushion_restitution: 0.7,
            ball_restitution: 0.95,
            rolling_friction: 1.5,
            min_velocity: 0.01,
            max_shot_power: 15.0,
            pocket_radius: 0.35,
            pocket_positions: vec![
                // 4 corners
                Vec3::new(-hw, 0.0, -hd),
                Vec3::new( hw, 0.0, -hd),
                Vec3::new(-hw, 0.0,  hd),
                Vec3::new( hw, 0.0,  hd),
                // 2 side pockets (middle of long rails)
                Vec3::new(-hw, 0.0, 0.0),
                Vec3::new( hw, 0.0, 0.0),
            ],
        }
    }
}

/// Game phase state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GamePhase {
    Aiming,
    PowerCharging,
    BallsMoving,
    BallsStopped,
    GameOver,
}

/// Overall game state.
#[derive(Resource)]
pub struct GameState {
    pub phase: GamePhase,
    pub current_player: u8,
    pub player_scores: [u32; 2],
    pub is_scratch: bool,
    pub potted_this_turn: Vec<u8>,
    pub winner: Option<u8>,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            phase: GamePhase::Aiming,
            current_player: 0,
            player_scores: [0, 0],
            is_scratch: false,
            potted_this_turn: Vec::new(),
            winner: None,
        }
    }
}

/// Shot aiming state.
#[derive(Resource)]
pub struct ShotState {
    pub aim_point: Vec3,
    pub aim_direction: Vec3,
    pub power: f32,
    pub aim_valid: bool,
    pub charge_time: f32,
}

impl Default for ShotState {
    fn default() -> Self {
        Self {
            aim_point: Vec3::ZERO,
            aim_direction: Vec3::NEG_Z,
            power: 0.0,
            aim_valid: false,
            charge_time: 0.0,
        }
    }
}

/// Tracks all ball entities and their on-table status.
#[derive(Resource)]
pub struct BallTracker {
    pub ball_entities: Vec<Entity>,
    pub on_table: [bool; 16],
}

impl Default for BallTracker {
    fn default() -> Self {
        Self {
            ball_entities: Vec::new(),
            on_table: [true; 16],
        }
    }
}
