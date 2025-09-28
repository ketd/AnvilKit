//! # 窗口配置和状态管理
//! 
//! 提供窗口的配置参数和状态管理功能。

use winit::dpi::{LogicalSize, PhysicalSize};
use winit::window::{Window, WindowAttributes, Fullscreen};
use anvilkit_core::error::{AnvilKitError, Result};

/// 窗口配置
/// 
/// 定义窗口的初始属性和行为参数。
/// 
/// # 示例
/// 
/// ```rust
/// use anvilkit_render::window::WindowConfig;
/// 
/// // 使用默认配置
/// let config = WindowConfig::default();
/// 
/// // 自定义配置
/// let config = WindowConfig::new()
///     .with_title("我的游戏")
///     .with_size(1920, 1080)
///     .with_fullscreen(true)
///     .with_vsync(true);
/// ```
#[derive(Debug, Clone)]
pub struct WindowConfig {
    /// 窗口标题
    pub title: String,
    /// 窗口宽度（逻辑像素）
    pub width: u32,
    /// 窗口高度（逻辑像素）
    pub height: u32,
    /// 是否全屏
    pub fullscreen: bool,
    /// 是否可调整大小
    pub resizable: bool,
    /// 是否可见
    pub visible: bool,
    /// 是否启用垂直同步
    pub vsync: bool,
    /// 最小窗口大小
    pub min_size: Option<(u32, u32)>,
    /// 最大窗口大小
    pub max_size: Option<(u32, u32)>,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "AnvilKit Application".to_string(),
            width: 1280,
            height: 720,
            fullscreen: false,
            resizable: true,
            visible: true,
            vsync: true,
            min_size: Some((320, 240)),
            max_size: None,
        }
    }
}

impl WindowConfig {
    /// 创建新的窗口配置
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_render::window::WindowConfig;
    /// 
    /// let config = WindowConfig::new();
    /// assert_eq!(config.title, "AnvilKit Application");
    /// ```
    pub fn new() -> Self {
        Self::default()
    }
    
    /// 设置窗口标题
    /// 
    /// # 参数
    /// 
    /// - `title`: 窗口标题字符串
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_render::window::WindowConfig;
    /// 
    /// let config = WindowConfig::new().with_title("我的应用");
    /// assert_eq!(config.title, "我的应用");
    /// ```
    pub fn with_title<T: Into<String>>(mut self, title: T) -> Self {
        self.title = title.into();
        self
    }
    
    /// 设置窗口大小
    /// 
    /// # 参数
    /// 
    /// - `width`: 窗口宽度（逻辑像素）
    /// - `height`: 窗口高度（逻辑像素）
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_render::window::WindowConfig;
    /// 
    /// let config = WindowConfig::new().with_size(1920, 1080);
    /// assert_eq!(config.width, 1920);
    /// assert_eq!(config.height, 1080);
    /// ```
    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }
    
    /// 设置是否全屏
    /// 
    /// # 参数
    /// 
    /// - `fullscreen`: 是否启用全屏模式
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_render::window::WindowConfig;
    /// 
    /// let config = WindowConfig::new().with_fullscreen(true);
    /// assert!(config.fullscreen);
    /// ```
    pub fn with_fullscreen(mut self, fullscreen: bool) -> Self {
        self.fullscreen = fullscreen;
        self
    }
    
    /// 设置是否可调整大小
    /// 
    /// # 参数
    /// 
    /// - `resizable`: 是否允许调整窗口大小
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_render::window::WindowConfig;
    /// 
    /// let config = WindowConfig::new().with_resizable(false);
    /// assert!(!config.resizable);
    /// ```
    pub fn with_resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }
    
    /// 设置是否启用垂直同步
    /// 
    /// # 参数
    /// 
    /// - `vsync`: 是否启用垂直同步
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_render::window::WindowConfig;
    /// 
    /// let config = WindowConfig::new().with_vsync(false);
    /// assert!(!config.vsync);
    /// ```
    pub fn with_vsync(mut self, vsync: bool) -> Self {
        self.vsync = vsync;
        self
    }
    
    /// 设置最小窗口大小
    /// 
    /// # 参数
    /// 
    /// - `min_size`: 最小窗口大小，None 表示无限制
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_render::window::WindowConfig;
    /// 
    /// let config = WindowConfig::new().with_min_size(Some((640, 480)));
    /// assert_eq!(config.min_size, Some((640, 480)));
    /// ```
    pub fn with_min_size(mut self, min_size: Option<(u32, u32)>) -> Self {
        self.min_size = min_size;
        self
    }
    
    /// 设置最大窗口大小
    /// 
    /// # 参数
    /// 
    /// - `max_size`: 最大窗口大小，None 表示无限制
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_render::window::WindowConfig;
    /// 
    /// let config = WindowConfig::new().with_max_size(Some((1920, 1080)));
    /// assert_eq!(config.max_size, Some((1920, 1080)));
    /// ```
    pub fn with_max_size(mut self, max_size: Option<(u32, u32)>) -> Self {
        self.max_size = max_size;
        self
    }
    
    /// 将配置转换为 winit 的 WindowAttributes
    /// 
    /// # 返回
    /// 
    /// 返回配置好的 WindowAttributes
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_render::window::WindowConfig;
    /// 
    /// let config = WindowConfig::new().with_title("测试窗口");
    /// let attributes = config.to_window_attributes();
    /// ```
    pub fn to_window_attributes(&self) -> WindowAttributes {
        let mut attributes = Window::default_attributes()
            .with_title(&self.title)
            .with_inner_size(LogicalSize::new(self.width, self.height))
            .with_resizable(self.resizable)
            .with_visible(self.visible);
        
        if let Some((min_width, min_height)) = self.min_size {
            attributes = attributes.with_min_inner_size(LogicalSize::new(min_width, min_height));
        }
        
        if let Some((max_width, max_height)) = self.max_size {
            attributes = attributes.with_max_inner_size(LogicalSize::new(max_width, max_height));
        }
        
        if self.fullscreen {
            attributes = attributes.with_fullscreen(Some(Fullscreen::Borderless(None)));
        }
        
        attributes
    }
}

/// 窗口状态
/// 
/// 跟踪窗口的当前状态和属性。
/// 
/// # 示例
/// 
/// ```rust
/// use anvilkit_render::window::WindowState;
/// 
/// let mut state = WindowState::new();
/// state.set_size(1920, 1080);
/// assert_eq!(state.size(), (1920, 1080));
/// ```
#[derive(Debug, Clone)]
pub struct WindowState {
    /// 当前窗口大小（物理像素）
    size: PhysicalSize<u32>,
    /// 缩放因子
    scale_factor: f64,
    /// 是否聚焦
    focused: bool,
    /// 是否最小化
    minimized: bool,
    /// 是否最大化
    maximized: bool,
    /// 是否全屏
    fullscreen: bool,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            size: PhysicalSize::new(1280, 720),
            scale_factor: 1.0,
            focused: true,
            minimized: false,
            maximized: false,
            fullscreen: false,
        }
    }
}

impl WindowState {
    /// 创建新的窗口状态
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_render::window::WindowState;
    /// 
    /// let state = WindowState::new();
    /// assert_eq!(state.size(), (1280, 720));
    /// ```
    pub fn new() -> Self {
        Self::default()
    }
    
    /// 获取窗口大小
    /// 
    /// # 返回
    /// 
    /// 返回 (宽度, 高度) 元组，单位为物理像素
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_render::window::WindowState;
    /// 
    /// let state = WindowState::new();
    /// let (width, height) = state.size();
    /// assert_eq!(width, 1280);
    /// assert_eq!(height, 720);
    /// ```
    pub fn size(&self) -> (u32, u32) {
        (self.size.width, self.size.height)
    }
    
    /// 设置窗口大小
    /// 
    /// # 参数
    /// 
    /// - `width`: 窗口宽度（物理像素）
    /// - `height`: 窗口高度（物理像素）
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_render::window::WindowState;
    /// 
    /// let mut state = WindowState::new();
    /// state.set_size(1920, 1080);
    /// assert_eq!(state.size(), (1920, 1080));
    /// ```
    pub fn set_size(&mut self, width: u32, height: u32) {
        self.size = PhysicalSize::new(width, height);
    }
    
    /// 获取缩放因子
    /// 
    /// # 返回
    /// 
    /// 返回当前的 DPI 缩放因子
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_render::window::WindowState;
    /// 
    /// let state = WindowState::new();
    /// assert_eq!(state.scale_factor(), 1.0);
    /// ```
    pub fn scale_factor(&self) -> f64 {
        self.scale_factor
    }
    
    /// 设置缩放因子
    /// 
    /// # 参数
    /// 
    /// - `scale_factor`: DPI 缩放因子
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_render::window::WindowState;
    /// 
    /// let mut state = WindowState::new();
    /// state.set_scale_factor(2.0);
    /// assert_eq!(state.scale_factor(), 2.0);
    /// ```
    pub fn set_scale_factor(&mut self, scale_factor: f64) {
        self.scale_factor = scale_factor;
    }
    
    /// 检查窗口是否聚焦
    /// 
    /// # 返回
    /// 
    /// 如果窗口当前聚焦则返回 true
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_render::window::WindowState;
    /// 
    /// let state = WindowState::new();
    /// assert!(state.is_focused());
    /// ```
    pub fn is_focused(&self) -> bool {
        self.focused
    }
    
    /// 设置窗口聚焦状态
    /// 
    /// # 参数
    /// 
    /// - `focused`: 是否聚焦
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_render::window::WindowState;
    /// 
    /// let mut state = WindowState::new();
    /// state.set_focused(false);
    /// assert!(!state.is_focused());
    /// ```
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }
    
    /// 检查窗口是否最小化
    /// 
    /// # 返回
    /// 
    /// 如果窗口当前最小化则返回 true
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_render::window::WindowState;
    /// 
    /// let state = WindowState::new();
    /// assert!(!state.is_minimized());
    /// ```
    pub fn is_minimized(&self) -> bool {
        self.minimized
    }
    
    /// 设置窗口最小化状态
    /// 
    /// # 参数
    /// 
    /// - `minimized`: 是否最小化
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_render::window::WindowState;
    /// 
    /// let mut state = WindowState::new();
    /// state.set_minimized(true);
    /// assert!(state.is_minimized());
    /// ```
    pub fn set_minimized(&mut self, minimized: bool) {
        self.minimized = minimized;
    }
    
    /// 检查窗口是否全屏
    /// 
    /// # 返回
    /// 
    /// 如果窗口当前全屏则返回 true
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_render::window::WindowState;
    /// 
    /// let state = WindowState::new();
    /// assert!(!state.is_fullscreen());
    /// ```
    pub fn is_fullscreen(&self) -> bool {
        self.fullscreen
    }
    
    /// 设置窗口全屏状态
    /// 
    /// # 参数
    /// 
    /// - `fullscreen`: 是否全屏
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_render::window::WindowState;
    /// 
    /// let mut state = WindowState::new();
    /// state.set_fullscreen(true);
    /// assert!(state.is_fullscreen());
    /// ```
    pub fn set_fullscreen(&mut self, fullscreen: bool) {
        self.fullscreen = fullscreen;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_window_config_creation() {
        let config = WindowConfig::new()
            .with_title("Test")
            .with_size(800, 600)
            .with_fullscreen(true)
            .with_resizable(false)
            .with_vsync(false);
        
        assert_eq!(config.title, "Test");
        assert_eq!(config.width, 800);
        assert_eq!(config.height, 600);
        assert!(config.fullscreen);
        assert!(!config.resizable);
        assert!(!config.vsync);
    }
    
    #[test]
    fn test_window_state_operations() {
        let mut state = WindowState::new();
        
        // 测试大小设置
        state.set_size(1920, 1080);
        assert_eq!(state.size(), (1920, 1080));
        
        // 测试缩放因子
        state.set_scale_factor(2.0);
        assert_eq!(state.scale_factor(), 2.0);
        
        // 测试状态标志
        state.set_focused(false);
        assert!(!state.is_focused());
        
        state.set_minimized(true);
        assert!(state.is_minimized());
        
        state.set_fullscreen(true);
        assert!(state.is_fullscreen());
    }
    
    #[test]
    fn test_window_attributes_conversion() {
        let config = WindowConfig::new()
            .with_title("Test Window")
            .with_size(1024, 768);
        
        let attributes = config.to_window_attributes();
        // 注意：无法直接测试 WindowAttributes 的内容，
        // 因为它们没有实现 PartialEq
        // 这里只是确保转换不会 panic
    }
}
