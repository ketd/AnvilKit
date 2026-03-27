//! # Camera Plugin
//!
//! Provides `CameraPlugin` to register camera controller systems.

use anvilkit_ecs::plugin::Plugin;
use anvilkit_ecs::app::App;
use anvilkit_ecs::schedule::AnvilKitSchedule;

use crate::systems::camera_controller_system;

/// Camera plugin — registers camera controller systems.
///
/// Adds the [`camera_controller_system`] to the [`AnvilKitSchedule::PostUpdate`]
/// schedule so that camera transforms are refreshed after game-logic updates.
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
        app.add_systems(AnvilKitSchedule::PostUpdate, camera_controller_system);
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
        use anvilkit_ecs::physics::DeltaTime;
        use anvilkit_input::prelude::InputState;

        let mut app = App::new();
        app.insert_resource(DeltaTime::default());
        app.insert_resource(InputState::default());
        app.add_plugins(CameraPlugin);
        // Plugin should register without panicking; the system is added to PostUpdate.
        // Run an update cycle to verify no scheduling errors.
        app.update();
    }
}
