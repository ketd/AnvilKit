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
    Sphere {
        /// Sphere radius.
        radius: f32,
    },
    /// 长方体（半尺寸）
    Cuboid {
        /// Half-extents along each axis.
        half_extents: Vec3,
    },
    /// 胶囊体
    Capsule {
        /// Half the height of the cylindrical section.
        half_height: f32,
        /// Radius of the capsule hemispheres.
        radius: f32,
    },
    /// 网格碰撞体（顶点 + 三角形索引）
    TriMesh {
        /// Mesh vertex positions.
        vertices: Vec<Vec3>,
        /// Triangle index triples.
        indices: Vec<[u32; 3]>,
    },
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
    /// The type of rigid body (dynamic, fixed, or kinematic).
    pub body_type: RigidBodyType,
    /// Mass of the rigid body in kilograms.
    pub mass: f32,
    /// Multiplier for gravity applied to this body (1.0 = normal gravity).
    pub gravity_scale: f32,
    /// Whether this body acts as a sensor (detects overlaps without physical response).
    pub is_sensor: bool,
    /// Linear velocity damping factor (resistance to linear motion).
    pub linear_damping: f32,
    /// Angular velocity damping factor (resistance to rotational motion).
    pub angular_damping: f32,
}

impl RigidBody {
    /// Creates a new rigid body with the given type and default physical properties.
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

    /// Creates a new dynamic rigid body.
    pub fn dynamic() -> Self { Self::new(RigidBodyType::Dynamic) }
    /// Creates a new fixed rigid body.
    pub fn fixed() -> Self { Self::new(RigidBodyType::Fixed) }
    /// Creates a new kinematic rigid body.
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
    /// The geometric shape of this collider.
    pub shape: ColliderShape,
    /// Surface friction coefficient (0.0 = frictionless, 1.0 = high friction).
    pub friction: f32,
    /// Bounciness coefficient (0.0 = no bounce, 1.0 = perfectly elastic).
    pub restitution: f32,
}

impl Collider {
    /// Creates a sphere collider with the given radius.
    pub fn sphere(radius: f32) -> Self {
        Self { shape: ColliderShape::Sphere { radius }, friction: 0.5, restitution: 0.0 }
    }

    /// Creates a box collider with the given half-extents.
    pub fn cuboid(half_extents: Vec3) -> Self {
        Self { shape: ColliderShape::Cuboid { half_extents }, friction: 0.5, restitution: 0.0 }
    }

    /// Creates a capsule collider with the given half-height and radius.
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
    /// Linear velocity vector (units per second).
    pub linear: Vec3,
    /// Angular velocity vector (radians per second around each axis).
    pub angular: Vec3,
}

impl Velocity {
    /// Creates a velocity with zero linear and angular components.
    pub fn zero() -> Self { Self { linear: Vec3::ZERO, angular: Vec3::ZERO } }
    /// Creates a velocity with the given linear component and zero angular velocity.
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
