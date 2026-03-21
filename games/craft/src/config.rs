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
