//! # 窗口管理模块
//! 
//! 提供基于 winit 的跨平台窗口管理功能，包括窗口创建、事件处理和应用生命周期管理。
//! 
//! ## 核心组件
//! 
//! - **RenderApp**: 实现 ApplicationHandler 的主应用结构
//! - **WindowConfig**: 窗口配置参数
//! - **WindowState**: 窗口状态管理
//! 
//! ## 设计理念
//! 
//! 本模块采用最新的 winit 0.29 API 设计，使用 ApplicationHandler trait 
//! 替代旧的事件循环模式，提供更好的跨平台兼容性和性能。
//! 
//! ## 使用示例
//! 
//! ```rust,no_run
//! use anvilkit_render::window::*;
//! use winit::event_loop::EventLoop;
//! 
//! // 创建事件循环和应用
//! let event_loop = EventLoop::new().unwrap();
//! let mut app = RenderApp::new(WindowConfig::default());
//! 
//! // 运行应用
//! event_loop.run_app(&mut app).unwrap();
//! ```

pub mod window;
pub mod events;

// 重新导出主要类型
pub use window::{WindowConfig, WindowState};
pub use events::RenderApp;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_window_config_default() {
        let config = WindowConfig::default();
        assert_eq!(config.title, "AnvilKit Application");
        assert_eq!(config.width, 1280);
        assert_eq!(config.height, 720);
        assert!(!config.fullscreen);
        assert!(config.resizable);
        assert!(config.visible);
    }
    
    #[test]
    fn test_window_config_builder() {
        let config = WindowConfig::new()
            .with_title("Test Window")
            .with_size(800, 600)
            .with_fullscreen(true);
            
        assert_eq!(config.title, "Test Window");
        assert_eq!(config.width, 800);
        assert_eq!(config.height, 600);
        assert!(config.fullscreen);
    }
}
