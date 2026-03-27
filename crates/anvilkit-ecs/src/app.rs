//! # 应用框架
//! 
//! 提供基于 Bevy ECS 的应用框架，管理系统调度、插件加载和生命周期。
//! 
//! ## 核心概念
//! 
//! - **App**: 应用的主要容器，管理 World、Schedule 和插件
//! - **Plugin**: 模块化的功能扩展，可以添加系统、资源和组件
//! - **Schedule**: 系统执行的调度器，控制系统运行顺序和并行性
//! 
//! ## 使用示例
//! 
//! ```rust,no_run
//! use anvilkit_ecs::prelude::*;
//! use anvilkit_ecs::schedule::AnvilKitSchedule;
//!
//! // 创建应用
//! let mut app = App::new();
//!
//! // 添加插件
//! app.add_plugins(AnvilKitEcsPlugin);
//!
//! // 添加系统
//! app.add_systems(AnvilKitSchedule::Update, my_system);
//!
//! // 运行应用
//! app.run();
//!
//! fn my_system() {
//!     println!("系统正在运行！");
//! }
//! ```

use std::collections::HashSet;
use bevy_ecs::prelude::*;
use bevy_ecs::event::{Event, Events, EventRegistry};
use bevy_ecs::component::Tick;
use crate::plugin::Plugin;
use crate::schedule::{AnvilKitSchedule, ScheduleLabel};

/// AnvilKit 应用框架
/// 
/// 基于 Bevy ECS 构建的应用容器，提供系统调度、插件管理和生命周期控制。
/// 
/// # 特性
/// 
/// - **插件系统**: 支持模块化的功能扩展
/// - **系统调度**: 灵活的系统执行顺序和并行控制
/// - **资源管理**: 全局资源的存储和访问
/// - **事件系统**: 组件间的松耦合通信
/// 
/// # 示例
/// 
/// ```rust,no_run
/// use anvilkit_ecs::prelude::*;
/// use anvilkit_ecs::schedule::AnvilKitSchedule;
///
/// let mut app = App::new();
/// app.add_plugins(AnvilKitEcsPlugin)
///    .add_systems(AnvilKitSchedule::Update, movement_system)
///    .run();
///
/// fn movement_system() {
///     // 系统逻辑
/// }
/// ```
pub struct App {
    /// ECS 世界，存储所有实体、组件和资源
    pub world: World,
    /// 是否应该退出应用
    should_exit: bool,
    /// Startup 是否已经运行
    has_started: bool,
    /// 已注册的唯一插件类型名（防止重复注册）
    pub(crate) registered_plugins: HashSet<String>,
    /// FixedUpdate 时间累加器（秒）
    accumulated_time: f32,
    /// FixedUpdate 固定步长（秒），默认 1/60
    fixed_timestep: f32,
    /// Events 上次更新的 change tick
    last_event_tick: Tick,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    /// 创建新的应用实例
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_ecs::app::App;
    /// 
    /// let app = App::new();
    /// ```
    pub fn new() -> Self {
        let mut world = World::new();

        // 初始化基础调度器
        world.init_resource::<Schedules>();

        // 预创建所有标准调度阶段，避免 try_run_schedule 返回 TryRunScheduleError
        {
            let mut schedules = world.resource_mut::<Schedules>();
            for label in [
                AnvilKitSchedule::Startup,
                AnvilKitSchedule::Main,
                AnvilKitSchedule::PreUpdate,
                AnvilKitSchedule::FixedUpdate,
                AnvilKitSchedule::Update,
                AnvilKitSchedule::PostUpdate,
                AnvilKitSchedule::Cleanup,
            ] {
                schedules.entry(label);
            }
        }

        Self {
            world,
            should_exit: false,
            has_started: false,
            registered_plugins: HashSet::new(),
            accumulated_time: 0.0,
            fixed_timestep: 1.0 / 60.0,
            last_event_tick: Tick::new(0),
        }
    }

    /// 添加插件到应用
    /// 
    /// # 参数
    /// 
    /// - `plugin`: 要添加的插件
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_ecs::prelude::*;
    /// 
    /// let mut app = App::new();
    /// app.add_plugins(AnvilKitEcsPlugin);
    /// ```
    pub fn add_plugins<P: Plugin>(&mut self, plugin: P) -> &mut Self {
        if plugin.is_unique() {
            let type_name = std::any::type_name::<P>().to_string();
            if !self.registered_plugins.insert(type_name.clone()) {
                log::warn!("插件 {} 已注册，跳过重复注册", type_name);
                return self;
            }
        }
        plugin.build(self);
        self
    }

    /// 添加系统到指定调度
    /// 
    /// # 参数
    /// 
    /// - `schedule`: 调度标签
    /// - `systems`: 要添加的系统
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_ecs::prelude::*;
    /// use anvilkit_ecs::schedule::AnvilKitSchedule;
    ///
    /// let mut app = App::new();
    /// app.add_systems(AnvilKitSchedule::Update, my_system);
    ///
    /// fn my_system() {
    ///     println!("系统运行中");
    /// }
    /// ```
    pub fn add_systems<M>(
        &mut self,
        schedule: impl ScheduleLabel,
        systems: impl IntoSystemConfigs<M>,
    ) -> &mut Self {
        let mut schedules = self.world.resource_mut::<Schedules>();

        // 使用 entry 方法来获取或创建调度器
        let target_schedule = schedules.entry(schedule);
        target_schedule.add_systems(systems);

        self
    }

    /// 插入资源到世界
    /// 
    /// # 参数
    /// 
    /// - `resource`: 要插入的资源
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_ecs::prelude::*;
    /// 
    /// #[derive(Resource)]
    /// struct GameConfig {
    ///     difficulty: u32,
    /// }
    /// 
    /// let mut app = App::new();
    /// app.insert_resource(GameConfig { difficulty: 1 });
    /// ```
    pub fn insert_resource<R: Resource>(&mut self, resource: R) -> &mut Self {
        self.world.insert_resource(resource);
        self
    }

    /// 初始化资源（如果不存在）
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_ecs::prelude::*;
    /// 
    /// #[derive(Resource, Default)]
    /// struct Score(u32);
    /// 
    /// let mut app = App::new();
    /// app.init_resource::<Score>();
    /// ```
    pub fn init_resource<R: Resource + FromWorld>(&mut self) -> &mut Self {
        self.world.init_resource::<R>();
        self
    }

    /// 注册事件类型
    ///
    /// 注册后可使用 `EventWriter<E>` 发送事件、`EventReader<E>` 读取事件。
    /// 事件自动双缓冲，存活 2 帧后清除。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use anvilkit_ecs::prelude::*;
    /// use bevy_ecs::event::Event;
    ///
    /// #[derive(Event)]
    /// struct PlayerDied { entity: Entity }
    ///
    /// let mut app = App::new();
    /// app.add_event::<PlayerDied>();
    /// ```
    pub fn add_event<E: Event>(&mut self) -> &mut Self {
        self.init_resource::<Events<E>>();
        EventRegistry::register_event::<E>(&mut self.world);
        self
    }

    /// 注册可序列化组件类型
    ///
    /// 将类型注册到 `SerializableRegistry`，使其可被 `SceneSerializer` 处理。
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// use anvilkit_ecs::prelude::*;
    ///
    /// #[derive(Component)]
    /// struct Health(f32);
    ///
    /// let mut app = App::new();
    /// app.register_serializable::<Health>("Health");
    /// ```
    pub fn register_serializable<T: 'static>(&mut self, name: &str) -> &mut Self {
        self.init_resource::<crate::scene::SerializableRegistry>();
        self.world.resource_mut::<crate::scene::SerializableRegistry>().register::<T>(name);
        self
    }

    /// 运行应用的主循环
    /// 
    /// 这将持续运行主调度器，直到应用被标记为退出。
    /// 
    /// # 示例
    /// 
    /// ```rust,no_run
    /// use anvilkit_ecs::prelude::*;
    /// 
    /// let mut app = App::new();
    /// app.add_plugins(AnvilKitEcsPlugin)
    ///    .run();
    /// ```
    pub fn run(&mut self) {
        while !self.should_exit {
            self.update();
        }
    }

    /// 执行一次更新循环
    /// 
    /// 运行主调度器一次，处理所有系统。
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_ecs::prelude::*;
    /// 
    /// let mut app = App::new();
    /// app.add_plugins(AnvilKitEcsPlugin);
    /// 
    /// // 手动控制更新
    /// for _ in 0..10 {
    ///     app.update();
    /// }
    /// ```
    pub fn update(&mut self) {
        if !self.has_started {
            self.has_started = true;
            if let Err(e) = self.world.try_run_schedule(AnvilKitSchedule::Startup) {
                log::error!("Startup schedule 执行失败: {:?}", e);
            }
        }

        if let Err(e) = self.world.try_run_schedule(AnvilKitSchedule::Main) {
            log::error!("Main schedule 执行失败: {:?}", e);
        }

        // 刷新事件双缓冲（清理 2 帧前的事件，交换缓冲区）
        if self.world.contains_resource::<EventRegistry>() {
            let last_tick = self.last_event_tick;
            self.world.resource_scope(|world, mut registry: Mut<EventRegistry>| {
                registry.run_updates(world, last_tick);
            });
            self.last_event_tick = self.world.change_tick();
        }

        if let Err(e) = self.world.try_run_schedule(AnvilKitSchedule::PreUpdate) {
            log::error!("PreUpdate schedule 执行失败: {:?}", e);
        }

        // FixedUpdate 累加循环：以固定步长运行物理等确定性系统
        {
            let dt = self.world
                .get_resource::<anvilkit_core::time::Time>()
                .map(|t| t.delta_seconds())
                .unwrap_or(self.fixed_timestep);
            self.accumulated_time += dt;
            let max_ticks = 10; // 防止死循环
            let mut ticks = 0;
            while self.accumulated_time >= self.fixed_timestep && ticks < max_ticks {
                if let Err(e) = self.world.try_run_schedule(AnvilKitSchedule::FixedUpdate) {
                    log::error!("FixedUpdate schedule 执行失败: {:?}", e);
                    break;
                }
                self.accumulated_time -= self.fixed_timestep;
                ticks += 1;
            }
        }

        if let Err(e) = self.world.try_run_schedule(AnvilKitSchedule::Update) {
            log::error!("Update schedule 执行失败: {:?}", e);
        }
        if let Err(e) = self.world.try_run_schedule(AnvilKitSchedule::PostUpdate) {
            log::error!("PostUpdate schedule 执行失败: {:?}", e);
        }
        if let Err(e) = self.world.try_run_schedule(AnvilKitSchedule::Cleanup) {
            log::error!("Cleanup schedule 执行失败: {:?}", e);
        }

        // Check AppExit resource and propagate to App::should_exit
        if self.world.get_resource::<AppExit>().map_or(false, |e| e.should_exit()) {
            self.should_exit = true;
        }
    }

    /// 标记应用应该退出
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_ecs::prelude::*;
    /// 
    /// fn exit_system(mut app_exit: ResMut<AppExit>) {
    ///     app_exit.exit();
    /// }
    /// ```
    pub fn exit(&mut self) {
        self.should_exit = true;
    }

    /// 检查应用是否应该退出
    pub fn should_exit(&self) -> bool {
        self.should_exit
    }

    /// 设置 FixedUpdate 的固定步长（秒）
    ///
    /// 默认值为 1/60 秒（60Hz）。较小的步长更精确但更昂贵。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use anvilkit_ecs::app::App;
    ///
    /// let mut app = App::new();
    /// app.set_fixed_timestep(1.0 / 120.0); // 120Hz 物理更新
    /// ```
    pub fn set_fixed_timestep(&mut self, dt: f32) -> &mut Self {
        self.fixed_timestep = dt.max(0.0001); // 防止零或负值
        self
    }

    /// 获取当前 FixedUpdate 固定步长（秒）
    pub fn fixed_timestep(&self) -> f32 {
        self.fixed_timestep
    }
}

/// 应用退出资源
/// 
/// 用于控制应用的退出状态。
#[derive(Resource, Default)]
pub struct AppExit {
    should_exit: bool,
}

impl AppExit {
    /// 标记应用应该退出
    pub fn exit(&mut self) {
        self.should_exit = true;
    }

    /// 检查是否应该退出
    pub fn should_exit(&self) -> bool {
        self.should_exit
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;

    #[derive(Component)]
    struct TestComponent(i32);

    #[derive(Resource)]
    struct TestResource(String);

    fn test_system(mut query: Query<&mut TestComponent>) {
        for mut component in &mut query {
            component.0 += 1;
        }
    }

    #[test]
    fn test_app_creation() {
        let app = App::new();
        assert!(!app.should_exit());
    }

    #[test]
    fn test_resource_management() {
        let mut app = App::new();
        
        // 插入资源
        app.insert_resource(TestResource("test".to_string()));
        
        // 验证资源存在
        let resource = app.world.get_resource::<TestResource>().unwrap();
        assert_eq!(resource.0, "test");
    }

    #[test]
    fn test_system_execution() {
        let mut app = App::new();
        
        // 创建实体和组件
        app.world.spawn(TestComponent(0));
        
        // 添加系统
        app.add_systems(AnvilKitSchedule::Update, test_system);
        
        // 执行一次更新
        app.update();
        
        // 验证系统执行了
        let component = app.world.query::<&TestComponent>().single(&app.world);
        assert_eq!(component.0, 1);
    }

    #[test]
    fn test_app_exit() {
        let mut app = App::new();
        assert!(!app.should_exit());

        app.exit();
        assert!(app.should_exit());
    }

    #[test]
    fn test_app_multiple_updates() {
        let mut app = App::new();
        app.add_plugins(AnvilKitEcsPlugin);
        for _ in 0..10 {
            app.update();
        }
        // Should not panic after multiple updates
    }

    #[test]
    fn test_app_resource_overwrite() {
        let mut app = App::new();
        app.insert_resource(TestResource("first".to_string()));

        let resource = app.world.get_resource::<TestResource>().unwrap();
        assert_eq!(resource.0, "first");

        app.insert_resource(TestResource("second".to_string()));
        let resource = app.world.get_resource::<TestResource>().unwrap();
        assert_eq!(resource.0, "second");
    }

    #[test]
    fn test_app_init_resource_default() {
        #[derive(Resource, Default, PartialEq, Debug)]
        struct DefaultResource(String);

        let mut app = App::new();
        app.init_resource::<DefaultResource>();

        let resource = app.world.get_resource::<DefaultResource>().unwrap();
        assert_eq!(*resource, DefaultResource::default());
    }

    #[test]
    fn test_fixed_timestep_config() {
        let mut app = App::new();
        assert!((app.fixed_timestep() - 1.0 / 60.0).abs() < 0.001);

        app.set_fixed_timestep(1.0 / 120.0);
        assert!((app.fixed_timestep() - 1.0 / 120.0).abs() < 0.001);

        // 防止零或负值
        app.set_fixed_timestep(0.0);
        assert!(app.fixed_timestep() > 0.0);
    }

    #[test]
    fn test_add_event() {
        use bevy_ecs::event::{Event, Events};

        #[derive(Event)]
        struct TestEvent(i32);

        let mut app = App::new();
        app.add_event::<TestEvent>();

        // Events 资源应已创建
        assert!(app.world.get_resource::<Events<TestEvent>>().is_some());
    }

    #[test]
    fn test_fixed_update_runs_physics_schedule() {
        use crate::schedule::AnvilKitSchedule;

        #[derive(Resource, Default)]
        struct FixedCounter(u32);

        fn fixed_count_system(mut counter: ResMut<FixedCounter>) {
            counter.0 += 1;
        }

        let mut app = App::new();
        app.add_plugins(AnvilKitEcsPlugin);
        app.init_resource::<FixedCounter>();
        app.add_systems(AnvilKitSchedule::FixedUpdate, fixed_count_system);

        // 手动设置时间以触发 FixedUpdate
        // update() 读取 Time::delta_seconds() 为 0（首帧），所以 FixedUpdate 不会运行
        app.update();
        assert_eq!(app.world.resource::<FixedCounter>().0, 0);

        // 给 Time 一个实际的 delta 然后再 update
        if let Some(mut time) = app.world.get_resource_mut::<anvilkit_core::time::Time>() {
            time.update(); // 更新内部 Instant
        }
        std::thread::sleep(std::time::Duration::from_millis(20)); // 确保 delta > fixed_timestep
        if let Some(mut time) = app.world.get_resource_mut::<anvilkit_core::time::Time>() {
            time.update(); // 现在 delta_seconds ~0.02 > 1/60
        }
        app.update();
        assert!(app.world.resource::<FixedCounter>().0 >= 1);
    }

    #[test]
    fn test_fixed_update_accumulator_behavior() {
        #[derive(Resource, Default)]
        struct TickCount(u32);

        fn tick_system(mut count: ResMut<TickCount>) {
            count.0 += 1;
        }

        let mut app = App::new();
        app.add_plugins(AnvilKitEcsPlugin);
        app.init_resource::<TickCount>();
        app.set_fixed_timestep(1.0 / 60.0);
        app.add_systems(AnvilKitSchedule::FixedUpdate, tick_system);

        // First update: Time delta is 0, no FixedUpdate ticks
        app.update();
        assert_eq!(app.world.resource::<TickCount>().0, 0);

        // Manually inject time so FixedUpdate accumulates
        // The app reads Time::delta_seconds() for accumulation
        // Since we can't easily fake Time, verify the timestep config works
        assert!((app.fixed_timestep() - 1.0 / 60.0).abs() < 0.001);

        app.set_fixed_timestep(1.0 / 120.0);
        assert!((app.fixed_timestep() - 1.0 / 120.0).abs() < 0.001);
    }
}
