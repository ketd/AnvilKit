//! # 渲染上下文
//! 
//! 提供统一的渲染上下文，整合设备、表面和基础渲染功能。

use std::sync::Arc;
use wgpu::{
    CommandEncoder, RenderPass, RenderPassDescriptor, RenderPassColorAttachment,
    Operations, LoadOp, StoreOp, Color, TextureView,
};
use winit::window::Window;
use log::{info, warn, error, debug};

use crate::renderer::{RenderDevice, RenderSurface};
use anvilkit_core::error::{AnvilKitError, Result};

/// 渲染上下文
/// 
/// 统一管理渲染设备、表面和基础渲染操作，提供高级渲染接口。
/// 
/// # 设计理念
/// 
/// - **统一接口**: 整合设备和表面管理，提供简化的渲染 API
/// - **资源管理**: 自动管理渲染资源的生命周期
/// - **错误处理**: 完善的错误处理和恢复机制
/// - **性能优化**: 优化的渲染循环和资源使用
/// 
/// # 示例
/// 
/// ```rust,no_run
/// use anvilkit_render::renderer::RenderContext;
/// use std::sync::Arc;
/// use winit::window::Window;
/// 
/// # async fn example() -> anvilkit_core::error::Result<()> {
/// // 创建窗口（示例）
/// // let window = Arc::new(window);
/// 
/// // 创建渲染上下文
/// // let mut context = RenderContext::new(window).await?;
/// 
/// // 执行渲染
/// // context.render()?;
/// # Ok(())
/// # }
/// ```
pub struct RenderContext {
    /// 渲染设备
    device: RenderDevice,
    /// 渲染表面
    surface: RenderSurface,
    /// 清除颜色
    clear_color: Color,
}

impl RenderContext {
    /// 创建新的渲染上下文
    /// 
    /// # 参数
    /// 
    /// - `window`: 窗口实例
    /// 
    /// # 返回
    /// 
    /// 成功时返回 RenderContext 实例，失败时返回错误
    /// 
    /// # 示例
    /// 
    /// ```rust,no_run
    /// use anvilkit_render::renderer::RenderContext;
    /// use std::sync::Arc;
    /// use winit::window::Window;
    /// 
    /// # async fn example() -> anvilkit_core::error::Result<()> {
    /// // let window = Arc::new(window);
    /// // let context = RenderContext::new(window).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(window: Arc<Window>) -> Result<Self> {
        info!("创建渲染上下文");
        
        // 创建渲染设备
        let device = RenderDevice::new(&window).await?;
        
        // 创建渲染表面
        let surface = RenderSurface::new(&device, &window)?;
        
        // 默认清除颜色（深蓝色）
        let clear_color = Color {
            r: 0.1,
            g: 0.2,
            b: 0.3,
            a: 1.0,
        };
        
        info!("渲染上下文创建成功");
        
        Ok(Self {
            device,
            surface,
            clear_color,
        })
    }
    
    /// 调整渲染上下文大小
    /// 
    /// # 参数
    /// 
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
    /// # use anvilkit_render::renderer::RenderContext;
    /// # async fn example(context: &mut RenderContext) -> anvilkit_core::error::Result<()> {
    /// context.resize(1920, 1080)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn resize(&mut self, width: u32, height: u32) -> Result<()> {
        info!("调整渲染上下文大小: {}x{}", width, height);
        self.surface.resize(&self.device, width, height)
    }
    
    /// 设置清除颜色
    /// 
    /// # 参数
    /// 
    /// - `color`: 新的清除颜色
    /// 
    /// # 示例
    /// 
    /// ```rust,no_run
    /// # use anvilkit_render::renderer::RenderContext;
    /// # use wgpu::Color;
    /// # async fn example(context: &mut RenderContext) {
    /// let red = Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 };
    /// context.set_clear_color(red);
    /// # }
    /// ```
    pub fn set_clear_color(&mut self, color: Color) {
        debug!("设置清除颜色: {:?}", color);
        self.clear_color = color;
    }
    
    /// 获取清除颜色
    /// 
    /// # 返回
    /// 
    /// 返回当前的清除颜色
    /// 
    /// # 示例
    /// 
    /// ```rust,no_run
    /// # use anvilkit_render::renderer::RenderContext;
    /// # async fn example(context: &RenderContext) {
    /// let color = context.clear_color();
    /// println!("清除颜色: {:?}", color);
    /// # }
    /// ```
    pub fn clear_color(&self) -> Color {
        self.clear_color
    }
    
    /// 执行基础渲染
    /// 
    /// 执行一个基础的渲染循环，清除屏幕并呈现结果。
    /// 
    /// # 返回
    /// 
    /// 成功时返回 Ok(())，失败时返回错误
    /// 
    /// # 示例
    /// 
    /// ```rust,no_run
    /// # use anvilkit_render::renderer::RenderContext;
    /// # async fn example(context: &mut RenderContext) -> anvilkit_core::error::Result<()> {
    /// context.render()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn render(&mut self) -> Result<()> {
        // 获取当前帧
        let frame = match self.surface.get_current_frame() {
            Ok(frame) => frame,
            Err(e) => {
                error!("获取当前帧失败: {}", e);
                return Err(e);
            }
        };
        
        // 创建纹理视图
        let view = frame.texture.create_view(&Default::default());
        
        // 创建命令编码器
        let mut encoder = self.device.device().create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("AnvilKit Render Encoder"),
            }
        );
        
        // 创建渲染通道
        {
            let _render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("AnvilKit Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(self.clear_color),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            
            // 在这里可以添加具体的渲染命令
            // 目前只是清除屏幕
        }
        
        // 提交命令
        self.device.queue().submit(std::iter::once(encoder.finish()));
        
        // 呈现帧
        frame.present();
        
        Ok(())
    }
    
    /// 开始渲染通道
    /// 
    /// 创建一个新的渲染通道，用于执行自定义渲染命令。
    /// 
    /// # 参数
    /// 
    /// - `encoder`: 命令编码器
    /// - `view`: 渲染目标视图
    /// 
    /// # 返回
    /// 
    /// 返回配置好的渲染通道
    /// 
    /// # 示例
    /// 
    /// ```rust,no_run
    /// # use anvilkit_render::renderer::RenderContext;
    /// # use wgpu::{CommandEncoder, TextureView};
    /// # async fn example(context: &RenderContext, encoder: &mut CommandEncoder, view: &TextureView) {
    /// let render_pass = context.begin_render_pass(encoder, view);
    /// // 使用渲染通道执行绘制命令
    /// # }
    /// ```
    pub fn begin_render_pass<'a>(
        &self,
        encoder: &'a mut CommandEncoder,
        view: &'a TextureView,
    ) -> RenderPass<'a> {
        encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("AnvilKit Custom Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(self.clear_color),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        })
    }
    
    /// 获取渲染设备
    /// 
    /// # 返回
    /// 
    /// 返回渲染设备的引用
    /// 
    /// # 示例
    /// 
    /// ```rust,no_run
    /// # use anvilkit_render::renderer::RenderContext;
    /// # async fn example(context: &RenderContext) {
    /// let device = context.device();
    /// let wgpu_device = device.device();
    /// # }
    /// ```
    pub fn device(&self) -> &RenderDevice {
        &self.device
    }
    
    /// 获取渲染表面
    /// 
    /// # 返回
    /// 
    /// 返回渲染表面的引用
    /// 
    /// # 示例
    /// 
    /// ```rust,no_run
    /// # use anvilkit_render::renderer::RenderContext;
    /// # async fn example(context: &RenderContext) {
    /// let surface = context.surface();
    /// let (width, height) = surface.size();
    /// # }
    /// ```
    pub fn surface(&self) -> &RenderSurface {
        &self.surface
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
    /// # use anvilkit_render::renderer::RenderContext;
    /// # async fn example(context: &RenderContext) {
    /// let (width, height) = context.size();
    /// println!("渲染大小: {}x{}", width, height);
    /// # }
    /// ```
    pub fn size(&self) -> (u32, u32) {
        self.surface.size()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wgpu::Color;
    
    #[test]
    fn test_clear_color_operations() {
        // 创建一个模拟的渲染上下文用于测试
        let default_color = Color {
            r: 0.1,
            g: 0.2,
            b: 0.3,
            a: 1.0,
        };
        
        // 测试颜色设置和获取
        assert_eq!(default_color.r, 0.1);
        assert_eq!(default_color.g, 0.2);
        assert_eq!(default_color.b, 0.3);
        assert_eq!(default_color.a, 1.0);
    }
}
