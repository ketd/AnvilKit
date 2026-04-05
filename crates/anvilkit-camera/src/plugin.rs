//! # Camera Plugin
//!
//! Provides `CameraPlugin` to register camera controller systems.

use bevy_app::{App, Plugin, PostUpdate};
use crate::systems::{
    camera_input_system,
    camera_mode_system,
    camera_effects_apply_system,
};
use crate::orbit::rig::camera_rig_system;
use crate::orbit::spring_arm::camera_spring_arm_system;
use crate::constraints::rail::camera_rail_system;
use crate::constraints::look_at::camera_look_at_system;
use crate::effects::transition::camera_transition_system;

/// Camera plugin — registers the camera system pipeline.
///
/// Adds the following systems to [`PostUpdate`] in order:
/// 1. [`camera_input_system`] — Reads mouse/keyboard, updates yaw/pitch/zoom
/// 2. [`camera_mode_system`] — Computes position/rotation per mode
/// 3. [`camera_effects_apply_system`] — Applies shake, bob, FOV offsets
///
/// # Example
///
/// ```rust,no_run
/// use bevy_app::App;
/// use anvilkit_camera::plugin::CameraPlugin;
///
/// App::new()
///     .add_plugins(CameraPlugin);
/// ```
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        // Register camera systems in order within PostUpdate.
        // Pipeline: rig → input → rail → mode → spring_arm → look_at → effects → transition
        app.add_systems(PostUpdate, camera_rig_system);
        app.add_systems(PostUpdate, camera_input_system);
        app.add_systems(PostUpdate, camera_rail_system);
        app.add_systems(PostUpdate, camera_mode_system);
        app.add_systems(PostUpdate, camera_spring_arm_system);
        app.add_systems(PostUpdate, camera_look_at_system);
        app.add_systems(PostUpdate, camera_effects_apply_system);
        app.add_systems(PostUpdate, camera_transition_system);
    }

    fn name(&self) -> &str {
        "CameraPlugin"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_plugin_name() {
        let plugin = CameraPlugin;
        assert_eq!(plugin.name(), "CameraPlugin");
    }

    #[test]
    fn test_camera_plugin_is_unique() {
        let plugin = CameraPlugin;
        assert!(plugin.is_unique());
    }

    #[test]
    fn test_camera_plugin_build() {
        use anvilkit_core::time::DeltaTime;
        use anvilkit_input::prelude::InputState;

        let mut app = App::new();
        app.insert_resource(DeltaTime::default());
        app.insert_resource(InputState::default());
        app.add_plugins(CameraPlugin);
        app.update();
    }
}
