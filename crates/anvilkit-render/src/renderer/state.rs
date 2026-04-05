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
    /// xyz = world position, w = light type (0=directional, 1=point, 2=spot).
    pub position_type: [f32; 4],
    /// xyz = light direction, w = attenuation range.
    pub direction_range: [f32; 4],
    /// rgb = linear color, w = intensity.
    pub color_intensity: [f32; 4],
    /// x = inner cone cosine, y = outer cone cosine, zw = reserved.
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

/// Cascade Shadow Maps 级数
pub const CSM_CASCADE_COUNT: usize = 3;

/// PBR 场景 Uniform (992 字节)
///
/// 包含 per-object 变换、材质参数、多光源数据和 CSM 矩阵。
/// 前 256 字节与旧布局兼容（light_dir/light_color 保留但多光源路径不使用）。
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PbrSceneUniform {
    /// Object-to-world model transform (64 bytes).
    pub model: [[f32; 4]; 4],
    /// Combined view-projection matrix (64 bytes).
    pub view_proj: [[f32; 4]; 4],
    /// Inverse-transpose of the model matrix for normal transforms (64 bytes).
    pub normal_matrix: [[f32; 4]; 4],
    /// Camera world-space position, w unused (16 bytes).
    pub camera_pos: [f32; 4],
    /// Legacy primary light direction, xyz = direction, w unused (16 bytes).
    pub light_dir: [f32; 4],
    /// Legacy primary light color, rgb = color, w = intensity (16 bytes).
    pub light_color: [f32; 4],
    /// Material parameters: [metallic, roughness, normal_scale, light_count] (16 bytes).
    pub material_params: [f32; 4],
    /// Multi-light array, up to `MAX_LIGHTS` entries (512 bytes).
    pub lights: [GpuLight; MAX_LIGHTS],
    /// Cascade shadow map view-projection matrices (3 × 64 = 192 bytes).
    pub cascade_view_projs: [[[f32; 4]; 4]; CSM_CASCADE_COUNT],
    /// Cascade far plane split distances in view-space [c0, c1, c2, shadow_texel_size] (16 bytes).
    pub cascade_splits: [f32; 4],
    /// Emissive factor rgb, w = cascade_count (16 bytes).
    pub emissive_factor: [f32; 4],
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
            cascade_view_projs: [glam::Mat4::IDENTITY.to_cols_array_2d(); CSM_CASCADE_COUNT],
            cascade_splits: [10.0, 30.0, 100.0, 1.0 / 2048.0],
            emissive_factor: [0.0, 0.0, 0.0, CSM_CASCADE_COUNT as f32],
        }
    }
}

/// 共享渲染状态
///
/// 持有与 GPU 表面和场景 Uniform 相关的资源。
/// RenderApp 在 GPU 初始化后将其插入 World。
#[derive(Resource)]
pub struct RenderState {
    /// Swapchain surface texture format.
    pub surface_format: wgpu::TextureFormat,
    /// Current surface dimensions (width, height) in pixels.
    pub surface_size: (u32, u32),
    /// GPU buffer holding the PBR scene uniform data.
    pub scene_uniform_buffer: wgpu::Buffer,
    /// Bind group exposing the scene uniform buffer to shaders.
    pub scene_bind_group: wgpu::BindGroup,
    /// Layout for the scene uniform bind group.
    pub scene_bind_group_layout: wgpu::BindGroupLayout,
    /// Depth buffer texture view for the main pass.
    pub depth_texture_view: wgpu::TextureView,
    /// HDR off-screen render target texture (retained for copy operations).
    pub hdr_texture: wgpu::Texture,
    /// HDR off-screen render target texture view.
    pub hdr_texture_view: wgpu::TextureView,
    /// Tone-mapping post-process render pipeline.
    pub tonemap_pipeline: wgpu::RenderPipeline,
    /// Bind group for the tone-mapping pass inputs.
    pub tonemap_bind_group: wgpu::BindGroup,
    /// Layout for the tone-mapping bind group.
    pub tonemap_bind_group_layout: wgpu::BindGroupLayout,
    /// Bind group for IBL environment and shadow map sampling (group 2).
    pub ibl_shadow_bind_group: wgpu::BindGroup,
    /// Layout for the IBL and shadow bind group.
    pub ibl_shadow_bind_group_layout: wgpu::BindGroupLayout,
    /// Shadow-only depth render pipeline.
    pub shadow_pipeline: wgpu::RenderPipeline,
    /// Shadow map depth texture view (D2Array for CSM sampling).
    pub shadow_map_view: wgpu::TextureView,
    /// Per-cascade shadow map layer views (for rendering into individual layers).
    pub shadow_cascade_views: Vec<wgpu::TextureView>,
    /// MSAA multi-sampled HDR color attachment texture view.
    pub hdr_msaa_texture_view: wgpu::TextureView,
    /// Bloom post-processing GPU resources (mip chain, pipelines, bind groups).
    pub bloom: Option<crate::renderer::bloom::BloomResources>,
    /// 后处理 GPU 资源集合（SSAO, DOF, MotionBlur, ColorGrading）
    pub post_process: crate::renderer::post_process::PostProcessResources,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pbr_scene_uniform_size() {
        // 768 (old fields before shadow_view_proj) + 192 (3 cascade matrices) + 16 (cascade_splits) + 16 (emissive) = 992
        assert_eq!(std::mem::size_of::<PbrSceneUniform>(), 992);
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
