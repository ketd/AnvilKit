pub mod theme;
pub mod main_menu;
pub mod pause_menu;
pub mod settings_menu;
pub mod inventory;
pub mod screen_dispatch;

use bevy_ecs::system::Resource;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CraftScreen {
    #[default]
    MainMenu,
    Playing,
    Paused,
    Inventory,
    Settings,
    /// Transient: triggers app exit.
    Quit,
    /// Transient: triggers save then return to main menu.
    SaveAndQuit,
}

/// Tracks which screen Settings should return to on "Back".
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsReturnTo {
    MainMenu,
    Paused,
}

impl Default for SettingsReturnTo {
    fn default() -> Self {
        Self::MainMenu
    }
}
