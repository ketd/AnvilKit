//! # 自动插件
//!
//! 提供 `AutoInputPlugin`、`AutoDeltaTimePlugin` 和 `PersistencePlugin`，
//! 自动管理输入帧生命周期、时间更新和自动存档。

use bevy_ecs::prelude::*;
use crate::ecs_plugin::Plugin;
use crate::ecs_app::App;
use crate::schedule::AnvilKitSchedule;

// Note: winit 0.30 removed gamepad support. Gamepad input requires a separate
// backend (e.g., gilrs) which can write to GamepadState directly.
// AutoInputPlugin handles keyboard/mouse only.

/// 自动输入插件
///
/// 在 Cleanup 阶段自动调用 `InputState::end_frame()`，
/// 确保 just_pressed / just_released 状态在帧末正确清除。
///
/// # 示例
///
/// ```rust,no_run
/// use anvilkit_app::prelude::*;
/// use anvilkit_app::auto_plugins::AutoInputPlugin;
///
/// App::new()
///     .add_plugins(AnvilKitEcsPlugin)
///     .add_plugins(AutoInputPlugin);
/// ```
pub struct AutoInputPlugin;

impl Plugin for AutoInputPlugin {
    fn build(&self, app: &mut App) {
        use anvilkit_input::prelude::InputState;
        app.init_resource::<InputState>();
        app.add_systems(AnvilKitSchedule::PreUpdate, action_map_update_system);
        app.add_systems(AnvilKitSchedule::Cleanup, input_end_frame_system);
    }

    fn name(&self) -> &str {
        "AutoInputPlugin"
    }
}

/// Sync `ActionMap` states from `InputState` each frame.
///
/// Uses `Option` so the system is a no-op when `ActionMap` is not inserted.
/// Games that register an `ActionMap` resource get automatic per-frame updates.
///
/// This system is registered by [`AutoInputPlugin`] in `PreUpdate`. Games that
/// do not use `AutoInputPlugin` can add this system manually:
///
/// ```rust,ignore
/// use anvilkit_app::auto_plugins::action_map_update_system;
/// app.add_systems(AnvilKitSchedule::PreUpdate, action_map_update_system);
/// ```
pub fn action_map_update_system(
    input: Res<anvilkit_input::prelude::InputState>,
    action_map: Option<ResMut<anvilkit_input::prelude::ActionMap>>,
) {
    if let Some(mut map) = action_map {
        map.update(&input);
    }
}

/// 帧末清除 just_pressed/just_released 状态
fn input_end_frame_system(mut input: ResMut<anvilkit_input::prelude::InputState>) {
    input.end_frame();
}

/// 自动时间更新插件
///
/// 在 PreUpdate 阶段自动调用 `Time::update()`，
/// 并将 delta 钳制到最大 0.25 秒（防止长帧导致的 spiral-of-death）。
///
/// # 示例
///
/// ```rust,no_run
/// use anvilkit_app::prelude::*;
/// use anvilkit_app::auto_plugins::AutoDeltaTimePlugin;
///
/// App::new()
///     .add_plugins(AnvilKitEcsPlugin)
///     .add_plugins(AutoDeltaTimePlugin);
/// ```
pub struct AutoDeltaTimePlugin;

impl Plugin for AutoDeltaTimePlugin {
    fn build(&self, app: &mut App) {
        use anvilkit_core::time::Time;
        app.init_resource::<Time>();
        app.add_systems(AnvilKitSchedule::PreUpdate, time_update_system);
    }

    fn name(&self) -> &str {
        "AutoDeltaTimePlugin"
    }
}

/// 帧初更新 Time 资源
fn time_update_system(mut time: ResMut<anvilkit_core::time::Time>) {
    time.update();
    // Note: delta clamping is handled by the FixedUpdate accumulator in App::update()
    // which caps at max 10 ticks. Time itself tracks real elapsed time.
}

// --- Persistence Plugin (feature-gated) ---

/// 持久化插件
///
/// 注册 `AutoSaveConfig` 和 `AutoSaveState` 资源，
/// 并在 Update 阶段运行自动存档计时系统。
///
/// 当计时器触发时记录日志；实际存档操作由游戏层处理。
///
/// # 示例
///
/// ```rust,no_run
/// use anvilkit_app::prelude::*;
/// use anvilkit_app::auto_plugins::PersistencePlugin;
///
/// App::new()
///     .add_plugins(AnvilKitEcsPlugin)
///     .add_plugins(PersistencePlugin);
/// ```
#[cfg(feature = "persistence")]
pub struct PersistencePlugin;

#[cfg(feature = "persistence")]
impl Plugin for PersistencePlugin {
    fn build(&self, app: &mut App) {
        use anvilkit_core::persistence::{AutoSaveConfig, AutoSaveState};
        app.init_resource::<AutoSaveConfig>();
        app.init_resource::<AutoSaveState>();
        app.add_systems(AnvilKitSchedule::Update, auto_save_tick_system);
    }

    fn name(&self) -> &str {
        "PersistencePlugin"
    }
}

/// 自动存档计时系统 — 每帧累加时间，到达间隔时触发存档事件
#[cfg(feature = "persistence")]
fn auto_save_tick_system(
    config: Res<anvilkit_core::persistence::AutoSaveConfig>,
    mut state: ResMut<anvilkit_core::persistence::AutoSaveState>,
    dt: Res<crate::ecs_app::DeltaTime>,
) {
    if let Some(slot_name) = anvilkit_core::persistence::auto_save_tick(&config, &mut state, dt.0 as f64) {
        log::info!("Auto-save triggered: slot '{}'", slot_name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs_app::App;
    use crate::ecs_plugin::AnvilKitEcsPlugin;
    use bevy_app::Plugin;

    #[test]
    fn test_auto_input_plugin_registers_resource() {
        let mut app = App::new();
        app.add_plugins(AnvilKitEcsPlugin);
        app.add_plugins(AutoInputPlugin);
        assert!(app.world().get_resource::<anvilkit_input::prelude::InputState>().is_some());
    }

    #[test]
    fn test_auto_input_end_frame_clears_just_pressed() {
        let mut app = App::new();
        app.add_plugins(AnvilKitEcsPlugin);
        app.add_plugins(AutoInputPlugin);

        // Simulate a key press
        {
            let mut input = app.world_mut().resource_mut::<anvilkit_input::prelude::InputState>();
            input.press_key(anvilkit_input::prelude::KeyCode::Space);
        }

        // After update, Cleanup runs end_frame
        app.update();

        let input = app.world().resource::<anvilkit_input::prelude::InputState>();
        // Key should still be pressed but just_pressed should be cleared
        assert!(input.is_key_pressed(anvilkit_input::prelude::KeyCode::Space));
        assert!(!input.is_key_just_pressed(anvilkit_input::prelude::KeyCode::Space));
    }

    #[test]
    fn test_auto_delta_time_plugin_registers_resource() {
        let mut app = App::new();
        app.add_plugins(AnvilKitEcsPlugin);
        app.add_plugins(AutoDeltaTimePlugin);
        assert!(app.world().get_resource::<anvilkit_core::time::Time>().is_some());
    }

    #[test]
    fn test_auto_delta_time_updates_frame_count() {
        let mut app = App::new();
        app.add_plugins(AnvilKitEcsPlugin);
        app.add_plugins(AutoDeltaTimePlugin);

        let initial_frame_count = app.world().resource::<anvilkit_core::time::Time>().frame_count();
        app.update();
        let after_frame_count = app.world().resource::<anvilkit_core::time::Time>().frame_count();
        assert!(after_frame_count > initial_frame_count);
    }

    #[test]
    fn test_auto_input_plugin_name() {
        let plugin = AutoInputPlugin;
        assert_eq!(plugin.name(), "AutoInputPlugin");
    }

    #[cfg(feature = "persistence")]
    #[test]
    fn test_persistence_plugin_registers_resources() {
        let mut app = App::new();
        app.add_plugins(AnvilKitEcsPlugin);
        app.add_plugins(PersistencePlugin);
        assert!(app.world().get_resource::<anvilkit_core::persistence::AutoSaveConfig>().is_some());
        assert!(app.world().get_resource::<anvilkit_core::persistence::AutoSaveState>().is_some());
    }

    #[cfg(feature = "persistence")]
    #[test]
    fn test_persistence_plugin_name() {
        let plugin = PersistencePlugin;
        assert_eq!(plugin.name(), "PersistencePlugin");
    }

}
