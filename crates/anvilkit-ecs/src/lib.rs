//! # AnvilKit ECS
//! 
//! AnvilKit ECS 是基于 Bevy ECS 构建的高性能实体组件系统模块，为 AnvilKit 游戏引擎提供
//! 数据驱动的架构支持。
//! 
//! ## 核心特性
//! 
//! - **高性能 ECS**: 基于 Bevy ECS 的成熟架构，支持并行系统执行
//! - **组件集成**: 与 anvilkit-core 的数学和时间系统无缝集成
//! - **插件系统**: 模块化的插件架构，支持功能扩展
//! - **变换层次**: 完整的 Transform 和 GlobalTransform 层次系统
//! - **系统调度**: 灵活的系统调度和执行顺序管理
//! 
//! ## 设计理念
//! 
//! 1. **数据驱动**: 组件存储数据，系统处理逻辑，实体仅作为标识符
//! 2. **缓存友好**: 组件按类型连续存储，提高内存访问效率
//! 3. **并行执行**: 系统可以并行运行，充分利用多核性能
//! 4. **类型安全**: 编译时检查组件访问，避免运行时错误
//! 
//! ## 使用示例
//! 
//! ```rust
//! use anvilkit_ecs::prelude::*;
//! use anvilkit_core::math::Transform;
//! 
//! // 定义组件
//! #[derive(Component)]
//! struct Player {
//!     name: String,
//!     health: f32,
//! }
//! 
//! #[derive(Component)]
//! struct Velocity {
//!     x: f32,
//!     y: f32,
//! }
//! 
//! // 定义系统
//! fn movement_system(mut query: Query<(&mut Transform, &Velocity)>) {
//!     for (mut transform, velocity) in &mut query {
//!         transform.translation.x += velocity.x;
//!         transform.translation.y += velocity.y;
//!     }
//! }
//! 
//! // 创建应用
//! let mut app = App::new();
//! app.add_plugins(AnvilKitEcsPlugin)
//!    .add_systems(Update, movement_system);
//! ```

pub mod app;
pub mod bundle;
pub mod component;
pub mod plugin;
pub mod schedule;
pub mod system;
pub mod transform;

/// 预导入模块，包含最常用的类型和 trait
pub mod prelude {
    pub use crate::app::*;
    pub use crate::bundle::*;
    pub use crate::component::*;
    pub use crate::plugin::*;
    pub use crate::schedule::*;
    pub use crate::system::*;
    pub use crate::transform::*;
    
    // 重新导出 Bevy ECS 的核心类型
    pub use bevy_ecs::prelude::*;
    
    // 重新导出 anvilkit-core 的相关类型
    pub use anvilkit_core::prelude::*;
}

// 版本信息
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;

    #[test]
    fn test_version_info() {
        assert!(!VERSION.is_empty());
        assert_eq!(NAME, "anvilkit-ecs");
    }

    #[test]
    fn test_prelude_imports() {
        // 测试预导入模块是否正常工作
        let _world = World::new();
        let _app = App::new();
    }
}
