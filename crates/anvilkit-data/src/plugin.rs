//! DataTablePlugin — registers data resources into ECS.

use bevy_ecs::prelude::*;
use crate::locale::Locale;

/// Plugin that initializes data/i18n resources.
pub struct DataTablePlugin;

impl DataTablePlugin {
    /// Register data resources into the ECS world.
    pub fn build(world: &mut World) {
        world.insert_resource(Locale::default());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_registers_locale() {
        let mut world = World::new();
        DataTablePlugin::build(&mut world);
        assert!(world.get_resource::<Locale>().is_some());
    }
}
