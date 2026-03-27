#![warn(missing_docs)]
//! # AnvilKit Camera
//!
//! Camera controller and effects system for the AnvilKit engine.

/// Camera controller component and control modes.
pub mod controller;
/// Camera visual effects (head bob, FOV shifts).
pub mod effects;
/// Camera plugin for engine integration.
pub mod plugin;
/// Camera controller ECS systems.
pub mod systems;

/// Prelude module re-exporting the most commonly used types.
pub mod prelude {
    pub use crate::controller::{CameraMode, CameraController};
    pub use crate::effects::CameraEffects;
    pub use crate::plugin::CameraPlugin;
    pub use crate::systems::camera_controller_system;
}
