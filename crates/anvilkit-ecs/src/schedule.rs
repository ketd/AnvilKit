//! # 系统调度
//! 
//! 提供系统执行的调度和组织功能，控制系统的运行顺序和并行性。
//! 
//! ## 核心概念
//! 
//! - **Schedule**: 系统的集合，定义了系统的执行顺序
//! - **SystemSet**: 系统的逻辑分组，用于批量操作和依赖管理
//! - **ScheduleLabel**: 调度的标识符，用于区分不同的调度阶段
//! 
//! ## 调度阶段
//! 
//! AnvilKit 定义了以下标准调度阶段：
//! 
//! 1. **Startup**: 应用启动时执行一次
//! 2. **PreUpdate**: 主更新前的准备阶段
//! 3. **Update**: 主要的游戏逻辑更新
//! 4. **PostUpdate**: 主更新后的清理和同步
//! 5. **Cleanup**: 帧结束时的清理工作
//! 
//! ## 使用示例
//! 
//! ```rust
//! use anvilkit_ecs::prelude::*;
//! use anvilkit_ecs::schedule::AnvilKitSchedule;
//!
//! fn setup_system() {
//!     println!("游戏初始化");
//! }
//!
//! fn update_system() {
//!     println!("游戏更新");
//! }
//!
//! fn cleanup_system() {
//!     println!("帧清理");
//! }
//!
//! let mut app = App::new();
//! app.add_systems(AnvilKitSchedule::Startup, setup_system)
//!    .add_systems(AnvilKitSchedule::Update, update_system)
//!    .add_systems(AnvilKitSchedule::PostUpdate, cleanup_system);
//! ```

use bevy_ecs::schedule::*;
pub use bevy_ecs::schedule::ScheduleLabel;

/// AnvilKit 调度标签
/// 
/// 定义了 AnvilKit 中使用的标准调度阶段。
/// 
/// # 调度顺序
/// 
/// 1. `Startup` - 应用启动时执行一次
/// 2. `Main` - 主循环调度器（包含以下子阶段）
///    - `PreUpdate` - 更新前准备
///    - `Update` - 主要更新逻辑
///    - `PostUpdate` - 更新后处理
///    - `Cleanup` - 帧结束清理
/// 
/// # 示例
/// 
/// ```rust
/// use anvilkit_ecs::prelude::*;
/// use anvilkit_ecs::schedule::AnvilKitSchedule;
///
/// fn my_startup_system() {
///     println!("应用启动");
/// }
///
/// fn my_update_system() {
///     println!("每帧更新");
/// }
///
/// let mut app = App::new();
/// app.add_systems(AnvilKitSchedule::Startup, my_startup_system)
///    .add_systems(AnvilKitSchedule::Update, my_update_system);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnvilKitSchedule {
    /// 应用启动时执行一次的系统
    /// 
    /// 用于初始化资源、设置场景、加载配置等一次性操作。
    Startup,
    
    /// 主循环调度器
    /// 
    /// 包含所有每帧执行的系统调度。
    Main,
    
    /// 主更新前的准备阶段
    /// 
    /// 用于输入处理、时间更新、状态准备等。
    PreUpdate,
    
    /// 主要的游戏逻辑更新
    /// 
    /// 包含游戏的核心逻辑，如移动、碰撞检测、AI 等。
    Update,
    
    /// 主更新后的处理阶段
    /// 
    /// 用于变换传播、渲染准备、物理同步等。
    PostUpdate,
    
    /// 帧结束时的清理工作
    /// 
    /// 用于清理临时数据、垃圾回收、统计信息更新等。
    Cleanup,
}

impl ScheduleLabel for AnvilKitSchedule {
    fn dyn_clone(&self) -> Box<dyn ScheduleLabel> {
        Box::new(self.clone())
    }

    fn as_dyn_eq(&self) -> &(dyn bevy_ecs::label::DynEq + 'static) {
        self
    }

    fn dyn_hash(&self, mut state: &mut dyn std::hash::Hasher) {
        use std::hash::Hash;
        self.hash(&mut state);
    }
}



/// 系统集合标签
/// 
/// 用于对相关系统进行分组，便于批量操作和依赖管理。
/// 
/// # 示例
/// 
/// ```rust
/// use anvilkit_ecs::prelude::*;
/// use anvilkit_ecs::schedule::AnvilKitSchedule;
///
/// fn physics_system() {
///     // 物理计算
/// }
///
/// fn collision_system() {
///     // 碰撞检测
/// }
///
/// let mut app = App::new();
/// app.add_systems(AnvilKitSchedule::Update, (
///     physics_system.in_set(AnvilKitSystemSet::Physics),
///     collision_system.in_set(AnvilKitSystemSet::Physics),
/// ));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub enum AnvilKitSystemSet {
    /// 输入处理系统集合
    /// 
    /// 处理键盘、鼠标、手柄等输入设备的系统。
    Input,
    
    /// 时间管理系统集合
    /// 
    /// 更新时间、计时器、帧率等时间相关的系统。
    Time,
    
    /// 物理系统集合
    /// 
    /// 物理模拟、碰撞检测、刚体更新等系统。
    Physics,
    
    /// 游戏逻辑系统集合
    /// 
    /// 游戏的核心逻辑，如 AI、状态机、游戏规则等。
    GameLogic,
    
    /// 变换系统集合
    /// 
    /// Transform 层次传播、坐标变换等系统。
    Transform,
    
    /// 渲染系统集合
    /// 
    /// 渲染准备、可见性计算、材质更新等系统。
    Render,
    
    /// 音频系统集合
    /// 
    /// 音频播放、音效处理、音量控制等系统。
    Audio,
    
    /// UI 系统集合
    /// 
    /// 用户界面更新、布局计算、事件处理等系统。
    UI,
    
    /// 网络系统集合
    /// 
    /// 网络通信、同步、序列化等系统。
    Network,
    
    /// 调试系统集合
    /// 
    /// 调试信息显示、性能监控、日志记录等系统。
    Debug,
}

/// 系统执行条件
///
/// 提供常用的系统执行条件，用于控制系统何时运行。
///
/// # 示例
///
/// ```rust
/// use anvilkit_ecs::prelude::*;
/// use bevy_ecs::schedule::common_conditions::*;
///
/// fn debug_system() {
///     println!("调试信息");
/// }
///
/// let mut app = App::new();
/// app.add_systems(AnvilKitSchedule::Update, debug_system.run_if(|| cfg!(debug_assertions)));
/// ```
pub use bevy_ecs::schedule::common_conditions;

/// 调度构建器
/// 
/// 提供便捷的方法来构建和配置调度器。
/// 
/// # 示例
/// 
/// ```rust
/// use anvilkit_ecs::prelude::*;
///
/// let schedule = ScheduleBuilder::new()
///     .add_systems_to_set(AnvilKitSystemSet::Input, (handle_keyboard, handle_mouse))
///     .add_systems_to_set(AnvilKitSystemSet::Physics, (update_physics, check_collisions))
///     .configure_sets((
///         AnvilKitSystemSet::Input.before(AnvilKitSystemSet::Physics),
///         AnvilKitSystemSet::Physics.before(AnvilKitSystemSet::GameLogic),
///     ))
///     .build();
///
/// fn handle_keyboard() {}
/// fn handle_mouse() {}
/// fn update_physics() {}
/// fn check_collisions() {}
/// ```
pub struct ScheduleBuilder {
    schedule: Schedule,
}

impl ScheduleBuilder {
    /// 创建新的调度构建器
    pub fn new() -> Self {
        Self {
            schedule: Schedule::default(),
        }
    }

    /// 添加系统到指定集合
    pub fn add_systems_to_set<M>(
        mut self,
        set: impl SystemSet,
        systems: impl IntoSystemConfigs<M>,
    ) -> Self {
        self.schedule.add_systems(systems.in_set(set));
        self
    }

    /// 配置系统集合的依赖关系
    pub fn configure_sets(
        mut self,
        sets: impl IntoSystemSetConfigs,
    ) -> Self {
        self.schedule.configure_sets(sets);
        self
    }

    /// 构建调度器
    pub fn build(self) -> Schedule {
        self.schedule
    }
}

impl Default for ScheduleBuilder {
    fn default() -> Self {
        Self::new()
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

    #[derive(Component)]
    #[allow(dead_code)]
    struct TestComponent;

    fn test_system(mut resource: ResMut<TestResource>) {
        resource.value += 1;
    }

    fn conditional_system(mut resource: ResMut<TestResource>) {
        resource.value += 10;
    }

    #[test]
    fn test_schedule_labels() {
        // 测试调度标签的唯一性
        assert_ne!(
            AnvilKitSchedule::Startup.intern(),
            AnvilKitSchedule::Update.intern()
        );
        assert_ne!(
            AnvilKitSchedule::PreUpdate.intern(),
            AnvilKitSchedule::PostUpdate.intern()
        );
    }

    #[test]
    fn test_system_sets() {
        // 测试系统集合的唯一性
        assert_ne!(AnvilKitSystemSet::Input, AnvilKitSystemSet::Physics);
        assert_ne!(AnvilKitSystemSet::GameLogic, AnvilKitSystemSet::Render);
    }

    #[test]
    fn test_run_conditions() {
        let mut app = App::new();
        app.init_resource::<TestResource>();
        
        // 添加条件系统
        app.add_systems(AnvilKitSchedule::Update,
            conditional_system.run_if(common_conditions::resource_exists::<TestResource>)
        );
        
        // 执行更新
        app.update();
        
        let resource = app.world.get_resource::<TestResource>().unwrap();
        assert_eq!(resource.value, 10);
    }

    #[test]
    fn test_schedule_builder() {
        let mut schedule = ScheduleBuilder::new()
            .add_systems_to_set(AnvilKitSystemSet::GameLogic, test_system)
            .build();
        
        let mut world = World::new();
        world.init_resource::<TestResource>();
        
        schedule.run(&mut world);
        
        let resource = world.get_resource::<TestResource>().unwrap();
        assert_eq!(resource.value, 1);
    }
}
