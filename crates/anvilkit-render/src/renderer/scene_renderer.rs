//! # 场景渲染编排层
//!
//! 封装多 pass 渲染管线编排，自动管理 window resize 时的 GPU 资源重建，
//! 以及后处理效果链的动态调度。
//!
//! ## 设计
//!
//! `SceneRenderer` 不是一个 ECS Resource，而是一组静态方法，
//! 操作 `RenderState` 和 `PostProcessResources`。这样避免了所有权问题，
//! 同时提供了共享的渲染逻辑给 `RenderApp` 和未来的 `DemoApp` 脚手架使用。

use crate::renderer::RenderDevice;
use crate::renderer::state::RenderState;
use crate::renderer::buffer::{
    create_depth_texture_msaa, create_hdr_render_target, create_hdr_msaa_texture, create_sampler,
};
use crate::renderer::post_process::PostProcessSettings;
use log::debug;

/// Pipeline 创建参数（从 RenderConfig 提取）
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    /// MSAA 采样数
    pub msaa_samples: u32,
    /// 清除颜色 [r, g, b, a]
    pub clear_color: [f32; 4],
    /// 默认剔除模式
    pub cull_mode: wgpu::Face,
}

/// 场景渲染编排器
///
/// 提供自动 resize 和后处理资源管理的静态方法。
pub struct SceneRenderer;

impl SceneRenderer {
    /// 处理窗口大小变化 — 重建所有 size-dependent GPU 资源
    ///
    /// 重建：depth texture, HDR RT, MSAA color, bloom mip chain, tonemap bind group,
    /// 以及所有后处理资源。
    ///
    /// # 参数
    ///
    /// - `device`: GPU 设备
    /// - `rs`: 可变 RenderState 引用
    /// - `width`, `height`: 新的窗口尺寸
    /// - `bloom_mip_count`: bloom mip chain 层数
    pub fn handle_resize(
        device: &RenderDevice,
        rs: &mut RenderState,
        width: u32,
        height: u32,
        bloom_mip_count: u32,
    ) {
        if width == 0 || height == 0 {
            return;
        }

        debug!("SceneRenderer: resize {}x{}", width, height);

        rs.surface_size = (width, height);

        // 重建 depth texture (MSAA)
        let (_, depth_view) = create_depth_texture_msaa(device, width, height, "ECS Depth MSAA");
        rs.depth_texture_view = depth_view;

        // 重建 HDR render target (resolve) + MSAA color
        let (hdr_tex, hdr_view) = create_hdr_render_target(device, width, height, "ECS HDR RT");
        let (_, hdr_msaa_view) = create_hdr_msaa_texture(device, width, height, "ECS HDR MSAA");
        let sampler = create_sampler(device, "ECS Sampler");

        // Resize bloom mip chain
        if let Some(ref mut bloom) = rs.bloom {
            bloom.resize(device, width, height, bloom_mip_count);
        }

        // 重建 tonemap bind group
        let bloom_view = rs.bloom.as_ref()
            .and_then(|b| b.mip_views.first());
        let bloom_view_ref = bloom_view.unwrap_or(&hdr_view);
        let new_bg = device.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ECS Tonemap BG"),
            layout: &rs.tonemap_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&hdr_view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sampler) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(bloom_view_ref) },
            ],
        });

        rs.hdr_texture = hdr_tex;
        rs.hdr_texture_view = hdr_view;
        rs.hdr_msaa_texture_view = hdr_msaa_view;
        rs.tonemap_bind_group = new_bg;

        // Resize 后处理资源
        rs.post_process.resize(device, width, height);
    }

    /// 确保后处理 GPU 资源已初始化
    ///
    /// 根据 `PostProcessSettings` 延迟创建需要的 GPU 资源。
    /// 应在每帧 render 之前调用。
    pub fn ensure_post_process_resources(
        device: &RenderDevice,
        rs: &mut RenderState,
        settings: &PostProcessSettings,
    ) {
        let (w, h) = rs.surface_size;
        rs.post_process.ensure_resources(device, w, h, settings);
    }

    /// 从 RenderConfig 读取渲染参数
    ///
    /// 用于 pipeline 创建时应用用户配置（MSAA、clear color、cull mode）。
    pub fn get_pipeline_config(config: &crate::plugin::RenderConfig) -> PipelineConfig {
        PipelineConfig {
            msaa_samples: config.msaa_samples,
            clear_color: config.clear_color,
            cull_mode: config.default_cull_mode,
        }
    }

    /// 按优先级排序相机
    ///
    /// 高优先级相机先渲染。用于多相机场景（如 minimap）。
    pub fn sort_cameras_by_priority(cameras: &mut [(i32, glam::Mat4)]) {
        cameras.sort_by(|a, b| a.0.cmp(&b.0));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scene_renderer_is_zero_sized() {
        // SceneRenderer 是纯静态方法集合，零大小
        assert_eq!(std::mem::size_of::<SceneRenderer>(), 0);
    }

    #[test]
    fn test_sort_cameras_by_priority() {
        let mut cameras = vec![
            (10, glam::Mat4::IDENTITY),
            (1, glam::Mat4::IDENTITY),
            (5, glam::Mat4::IDENTITY),
        ];
        SceneRenderer::sort_cameras_by_priority(&mut cameras);
        assert_eq!(cameras[0].0, 1);
        assert_eq!(cameras[1].0, 5);
        assert_eq!(cameras[2].0, 10);
    }
}
