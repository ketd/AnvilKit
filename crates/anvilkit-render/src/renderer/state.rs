//! # 渲染状态共享资源
//!
//! 在 GPU 初始化后由 RenderApp 插入到 ECS World，
//! 供渲染系统读取表面信息和场景 Uniform。

use bevy_ecs::prelude::*;

/// PBR 场景 Uniform (256 字节)
///
/// 包含所有 PBR 直接光照所需的 per-object 数据：
/// model/view_proj/normal_matrix 变换 + 相机位置 + 方向光 + 材质参数。
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PbrSceneUniform {
    pub model: [[f32; 4]; 4],           // 64 bytes
    pub view_proj: [[f32; 4]; 4],       // 64 bytes
    pub normal_matrix: [[f32; 4]; 4],   // 64 bytes
    pub camera_pos: [f32; 4],           // 16 bytes
    pub light_dir: [f32; 4],            // 16 bytes
    pub light_color: [f32; 4],          // 16 bytes (rgb + intensity)
    pub material_params: [f32; 4],      // 16 bytes (metallic, roughness, normal_scale, 0)
}

impl Default for PbrSceneUniform {
    fn default() -> Self {
        Self {
            model: glam::Mat4::IDENTITY.to_cols_array_2d(),
            view_proj: glam::Mat4::IDENTITY.to_cols_array_2d(),
            normal_matrix: glam::Mat4::IDENTITY.to_cols_array_2d(),
            camera_pos: [0.0; 4],
            light_dir: [0.0, -1.0, 0.0, 0.0],
            light_color: [1.0, 1.0, 1.0, 3.0],
            material_params: [0.0, 0.5, 1.0, 0.0],
        }
    }
}

/// 共享渲染状态
///
/// 持有与 GPU 表面和场景 Uniform 相关的资源。
/// RenderApp 在 GPU 初始化后将其插入 World。
#[derive(Resource)]
pub struct RenderState {
    pub surface_format: wgpu::TextureFormat,
    pub surface_size: (u32, u32),
    pub scene_uniform_buffer: wgpu::Buffer,
    pub scene_bind_group: wgpu::BindGroup,
    pub scene_bind_group_layout: wgpu::BindGroupLayout,
    pub depth_texture_view: wgpu::TextureView,
    // HDR multi-pass rendering
    pub hdr_texture_view: wgpu::TextureView,
    pub tonemap_pipeline: wgpu::RenderPipeline,
    pub tonemap_bind_group: wgpu::BindGroup,
    pub tonemap_bind_group_layout: wgpu::BindGroupLayout,
    // IBL
    pub ibl_bind_group: wgpu::BindGroup,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pbr_scene_uniform_size() {
        assert_eq!(std::mem::size_of::<PbrSceneUniform>(), 256);
    }

    #[test]
    fn test_pbr_scene_uniform_default() {
        let u = PbrSceneUniform::default();
        // model should be identity
        assert_eq!(u.model[0][0], 1.0);
        assert_eq!(u.model[1][1], 1.0);
        // default light direction points down
        assert_eq!(u.light_dir[1], -1.0);
        // default roughness = 0.5
        assert_eq!(u.material_params[1], 0.5);
        // default normal_scale = 1.0
        assert_eq!(u.material_params[2], 1.0);
    }
}
