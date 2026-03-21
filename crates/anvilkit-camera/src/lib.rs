#![warn(missing_docs)]

pub mod controller;
pub mod effects;
pub mod systems;

pub mod prelude {
    pub use crate::controller::{CameraMode, CameraController};
    pub use crate::effects::CameraEffects;
    pub use crate::systems::camera_controller_system;
}
