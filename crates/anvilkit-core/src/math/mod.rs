//! # 数学系统
//!
//! AnvilKit 的数学系统提供了游戏开发中常用的数学类型和操作。
//!
//! ## 模块组织
//!
//! - [`transform`]: 3D 变换和层次结构
//! - [`aabb`]: Axis-aligned bounding boxes
//! - [`frustum`]: View frustum for culling
//! - [`raycast`]: Ray casting

pub mod transform;
pub mod aabb;
pub mod frustum;
pub mod raycast;

// 重新导出主要类型
pub use transform::{Transform, GlobalTransform};
pub use aabb::Aabb;
pub use frustum::Frustum;

/// 速度组件 — linear + angular velocity
#[cfg_attr(feature = "bevy_ecs", derive(bevy_ecs::prelude::Component))]
#[derive(Debug, Clone, Copy)]
pub struct Velocity {
    /// Linear velocity vector (units per second).
    pub linear: glam::Vec3,
    /// Angular velocity vector (radians per second around each axis).
    pub angular: glam::Vec3,
}

impl Velocity {
    /// Creates a velocity with zero linear and angular components.
    pub fn zero() -> Self { Self { linear: glam::Vec3::ZERO, angular: glam::Vec3::ZERO } }
    /// Creates a velocity with the given linear component and zero angular velocity.
    pub fn linear(linear: glam::Vec3) -> Self { Self { linear, angular: glam::Vec3::ZERO } }
}

impl Default for Velocity {
    fn default() -> Self { Self::zero() }
}
