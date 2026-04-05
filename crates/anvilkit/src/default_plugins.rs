//! # 默认插件集
//!
//! 提供 `DefaultPlugins` 一站式初始化引擎核心系统。

use anvilkit_app::ecs_app::{App, Plugin};
use anvilkit_app::ecs_plugin::AnvilKitEcsPlugin;
use anvilkit_app::auto_plugins::{AutoInputPlugin, AutoDeltaTimePlugin};
use anvilkit_render::plugin::RenderPlugin;
use anvilkit_render::prelude::WindowConfig;
use anvilkit_render::transform::TransformPlugin;
use anvilkit_audio::AudioPlugin;

/// 默认插件集 — 一站式初始化引擎核心系统
///
/// 包含：
/// - `AnvilKitEcsPlugin` — ECS 调度
/// - `TransformPlugin` — Transform 层次传播
/// - `RenderPlugin` — GPU 设备 + 窗口 + 渲染系统
/// - `AudioPlugin` — 音频引擎初始化
/// - `AutoInputPlugin` — 自动输入帧管理
/// - `AutoDeltaTimePlugin` — 自动时间更新
pub struct DefaultPlugins {
    window_config: WindowConfig,
}

impl Default for DefaultPlugins {
    fn default() -> Self {
        Self {
            window_config: WindowConfig::default(),
        }
    }
}

impl DefaultPlugins {
    /// 创建默认插件集
    pub fn new() -> Self {
        Self::default()
    }

    /// 自定义窗口配置
    pub fn with_window(mut self, config: WindowConfig) -> Self {
        self.window_config = config;
        self
    }
}

impl Plugin for DefaultPlugins {
    fn build(&self, app: &mut App) {
        // 1. ECS 核心（调度器、时间）
        app.add_plugins(AnvilKitEcsPlugin);

        // 2. Transform 层次传播
        app.add_plugins(TransformPlugin);

        // 3. 渲染（GPU 设备、窗口、渲染系统、输入转发）
        app.add_plugins(
            RenderPlugin::new().with_window_config(self.window_config.clone())
        );

        // 4. 音频引擎
        app.add_plugins(AudioPlugin);

        // 5. 自动输入帧管理
        app.add_plugins(AutoInputPlugin);

        // 6. 自动时间更新
        app.add_plugins(AutoDeltaTimePlugin);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_plugins_creation() {
        let _plugins = DefaultPlugins::new();
    }

    #[test]
    fn test_with_window_config() {
        let plugins = DefaultPlugins::new()
            .with_window(WindowConfig::new().with_title("Test"));
        assert_eq!(plugins.window_config.title, "Test");
    }
}
