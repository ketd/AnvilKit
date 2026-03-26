//! # 渲染器模块
//!
//! 提供基于 wgpu 的跨平台图形渲染功能，包括设备管理、表面配置和渲染管线。
//!
//! ## 核心组件
//!
//! - **RenderDevice**: GPU 设备和适配器管理
//! - **RenderSurface**: 窗口表面和交换链管理
//! - **RenderPipeline**: 渲染管线抽象
//! - **RenderState**: ECS 共享渲染状态
//! - **RenderAssets**: GPU 资产管理
//!
//! ## 设计理念
//!
//! 本模块采用现代图形 API 设计，支持多种后端（Vulkan、Metal、D3D12、OpenGL、WebGPU），
//! 提供高性能的零成本抽象和灵活的渲染管线配置。

pub mod device;
pub mod surface;
pub mod pipeline;
pub mod buffer;
pub mod assets;
pub mod draw;
pub mod state;
pub mod ibl;
pub mod sprite;
pub mod ui;
pub mod particle;
pub mod debug;
pub mod raycast;
pub mod line;
pub mod text;
pub mod buffer_pool;
pub mod bloom;
pub mod ssao;
pub mod dof;
pub mod motion_blur;
pub mod color_grading;
pub mod debug_renderer;
pub mod post_process;
#[cfg(feature = "capture")]
pub mod capture;

// 重新导出主要类型
pub use device::RenderDevice;
pub use surface::RenderSurface;
pub use pipeline::{RenderPipelineBuilder, BasicRenderPipeline};
pub use buffer::{
    Vertex, ColorVertex, MeshVertex, PbrVertex, SkinnedVertex,
    create_vertex_buffer, create_index_buffer, create_index_buffer_u32,
    create_uniform_buffer, create_depth_texture, create_hdr_render_target,
    DEPTH_FORMAT, HDR_FORMAT,
    create_texture, create_texture_linear, create_sampler,
    create_shadow_map, create_shadow_sampler, SHADOW_MAP_SIZE,
    create_depth_texture_msaa, create_hdr_msaa_texture, MSAA_SAMPLE_COUNT,
};
pub use state::{PbrSceneUniform, GpuLight, MAX_LIGHTS};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_exports() {
        let _: Option<RenderDevice> = None;
        let _: Option<RenderSurface> = None;
        let _: Option<BasicRenderPipeline> = None;
    }
}
