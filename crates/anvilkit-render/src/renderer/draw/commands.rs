//! 绘制命令和绘制命令列表

use bevy_ecs::prelude::*;
use anvilkit_describe::Describe;
use glam::Mat4;

use crate::renderer::assets::{MeshHandle, MaterialHandle};

/// 材质参数组件
///
/// 附加到实体上以控制 PBR 材质参数。
/// 如果实体没有此组件，render_extract_system 使用默认值 (metallic=0, roughness=0.5, normal_scale=1.0)。
#[derive(Debug, Clone, Component, Describe)]
/// Per-entity PBR material parameter overrides.
pub struct MaterialParams {
    /// Metalness factor (0 = dielectric, 1 = metal).
    #[describe(hint = "0 = dielectric, 1 = full metal", range = "0.0..1.0", default = "0.0")]
    pub metallic: f32,
    /// Surface roughness (0 = mirror, 1 = diffuse).
    #[describe(hint = "0 = mirror-smooth, 1 = fully rough", range = "0.0..1.0", default = "0.5")]
    pub roughness: f32,
    /// Normal map intensity multiplier.
    #[describe(hint = "Normal map intensity multiplier", range = "0.0..2.0", default = "1.0")]
    pub normal_scale: f32,
    /// Emissive color factor [R, G, B].
    #[describe(hint = "Emissive color factor [R,G,B]", default = "[0.0, 0.0, 0.0]")]
    pub emissive_factor: [f32; 3],
}

impl Default for MaterialParams {
    fn default() -> Self {
        Self {
            metallic: 0.0,
            roughness: 0.5,
            normal_scale: 1.0,
            emissive_factor: [0.0; 3],
        }
    }
}

/// 单个绘制命令
pub struct DrawCommand {
    /// Handle to the GPU mesh to draw.
    pub mesh: MeshHandle,
    /// Handle to the GPU material (pipeline + bind group).
    pub material: MaterialHandle,
    /// Object-to-world transform matrix.
    pub model_matrix: Mat4,
    /// PBR metalness factor for this draw.
    pub metallic: f32,
    /// PBR roughness factor for this draw.
    pub roughness: f32,
    /// Normal map intensity for this draw.
    pub normal_scale: f32,
    /// Emissive color factor [R, G, B] for this draw.
    pub emissive_factor: [f32; 3],
}

/// 每帧的绘制命令列表
///
/// 由 render_extract_system 填充，由 RenderApp::render_ecs() 消费。
/// 支持按 mesh+material 排序分组以减少管线状态切换。
#[derive(Resource, Default)]
pub struct DrawCommandList {
    /// Collected draw commands for the current frame.
    pub commands: Vec<DrawCommand>,
}

impl DrawCommandList {
    /// Removes all draw commands from the list.
    pub fn clear(&mut self) {
        self.commands.clear();
    }

    /// Appends a draw command to the list.
    pub fn push(&mut self, cmd: DrawCommand) {
        self.commands.push(cmd);
    }

    /// 按 (material, mesh) 排序以实现批处理
    ///
    /// 相同 material 的命令排在一起，减少管线状态切换。
    /// 相同 mesh 的命令排在一起，减少顶点缓冲区切换。
    pub fn sort_for_batching(&mut self) {
        self.commands.sort_by(|a, b| {
            a.material.index().cmp(&b.material.index())
                .then(a.mesh.index().cmp(&b.mesh.index()))
        });
    }
}
