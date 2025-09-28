//! # AnvilKit 渲染系统
//! 
//! AnvilKit 的渲染模块提供了基于 wgpu 和 winit 的跨平台图形渲染功能。
//! 
//! ## 核心特性
//! 
//! - **跨平台支持**: 支持 Windows、macOS、Linux 和 Web 平台
//! - **现代图形 API**: 基于 wgpu，支持 Vulkan、Metal、D3D12、OpenGL 和 WebGPU
//! - **ECS 集成**: 与 AnvilKit ECS 系统无缝集成
//! - **插件架构**: 模块化的渲染插件系统
//! - **高性能**: 零成本抽象和 GPU 优化
//! 
//! ## 架构设计
//! 
//! 渲染系统采用分层架构：
//! - **窗口层**: 基于 winit 的窗口管理和事件处理
//! - **设备层**: wgpu 设备、适配器和表面管理
//! - **渲染层**: 渲染管线、资源和绘制命令
//! - **集成层**: ECS 插件和组件系统
//! 
//! ## 使用示例
//! 
//! ```rust,no_run
//! use anvilkit_render::prelude::*;
//! use anvilkit_ecs::prelude::*;
//! 
//! // 创建应用并添加渲染插件
//! let mut app = App::new();
//! app.add_plugins(RenderPlugin::default())
//!    .run();
//! ```

pub mod window;
pub mod renderer;
pub mod plugin;

/// 预导入模块
/// 
/// 包含最常用的类型和 trait，方便用户导入。
pub mod prelude {
    pub use crate::window::{RenderApp, WindowConfig};
    pub use crate::renderer::{RenderDevice, RenderSurface, RenderContext};
    pub use crate::plugin::RenderPlugin;
    
    // 重新导出核心依赖的常用类型
    pub use wgpu::{
        Device, Queue, Surface, SurfaceConfiguration, TextureFormat,
        RenderPipeline, RenderPass, CommandEncoder, Buffer, Texture,
        BindGroup, BindGroupLayout, PipelineLayout,
    };
    
    pub use winit::{
        event::{Event, WindowEvent, DeviceEvent},
        event_loop::{EventLoop, ActiveEventLoop},
        window::{Window, WindowId},
        application::ApplicationHandler,
    };
    
    // 重新导出 AnvilKit 核心类型
    pub use anvilkit_core::prelude::*;
    pub use anvilkit_ecs::prelude::*;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_prelude_imports() {
        // 测试预导入模块是否正确导出所需类型
        use crate::prelude::*;
        
        // 这些类型应该可以访问
        let _: Option<Device> = None;
        let _: Option<Queue> = None;
        let _: Option<Window> = None;
        let _: Option<EventLoop<()>> = None;
    }
    
    #[test]
    fn test_version_info() {
        // 测试版本信息
        assert_eq!(env!("CARGO_PKG_NAME"), "anvilkit-render");
        assert_eq!(env!("CARGO_PKG_VERSION"), "0.1.0");
    }
}
