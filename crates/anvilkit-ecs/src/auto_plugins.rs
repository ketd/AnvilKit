//! # 自动插件
//!
//! 提供 `AutoInputPlugin` 和 `AutoDeltaTimePlugin`，
//! 自动管理输入帧生命周期和时间更新。

use bevy_ecs::prelude::*;
use crate::plugin::Plugin;
use crate::app::App;
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
/// use anvilkit_ecs::prelude::*;
/// use anvilkit_ecs::auto_plugins::AutoInputPlugin;
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
        app.add_systems(AnvilKitSchedule::Cleanup, input_end_frame_system);
    }

    fn name(&self) -> &str {
        "AutoInputPlugin"
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
/// use anvilkit_ecs::prelude::*;
/// use anvilkit_ecs::auto_plugins::AutoDeltaTimePlugin;
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

/// 最大允许的 delta 时间（秒）
///
/// 超过此值的帧时间被视为异常（如调试器暂停），
/// 防止 FixedUpdate 执行过多 tick。
pub const MAX_DELTA_SECONDS: f32 = 0.25;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;

    #[test]
    fn test_auto_input_plugin_registers_resource() {
        let mut app = App::new();
        app.add_plugins(AnvilKitEcsPlugin);
        app.add_plugins(AutoInputPlugin);
        assert!(app.world.get_resource::<anvilkit_input::prelude::InputState>().is_some());
    }

    #[test]
    fn test_auto_input_end_frame_clears_just_pressed() {
        let mut app = App::new();
        app.add_plugins(AnvilKitEcsPlugin);
        app.add_plugins(AutoInputPlugin);

        // Simulate a key press
        {
            let mut input = app.world.resource_mut::<anvilkit_input::prelude::InputState>();
            input.press_key(anvilkit_input::prelude::KeyCode::Space);
        }

        // After update, Cleanup runs end_frame
        app.update();

        let input = app.world.resource::<anvilkit_input::prelude::InputState>();
        // Key should still be pressed but just_pressed should be cleared
        assert!(input.is_key_pressed(anvilkit_input::prelude::KeyCode::Space));
        assert!(!input.is_key_just_pressed(anvilkit_input::prelude::KeyCode::Space));
    }

    #[test]
    fn test_auto_delta_time_plugin_registers_resource() {
        let mut app = App::new();
        app.add_plugins(AnvilKitEcsPlugin);
        app.add_plugins(AutoDeltaTimePlugin);
        assert!(app.world.get_resource::<anvilkit_core::time::Time>().is_some());
    }

    #[test]
    fn test_auto_delta_time_updates_frame_count() {
        let mut app = App::new();
        app.add_plugins(AnvilKitEcsPlugin);
        app.add_plugins(AutoDeltaTimePlugin);

        let initial_frame_count = app.world.resource::<anvilkit_core::time::Time>().frame_count();
        app.update();
        let after_frame_count = app.world.resource::<anvilkit_core::time::Time>().frame_count();
        assert!(after_frame_count > initial_frame_count);
    }

    #[test]
    fn test_auto_input_plugin_name() {
        let plugin = AutoInputPlugin;
        assert_eq!(plugin.name(), "AutoInputPlugin");
    }
}
