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

/// Physics component types (RigidBody, Collider, Velocity, etc.).
pub mod components;
/// Collision event types.
pub mod events;
/// AABB collision detection and velocity integration.
pub mod aabb;
/// Rapier3d physics integration (feature-gated).
#[cfg(feature = "rapier")]
pub mod rapier;

pub use components::*;
pub use events::*;
pub use aabb::*;
#[cfg(feature = "rapier")]
pub use rapier::rapier_integration::*;
