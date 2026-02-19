//! # 物理组件
//!
//! 定义 ECS 物理组件类型，用于与 rapier 物理引擎集成。
//! 组件定义不依赖 rapier，实际物理模拟由 PhysicsPlugin 提供。
//!
//! ## 使用示例
//!
//! ```rust
//! use anvilkit_ecs::physics::{RigidBody, RigidBodyType, Collider, ColliderShape, Velocity};
//! use glam::Vec3;
//!
//! let body = RigidBody::new(RigidBodyType::Dynamic);
//! assert_eq!(body.body_type, RigidBodyType::Dynamic);
//!
//! let collider = Collider::sphere(0.5);
//! let velocity = Velocity::linear(Vec3::new(1.0, 0.0, 0.0));
//! ```

use bevy_ecs::prelude::*;
use glam::Vec3;

/// 刚体类型
///
/// # 示例
///
/// ```rust
/// use anvilkit_ecs::physics::RigidBodyType;
///
/// let dynamic = RigidBodyType::Dynamic;
/// let fixed = RigidBodyType::Fixed;
/// assert_ne!(dynamic, fixed);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RigidBodyType {
    /// 动态刚体（受力和重力影响）
    Dynamic,
    /// 固定刚体（不移动）
    Fixed,
    /// 运动学刚体（手动控制位置，不受力影响）
    Kinematic,
}

/// 碰撞体形状
///
/// # 示例
///
/// ```rust
/// use anvilkit_ecs::physics::ColliderShape;
///
/// let sphere = ColliderShape::Sphere { radius: 1.0 };
/// let cube = ColliderShape::Cuboid { half_extents: glam::Vec3::ONE };
/// ```
#[derive(Debug, Clone)]
pub enum ColliderShape {
    /// 球体
    Sphere { radius: f32 },
    /// 长方体（半尺寸）
    Cuboid { half_extents: Vec3 },
    /// 胶囊体
    Capsule { half_height: f32, radius: f32 },
    /// 网格碰撞体（顶点 + 三角形索引）
    TriMesh { vertices: Vec<Vec3>, indices: Vec<[u32; 3]> },
}

/// 刚体组件
///
/// 附加到实体上表示该实体参与物理模拟。
///
/// # 示例
///
/// ```rust
/// use anvilkit_ecs::physics::{RigidBody, RigidBodyType};
///
/// let body = RigidBody::new(RigidBodyType::Dynamic);
/// assert!(!body.is_sensor);
/// assert_eq!(body.mass, 1.0);
/// ```
#[derive(Debug, Clone, Component)]
pub struct RigidBody {
    pub body_type: RigidBodyType,
    pub mass: f32,
    pub gravity_scale: f32,
    pub is_sensor: bool,
    pub linear_damping: f32,
    pub angular_damping: f32,
}

impl RigidBody {
    pub fn new(body_type: RigidBodyType) -> Self {
        Self {
            body_type,
            mass: 1.0,
            gravity_scale: 1.0,
            is_sensor: false,
            linear_damping: 0.0,
            angular_damping: 0.05,
        }
    }

    pub fn dynamic() -> Self { Self::new(RigidBodyType::Dynamic) }
    pub fn fixed() -> Self { Self::new(RigidBodyType::Fixed) }
    pub fn kinematic() -> Self { Self::new(RigidBodyType::Kinematic) }
}

/// 碰撞体组件
///
/// 定义实体的碰撞形状和物理材质。
///
/// # 示例
///
/// ```rust
/// use anvilkit_ecs::physics::Collider;
///
/// let sphere = Collider::sphere(0.5);
/// assert_eq!(sphere.friction, 0.5);
/// assert_eq!(sphere.restitution, 0.0);
/// ```
#[derive(Debug, Clone, Component)]
pub struct Collider {
    pub shape: ColliderShape,
    pub friction: f32,
    pub restitution: f32,
}

impl Collider {
    pub fn sphere(radius: f32) -> Self {
        Self { shape: ColliderShape::Sphere { radius }, friction: 0.5, restitution: 0.0 }
    }

    pub fn cuboid(half_extents: Vec3) -> Self {
        Self { shape: ColliderShape::Cuboid { half_extents }, friction: 0.5, restitution: 0.0 }
    }

    pub fn capsule(half_height: f32, radius: f32) -> Self {
        Self { shape: ColliderShape::Capsule { half_height, radius }, friction: 0.5, restitution: 0.0 }
    }
}

/// 速度组件
///
/// # 示例
///
/// ```rust
/// use anvilkit_ecs::physics::Velocity;
/// use glam::Vec3;
///
/// let vel = Velocity::linear(Vec3::X);
/// assert_eq!(vel.linear, Vec3::X);
/// assert_eq!(vel.angular, Vec3::ZERO);
/// ```
#[derive(Debug, Clone, Copy, Component)]
pub struct Velocity {
    pub linear: Vec3,
    pub angular: Vec3,
}

impl Velocity {
    pub fn zero() -> Self { Self { linear: Vec3::ZERO, angular: Vec3::ZERO } }
    pub fn linear(linear: Vec3) -> Self { Self { linear, angular: Vec3::ZERO } }
}

impl Default for Velocity {
    fn default() -> Self { Self::zero() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rigid_body() {
        let body = RigidBody::dynamic();
        assert_eq!(body.body_type, RigidBodyType::Dynamic);
        assert_eq!(body.mass, 1.0);
    }

    #[test]
    fn test_collider() {
        let c = Collider::sphere(1.0);
        assert_eq!(c.friction, 0.5);
        match c.shape {
            ColliderShape::Sphere { radius } => assert_eq!(radius, 1.0),
            _ => panic!("wrong shape"),
        }
    }

    #[test]
    fn test_velocity() {
        let v = Velocity::linear(Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(v.linear, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(v.angular, Vec3::ZERO);
    }
}
