//! # 插件系统
//! 
//! 提供模块化的插件架构，允许功能的组合和扩展。
//! 
//! ## 设计理念
//! 
//! - **模块化**: 每个插件负责特定的功能领域
//! - **可组合**: 插件可以相互依赖和组合
//! - **可配置**: 插件可以接受配置参数
//! - **生命周期**: 插件在应用构建时初始化
//! 
//! ## 使用示例
//! 
//! ```rust
//! use anvilkit_ecs::prelude::*;
//! 
//! // 定义自定义插件
//! struct MyPlugin {
//!     config: MyConfig,
//! }
//! 
//! #[derive(Default)]
//! struct MyConfig {
//!     enabled: bool,
//! }
//! 
//! impl Plugin for MyPlugin {
//!     fn build(&self, app: &mut App) {
//!         if self.config.enabled {
//!             app.add_systems(Update, my_system);
//!         }
//!     }
//! }
//! 
//! fn my_system() {
//!     println!("我的系统正在运行！");
//! }
//! ```

use crate::app::App;

/// 插件 trait
/// 
/// 所有插件都必须实现此 trait，用于向应用添加功能。
/// 
/// # 设计原则
/// 
/// 1. **单一职责**: 每个插件应该专注于一个特定的功能领域
/// 2. **无副作用**: 插件的构建过程应该是确定性的
/// 3. **可测试**: 插件应该易于单独测试
/// 4. **文档化**: 插件应该有清晰的文档说明其功能和用法
/// 
/// # 示例
/// 
/// ```rust
/// use anvilkit_ecs::prelude::*;
/// 
/// struct LoggingPlugin;
/// 
/// impl Plugin for LoggingPlugin {
///     fn build(&self, app: &mut App) {
///         app.add_systems(Startup, setup_logging)
///            .add_systems(Update, log_frame_count);
///     }
/// }
/// 
/// fn setup_logging() {
///     println!("日志系统已初始化");
/// }
/// 
/// fn log_frame_count() {
///     // 记录帧数逻辑
/// }
/// ```
pub trait Plugin: Send + Sync {
    /// 构建插件
    /// 
    /// 在此方法中添加系统、资源、组件等到应用。
    /// 
    /// # 参数
    /// 
    /// - `app`: 要配置的应用实例
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_ecs::prelude::*;
    /// 
    /// struct MyPlugin;
    /// 
    /// impl Plugin for MyPlugin {
    ///     fn build(&self, app: &mut App) {
    ///         app.insert_resource(MyResource::default())
    ///            .add_systems(Update, my_system);
    ///     }
    /// }
    /// 
    /// #[derive(Resource, Default)]
    /// struct MyResource {
    ///     value: i32,
    /// }
    /// 
    /// fn my_system(mut resource: ResMut<MyResource>) {
    ///     resource.value += 1;
    /// }
    /// ```
    fn build(&self, app: &mut App);

    /// 插件名称
    /// 
    /// 返回插件的唯一标识名称，用于调试和日志记录。
    /// 
    /// # 默认实现
    /// 
    /// 默认返回类型名称。
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    /// 插件是否唯一
    /// 
    /// 如果返回 `true`，则同一类型的插件只能添加一次。
    /// 
    /// # 默认实现
    /// 
    /// 默认返回 `true`。
    fn is_unique(&self) -> bool {
        true
    }
}

/// AnvilKit ECS 核心插件
/// 
/// 提供 ECS 系统的基础功能，包括：
/// - Transform 层次系统
/// - 时间管理
/// - 基础调度器设置
/// 
/// # 示例
/// 
/// ```rust
/// use anvilkit_ecs::prelude::*;
/// 
/// let mut app = App::new();
/// app.add_plugins(AnvilKitEcsPlugin);
/// ```
pub struct AnvilKitEcsPlugin;

impl Plugin for AnvilKitEcsPlugin {
    fn build(&self, app: &mut App) {
        use crate::schedule::{AnvilKitSchedule, ScheduleLabel};
        use crate::transform::TransformPlugin;
        use anvilkit_core::time::Time;
        
        // 添加核心资源
        app.init_resource::<Time>()
           .insert_resource(crate::app::AppExit::default());

        // 设置基础调度器
        self.setup_schedules(app);
        
        // 添加 Transform 插件
        app.add_plugins(TransformPlugin);
    }

    fn name(&self) -> &str {
        "AnvilKitEcsPlugin"
    }
}

impl AnvilKitEcsPlugin {
    /// 设置基础调度器
    fn setup_schedules(&self, app: &mut App) {
        use bevy_ecs::schedule::*;
        use crate::schedule::AnvilKitSchedule;
        
        // 创建主要的调度器
        app.world.add_schedule(Schedule::new(AnvilKitSchedule::Main));
        app.world.add_schedule(Schedule::new(AnvilKitSchedule::Startup));
        app.world.add_schedule(Schedule::new(AnvilKitSchedule::PreUpdate));
        app.world.add_schedule(Schedule::new(AnvilKitSchedule::Update));
        app.world.add_schedule(Schedule::new(AnvilKitSchedule::PostUpdate));
        app.world.add_schedule(Schedule::new(AnvilKitSchedule::Cleanup));
    }
}

/// 插件组
/// 
/// 用于将多个相关插件组合在一起。
/// 
/// # 示例
/// 
/// ```rust
/// use anvilkit_ecs::prelude::*;
/// 
/// struct GamePlugins;
/// 
/// impl Plugin for GamePlugins {
///     fn build(&self, app: &mut App) {
///         app.add_plugins(AnvilKitEcsPlugin)
///            .add_plugins(PhysicsPlugin)
///            .add_plugins(RenderPlugin);
///     }
/// }
/// 
/// struct PhysicsPlugin;
/// struct RenderPlugin;
/// 
/// impl Plugin for PhysicsPlugin {
///     fn build(&self, app: &mut App) {
///         // 物理系统设置
///     }
/// }
/// 
/// impl Plugin for RenderPlugin {
///     fn build(&self, app: &mut App) {
///         // 渲染系统设置
///     }
/// }
/// ```
pub struct PluginGroup<T> {
    plugins: Vec<T>,
}

impl<T> PluginGroup<T> {
    /// 创建新的插件组
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    /// 添加插件到组
    pub fn add(mut self, plugin: T) -> Self {
        self.plugins.push(plugin);
        self
    }

    /// 获取插件列表
    pub fn plugins(&self) -> &[T] {
        &self.plugins
    }
}

impl<T> Default for PluginGroup<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Plugin> Plugin for PluginGroup<T> {
    fn build(&self, app: &mut App) {
        for plugin in &self.plugins {
            plugin.build(app);
        }
    }

    fn name(&self) -> &str {
        "PluginGroup"
    }

    fn is_unique(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;

    #[derive(Resource, Default)]
    struct TestResource {
        value: i32,
    }

    struct TestPlugin {
        initial_value: i32,
    }

    impl Plugin for TestPlugin {
        fn build(&self, app: &mut App) {
            app.insert_resource(TestResource {
                value: self.initial_value,
            });
        }
    }

    fn increment_system(mut resource: ResMut<TestResource>) {
        resource.value += 1;
    }

    #[test]
    fn test_plugin_trait() {
        let mut app = App::new();
        let plugin = TestPlugin { initial_value: 42 };
        
        plugin.build(&mut app);
        
        let resource = app.world.get_resource::<TestResource>().unwrap();
        assert_eq!(resource.value, 42);
    }

    #[test]
    fn test_anvilkit_ecs_plugin() {
        let mut app = App::new();
        app.add_plugins(AnvilKitEcsPlugin);
        
        // 验证核心资源已添加
        assert!(app.world.get_resource::<Time>().is_some());
        assert!(app.world.get_resource::<crate::app::AppExit>().is_some());
    }

    #[test]
    fn test_plugin_group() {
        let mut app = App::new();
        
        let plugin_group = PluginGroup::new()
            .add(TestPlugin { initial_value: 10 })
            .add(TestPlugin { initial_value: 20 }); // 这会覆盖前一个
        
        plugin_group.build(&mut app);
        
        let resource = app.world.get_resource::<TestResource>().unwrap();
        assert_eq!(resource.value, 20); // 最后一个插件的值
    }

    #[test]
    fn test_plugin_name() {
        let plugin = TestPlugin { initial_value: 0 };
        assert!(plugin.name().contains("TestPlugin"));
    }

    #[test]
    fn test_plugin_uniqueness() {
        let plugin = TestPlugin { initial_value: 0 };
        assert!(plugin.is_unique());
        
        let plugin_group = PluginGroup::<TestPlugin>::new();
        assert!(!plugin_group.is_unique());
    }
}
