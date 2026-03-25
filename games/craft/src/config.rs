//! Game configuration constants.

// --- World ---

/// Chunk load/unload radius (in chunks).
pub const LOAD_RADIUS: i32 = 7;

/// Water surface Y level.
pub const WATER_LEVEL: i32 = 28;

/// Height of tree trunks (in blocks).
pub const TRUNK_HEIGHT: usize = 5;

/// Cloud layer Y range.
pub const CLOUD_Y_MIN: usize = 64;
pub const CLOUD_Y_MAX: usize = 67;

// --- Camera / Rendering ---

/// Vertical field of view (degrees).
pub const FOV: f32 = 70.0;

/// Near clip plane distance.
pub const NEAR_PLANE: f32 = 0.1;

/// Far clip plane distance.
pub const FAR_PLANE: f32 = 500.0;

// --- Gameplay ---

/// Maximum raycast distance for block interaction.
pub const RAYCAST_MAX_DIST: f32 = 10.0;

/// Number of dirty chunks to remesh per frame (budget).
pub const REMESH_BUDGET: usize = 4;

// --- Physics ---

pub const GRAVITY: f32 = 20.0;
pub const JUMP_VEL: f32 = 8.0;
pub const TERMINAL_VELOCITY: f32 = 50.0;
pub const PLAYER_WIDTH: f32 = 0.6;
pub const PLAYER_HEIGHT: f32 = 1.8;
pub const EYE_OFFSET: f32 = 1.6;
pub const SPRINT_MULTIPLIER: f32 = 1.5;

// --- World generation ---

pub const NOISE_HEIGHT_SCALE: f64 = 0.005;
pub const NOISE_DETAIL_SCALE: f64 = 0.01;
pub const BASE_HEIGHT: f64 = 32.0;
pub const HEIGHT_AMP1: f64 = 16.0;
pub const HEIGHT_AMP2: f64 = 8.0;
pub const TREE_NOISE_SCALE: f64 = 0.3;
pub const TREE_THRESHOLD: f64 = 0.84;
pub const LEAF_RADIUS: i32 = 3;
pub const CAVE_NOISE_SCALE: f64 = 0.05;
pub const CAVE_THRESHOLD: f64 = 0.3;
pub const CLOUD_NOISE_SCALE: f64 = 0.01;
pub const CLOUD_THRESHOLD: f64 = 0.75;

// --- Other ---

pub const DAY_NIGHT_DURATION: f32 = 600.0;
pub const DEFAULT_SEED: u32 = 42;
pub const MAX_WORKER_THREADS: usize = 8;
