//! # 渲染管线管理
//! 
//! 提供渲染管线的创建、配置和管理功能。

use wgpu::{
    RenderPipeline, RenderPipelineDescriptor, VertexState, FragmentState,
    PrimitiveState, MultisampleState, PipelineLayoutDescriptor,
    ShaderModule, ShaderModuleDescriptor, ShaderSource,
    VertexBufferLayout, ColorTargetState, BlendState, ColorWrites,
    PrimitiveTopology, FrontFace, Face, PolygonMode,
    TextureFormat, Device,
};
use log::{info, warn, error, debug};

use crate::renderer::RenderDevice;
use anvilkit_core::error::{AnvilKitError, Result};

/// 渲染管线构建器
/// 
/// 提供流式 API 来配置和创建渲染管线。
/// 
/// # 设计理念
/// 
/// - **流式配置**: 使用构建器模式简化管线配置
/// - **默认值**: 提供合理的默认配置参数
/// - **类型安全**: 编译时检查配置的正确性
/// - **灵活性**: 支持自定义着色器和状态配置
/// 
/// # 示例
/// 
/// ```rust,no_run
/// use anvilkit_render::renderer::{RenderDevice, RenderPipelineBuilder};
/// use wgpu::TextureFormat;
/// 
/// # async fn example(device: &RenderDevice) -> anvilkit_core::error::Result<()> {
/// let pipeline = RenderPipelineBuilder::new()
///     .with_vertex_shader("vertex_shader.wgsl")
///     .with_fragment_shader("fragment_shader.wgsl")
///     .with_format(TextureFormat::Bgra8UnormSrgb)
///     .build(device)?;
/// # Ok(())
/// # }
/// ```
pub struct RenderPipelineBuilder {
    /// 顶点着色器源码
    vertex_shader: Option<String>,
    /// 片段着色器源码
    fragment_shader: Option<String>,
    /// 渲染目标格式
    format: Option<TextureFormat>,
    /// 图元拓扑
    topology: PrimitiveTopology,
    /// 多重采样状态
    multisample_count: u32,
    /// 标签
    label: Option<String>,
}

impl Default for RenderPipelineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderPipelineBuilder {
    /// 创建新的渲染管线构建器
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_render::renderer::RenderPipelineBuilder;
    /// 
    /// let builder = RenderPipelineBuilder::new();
    /// ```
    pub fn new() -> Self {
        Self {
            vertex_shader: None,
            fragment_shader: None,
            format: None,
            topology: PrimitiveTopology::TriangleList,
            multisample_count: 1,
            label: None,
        }
    }
    
    /// 设置顶点着色器
    /// 
    /// # 参数
    /// 
    /// - `source`: 着色器源码
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_render::renderer::RenderPipelineBuilder;
    /// 
    /// let builder = RenderPipelineBuilder::new()
    ///     .with_vertex_shader("vertex_shader.wgsl");
    /// ```
    pub fn with_vertex_shader<S: Into<String>>(mut self, source: S) -> Self {
        self.vertex_shader = Some(source.into());
        self
    }
    
    /// 设置片段着色器
    /// 
    /// # 参数
    /// 
    /// - `source`: 着色器源码
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_render::renderer::RenderPipelineBuilder;
    /// 
    /// let builder = RenderPipelineBuilder::new()
    ///     .with_fragment_shader("fragment_shader.wgsl");
    /// ```
    pub fn with_fragment_shader<S: Into<String>>(mut self, source: S) -> Self {
        self.fragment_shader = Some(source.into());
        self
    }
    
    /// 设置渲染目标格式
    /// 
    /// # 参数
    /// 
    /// - `format`: 纹理格式
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_render::renderer::RenderPipelineBuilder;
    /// use wgpu::TextureFormat;
    /// 
    /// let builder = RenderPipelineBuilder::new()
    ///     .with_format(TextureFormat::Bgra8UnormSrgb);
    /// ```
    pub fn with_format(mut self, format: TextureFormat) -> Self {
        self.format = Some(format);
        self
    }
    
    /// 设置图元拓扑
    /// 
    /// # 参数
    /// 
    /// - `topology`: 图元拓扑类型
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_render::renderer::RenderPipelineBuilder;
    /// use wgpu::PrimitiveTopology;
    /// 
    /// let builder = RenderPipelineBuilder::new()
    ///     .with_topology(PrimitiveTopology::LineList);
    /// ```
    pub fn with_topology(mut self, topology: PrimitiveTopology) -> Self {
        self.topology = topology;
        self
    }
    
    /// 设置多重采样数量
    /// 
    /// # 参数
    /// 
    /// - `count`: 采样数量
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_render::renderer::RenderPipelineBuilder;
    /// 
    /// let builder = RenderPipelineBuilder::new()
    ///     .with_multisample_count(4);
    /// ```
    pub fn with_multisample_count(mut self, count: u32) -> Self {
        self.multisample_count = count;
        self
    }
    
    /// 设置标签
    /// 
    /// # 参数
    /// 
    /// - `label`: 管线标签
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_render::renderer::RenderPipelineBuilder;
    /// 
    /// let builder = RenderPipelineBuilder::new()
    ///     .with_label("My Render Pipeline");
    /// ```
    pub fn with_label<S: Into<String>>(mut self, label: S) -> Self {
        self.label = Some(label.into());
        self
    }
    
    /// 构建渲染管线
    /// 
    /// # 参数
    /// 
    /// - `device`: 渲染设备
    /// 
    /// # 返回
    /// 
    /// 成功时返回 BasicRenderPipeline，失败时返回错误
    /// 
    /// # 示例
    /// 
    /// ```rust,no_run
    /// use anvilkit_render::renderer::{RenderDevice, RenderPipelineBuilder};
    /// use wgpu::TextureFormat;
    /// 
    /// # async fn example(device: &RenderDevice) -> anvilkit_core::error::Result<()> {
    /// let pipeline = RenderPipelineBuilder::new()
    ///     .with_vertex_shader("vertex_shader.wgsl")
    ///     .with_fragment_shader("fragment_shader.wgsl")
    ///     .with_format(TextureFormat::Bgra8UnormSrgb)
    ///     .build(device)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn build(self, device: &RenderDevice) -> Result<BasicRenderPipeline> {
        let vertex_shader = self.vertex_shader
            .ok_or_else(|| AnvilKitError::Render("缺少顶点着色器".to_string()))?;
        
        let fragment_shader = self.fragment_shader
            .ok_or_else(|| AnvilKitError::Render("缺少片段着色器".to_string()))?;
        
        let format = self.format
            .ok_or_else(|| AnvilKitError::Render("缺少渲染目标格式".to_string()))?;
        
        BasicRenderPipeline::new(
            device,
            &vertex_shader,
            &fragment_shader,
            format,
            self.topology,
            self.multisample_count,
            self.label.as_deref(),
        )
    }
}

/// 基础渲染管线
/// 
/// 封装 wgpu 渲染管线，提供基础的渲染功能。
/// 
/// # 示例
/// 
/// ```rust,no_run
/// use anvilkit_render::renderer::{RenderDevice, BasicRenderPipeline};
/// use wgpu::{TextureFormat, PrimitiveTopology};
/// 
/// # async fn example(device: &RenderDevice) -> anvilkit_core::error::Result<()> {
/// let pipeline = BasicRenderPipeline::new(
///     device,
///     "vertex_shader.wgsl",
///     "fragment_shader.wgsl",
///     TextureFormat::Bgra8UnormSrgb,
///     PrimitiveTopology::TriangleList,
///     1,
///     Some("Basic Pipeline"),
/// )?;
/// # Ok(())
/// # }
/// ```
pub struct BasicRenderPipeline {
    /// wgpu 渲染管线
    pipeline: RenderPipeline,
    /// 顶点着色器模块
    vertex_shader: ShaderModule,
    /// 片段着色器模块
    fragment_shader: ShaderModule,
}

impl BasicRenderPipeline {
    /// 创建新的基础渲染管线
    /// 
    /// # 参数
    /// 
    /// - `device`: 渲染设备
    /// - `vertex_source`: 顶点着色器源码
    /// - `fragment_source`: 片段着色器源码
    /// - `format`: 渲染目标格式
    /// - `topology`: 图元拓扑
    /// - `multisample_count`: 多重采样数量
    /// - `label`: 可选的标签
    /// 
    /// # 返回
    /// 
    /// 成功时返回 BasicRenderPipeline，失败时返回错误
    pub fn new(
        device: &RenderDevice,
        vertex_source: &str,
        fragment_source: &str,
        format: TextureFormat,
        topology: PrimitiveTopology,
        multisample_count: u32,
        label: Option<&str>,
    ) -> Result<Self> {
        info!("创建基础渲染管线: {:?}", label);
        
        let wgpu_device = device.device();
        
        // 创建着色器模块
        let vertex_shader = Self::create_shader_module(
            wgpu_device,
            vertex_source,
            Some("Vertex Shader"),
        )?;
        
        let fragment_shader = Self::create_shader_module(
            wgpu_device,
            fragment_source,
            Some("Fragment Shader"),
        )?;
        
        // 创建管线布局
        let layout = wgpu_device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Basic Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });
        
        // 创建渲染管线
        let pipeline = wgpu_device.create_render_pipeline(&RenderPipelineDescriptor {
            label,
            layout: Some(&layout),
            vertex: VertexState {
                module: &vertex_shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            primitive: PrimitiveState {
                topology,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: multisample_count,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(FragmentState {
                module: &fragment_shader,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });
        
        info!("基础渲染管线创建成功");
        
        Ok(Self {
            pipeline,
            vertex_shader,
            fragment_shader,
        })
    }
    
    /// 创建着色器模块
    /// 
    /// # 参数
    /// 
    /// - `device`: GPU 设备
    /// - `source`: 着色器源码
    /// - `label`: 可选的标签
    /// 
    /// # 返回
    /// 
    /// 成功时返回 ShaderModule，失败时返回错误
    fn create_shader_module(
        device: &Device,
        source: &str,
        label: Option<&str>,
    ) -> Result<ShaderModule> {
        debug!("创建着色器模块: {:?}", label);
        
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label,
            source: ShaderSource::Wgsl(source.into()),
        });
        
        Ok(shader)
    }
    
    /// 获取渲染管线
    /// 
    /// # 返回
    /// 
    /// 返回 wgpu 渲染管线的引用
    /// 
    /// # 示例
    /// 
    /// ```rust,no_run
    /// # use anvilkit_render::renderer::BasicRenderPipeline;
    /// # async fn example(pipeline: &BasicRenderPipeline) {
    /// let wgpu_pipeline = pipeline.pipeline();
    /// // 使用管线进行渲染
    /// # }
    /// ```
    pub fn pipeline(&self) -> &RenderPipeline {
        &self.pipeline
    }
    
    /// 获取顶点着色器
    /// 
    /// # 返回
    /// 
    /// 返回顶点着色器模块的引用
    /// 
    /// # 示例
    /// 
    /// ```rust,no_run
    /// # use anvilkit_render::renderer::BasicRenderPipeline;
    /// # async fn example(pipeline: &BasicRenderPipeline) {
    /// let vertex_shader = pipeline.vertex_shader();
    /// # }
    /// ```
    pub fn vertex_shader(&self) -> &ShaderModule {
        &self.vertex_shader
    }
    
    /// 获取片段着色器
    /// 
    /// # 返回
    /// 
    /// 返回片段着色器模块的引用
    /// 
    /// # 示例
    /// 
    /// ```rust,no_run
    /// # use anvilkit_render::renderer::BasicRenderPipeline;
    /// # async fn example(pipeline: &BasicRenderPipeline) {
    /// let fragment_shader = pipeline.fragment_shader();
    /// # }
    /// ```
    pub fn fragment_shader(&self) -> &ShaderModule {
        &self.fragment_shader
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wgpu::{TextureFormat, PrimitiveTopology};
    
    #[test]
    fn test_pipeline_builder_creation() {
        let builder = RenderPipelineBuilder::new()
            .with_vertex_shader("vertex.wgsl")
            .with_fragment_shader("fragment.wgsl")
            .with_format(TextureFormat::Bgra8UnormSrgb)
            .with_topology(PrimitiveTopology::LineList)
            .with_multisample_count(4)
            .with_label("Test Pipeline");
        
        assert_eq!(builder.vertex_shader.as_ref().unwrap(), "vertex.wgsl");
        assert_eq!(builder.fragment_shader.as_ref().unwrap(), "fragment.wgsl");
        assert_eq!(builder.format.unwrap(), TextureFormat::Bgra8UnormSrgb);
        assert_eq!(builder.topology, PrimitiveTopology::LineList);
        assert_eq!(builder.multisample_count, 4);
        assert_eq!(builder.label.as_ref().unwrap(), "Test Pipeline");
    }
    
    #[test]
    fn test_pipeline_builder_defaults() {
        let builder = RenderPipelineBuilder::new();
        
        assert!(builder.vertex_shader.is_none());
        assert!(builder.fragment_shader.is_none());
        assert!(builder.format.is_none());
        assert_eq!(builder.topology, PrimitiveTopology::TriangleList);
        assert_eq!(builder.multisample_count, 1);
        assert!(builder.label.is_none());
    }
}
