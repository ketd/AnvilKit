//! # 插件系统
//!
//! Provides `AnvilKitEcsPlugin` for core ECS setup.

pub use bevy_app::Plugin;

use crate::ecs_app::App;
use anvilkit_core::time::Time;

/// AnvilKit ECS 核心插件
///
/// 提供 ECS 系统的基础功能，包括：
/// - 时间管理
/// - 基础调度器设置
///
/// Note: TransformPlugin is no longer added here (it lives in anvilkit-render).
/// Games and DefaultPlugins should add it explicitly if needed.
pub struct AnvilKitEcsPlugin;

impl Plugin for AnvilKitEcsPlugin {
    fn build(&self, app: &mut App) {
        // 添加核心资源
        app.init_resource::<Time>();

        // 设置基础调度器
        self.setup_schedules(app);
    }
}

impl AnvilKitEcsPlugin {
    /// 设置基础调度器并注册到 bevy 的 MainScheduleOrder
    fn setup_schedules(&self, app: &mut App) {
        use bevy_ecs::schedule::*;
        use bevy_app::MainScheduleOrder;
        use crate::schedule::{AnvilKitSchedule, AnvilKitSystemSet};

        // Register all AnvilKit schedules with the world
        app.init_schedule(AnvilKitSchedule::Startup);
        app.init_schedule(AnvilKitSchedule::Main);
        app.init_schedule(AnvilKitSchedule::PreUpdate);
        app.init_schedule(AnvilKitSchedule::FixedUpdate);
        app.init_schedule(AnvilKitSchedule::Update);
        app.init_schedule(AnvilKitSchedule::PostUpdate);
        app.init_schedule(AnvilKitSchedule::Cleanup);

        {
            let mut order = app.world_mut().resource_mut::<MainScheduleOrder>();
            order.insert_after(bevy_app::PreUpdate, AnvilKitSchedule::PreUpdate);
            order.insert_after(AnvilKitSchedule::PreUpdate, AnvilKitSchedule::FixedUpdate);
            order.insert_after(bevy_app::Update, AnvilKitSchedule::Update);
            order.insert_after(bevy_app::PostUpdate, AnvilKitSchedule::PostUpdate);
            order.insert_after(bevy_app::Last, AnvilKitSchedule::Cleanup);
        }

        let configure_order = |schedule: &mut Schedule| {
            schedule.configure_sets((
                AnvilKitSystemSet::Input,
                AnvilKitSystemSet::Time.after(AnvilKitSystemSet::Input),
                AnvilKitSystemSet::Physics.after(AnvilKitSystemSet::Time),
                AnvilKitSystemSet::GameLogic.after(AnvilKitSystemSet::Physics),
                AnvilKitSystemSet::Transform.after(AnvilKitSystemSet::GameLogic),
                AnvilKitSystemSet::Render.after(AnvilKitSystemSet::Transform),
                AnvilKitSystemSet::Audio.after(AnvilKitSystemSet::Render),
                AnvilKitSystemSet::UI.after(AnvilKitSystemSet::Audio),
                AnvilKitSystemSet::Network.after(AnvilKitSystemSet::UI),
                AnvilKitSystemSet::Debug.after(AnvilKitSystemSet::Network),
            ));
        };

        if let Some(mut schedules) = app.world_mut().get_resource_mut::<Schedules>() {
            if let Some(schedule) = schedules.get_mut(AnvilKitSchedule::Update) {
                configure_order(schedule);
            }
            if let Some(schedule) = schedules.get_mut(AnvilKitSchedule::FixedUpdate) {
                configure_order(schedule);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_anvilkit_ecs_plugin() {
        let mut app = App::new();
        app.add_plugins(AnvilKitEcsPlugin);

        assert!(app.world().get_resource::<Time>().is_some());
    }
}
