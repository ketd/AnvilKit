//! # 默认插件集
//!
//! 提供 `DefaultPlugins` 一站式初始化所有引擎核心系统。
//!
//! ## 使用示例
//!
//! ```rust,no_run
//! use anvilkit::prelude::*;
//!
//! let mut app = App::new();
//! app.add_plugins(DefaultPlugins::new());
//! ```

use anvilkit_ecs::prelude::*;
use anvilkit_ecs::plugin::Plugin;
use anvilkit_render::plugin::RenderPlugin;
use anvilkit_render::prelude::WindowConfig;
use anvilkit_audio::AudioPlugin;

/// 默认插件集 — 一站式初始化引擎核心系统
///
/// 包含：
/// - `AnvilKitEcsPlugin` — ECS 调度 + Transform 传播
/// - `RenderPlugin` — GPU 设备 + 窗口 + 渲染系统
/// - `AudioPlugin` — 音频引擎初始化
///
/// # 示例
///
/// ```rust,no_run
/// use anvilkit::prelude::*;
///
/// App::new()
///     .add_plugins(DefaultPlugins::new())
///     .add_systems(AnvilKitSchedule::Startup, setup)
///     .run();
///
/// fn setup() {
///     // 设置场景
/// }
/// ```
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
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use anvilkit::prelude::*;
    ///
    /// App::new()
    ///     .add_plugins(
    ///         DefaultPlugins::new()
    ///             .with_window(WindowConfig::new().with_title("My Game").with_size(1920, 1080))
    ///     );
    /// ```
    pub fn with_window(mut self, config: WindowConfig) -> Self {
        self.window_config = config;
        self
    }
}

impl Plugin for DefaultPlugins {
    fn build(&self, app: &mut App) {
        // 1. ECS 核心（调度器、时间、Transform）
        app.add_plugins(AnvilKitEcsPlugin);

        // 2. 渲染（GPU 设备、窗口、渲染系统、输入转发）
        app.add_plugins(
            RenderPlugin::new().with_window_config(self.window_config.clone())
        );

        // 3. 音频引擎
        app.add_plugins(AudioPlugin);

        // 4. 自动输入帧管理
        app.add_plugins(anvilkit_ecs::auto_plugins::AutoInputPlugin);

        // 5. 自动时间更新
        app.add_plugins(anvilkit_ecs::auto_plugins::AutoDeltaTimePlugin);
    }

    fn name(&self) -> &str {
        "DefaultPlugins"
    }

    fn is_unique(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_plugins_creation() {
        let plugins = DefaultPlugins::new();
        assert_eq!(plugins.name(), "DefaultPlugins");
    }

    #[test]
    fn test_with_window_config() {
        let plugins = DefaultPlugins::new()
            .with_window(WindowConfig::new().with_title("Test"));
        assert_eq!(plugins.window_config.title, "Test");
    }
}
