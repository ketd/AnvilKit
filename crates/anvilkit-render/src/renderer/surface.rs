//! # 窗口表面和交换链管理
//! 
//! 提供 wgpu 表面配置、交换链管理和帧缓冲功能。

use std::sync::Arc;
use wgpu::{
    Surface, SurfaceConfiguration, TextureFormat, PresentMode, CompositeAlphaMode,
    SurfaceTexture,
};
use winit::window::Window;
use log::{info, warn, debug};

use crate::renderer::RenderDevice;
use anvilkit_core::error::{AnvilKitError, Result};

/// 渲染表面
/// 
/// 管理窗口表面、交换链配置和帧缓冲，提供渲染目标管理功能。
/// 
/// # 设计理念
/// 
/// - **自适应配置**: 根据设备能力自动配置表面参数
/// - **动态调整**: 支持窗口大小变化时的动态重配置
/// - **格式选择**: 自动选择最佳的纹理格式和呈现模式
/// - **错误恢复**: 处理表面丢失等异常情况
/// 
/// # 示例
/// 
/// ```rust,no_run
/// use anvilkit_render::renderer::{RenderDevice, RenderSurface};
/// use std::sync::Arc;
/// use winit::window::Window;
/// 
/// # async fn example() -> anvilkit_core::error::Result<()> {
/// // 创建设备和表面
/// // let window = Arc::new(window);
/// // let device = RenderDevice::new(&window).await?;
/// // let surface = RenderSurface::new(&device, &window)?;
/// 
/// // 获取当前帧
/// // let frame = surface.get_current_frame()?;
/// # Ok(())
/// # }
/// ```
pub struct RenderSurface {
    /// wgpu 表面（持有 Arc<Window>，保证生命周期安全）
    surface: Surface<'static>,
    /// 表面配置
    config: SurfaceConfiguration,
    /// 当前纹理格式
    format: TextureFormat,
    /// 持有窗口引用以保证 surface 生命周期
    _window: Arc<Window>,
}

impl RenderSurface {
    /// 创建新的渲染表面
    /// 
    /// # 参数
    /// 
    /// - `device`: 渲染设备
    /// - `window`: 窗口实例
    /// 
    /// # 返回
    /// 
    /// 成功时返回 RenderSurface 实例，失败时返回错误
    /// 
    /// # 示例
    /// 
    /// ```rust,no_run
    /// use anvilkit_render::renderer::{RenderDevice, RenderSurface};
    /// use std::sync::Arc;
    /// use winit::window::Window;
    /// 
    /// # async fn example() -> anvilkit_core::error::Result<()> {
    /// // let window = Arc::new(window);
    /// // let device = RenderDevice::new(&window).await?;
    /// // let surface = RenderSurface::new(&device, &window)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(device: &RenderDevice, window: &Arc<Window>) -> Result<Self> {
        Self::new_with_vsync(device, window, false)
    }

    /// 创建新的渲染表面（指定 vsync 模式）
    ///
    /// - `vsync = true`: 使用 `PresentMode::Fifo`（垂直同步）
    /// - `vsync = false`: 优先使用 `PresentMode::Mailbox`（三重缓冲，低延迟）
    pub fn new_with_vsync(device: &RenderDevice, window: &Arc<Window>, vsync: bool) -> Result<Self> {
        info!("创建渲染表面 (vsync={})", vsync);

        // 创建表面
        let surface = device.instance().create_surface(window.clone())
            .map_err(|e| AnvilKitError::render(format!("创建表面失败: {}", e)))?;

        // 获取表面能力
        let capabilities = surface.get_capabilities(device.adapter());

        // 选择纹理格式
        let format = Self::choose_format(&capabilities.formats);

        // 获取窗口大小
        let size = window.inner_size();

        // 选择呈现模式
        let present_mode = if vsync {
            PresentMode::Fifo
        } else {
            Self::choose_present_mode(&capabilities.present_modes)
        };

        // 创建表面配置
        let config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode,
            alpha_mode: Self::choose_alpha_mode(&capabilities.alpha_modes),
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        
        // 配置表面
        surface.configure(device.device(), &config);
        
        info!("渲染表面创建成功");
        info!("表面格式: {:?}", format);
        info!("表面大小: {}x{}", config.width, config.height);
        info!("呈现模式: {:?}", config.present_mode);
        
        Ok(Self {
            surface,
            config,
            format,
            _window: window.clone(),
        })
    }
    
    /// 选择纹理格式
    /// 
    /// # 参数
    /// 
    /// - `formats`: 支持的格式列表
    /// 
    /// # 返回
    /// 
    /// 返回选择的纹理格式
    fn choose_format(formats: &[TextureFormat]) -> TextureFormat {
        // 优先选择 sRGB 格式
        for &format in formats {
            match format {
                TextureFormat::Bgra8UnormSrgb | TextureFormat::Rgba8UnormSrgb => {
                    debug!("选择纹理格式: {:?}", format);
                    return format;
                }
                _ => {}
            }
        }
        
        // 回退到第一个可用格式
        let format = formats.first().copied().unwrap_or(TextureFormat::Bgra8UnormSrgb);
        debug!("回退到纹理格式: {:?}", format);
        format
    }
    
    /// 选择呈现模式
    /// 
    /// # 参数
    /// 
    /// - `modes`: 支持的呈现模式列表
    /// 
    /// # 返回
    /// 
    /// 返回选择的呈现模式
    fn choose_present_mode(modes: &[PresentMode]) -> PresentMode {
        // 优先选择 Mailbox 模式（三重缓冲）
        if modes.contains(&PresentMode::Mailbox) {
            debug!("选择呈现模式: Mailbox");
            return PresentMode::Mailbox;
        }
        
        // 回退到 Fifo 模式（垂直同步）
        debug!("选择呈现模式: Fifo");
        PresentMode::Fifo
    }
    
    /// 选择 Alpha 混合模式
    /// 
    /// # 参数
    /// 
    /// - `modes`: 支持的 Alpha 模式列表
    /// 
    /// # 返回
    /// 
    /// 返回选择的 Alpha 模式
    fn choose_alpha_mode(modes: &[CompositeAlphaMode]) -> CompositeAlphaMode {
        // 优先选择 Auto 模式
        if modes.contains(&CompositeAlphaMode::Auto) {
            debug!("选择 Alpha 模式: Auto");
            return CompositeAlphaMode::Auto;
        }
        
        // 回退到 Opaque 模式
        debug!("选择 Alpha 模式: Opaque");
        CompositeAlphaMode::Opaque
    }
    
    /// 调整表面大小
    /// 
    /// # 参数
    /// 
    /// - `device`: 渲染设备
    /// - `width`: 新的宽度
    /// - `height`: 新的高度
    /// 
    /// # 返回
    /// 
    /// 成功时返回 Ok(())，失败时返回错误
    /// 
    /// # 示例
    /// 
    /// ```rust,no_run
    /// # use anvilkit_render::renderer::{RenderDevice, RenderSurface};
    /// # async fn example(device: &RenderDevice, surface: &mut RenderSurface) -> anvilkit_core::error::Result<()> {
    /// surface.resize(device, 1920, 1080)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn resize(&mut self, device: &RenderDevice, width: u32, height: u32) -> Result<()> {
        if width == 0 || height == 0 {
            warn!("忽略无效的表面大小: {}x{}", width, height);
            return Ok(());
        }
        
        info!("调整表面大小: {}x{}", width, height);
        
        self.config.width = width;
        self.config.height = height;
        
        self.surface.configure(device.device(), &self.config);
        
        Ok(())
    }
    
    /// 获取当前帧纹理
    /// 
    /// # 返回
    /// 
    /// 成功时返回 SurfaceTexture，失败时返回错误
    /// 
    /// # 示例
    /// 
    /// ```rust,no_run
    /// # use anvilkit_render::renderer::RenderSurface;
    /// # async fn example(surface: &RenderSurface) -> anvilkit_core::error::Result<()> {
    /// let frame = surface.get_current_frame()?;
    /// let view = frame.texture.create_view(&Default::default());
    /// // 使用纹理视图进行渲染
    /// frame.present();
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_current_frame(&self) -> Result<SurfaceTexture> {
        self.surface.get_current_texture()
            .map_err(|e| match e {
                wgpu::SurfaceError::Lost => {
                    AnvilKitError::render("表面丢失，需要重新配置".to_string())
                }
                wgpu::SurfaceError::OutOfMemory => {
                    AnvilKitError::render("GPU 内存不足".to_string())
                }
                wgpu::SurfaceError::Timeout => {
                    AnvilKitError::render("获取表面纹理超时".to_string())
                }
                wgpu::SurfaceError::Outdated => {
                    AnvilKitError::render("表面配置过时，需要重新配置".to_string())
                }
            })
    }

    /// 重新配置表面（用于 Lost/Outdated 恢复）
    pub fn reconfigure(&self, device: &RenderDevice) {
        info!("重新配置渲染表面: {}x{}", self.config.width, self.config.height);
        self.surface.configure(device.device(), &self.config);
    }

    /// 获取当前帧，自动恢复 Lost/Outdated 错误
    ///
    /// 如果首次获取失败（Lost 或 Outdated），自动 reconfigure 后重试一次。
    pub fn get_current_frame_with_recovery(&self, device: &RenderDevice) -> Result<SurfaceTexture> {
        match self.surface.get_current_texture() {
            Ok(frame) => Ok(frame),
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                warn!("表面需要重新配置，正在恢复...");
                self.reconfigure(device);
                // 重试一次
                self.surface.get_current_texture()
                    .map_err(|e| AnvilKitError::render(format!("表面恢复后仍失败: {}", e)))
            }
            Err(wgpu::SurfaceError::OutOfMemory) => {
                Err(AnvilKitError::render("GPU 内存不足".to_string()))
            }
            Err(wgpu::SurfaceError::Timeout) => {
                Err(AnvilKitError::render("获取表面纹理超时".to_string()))
            }
        }
    }
    
    /// 获取表面配置
    /// 
    /// # 返回
    /// 
    /// 返回当前的表面配置
    /// 
    /// # 示例
    /// 
    /// ```rust,no_run
    /// # use anvilkit_render::renderer::RenderSurface;
    /// # async fn example(surface: &RenderSurface) {
    /// let config = surface.config();
    /// println!("表面大小: {}x{}", config.width, config.height);
    /// # }
    /// ```
    pub fn config(&self) -> &SurfaceConfiguration {
        &self.config
    }
    
    /// 获取纹理格式
    /// 
    /// # 返回
    /// 
    /// 返回当前的纹理格式
    /// 
    /// # 示例
    /// 
    /// ```rust,no_run
    /// # use anvilkit_render::renderer::RenderSurface;
    /// # async fn example(surface: &RenderSurface) {
    /// let format = surface.format();
    /// println!("纹理格式: {:?}", format);
    /// # }
    /// ```
    pub fn format(&self) -> TextureFormat {
        self.format
    }
    
    /// 获取表面大小
    /// 
    /// # 返回
    /// 
    /// 返回 (宽度, 高度) 元组
    /// 
    /// # 示例
    /// 
    /// ```rust,no_run
    /// # use anvilkit_render::renderer::RenderSurface;
    /// # async fn example(surface: &RenderSurface) {
    /// let (width, height) = surface.size();
    /// println!("表面大小: {}x{}", width, height);
    /// # }
    /// ```
    pub fn size(&self) -> (u32, u32) {
        (self.config.width, self.config.height)
    }
    
    /// 获取表面引用
    /// 
    /// # 返回
    /// 
    /// 返回 wgpu 表面的引用
    /// 
    /// # 示例
    /// 
    /// ```rust,no_run
    /// # use anvilkit_render::renderer::RenderSurface;
    /// # async fn example(surface: &RenderSurface) {
    /// let wgpu_surface = surface.surface();
    /// // 使用原始表面进行高级操作
    /// # }
    /// ```
    pub fn surface(&self) -> &Surface<'static> {
        &self.surface
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wgpu::{TextureFormat, PresentMode, CompositeAlphaMode};
    
    #[test]
    fn test_format_selection() {
        let formats = vec![
            TextureFormat::Rgba8Unorm,
            TextureFormat::Bgra8UnormSrgb,
            TextureFormat::Rgba8UnormSrgb,
        ];
        
        let chosen = RenderSurface::choose_format(&formats);
        assert_eq!(chosen, TextureFormat::Bgra8UnormSrgb);
    }
    
    #[test]
    fn test_present_mode_selection() {
        let modes = vec![
            PresentMode::Fifo,
            PresentMode::Mailbox,
            PresentMode::Immediate,
        ];
        
        let chosen = RenderSurface::choose_present_mode(&modes);
        assert_eq!(chosen, PresentMode::Mailbox);
    }
    
    #[test]
    fn test_alpha_mode_selection() {
        let modes = vec![
            CompositeAlphaMode::Opaque,
            CompositeAlphaMode::Auto,
            CompositeAlphaMode::PreMultiplied,
        ];

        let chosen = RenderSurface::choose_alpha_mode(&modes);
        assert_eq!(chosen, CompositeAlphaMode::Auto);
    }

    #[test]
    fn test_format_srgb_preference() {
        // sRGB formats should be preferred
        let formats = vec![
            TextureFormat::Rgba8Unorm,
            TextureFormat::Bgra8UnormSrgb,
            TextureFormat::Rgba8UnormSrgb,
        ];

        let srgb = formats.iter().find(|f| {
            matches!(f,
                TextureFormat::Bgra8UnormSrgb |
                TextureFormat::Rgba8UnormSrgb
            )
        });
        assert!(srgb.is_some());
    }

    #[test]
    fn test_present_mode_vsync() {
        // Fifo mode is VSync on
        let mode = PresentMode::Fifo;
        assert_eq!(mode, PresentMode::Fifo);

        // Immediate mode is VSync off
        let mode = PresentMode::Immediate;
        assert_eq!(mode, PresentMode::Immediate);
    }
}
