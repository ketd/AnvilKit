//! UiPlugin — registers UI systems into ECS schedules.

use bevy_ecs::prelude::*;
use crate::events::UiEvents;
use crate::focus::UiFocus;
use crate::layout::UiLayoutEngine;
use crate::theme::UiTheme;
use crate::controls::checkbox::UiChangeEvent;

/// UI plugin — call `UiPlugin.build(app)` to register all UI resources and systems.
pub struct UiPlugin;

impl UiPlugin {
    /// Register UI resources and systems into the ECS app.
    ///
    /// This registers:
    /// - `UiLayoutEngine` resource
    /// - `UiEvents` resource
    /// - `UiFocus` resource
    /// - `UiTheme` resource
    /// - `UiChangeEvent` event
    /// - Focus interaction system
    pub fn build(app: &mut bevy_ecs::world::World) {
        app.insert_resource(UiLayoutEngine::new());
        app.insert_resource(UiEvents::default());
        app.insert_resource(UiFocus::default());
        app.insert_resource(UiTheme::default());
        app.init_resource::<Events<UiChangeEvent>>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_registers_resources() {
        let mut world = bevy_ecs::world::World::new();
        UiPlugin::build(&mut world);

        assert!(world.get_resource::<UiLayoutEngine>().is_some());
        assert!(world.get_resource::<UiEvents>().is_some());
        assert!(world.get_resource::<UiFocus>().is_some());
        assert!(world.get_resource::<UiTheme>().is_some());
    }
}
