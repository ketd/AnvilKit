//! # 绘制命令、相机资源和场景灯光
//!
//! 提供 ECS 渲染系统的中间表示：绘制命令列表、活动相机信息和场景灯光。

use bevy_ecs::prelude::*;
use glam::{Mat4, Vec3};

use crate::renderer::assets::{MeshHandle, MaterialHandle};

/// 活动相机资源
///
/// 由 camera_system 每帧写入，包含当前激活相机的视图投影矩阵。
#[derive(Resource)]
pub struct ActiveCamera {
    pub view_proj: Mat4,
    pub camera_pos: Vec3,
}

impl Default for ActiveCamera {
    fn default() -> Self {
        Self {
            view_proj: Mat4::IDENTITY,
            camera_pos: Vec3::ZERO,
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

/// 场景灯光资源
///
/// 持有场景中的灯光信息，由 RenderPlugin 注册默认值。
#[derive(Resource)]
pub struct SceneLights {
    pub directional: DirectionalLight,
}

impl Default for SceneLights {
    fn default() -> Self {
        Self {
            directional: DirectionalLight::default(),
        }
    }
}

/// 材质参数组件
///
/// 附加到实体上以控制 PBR 材质参数。
/// 如果实体没有此组件，render_extract_system 使用默认值 (metallic=0, roughness=0.5, normal_scale=1.0)。
#[derive(Debug, Clone, Component)]
pub struct MaterialParams {
    pub metallic: f32,
    pub roughness: f32,
    pub normal_scale: f32,
}

impl Default for MaterialParams {
    fn default() -> Self {
        Self {
            metallic: 0.0,
            roughness: 0.5,
            normal_scale: 1.0,
        }
    }
}

/// 单个绘制命令
pub struct DrawCommand {
    pub mesh: MeshHandle,
    pub material: MaterialHandle,
    pub model_matrix: Mat4,
    pub metallic: f32,
    pub roughness: f32,
    pub normal_scale: f32,
}

/// 每帧的绘制命令列表
///
/// 由 render_extract_system 填充，由 RenderApp::render_ecs() 消费。
#[derive(Resource, Default)]
pub struct DrawCommandList {
    pub commands: Vec<DrawCommand>,
}

impl DrawCommandList {
    pub fn clear(&mut self) {
        self.commands.clear();
    }

    pub fn push(&mut self, cmd: DrawCommand) {
        self.commands.push(cmd);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_directional_light_default() {
        let light = DirectionalLight::default();
        assert!(light.direction.length() > 0.99);
        assert!(light.intensity > 0.0);
    }

    #[test]
    fn test_scene_lights_default() {
        let lights = SceneLights::default();
        assert!(lights.directional.intensity > 0.0);
    }

    #[test]
    fn test_material_params_default() {
        let params = MaterialParams::default();
        assert_eq!(params.metallic, 0.0);
        assert_eq!(params.roughness, 0.5);
        assert_eq!(params.normal_scale, 1.0);
    }
}
