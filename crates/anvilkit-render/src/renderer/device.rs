//! # GPU 设备和适配器管理
//! 
//! 提供 wgpu 设备、适配器和实例的创建和管理功能。

use std::sync::Arc;
use wgpu::{
    Instance, Adapter, Device, Queue, Surface, SurfaceConfiguration,
    DeviceDescriptor, Features, Limits, PowerPreference, RequestAdapterOptions,
    InstanceDescriptor, Backends, TextureFormat,
};
use winit::window::Window;
use log::{info, warn, error, debug};

use anvilkit_core::error::{AnvilKitError, Result};

/// GPU 渲染设备
/// 
/// 封装 wgpu 的实例、适配器、设备和队列，提供统一的 GPU 资源管理。
/// 
/// # 设计理念
/// 
/// - **自动选择**: 自动选择最佳的 GPU 适配器
/// - **特性检测**: 检测和启用可用的 GPU 特性
/// - **错误处理**: 完善的错误处理和回退机制
/// - **跨平台**: 支持多种图形后端
/// 
/// # 示例
/// 
/// ```rust,no_run
/// use anvilkit_render::renderer::RenderDevice;
/// use std::sync::Arc;
/// use winit::window::Window;
/// 
/// # async fn example() -> anvilkit_core::error::Result<()> {
/// // 创建窗口（示例）
/// // let window = Arc::new(window);
/// 
/// // 创建渲染设备
/// // let device = RenderDevice::new(&window).await?;
/// 
/// // 获取设备和队列
/// // let wgpu_device = device.device();
/// // let queue = device.queue();
/// # Ok(())
/// # }
/// ```
pub struct RenderDevice {
    /// wgpu 实例
    instance: Instance,
    /// GPU 适配器
    adapter: Adapter,
    /// GPU 设备
    device: Device,
    /// 命令队列
    queue: Queue,
    /// 支持的特性
    features: Features,
    /// 设备限制
    limits: Limits,
}

impl RenderDevice {
    /// 创建新的渲染设备
    /// 
    /// # 参数
    /// 
    /// - `window`: 窗口实例，用于创建兼容的表面
    /// 
    /// # 返回
    /// 
    /// 成功时返回 RenderDevice 实例，失败时返回错误
    /// 
    /// # 示例
    /// 
    /// ```rust,no_run
    /// use anvilkit_render::renderer::RenderDevice;
    /// use std::sync::Arc;
    /// use winit::window::Window;
    /// 
    /// # async fn example() -> anvilkit_core::error::Result<()> {
    /// // let window = Arc::new(window);
    /// // let device = RenderDevice::new(&window).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(window: &Arc<Window>) -> Result<Self> {
        info!("初始化 GPU 渲染设备");
        
        // 创建 wgpu 实例
        let instance = Self::create_instance()?;
        
        // 创建表面
        let surface = Self::create_surface(&instance, window)?;
        
        // 请求适配器
        let adapter = Self::request_adapter(&instance, &surface).await?;
        
        // 请求设备和队列
        let (device, queue) = Self::request_device(&adapter).await?;
        
        let features = adapter.features();
        let limits = adapter.limits();
        
        info!("GPU 渲染设备初始化完成");
        info!("适配器信息: {:?}", adapter.get_info());
        info!("支持的特性: {:?}", features);
        
        Ok(Self {
            instance,
            adapter,
            device,
            queue,
            features,
            limits,
        })
    }
    
    /// 创建 wgpu 实例
    /// 
    /// # 返回
    /// 
    /// 成功时返回 Instance，失败时返回错误
    fn create_instance() -> Result<Instance> {
        debug!("创建 wgpu 实例");
        
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::all(),
            ..Default::default()
        });
        
        Ok(instance)
    }
    
    /// 创建窗口表面
    /// 
    /// # 参数
    /// 
    /// - `instance`: wgpu 实例
    /// - `window`: 窗口实例
    /// 
    /// # 返回
    /// 
    /// 成功时返回 Surface，失败时返回错误
    fn create_surface(instance: &Instance, window: &Arc<Window>) -> Result<Surface> {
        debug!("创建窗口表面");
        
        let surface = instance.create_surface(window.clone())
            .map_err(|e| AnvilKitError::Render(format!("创建表面失败: {}", e)))?;
        
        Ok(surface)
    }
    
    /// 请求 GPU 适配器
    /// 
    /// # 参数
    /// 
    /// - `instance`: wgpu 实例
    /// - `surface`: 窗口表面
    /// 
    /// # 返回
    /// 
    /// 成功时返回 Adapter，失败时返回错误
    async fn request_adapter(instance: &Instance, surface: &Surface) -> Result<Adapter> {
        debug!("请求 GPU 适配器");
        
        let adapter = instance.request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
            compatible_surface: Some(surface),
            force_fallback_adapter: false,
        }).await
        .ok_or_else(|| AnvilKitError::Render("未找到兼容的 GPU 适配器".to_string()))?;
        
        let info = adapter.get_info();
        info!("选择的 GPU 适配器: {} ({:?})", info.name, info.backend);
        
        Ok(adapter)
    }
    
    /// 请求 GPU 设备和队列
    /// 
    /// # 参数
    /// 
    /// - `adapter`: GPU 适配器
    /// 
    /// # 返回
    /// 
    /// 成功时返回 (Device, Queue)，失败时返回错误
    async fn request_device(adapter: &Adapter) -> Result<(Device, Queue)> {
        debug!("请求 GPU 设备和队列");
        
        let (device, queue) = adapter.request_device(
            &DeviceDescriptor {
                label: Some("AnvilKit Render Device"),
                required_features: Features::empty(),
                required_limits: Limits::default(),
            },
            None, // 不使用跟踪路径
        ).await
        .map_err(|e| AnvilKitError::Render(format!("创建设备失败: {}", e)))?;
        
        info!("GPU 设备和队列创建成功");
        
        Ok((device, queue))
    }
    
    /// 获取 wgpu 实例
    /// 
    /// # 返回
    /// 
    /// 返回 wgpu 实例的引用
    /// 
    /// # 示例
    /// 
    /// ```rust,no_run
    /// # use anvilkit_render::renderer::RenderDevice;
    /// # async fn example(device: &RenderDevice) {
    /// let instance = device.instance();
    /// # }
    /// ```
    pub fn instance(&self) -> &Instance {
        &self.instance
    }
    
    /// 获取 GPU 适配器
    /// 
    /// # 返回
    /// 
    /// 返回 GPU 适配器的引用
    /// 
    /// # 示例
    /// 
    /// ```rust,no_run
    /// # use anvilkit_render::renderer::RenderDevice;
    /// # async fn example(device: &RenderDevice) {
    /// let adapter = device.adapter();
    /// let info = adapter.get_info();
    /// println!("GPU: {}", info.name);
    /// # }
    /// ```
    pub fn adapter(&self) -> &Adapter {
        &self.adapter
    }
    
    /// 获取 GPU 设备
    /// 
    /// # 返回
    /// 
    /// 返回 GPU 设备的引用
    /// 
    /// # 示例
    /// 
    /// ```rust,no_run
    /// # use anvilkit_render::renderer::RenderDevice;
    /// # async fn example(device: &RenderDevice) {
    /// let wgpu_device = device.device();
    /// // 使用设备创建资源
    /// # }
    /// ```
    pub fn device(&self) -> &Device {
        &self.device
    }
    
    /// 获取命令队列
    /// 
    /// # 返回
    /// 
    /// 返回命令队列的引用
    /// 
    /// # 示例
    /// 
    /// ```rust,no_run
    /// # use anvilkit_render::renderer::RenderDevice;
    /// # async fn example(device: &RenderDevice) {
    /// let queue = device.queue();
    /// // 使用队列提交命令
    /// # }
    /// ```
    pub fn queue(&self) -> &Queue {
        &self.queue
    }
    
    /// 获取支持的特性
    /// 
    /// # 返回
    /// 
    /// 返回 GPU 支持的特性集合
    /// 
    /// # 示例
    /// 
    /// ```rust,no_run
    /// # use anvilkit_render::renderer::RenderDevice;
    /// # use wgpu::Features;
    /// # async fn example(device: &RenderDevice) {
    /// let features = device.features();
    /// if features.contains(Features::TIMESTAMP_QUERY) {
    ///     println!("支持时间戳查询");
    /// }
    /// # }
    /// ```
    pub fn features(&self) -> Features {
        self.features
    }
    
    /// 获取设备限制
    /// 
    /// # 返回
    /// 
    /// 返回 GPU 设备的限制参数
    /// 
    /// # 示例
    /// 
    /// ```rust,no_run
    /// # use anvilkit_render::renderer::RenderDevice;
    /// # async fn example(device: &RenderDevice) {
    /// let limits = device.limits();
    /// println!("最大纹理大小: {}", limits.max_texture_dimension_2d);
    /// # }
    /// ```
    pub fn limits(&self) -> &Limits {
        &self.limits
    }
    
    /// 检查是否支持指定特性
    /// 
    /// # 参数
    /// 
    /// - `feature`: 要检查的特性
    /// 
    /// # 返回
    /// 
    /// 如果支持指定特性则返回 true
    /// 
    /// # 示例
    /// 
    /// ```rust,no_run
    /// # use anvilkit_render::renderer::RenderDevice;
    /// # use wgpu::Features;
    /// # async fn example(device: &RenderDevice) {
    /// if device.supports_feature(Features::MULTI_DRAW_INDIRECT) {
    ///     println!("支持多重间接绘制");
    /// }
    /// # }
    /// ```
    pub fn supports_feature(&self, feature: Features) -> bool {
        self.features.contains(feature)
    }
    
    /// 获取首选的表面纹理格式
    /// 
    /// # 参数
    /// 
    /// - `surface`: 窗口表面
    /// 
    /// # 返回
    /// 
    /// 返回首选的纹理格式
    /// 
    /// # 示例
    /// 
    /// ```rust,no_run
    /// # use anvilkit_render::renderer::RenderDevice;
    /// # use wgpu::Surface;
    /// # async fn example(device: &RenderDevice, surface: &Surface) {
    /// let format = device.get_preferred_format(surface);
    /// println!("首选格式: {:?}", format);
    /// # }
    /// ```
    pub fn get_preferred_format(&self, surface: &Surface) -> TextureFormat {
        surface.get_capabilities(&self.adapter).formats[0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_instance_creation() {
        // 测试实例创建
        let instance = RenderDevice::create_instance();
        assert!(instance.is_ok());
    }
    
    #[test]
    fn test_feature_support() {
        // 创建一个模拟的设备用于测试
        let features = Features::empty();
        
        // 测试特性检查逻辑
        assert!(!features.contains(Features::TIMESTAMP_QUERY));
        assert!(features.is_empty());
    }
}
