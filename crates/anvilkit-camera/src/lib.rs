#![warn(missing_docs)]
//! # AnvilKit Camera
//!
//! Camera controller, effects, and supporting systems for the AnvilKit engine.
//!
//! ## Module Structure
//!
//! ```text
//! anvilkit_camera
//! ├── controller       — CameraMode + CameraController
//! ├── input_curve      — Dead zone + response curve
//! ├── systems          — Core ECS systems (input, mode, effects)
//! ├── plugin           — CameraPlugin registration
//! ├── orbit/           — Orbit subsystem
//! │   ├── OrbitState   — Distance, target, limits
//! │   ├── rig          — Entity follow with offset + damping
//! │   └── spring_arm   — Collision avoidance
//! ├── effects/         — Visual effects subsystem
//! │   ├── CameraEffects — Trauma shake, head bob, FOV
//! │   ├── noise        — Perlin gradient noise
//! │   └── transition   — Smooth camera blending
//! └── constraints/     — Camera constraints
//!     ├── look_at      — Soft look-at with dead zone
//!     └── rail         — Dolly/path camera
//! ```

/// Camera controller component and control modes.
pub mod controller;
/// Input curve utilities (dead zone, response curve).
pub mod input_curve;
/// Camera plugin for engine integration.
pub mod plugin;
/// Camera controller ECS systems.
pub mod systems;

/// Orbit subsystem: orbit state, entity rig, spring arm collision.
pub mod orbit;
/// Effects subsystem: trauma shake, noise, transitions.
pub mod effects;
/// Constraints subsystem: look-at, rail/dolly.
pub mod constraints;

/// Prelude module re-exporting the most commonly used types.
pub mod prelude {
    pub use crate::controller::{CameraMode, CameraController};
    pub use crate::effects::CameraEffects;
    pub use crate::effects::transition::{CameraTransition, EasingType};
    pub use crate::input_curve::InputCurve;
    pub use crate::orbit::OrbitState;
    pub use crate::orbit::rig::CameraRig;
    pub use crate::orbit::spring_arm::SpringArm;
    pub use crate::constraints::look_at::LookAtTarget;
    pub use crate::constraints::rail::{CameraRail, RailInterpolation};
    pub use crate::plugin::CameraPlugin;
    pub use crate::systems::{
        camera_input_system,
        camera_mode_system,
        camera_effects_apply_system,
        camera_controller_system,
    };
}
