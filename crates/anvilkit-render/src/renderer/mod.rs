//! # 渲染器模块
//! 
//! 提供基于 wgpu 的跨平台图形渲染功能，包括设备管理、表面配置和渲染管线。
//! 
//! ## 核心组件
//! 
//! - **RenderDevice**: GPU 设备和适配器管理
//! - **RenderSurface**: 窗口表面和交换链管理
//! - **RenderContext**: 统一的渲染上下文
//! - **RenderPipeline**: 渲染管线抽象
//! 
//! ## 设计理念
//! 
//! 本模块采用现代图形 API 设计，支持多种后端（Vulkan、Metal、D3D12、OpenGL、WebGPU），
//! 提供高性能的零成本抽象和灵活的渲染管线配置。
//! 
//! ## 使用示例
//! 
//! ```rust,no_run
//! use anvilkit_render::renderer::*;
//! use std::sync::Arc;
//! use winit::window::Window;
//! 
//! # async fn example() -> anvilkit_core::error::Result<()> {
//! // 创建窗口（示例）
//! // let window = Arc::new(window);
//! 
//! // 创建渲染上下文
//! // let render_context = RenderContext::new(window).await?;
//! 
//! // 执行渲染
//! // render_context.render()?;
//! # Ok(())
//! # }
//! ```

pub mod device;
pub mod surface;
pub mod context;
pub mod pipeline;

// 重新导出主要类型
pub use device::RenderDevice;
pub use surface::RenderSurface;
pub use context::RenderContext;
pub use pipeline::{RenderPipelineBuilder, BasicRenderPipeline};

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_module_exports() {
        // 测试模块导出是否正确
        // 这些类型应该可以访问
        let _: Option<RenderDevice> = None;
        let _: Option<RenderSurface> = None;
        let _: Option<RenderContext> = None;
        let _: Option<BasicRenderPipeline> = None;
    }
}
