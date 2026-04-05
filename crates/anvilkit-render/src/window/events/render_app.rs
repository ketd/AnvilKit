use std::sync::Arc;
use std::time::Instant;
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;
use log::info;

use bevy_app::App;
use crate::window::{WindowConfig, WindowState};
use crate::renderer::{RenderDevice, RenderSurface};
use anvilkit_core::error::{AnvilKitError, Result};

/// 渲染应用
///
/// 实现 ApplicationHandler trait，管理窗口生命周期和渲染循环。
/// 使用 winit 0.30 API 设计，提供跨平台兼容性。
///
/// # 设计理念
///
/// - **延迟初始化**: 在 `resumed` 事件中创建窗口和渲染上下文
/// - **事件驱动**: 响应窗口事件和设备事件
/// - **ECS 集成**: 持有 App 并每帧调用 update()
///
/// # 示例
///
/// ```rust,no_run
/// use anvilkit_render::window::{RenderApp, WindowConfig};
/// use anvilkit_render::prelude::*;
/// use winit::event_loop::EventLoop;
///
/// let mut app = App::new();
/// app.add_plugins(RenderPlugin::default());
///
/// RenderApp::run(app);
/// ```
pub struct RenderApp {
    /// 窗口配置
    pub(super) config: WindowConfig,
    /// 窗口实例（延迟初始化）
    pub(super) window: Option<Arc<Window>>,
    /// 窗口状态
    pub(super) window_state: WindowState,
    /// 渲染设备（延迟初始化）
    pub(super) render_device: Option<RenderDevice>,
    /// 渲染表面（延迟初始化，内部持有 Arc<Window>）
    pub(super) render_surface: Option<RenderSurface>,

    /// 是否请求退出
    pub(super) exit_requested: bool,

    // --- ECS fields ---
    /// ECS App（当通过 RenderApp::run() 启动时持有）
    pub(super) app: Option<App>,
    /// GPU 是否已初始化并注入到 ECS World
    pub(super) gpu_initialized: bool,

    /// 上一帧时间戳，用于计算真实帧时间
    pub(super) last_frame_time: Instant,

    /// 帧捕获资源（capture feature 启用时）
    #[cfg(feature = "capture")]
    pub(super) capture_resources: Option<crate::renderer::capture::CaptureResources>,
}

impl RenderApp {
    /// 创建新的渲染应用
    ///
    /// # 参数
    ///
    /// - `config`: 窗口配置参数
    ///
    /// # 示例
    ///
    /// ```rust
    /// use anvilkit_render::window::{RenderApp, WindowConfig};
    ///
    /// let config = WindowConfig::new().with_title("我的应用");
    /// let app = RenderApp::new(config);
    /// ```
    pub fn new(config: WindowConfig) -> Self {
        info!("创建渲染应用: {}", config.title);

        Self {
            config,
            window: None,
            window_state: WindowState::new(),
            render_device: None,
            render_surface: None,
            exit_requested: false,
            app: None,
            gpu_initialized: false,
            last_frame_time: Instant::now(),
            #[cfg(feature = "capture")]
            capture_resources: None,
        }
    }

    /// ECS 驱动的入口点
    ///
    /// 创建 EventLoop、窗口，运行 winit 主循环。
    /// 每帧调用 `app.update()` 然后执行 GPU 渲染。
    ///
    /// # 参数
    ///
    /// - `app`: 已配置好 RenderPlugin 和系统的 ECS App
    pub fn run(app: App) {
        let event_loop = winit::event_loop::EventLoop::new().unwrap();

        // 从 App 中读取 RenderConfig 获取 WindowConfig
        let window_config = app.world().get_resource::<crate::plugin::RenderConfig>()
            .map(|c| c.window_config.clone())
            .unwrap_or_default();

        let mut render_app = Self::new(window_config);
        render_app.app = Some(app);

        event_loop.run_app(&mut render_app).unwrap();
    }

    /// 获取窗口配置
    pub fn config(&self) -> &WindowConfig {
        &self.config
    }

    /// 获取窗口状态
    pub fn window_state(&self) -> &WindowState {
        &self.window_state
    }

    /// 获取窗口实例
    pub fn window(&self) -> Option<&Arc<Window>> {
        self.window.as_ref()
    }

    /// 请求退出应用
    pub fn request_exit(&mut self) {
        info!("请求退出应用");
        self.exit_requested = true;
    }

    /// 检查是否请求退出
    pub fn is_exit_requested(&self) -> bool {
        self.exit_requested
    }

    /// 获取渲染设备（初始化后可用）
    pub fn render_device(&self) -> Option<&RenderDevice> {
        self.render_device.as_ref()
    }

    /// 获取渲染表面格式（初始化后可用）
    pub fn surface_format(&self) -> Option<wgpu::TextureFormat> {
        self.render_surface.as_ref().map(|s| s.format())
    }

    /// 获取当前帧的 SurfaceTexture（用于外部渲染）
    pub fn get_current_frame(&self) -> Option<wgpu::SurfaceTexture> {
        self.render_surface.as_ref().and_then(|s| s.get_current_frame().ok())
    }

    // --- Internal methods ---

    /// 创建窗口
    pub(super) fn create_window(&mut self, event_loop: &ActiveEventLoop) -> Result<()> {
        if self.window.is_some() {
            return Ok(());
        }

        info!("创建窗口: {} ({}x{})",
              self.config.title, self.config.width, self.config.height);

        let attributes = self.config.to_window_attributes();
        let window = event_loop.create_window(attributes)
            .map_err(|e| AnvilKitError::render(format!("创建窗口失败: {}", e)))?;

        let size = window.inner_size();
        self.window_state.set_size(size.width, size.height);
        self.window_state.set_scale_factor(window.scale_factor());

        self.window = Some(Arc::new(window));

        info!("窗口创建成功");
        Ok(())
    }

    /// 初始化渲染资源
    pub(super) async fn init_render(&mut self) -> Result<()> {
        if self.render_device.is_some() {
            return Ok(());
        }

        let window = self.window.as_ref()
            .ok_or_else(|| AnvilKitError::render("窗口未创建".to_string()))?;

        info!("初始化渲染设备和表面");

        let device = RenderDevice::new(window).await?;
        let surface = RenderSurface::new_with_vsync(&device, window, self.config.vsync)?;

        self.render_device = Some(device);
        self.render_surface = Some(surface);

        info!("渲染设备和表面初始化成功");
        Ok(())
    }
}
