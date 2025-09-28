//! # 事件处理和应用生命周期
//! 
//! 基于 winit 0.29 的 ApplicationHandler 实现应用生命周期管理和事件处理。

use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::{WindowEvent, DeviceEvent, DeviceId},
    event_loop::{ActiveEventLoop, ControlFlow},
    window::{Window, WindowId},
    dpi::PhysicalSize,
};
use log::{info, warn, error, debug};

use crate::window::{WindowConfig, WindowState};
use crate::renderer::RenderContext;
use anvilkit_core::error::{AnvilKitError, Result};

/// 渲染应用
/// 
/// 实现 ApplicationHandler trait，管理窗口生命周期和渲染循环。
/// 使用最新的 winit 0.29 API 设计，提供跨平台兼容性。
/// 
/// # 设计理念
/// 
/// - **延迟初始化**: 在 `resumed` 事件中创建窗口和渲染上下文
/// - **事件驱动**: 响应窗口事件和设备事件
/// - **资源管理**: 自动管理窗口和渲染资源的生命周期
/// 
/// # 示例
/// 
/// ```rust,no_run
/// use anvilkit_render::window::{RenderApp, WindowConfig};
/// use winit::event_loop::EventLoop;
/// 
/// // 创建事件循环和应用
/// let event_loop = EventLoop::new().unwrap();
/// let mut app = RenderApp::new(WindowConfig::default());
/// 
/// // 运行应用
/// event_loop.run_app(&mut app).unwrap();
/// ```
pub struct RenderApp {
    /// 窗口配置
    config: WindowConfig,
    /// 窗口实例（延迟初始化）
    window: Option<Arc<Window>>,
    /// 窗口状态
    window_state: WindowState,
    /// 渲染上下文（延迟初始化）
    render_context: Option<RenderContext>,
    /// 是否请求退出
    exit_requested: bool,
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
            render_context: None,
            exit_requested: false,
        }
    }
    
    /// 获取窗口配置
    /// 
    /// # 返回
    /// 
    /// 返回当前的窗口配置
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_render::window::{RenderApp, WindowConfig};
    /// 
    /// let app = RenderApp::new(WindowConfig::default());
    /// let config = app.config();
    /// assert_eq!(config.title, "AnvilKit Application");
    /// ```
    pub fn config(&self) -> &WindowConfig {
        &self.config
    }
    
    /// 获取窗口状态
    /// 
    /// # 返回
    /// 
    /// 返回当前的窗口状态
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_render::window::{RenderApp, WindowConfig};
    /// 
    /// let app = RenderApp::new(WindowConfig::default());
    /// let state = app.window_state();
    /// assert_eq!(state.size(), (1280, 720));
    /// ```
    pub fn window_state(&self) -> &WindowState {
        &self.window_state
    }
    
    /// 获取窗口实例
    /// 
    /// # 返回
    /// 
    /// 返回窗口实例的引用，如果窗口尚未创建则返回 None
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_render::window::{RenderApp, WindowConfig};
    /// 
    /// let app = RenderApp::new(WindowConfig::default());
    /// // 窗口在 resumed 事件之前不会创建
    /// assert!(app.window().is_none());
    /// ```
    pub fn window(&self) -> Option<&Arc<Window>> {
        self.window.as_ref()
    }
    
    /// 请求退出应用
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_render::window::{RenderApp, WindowConfig};
    /// 
    /// let mut app = RenderApp::new(WindowConfig::default());
    /// app.request_exit();
    /// ```
    pub fn request_exit(&mut self) {
        info!("请求退出应用");
        self.exit_requested = true;
    }
    
    /// 检查是否请求退出
    /// 
    /// # 返回
    /// 
    /// 如果应用请求退出则返回 true
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_render::window::{RenderApp, WindowConfig};
    /// 
    /// let mut app = RenderApp::new(WindowConfig::default());
    /// assert!(!app.is_exit_requested());
    /// 
    /// app.request_exit();
    /// assert!(app.is_exit_requested());
    /// ```
    pub fn is_exit_requested(&self) -> bool {
        self.exit_requested
    }
    
    /// 创建窗口
    /// 
    /// # 参数
    /// 
    /// - `event_loop`: 活动的事件循环
    /// 
    /// # 返回
    /// 
    /// 成功时返回 Ok(())，失败时返回错误
    fn create_window(&mut self, event_loop: &ActiveEventLoop) -> Result<()> {
        if self.window.is_some() {
            warn!("窗口已经存在，跳过创建");
            return Ok(());
        }
        
        info!("创建窗口: {} ({}x{})", 
              self.config.title, self.config.width, self.config.height);
        
        let attributes = self.config.to_window_attributes();
        let window = event_loop.create_window(attributes)
            .map_err(|e| AnvilKitError::Render(format!("创建窗口失败: {}", e)))?;
        
        // 更新窗口状态
        let size = window.inner_size();
        self.window_state.set_size(size.width, size.height);
        self.window_state.set_scale_factor(window.scale_factor());
        
        self.window = Some(Arc::new(window));
        
        info!("窗口创建成功");
        Ok(())
    }
    
    /// 创建渲染上下文
    /// 
    /// # 返回
    /// 
    /// 成功时返回 Ok(())，失败时返回错误
    async fn create_render_context(&mut self) -> Result<()> {
        if self.render_context.is_some() {
            warn!("渲染上下文已经存在，跳过创建");
            return Ok(());
        }
        
        let window = self.window.as_ref()
            .ok_or_else(|| AnvilKitError::Render("窗口未创建".to_string()))?;
        
        info!("创建渲染上下文");
        
        let render_context = RenderContext::new(window.clone()).await?;
        self.render_context = Some(render_context);
        
        info!("渲染上下文创建成功");
        Ok(())
    }
    
    /// 处理窗口大小变化
    /// 
    /// # 参数
    /// 
    /// - `new_size`: 新的窗口大小
    fn handle_resize(&mut self, new_size: PhysicalSize<u32>) {
        debug!("窗口大小变化: {}x{}", new_size.width, new_size.height);
        
        self.window_state.set_size(new_size.width, new_size.height);
        
        if let Some(render_context) = &mut self.render_context {
            if let Err(e) = render_context.resize(new_size.width, new_size.height) {
                error!("调整渲染上下文大小失败: {}", e);
            }
        }
    }
    
    /// 处理缩放因子变化
    /// 
    /// # 参数
    /// 
    /// - `scale_factor`: 新的缩放因子
    fn handle_scale_factor_changed(&mut self, scale_factor: f64) {
        debug!("缩放因子变化: {}", scale_factor);
        self.window_state.set_scale_factor(scale_factor);
    }
    
    /// 执行渲染
    fn render(&mut self) {
        if let Some(render_context) = &mut self.render_context {
            if let Err(e) = render_context.render() {
                error!("渲染失败: {}", e);
            }
        }
    }
}

impl ApplicationHandler for RenderApp {
    /// 应用恢复事件
    /// 
    /// 在此事件中进行延迟初始化，创建窗口和渲染上下文。
    /// 这是 winit 0.29 推荐的初始化模式。
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        info!("应用恢复");
        
        // 创建窗口
        if let Err(e) = self.create_window(event_loop) {
            error!("创建窗口失败: {}", e);
            event_loop.exit();
            return;
        }
        
        // 创建渲染上下文（异步）
        let window = self.window.clone();
        if window.is_some() {
            // 使用 pollster 运行异步代码
            if let Err(e) = pollster::block_on(self.create_render_context()) {
                error!("创建渲染上下文失败: {}", e);
                event_loop.exit();
                return;
            }
        }
        
        // 请求重绘
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
    
    /// 窗口事件处理
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                info!("收到窗口关闭请求");
                self.request_exit();
                event_loop.exit();
            }
            
            WindowEvent::Resized(new_size) => {
                self.handle_resize(new_size);
            }
            
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.handle_scale_factor_changed(scale_factor);
            }
            
            WindowEvent::Focused(focused) => {
                debug!("窗口焦点变化: {}", focused);
                self.window_state.set_focused(focused);
            }
            
            WindowEvent::Occluded(occluded) => {
                debug!("窗口遮挡状态: {}", occluded);
                self.window_state.set_minimized(occluded);
            }
            
            WindowEvent::RedrawRequested => {
                self.render();
            }
            
            _ => {}
        }
    }
    
    /// 设备事件处理
    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        _event: DeviceEvent,
    ) {
        // 处理设备事件（鼠标、键盘等）
        // 目前暂时留空，后续可以添加输入处理
    }
    
    /// 即将等待事件
    /// 
    /// 在事件循环即将阻塞等待新事件时调用。
    /// 可以在此处执行帧更新逻辑。
    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // 请求重绘以维持渲染循环
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_render_app_creation() {
        let config = WindowConfig::new().with_title("Test App");
        let app = RenderApp::new(config);
        
        assert_eq!(app.config().title, "Test App");
        assert!(app.window().is_none());
        assert!(!app.is_exit_requested());
    }
    
    #[test]
    fn test_exit_request() {
        let mut app = RenderApp::new(WindowConfig::default());
        
        assert!(!app.is_exit_requested());
        app.request_exit();
        assert!(app.is_exit_requested());
    }
    
    #[test]
    fn test_window_state_updates() {
        let mut app = RenderApp::new(WindowConfig::default());
        
        // 测试大小变化处理
        let new_size = PhysicalSize::new(1920, 1080);
        app.handle_resize(new_size);
        assert_eq!(app.window_state().size(), (1920, 1080));
        
        // 测试缩放因子变化处理
        app.handle_scale_factor_changed(2.0);
        assert_eq!(app.window_state().scale_factor(), 2.0);
    }
}
