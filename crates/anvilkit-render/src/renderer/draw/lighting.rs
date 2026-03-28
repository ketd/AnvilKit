//! 活动相机资源和场景灯光

use bevy_ecs::prelude::*;
use glam::{Mat4, Vec3};

/// 活动相机资源
///
/// 由 camera_system 每帧写入，包含当前激活相机的视图投影矩阵。
#[derive(Resource)]
pub struct ActiveCamera {
    /// Combined view-projection matrix of the active camera.
    pub view_proj: Mat4,
    /// World-space position of the active camera.
    pub camera_pos: Vec3,
    /// Vertical field of view in radians (used by CSM shadow mapping).
    pub fov_radians: f32,
}

impl Default for ActiveCamera {
    fn default() -> Self {
        Self {
            view_proj: Mat4::IDENTITY,
            camera_pos: Vec3::ZERO,
            fov_radians: std::f32::consts::FRAC_PI_4,
        }
    }
}

/// 方向光
#[derive(Debug, Clone)]
pub struct DirectionalLight {
    /// 光照方向（从光源指向场景）
    pub direction: Vec3,
    /// 光照颜色 (linear RGB)
    pub color: Vec3,
    /// 光照强度
    pub intensity: f32,
}

impl Default for DirectionalLight {
    fn default() -> Self {
        Self {
            direction: Vec3::new(-0.5, -0.8, 0.3).normalize(),
            color: Vec3::new(1.0, 0.95, 0.9),
            intensity: 5.0,
        }
    }
}

/// 点光源
#[derive(Debug, Clone)]
pub struct PointLight {
    /// 世界空间位置
    pub position: Vec3,
    /// 光照颜色 (linear RGB)
    pub color: Vec3,
    /// 光照强度
    pub intensity: f32,
    /// 衰减距离（超出此距离光照为零）
    pub range: f32,
}

impl Default for PointLight {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 3.0, 0.0),
            color: Vec3::ONE,
            intensity: 5.0,
            range: 10.0,
        }
    }
}

/// 聚光灯
#[derive(Debug, Clone)]
pub struct SpotLight {
    /// 世界空间位置
    pub position: Vec3,
    /// 光照方向（从光源指向场景）
    pub direction: Vec3,
    /// 光照颜色 (linear RGB)
    pub color: Vec3,
    /// 光照强度
    pub intensity: f32,
    /// 衰减距离
    pub range: f32,
    /// 内锥角（弧度），全亮区域
    pub inner_cone_angle: f32,
    /// 外锥角（弧度），衰减到零
    pub outer_cone_angle: f32,
}

impl Default for SpotLight {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 3.0, 0.0),
            direction: Vec3::new(0.0, -1.0, 0.0),
            color: Vec3::ONE,
            intensity: 10.0,
            range: 15.0,
            inner_cone_angle: 0.35,  // ~20 degrees
            outer_cone_angle: 0.52,  // ~30 degrees
        }
    }
}

/// 最大阴影投射光源数量
///
/// 限制同时投射阴影的光源数量以控制 GPU 内存和性能。
/// 超出此限制的光源不会投射阴影（但仍参与光照计算）。
pub const MAX_SHADOW_LIGHTS: usize = 4;

/// 场景灯光资源
///
/// 持有场景中所有灯光信息，最多 8 盏（1 方向光 + 点光/聚光组合）。
/// 其中最多 [`MAX_SHADOW_LIGHTS`] 个光源可同时投射阴影。
#[derive(Resource)]
pub struct SceneLights {
    /// The primary directional (sun) light.
    pub directional: DirectionalLight,
    /// All active point lights in the scene.
    pub point_lights: Vec<PointLight>,
    /// All active spot lights in the scene.
    pub spot_lights: Vec<SpotLight>,
}

impl Default for SceneLights {
    fn default() -> Self {
        Self {
            directional: DirectionalLight::default(),
            point_lights: Vec::new(),
            spot_lights: Vec::new(),
        }
    }
}
