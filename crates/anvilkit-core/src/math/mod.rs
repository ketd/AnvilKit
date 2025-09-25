//! # 数学系统
//! 
//! AnvilKit 的数学系统提供了游戏开发中常用的数学类型和操作。
//! 
//! ## 模块组织
//! 
//! - [`transform`]: 3D 变换和层次结构
//! - [`geometry`]: 几何图形和边界框
//! - [`interpolation`]: 插值和动画支持
//! - [`constants`]: 数学常量和工具函数
//! 
//! ## 设计原则
//! 
//! 1. **统一但非均一**: 提供统一的 API，但针对 2D/3D 进行优化
//! 2. **性能优先**: 使用 SIMD 优化和缓存友好的数据布局
//! 3. **类型安全**: 利用 Rust 的类型系统防止常见错误
//! 4. **可扩展性**: 通过 trait 提供可扩展的接口

pub mod transform;
pub mod geometry;
pub mod interpolation;
pub mod constants;

// 重新导出主要类型
pub use transform::{Transform, GlobalTransform};
pub use geometry::{Rect, Circle, Bounds2D, Bounds3D};
pub use interpolation::{Lerp, Slerp, Interpolate};
pub use constants::*;

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;

    #[test]
    fn test_math_module_integration() {
        // 测试不同模块之间的集成
        let transform = Transform::from_xyz(1.0, 2.0, 3.0);
        let bounds = Bounds3D::from_center_size(transform.translation, Vec3::ONE);
        
        assert!(bounds.contains(transform.translation));
    }
}
