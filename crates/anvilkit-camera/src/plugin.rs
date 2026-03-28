//! # Camera Plugin
//!
//! Provides `CameraPlugin` to register camera controller systems.

use anvilkit_ecs::plugin::Plugin;
use anvilkit_ecs::app::App;
use anvilkit_ecs::schedule::AnvilKitSchedule;
#[cfg(feature = "persistence")]
use bevy_ecs::prelude::*;

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
#[cfg(feature = "persistence")]
use crate::controller::CameraController;

/// Camera plugin — registers the camera system pipeline.
///
/// Adds the following systems to [`AnvilKitSchedule::PostUpdate`] in order:
/// 1. [`camera_input_system`] — Reads mouse/keyboard, updates yaw/pitch/zoom
/// 2. [`camera_mode_system`] — Computes position/rotation per mode
/// 3. [`camera_effects_apply_system`] — Applies shake, bob, FOV offsets
///
/// With the `persistence` feature, also adds a `PreUpdate` system that syncs
/// mouse sensitivity from [`Settings`](anvilkit_core::persistence::Settings).
///
/// # Example
///
/// ```rust,no_run
/// use anvilkit_ecs::prelude::*;
/// use anvilkit_camera::plugin::CameraPlugin;
///
/// App::new()
///     .add_plugins(AnvilKitEcsPlugin)
///     .add_plugins(CameraPlugin);
/// ```
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        // Register camera systems in order within PostUpdate.
        // Pipeline: rig → input → rail → mode → spring_arm → look_at → effects → transition
        app.add_systems(AnvilKitSchedule::PostUpdate, camera_rig_system);
        app.add_systems(AnvilKitSchedule::PostUpdate, camera_input_system);
        app.add_systems(AnvilKitSchedule::PostUpdate, camera_rail_system);
        app.add_systems(AnvilKitSchedule::PostUpdate, camera_mode_system);
        app.add_systems(AnvilKitSchedule::PostUpdate, camera_spring_arm_system);
        app.add_systems(AnvilKitSchedule::PostUpdate, camera_look_at_system);
        app.add_systems(AnvilKitSchedule::PostUpdate, camera_effects_apply_system);
        app.add_systems(AnvilKitSchedule::PostUpdate, camera_transition_system);

        #[cfg(feature = "persistence")]
        app.add_systems(AnvilKitSchedule::PreUpdate, camera_settings_sync_system);
    }

    fn name(&self) -> &str {
        "CameraPlugin"
    }
}

/// Syncs `Settings.input.mouse_sensitivity` into every [`CameraController`].
#[cfg(feature = "persistence")]
fn camera_settings_sync_system(
    settings: Option<Res<anvilkit_core::persistence::Settings>>,
    mut query: Query<&mut CameraController>,
) {
    let Some(settings) = settings else { return };
    for mut cc in &mut query {
        cc.mouse_sensitivity = settings.input.mouse_sensitivity;
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
        use anvilkit_ecs::physics::DeltaTime;
        use anvilkit_input::prelude::InputState;

        let mut app = App::new();
        app.insert_resource(DeltaTime::default());
        app.insert_resource(InputState::default());
        app.add_plugins(CameraPlugin);
        app.update();
    }
}
