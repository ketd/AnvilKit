//! # DemoApp 共享脚手架
//!
//! 封装 examples 中的共同 winit + wgpu 初始化逻辑，
//! 让 demo 只需关注场景设置和每帧逻辑。
//!
//! ## 用法
//!
//! ```rust,ignore
//! use anvilkit_render::demo_app::DemoApp;
//! use anvilkit_render::prelude::*;
//!
//! DemoApp::run("My Demo", 800, 600, |app, device| {
//!     // 初始化场景：spawn entities, upload meshes
//! });
//! ```

use crate::window::WindowConfig;
use crate::plugin::RenderPlugin;
use anvilkit_ecs::prelude::*;

/// Demo 应用脚手架
///
/// 提供标准化的 winit + ECS 初始化流程。
/// 用户只需提供窗口标题、尺寸和场景初始化回调。
pub struct DemoApp {
    /// 窗口标题
    pub title: String,
    /// 窗口宽度
    pub width: u32,
    /// 窗口高度
    pub height: u32,
}

impl DemoApp {
    /// 创建新的 DemoApp 配置
    pub fn new(title: impl Into<String>, width: u32, height: u32) -> Self {
        Self {
            title: title.into(),
            width,
            height,
        }
    }

    /// 获取窗口配置
    pub fn window_config(&self) -> WindowConfig {
        WindowConfig::new()
            .with_title(&self.title)
            .with_size(self.width, self.height)
    }

    /// 创建预配置的 App（含 ECS + Render 插件）
    pub fn create_app(&self) -> App {
        let mut app = App::new();
        app.add_plugins(
            RenderPlugin::new()
                .with_window_config(self.window_config())
        );
        app
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_demo_app_creation() {
        let demo = DemoApp::new("Test Demo", 800, 600);
        assert_eq!(demo.title, "Test Demo");
        assert_eq!(demo.width, 800);
        assert_eq!(demo.height, 600);
    }

    #[test]
    fn test_demo_app_window_config() {
        let demo = DemoApp::new("My Game", 1920, 1080);
        let config = demo.window_config();
        assert_eq!(config.title, "My Game");
        assert_eq!(config.width, 1920);
        assert_eq!(config.height, 1080);
    }

    #[test]
    fn test_demo_app_create_app() {
        let demo = DemoApp::new("Test", 640, 480);
        let _app = demo.create_app();
        // Should not panic — App + RenderPlugin initialization works
    }
}
