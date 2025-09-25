//! # AnvilKit Core
//! 
//! AnvilKit 游戏引擎的核心基础设施库。
//! 
//! 本 crate 提供了 AnvilKit 生态系统中使用的基础构建块：
//! - **数学系统**: 变换、几何图形、插值和数学常量
//! - **时间管理**: 帧时间跟踪、计时器和时间工具
//! - **错误处理**: 统一的错误类型和结果处理
//! 
//! ## 快速开始
//! 
//! ```rust
//! use anvilkit_core::prelude::*;
//! 
//! // 创建一个 3D 变换
//! let transform = Transform::from_xyz(1.0, 2.0, 3.0)
//!     .with_rotation(Quat::from_rotation_y(std::f32::consts::PI / 4.0))
//!     .with_scale(Vec3::splat(2.0));
//! 
//! // 创建时间管理器
//! let mut time = Time::new();
//! time.update();
//! 
//! println!("Delta time: {:.3}s", time.delta_seconds());
//! ```
//! 
//! ## 特性标志
//! 
//! - `serde`: 启用序列化支持
//! - `debug`: 启用调试功能和额外的验证

pub mod math;
pub mod time;
pub mod error;

/// 预导入模块，包含最常用的类型和函数
pub mod prelude {
    // 数学类型
    pub use crate::math::{Transform, GlobalTransform};
    pub use crate::math::geometry::{Rect, Circle, Bounds2D, Bounds3D};
    pub use crate::math::interpolation::{Lerp, Slerp, Interpolate};
    
    // 时间类型
    pub use crate::time::{Time, Timer};
    
    // 错误类型
    pub use crate::error::{AnvilKitError, Result};
    
    // 重新导出 glam 的常用类型
    pub use glam::{
        Vec2, Vec3, Vec4,
        Mat3, Mat4,
        Quat,
        UVec2, UVec3, UVec4,
        IVec2, IVec3, IVec4,
    };

    // 数学常量
    pub use std::f32::consts as math_consts;
}

// 重新导出核心模块
pub use math::*;
pub use time::*;
pub use error::*;

// 重新导出常用的 glam 类型
pub use glam::{
    Vec2, Vec3, Vec4,
    Mat3, Mat4,
    Quat,
    UVec2, UVec3, UVec4,
    IVec2, IVec3, IVec4,
};

/// AnvilKit Core 的版本信息
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// AnvilKit Core 的构建信息
pub const BUILD_INFO: &str = concat!(
    "AnvilKit Core v",
    env!("CARGO_PKG_VERSION"),
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_info() {
        assert!(!VERSION.is_empty());
        assert!(BUILD_INFO.contains(VERSION));
    }

    #[test]
    fn test_prelude_imports() {
        use crate::prelude::*;
        
        // 测试可以使用预导入的类型
        let _transform = Transform::IDENTITY;
        let _time = Time::new();
        let _rect = Rect::new(Vec2::ZERO, Vec2::ONE);
    }
}
