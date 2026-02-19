//! # 渲染状态共享资源
//!
//! 在 GPU 初始化后由 RenderApp 插入到 ECS World，
//! 供渲染系统读取表面信息和场景 Uniform。

use bevy_ecs::prelude::*;

/// GPU 端单个光源数据 (64 字节)
///
/// | 字段 | 含义 |
/// |------|------|
/// | position_type | xyz=位置(点光/聚光), w=类型 (0=方向光, 1=点光, 2=聚光) |
/// | direction_range | xyz=方向, w=衰减距离 |
/// | color_intensity | rgb=颜色(linear), w=强度 |
/// | params | x=inner_cone_cos, y=outer_cone_cos, z=0, w=0 |
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuLight {
    pub position_type: [f32; 4],
    pub direction_range: [f32; 4],
    pub color_intensity: [f32; 4],
    pub params: [f32; 4],
}

impl Default for GpuLight {
    fn default() -> Self {
        Self {
            position_type: [0.0; 4],
            direction_range: [0.0, -1.0, 0.0, 0.0],
            color_intensity: [0.0; 4],
            params: [0.0; 4],
        }
    }
}

/// 最大光源数量
pub const MAX_LIGHTS: usize = 8;

/// PBR 场景 Uniform (768 字节)
///
/// 包含 per-object 变换、材质参数和多光源数据。
/// 前 256 字节与旧布局兼容（light_dir/light_color 保留但多光源路径不使用）。
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PbrSceneUniform {
    pub model: [[f32; 4]; 4],           // 64 bytes
    pub view_proj: [[f32; 4]; 4],       // 64 bytes
    pub normal_matrix: [[f32; 4]; 4],   // 64 bytes
    pub camera_pos: [f32; 4],           // 16 bytes
    pub light_dir: [f32; 4],            // 16 bytes (legacy / lights[0] shortcut)
    pub light_color: [f32; 4],          // 16 bytes (legacy / lights[0] shortcut)
    pub material_params: [f32; 4],      // 16 bytes (metallic, roughness, normal_scale, light_count)
    // Multi-light array
    pub lights: [GpuLight; MAX_LIGHTS], // 512 bytes (8 * 64)
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
            lights: [GpuLight::default(); MAX_LIGHTS],
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
        assert_eq!(std::mem::size_of::<PbrSceneUniform>(), 768);
    }

    #[test]
    fn test_gpu_light_size() {
        assert_eq!(std::mem::size_of::<GpuLight>(), 64);
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
