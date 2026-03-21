//! # 顶点和索引缓冲区管理
//!
//! 提供类型安全的 GPU 缓冲区创建和顶点数据定义。

use wgpu::{Buffer, BufferUsages, VertexBufferLayout, VertexStepMode, VertexAttribute, VertexFormat};
use bytemuck::{Pod, Zeroable};

use crate::renderer::RenderDevice;

/// 顶点数据 trait
///
/// 实现此 trait 的类型可以安全地用作 GPU 顶点缓冲区数据。
///
/// # 示例
///
/// ```rust
/// use anvilkit_render::renderer::buffer::{Vertex, ColorVertex};
///
/// let layout = ColorVertex::layout();
/// assert_eq!(layout.array_stride, std::mem::size_of::<ColorVertex>() as u64);
/// ```
pub trait Vertex: Pod + Zeroable {
    /// 返回此顶点类型的缓冲区布局描述
    fn layout() -> VertexBufferLayout<'static>;
}

/// 带颜色的顶点
///
/// 包含 3D 位置和 RGB 颜色的基础顶点类型。
///
/// # 内存布局
///
/// | 偏移 | 属性 | 格式 |
/// |------|------|------|
/// | 0 | position | Float32x3 |
/// | 12 | color | Float32x3 |
///
/// # 示例
///
/// ```rust
/// use anvilkit_render::renderer::buffer::ColorVertex;
///
/// let vertex = ColorVertex {
///     position: [0.0, 0.5, 0.0],
///     color: [1.0, 0.0, 0.0],
/// };
/// assert_eq!(vertex.position[1], 0.5);
/// ```
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct ColorVertex {
    /// 3D 位置 (x, y, z)
    pub position: [f32; 3],
    /// RGB 颜色 (r, g, b)
    pub color: [f32; 3],
}

impl Vertex for ColorVertex {
    fn layout() -> VertexBufferLayout<'static> {
        const ATTRIBUTES: &[VertexAttribute] = &[
            // position: location 0
            VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: VertexFormat::Float32x3,
            },
            // color: location 1
            VertexAttribute {
                offset: std::mem::size_of::<[f32; 3]>() as u64,
                shader_location: 1,
                format: VertexFormat::Float32x3,
            },
        ];

        VertexBufferLayout {
            array_stride: std::mem::size_of::<ColorVertex>() as u64,
            step_mode: VertexStepMode::Vertex,
            attributes: ATTRIBUTES,
        }
    }
}

/// 网格顶点（位置 + 法线 + 纹理坐标）
///
/// 用于 glTF 网格渲染的标准顶点格式。
///
/// # 内存布局
///
/// | 偏移 | 属性 | 格式 |
/// |------|------|------|
/// | 0 | position | Float32x3 |
/// | 12 | normal | Float32x3 |
/// | 24 | texcoord | Float32x2 |
///
/// # 示例
///
/// ```rust
/// use anvilkit_render::renderer::buffer::MeshVertex;
///
/// let vertex = MeshVertex {
///     position: [0.0, 1.0, 0.0],
///     normal: [0.0, 1.0, 0.0],
///     texcoord: [0.5, 0.5],
/// };
/// assert_eq!(std::mem::size_of::<MeshVertex>(), 32);
/// ```
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct MeshVertex {
    /// 3D 位置 (x, y, z)
    pub position: [f32; 3],
    /// 表面法线 (x, y, z)
    pub normal: [f32; 3],
    /// 纹理坐标 (u, v)
    pub texcoord: [f32; 2],
}

impl Vertex for MeshVertex {
    fn layout() -> VertexBufferLayout<'static> {
        const ATTRIBUTES: &[VertexAttribute] = &[
            VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: VertexFormat::Float32x3,
            },
            VertexAttribute {
                offset: 12,
                shader_location: 1,
                format: VertexFormat::Float32x3,
            },
            VertexAttribute {
                offset: 24,
                shader_location: 2,
                format: VertexFormat::Float32x2,
            },
        ];

        VertexBufferLayout {
            array_stride: std::mem::size_of::<MeshVertex>() as u64,
            step_mode: VertexStepMode::Vertex,
            attributes: ATTRIBUTES,
        }
    }
}

/// PBR 顶点（位置 + 法线 + 纹理坐标 + 切线）
///
/// 用于支持法线贴图的 PBR 渲染管线。
///
/// # 内存布局
///
/// | 偏移 | 属性 | 格式 |
/// |------|------|------|
/// | 0 | position | Float32x3 |
/// | 12 | normal | Float32x3 |
/// | 24 | texcoord | Float32x2 |
/// | 32 | tangent | Float32x4 |
///
/// # 示例
///
/// ```rust
/// use anvilkit_render::renderer::buffer::PbrVertex;
///
/// let vertex = PbrVertex {
///     position: [0.0, 1.0, 0.0],
///     normal: [0.0, 1.0, 0.0],
///     texcoord: [0.5, 0.5],
///     tangent: [1.0, 0.0, 0.0, 1.0],
/// };
/// assert_eq!(std::mem::size_of::<PbrVertex>(), 48);
/// ```
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct PbrVertex {
    /// 3D 位置 (x, y, z)
    pub position: [f32; 3],
    /// 表面法线 (x, y, z)
    pub normal: [f32; 3],
    /// 纹理坐标 (u, v)
    pub texcoord: [f32; 2],
    /// 切线 (x, y, z, w) — w 是 bitangent sign
    pub tangent: [f32; 4],
}

impl Vertex for PbrVertex {
    fn layout() -> VertexBufferLayout<'static> {
        const ATTRIBUTES: &[VertexAttribute] = &[
            VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: VertexFormat::Float32x3,
            },
            VertexAttribute {
                offset: 12,
                shader_location: 1,
                format: VertexFormat::Float32x3,
            },
            VertexAttribute {
                offset: 24,
                shader_location: 2,
                format: VertexFormat::Float32x2,
            },
            VertexAttribute {
                offset: 32,
                shader_location: 3,
                format: VertexFormat::Float32x4,
            },
        ];

        VertexBufferLayout {
            array_stride: std::mem::size_of::<PbrVertex>() as u64,
            step_mode: VertexStepMode::Vertex,
            attributes: ATTRIBUTES,
        }
    }
}

/// 带骨骼蒙皮的 PBR 顶点
///
/// 在 PbrVertex 基础上增加 joint_indices 和 joint_weights，
/// 用于 GPU 端骨骼动画蒙皮。
///
/// # 内存布局
///
/// | 偏移 | 属性 | 格式 |
/// |------|------|------|
/// | 0 | position | Float32x3 |
/// | 12 | normal | Float32x3 |
/// | 24 | texcoord | Float32x2 |
/// | 32 | tangent | Float32x4 |
/// | 48 | joint_indices | Uint16x4 |
/// | 56 | joint_weights | Float32x4 |
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct SkinnedVertex {
    /// 3D 位置 (x, y, z)
    pub position: [f32; 3],
    /// 表面法线 (x, y, z)
    pub normal: [f32; 3],
    /// 纹理坐标 (u, v)
    pub texcoord: [f32; 2],
    /// 切线 (x, y, z, w) — w is bitangent sign
    pub tangent: [f32; 4],
    /// Indices of the 4 influencing skeleton joints
    pub joint_indices: [u16; 4],
    /// Blend weights for the 4 influencing joints
    pub joint_weights: [f32; 4],
}

impl Vertex for SkinnedVertex {
    fn layout() -> VertexBufferLayout<'static> {
        const ATTRIBUTES: &[VertexAttribute] = &[
            VertexAttribute { offset: 0, shader_location: 0, format: VertexFormat::Float32x3 },
            VertexAttribute { offset: 12, shader_location: 1, format: VertexFormat::Float32x3 },
            VertexAttribute { offset: 24, shader_location: 2, format: VertexFormat::Float32x2 },
            VertexAttribute { offset: 32, shader_location: 3, format: VertexFormat::Float32x4 },
            VertexAttribute { offset: 48, shader_location: 4, format: VertexFormat::Uint16x4 },
            VertexAttribute { offset: 56, shader_location: 5, format: VertexFormat::Float32x4 },
        ];

        VertexBufferLayout {
            array_stride: std::mem::size_of::<SkinnedVertex>() as u64,
            step_mode: VertexStepMode::Vertex,
            attributes: ATTRIBUTES,
        }
    }
}

/// 创建顶点缓冲区
///
/// 将顶点数据上传到 GPU 内存。
///
/// # 参数
///
/// - `device`: 渲染设备
/// - `label`: 缓冲区标签（用于调试）
/// - `vertices`: 顶点数据切片
///
/// # 示例
///
/// ```rust,no_run
/// use anvilkit_render::renderer::buffer::{ColorVertex, create_vertex_buffer};
/// use anvilkit_render::renderer::RenderDevice;
///
/// # async fn example(device: &RenderDevice) {
/// let vertices = [
///     ColorVertex { position: [0.0, 0.5, 0.0], color: [1.0, 0.0, 0.0] },
///     ColorVertex { position: [-0.5, -0.5, 0.0], color: [0.0, 1.0, 0.0] },
///     ColorVertex { position: [0.5, -0.5, 0.0], color: [0.0, 0.0, 1.0] },
/// ];
/// let buffer = create_vertex_buffer(device, "Triangle", &vertices);
/// # }
/// ```
pub fn create_vertex_buffer<V: Vertex>(
    device: &RenderDevice,
    label: &str,
    vertices: &[V],
) -> Buffer {
    use wgpu::util::{BufferInitDescriptor, DeviceExt};

    device.device().create_buffer_init(&BufferInitDescriptor {
        label: Some(label),
        contents: bytemuck::cast_slice(vertices),
        usage: BufferUsages::VERTEX,
    })
}

/// 创建索引缓冲区
///
/// 将索引数据上传到 GPU 内存。
///
/// # 参数
///
/// - `device`: 渲染设备
/// - `label`: 缓冲区标签
/// - `indices`: 16 位索引数据切片
///
/// # 示例
///
/// ```rust,no_run
/// use anvilkit_render::renderer::buffer::create_index_buffer;
/// use anvilkit_render::renderer::RenderDevice;
///
/// # async fn example(device: &RenderDevice) {
/// let indices: &[u16] = &[0, 1, 2];
/// let buffer = create_index_buffer(device, "Triangle Indices", indices);
/// # }
/// ```
pub fn create_index_buffer(
    device: &RenderDevice,
    label: &str,
    indices: &[u16],
) -> Buffer {
    use wgpu::util::{BufferInitDescriptor, DeviceExt};

    device.device().create_buffer_init(&BufferInitDescriptor {
        label: Some(label),
        contents: bytemuck::cast_slice(indices),
        usage: BufferUsages::INDEX,
    })
}

/// 创建 u32 索引缓冲区
///
/// 用于 glTF 模型等需要超过 65535 个顶点的网格。
///
/// # 示例
///
/// ```rust,no_run
/// use anvilkit_render::renderer::buffer::create_index_buffer_u32;
/// use anvilkit_render::renderer::RenderDevice;
///
/// # async fn example(device: &RenderDevice) {
/// let indices: &[u32] = &[0, 1, 2, 2, 3, 0];
/// let buffer = create_index_buffer_u32(device, "Mesh Indices", indices);
/// # }
/// ```
pub fn create_index_buffer_u32(
    device: &RenderDevice,
    label: &str,
    indices: &[u32],
) -> Buffer {
    use wgpu::util::{BufferInitDescriptor, DeviceExt};

    device.device().create_buffer_init(&BufferInitDescriptor {
        label: Some(label),
        contents: bytemuck::cast_slice(indices),
        usage: BufferUsages::INDEX,
    })
}

/// AnvilKit 标准深度纹理格式
pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

/// AnvilKit HDR 渲染目标格式
pub const HDR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

/// 默认阴影贴图分辨率
pub const SHADOW_MAP_SIZE: u32 = 2048;

/// MSAA 采样数（设为 1 可禁用 MSAA）
pub const MSAA_SAMPLE_COUNT: u32 = 4;

/// 创建阴影深度贴图
///
/// 使用 Depth32Float 格式，可作为渲染附件（shadow pass）和纹理采样（main pass）。
///
/// # 示例
///
/// ```rust,no_run
/// use anvilkit_render::renderer::buffer::create_shadow_map;
/// use anvilkit_render::renderer::RenderDevice;
///
/// # async fn example(device: &RenderDevice) {
/// let (shadow_tex, shadow_view) = create_shadow_map(device, 2048, "Shadow Map");
/// # }
/// ```
pub fn create_shadow_map(
    device: &RenderDevice,
    size: u32,
    label: &str,
) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.device().create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: DEPTH_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    (texture, view)
}

/// 创建阴影比较采样器
///
/// 使用 `LessEqual` 比较函数，用于 `textureSampleCompare()` 调用。
///
/// # 示例
///
/// ```rust,no_run
/// use anvilkit_render::renderer::buffer::create_shadow_sampler;
/// use anvilkit_render::renderer::RenderDevice;
///
/// # async fn example(device: &RenderDevice) {
/// let sampler = create_shadow_sampler(device, "Shadow Sampler");
/// # }
/// ```
pub fn create_shadow_sampler(device: &RenderDevice, label: &str) -> wgpu::Sampler {
    device.device().create_sampler(&wgpu::SamplerDescriptor {
        label: Some(label),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        compare: Some(wgpu::CompareFunction::LessEqual),
        ..Default::default()
    })
}

/// 创建 Uniform 缓冲区
///
/// 使用 `UNIFORM | COPY_DST` 用法创建，支持每帧通过 `queue.write_buffer()` 更新。
///
/// # 参数
///
/// - `device`: 渲染设备
/// - `label`: 缓冲区标签
/// - `contents`: 初始数据（字节切片）
///
/// # 示例
///
/// ```rust,no_run
/// use anvilkit_render::renderer::buffer::create_uniform_buffer;
/// use anvilkit_render::renderer::RenderDevice;
///
/// # async fn example(device: &RenderDevice) {
/// let mvp_data = [0u8; 64]; // 4x4 f32 矩阵
/// let buffer = create_uniform_buffer(device, "MVP Uniform", &mvp_data);
/// # }
/// ```
pub fn create_uniform_buffer(
    device: &RenderDevice,
    label: &str,
    contents: &[u8],
) -> Buffer {
    use wgpu::util::{BufferInitDescriptor, DeviceExt};

    device.device().create_buffer_init(&BufferInitDescriptor {
        label: Some(label),
        contents,
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
    })
}

/// 创建深度纹理和视图
///
/// 在窗口大小变化时需要重新创建。
///
/// # 参数
///
/// - `device`: 渲染设备
/// - `width`: 纹理宽度
/// - `height`: 纹理高度
/// - `label`: 纹理标签
///
/// # 返回
///
/// 返回 (Texture, TextureView) 元组
///
/// # 示例
///
/// ```rust,no_run
/// use anvilkit_render::renderer::buffer::create_depth_texture;
/// use anvilkit_render::renderer::RenderDevice;
///
/// # async fn example(device: &RenderDevice) {
/// let (texture, view) = create_depth_texture(device, 800, 600, "Depth");
/// # }
/// ```
pub fn create_depth_texture(
    device: &RenderDevice,
    width: u32,
    height: u32,
    label: &str,
) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.device().create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: DEPTH_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    (texture, view)
}

/// 创建 HDR 渲染目标纹理和视图
///
/// 使用 `Rgba16Float` 格式的离屏渲染目标，用于 HDR 渲染管线。
/// 场景先渲染到 HDR RT，再通过后处理 pass 进行 tone mapping 输出到 swapchain。
///
/// # 参数
///
/// - `device`: 渲染设备
/// - `width`: 纹理宽度
/// - `height`: 纹理高度
/// - `label`: 纹理标签
///
/// # 返回
///
/// 返回 (Texture, TextureView) 元组
///
/// # 示例
///
/// ```rust,no_run
/// use anvilkit_render::renderer::buffer::create_hdr_render_target;
/// use anvilkit_render::renderer::RenderDevice;
///
/// # async fn example(device: &RenderDevice) {
/// let (hdr_texture, hdr_view) = create_hdr_render_target(device, 800, 600, "HDR RT");
/// # }
/// ```
pub fn create_hdr_render_target(
    device: &RenderDevice,
    width: u32,
    height: u32,
    label: &str,
) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.device().create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: HDR_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    (texture, view)
}

/// 创建 MSAA 深度纹理（sample_count=MSAA_SAMPLE_COUNT）
pub fn create_depth_texture_msaa(
    device: &RenderDevice,
    width: u32,
    height: u32,
    label: &str,
) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.device().create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: MSAA_SAMPLE_COUNT,
        dimension: wgpu::TextureDimension::D2,
        format: DEPTH_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    (texture, view)
}

/// 创建 MSAA HDR 颜色纹理（sample_count=MSAA_SAMPLE_COUNT，仅 RENDER_ATTACHMENT）
///
/// 此纹理用作 MSAA 渲染附件，resolve 到单采样 HDR RT 后由 tonemap pass 采样。
pub fn create_hdr_msaa_texture(
    device: &RenderDevice,
    width: u32,
    height: u32,
    label: &str,
) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.device().create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: MSAA_SAMPLE_COUNT,
        dimension: wgpu::TextureDimension::D2,
        format: HDR_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    (texture, view)
}

/// 从 RGBA 数据创建 GPU 纹理和视图
///
/// # 参数
///
/// - `device`: 渲染设备
/// - `queue`: GPU 命令队列（用于写入纹理数据）
/// - `width`: 纹理宽度
/// - `height`: 纹理高度
/// - `data`: RGBA 像素数据（每像素 4 字节）
/// - `label`: 纹理标签
///
/// # 示例
///
/// ```rust,no_run
/// use anvilkit_render::renderer::buffer::create_texture;
/// use anvilkit_render::renderer::RenderDevice;
///
/// # async fn example(device: &RenderDevice) {
/// let rgba = vec![255u8; 4 * 4 * 4]; // 4x4 白色纹理
/// let (texture, view) = create_texture(device, 4, 4, &rgba, "White Texture");
/// # }
/// ```
pub fn create_texture(
    device: &RenderDevice,
    width: u32,
    height: u32,
    data: &[u8],
    label: &str,
) -> (wgpu::Texture, wgpu::TextureView) {
    let size = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };

    let texture = device.device().create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    device.queue().write_texture(
        wgpu::ImageCopyTexture {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        data,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(4 * width),
            rows_per_image: Some(height),
        },
        size,
    );

    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    (texture, view)
}

/// 从 RGBA 数据创建线性空间 GPU 纹理和视图
///
/// 与 `create_texture` 相同，但使用 `Rgba8Unorm`（线性空间）而非 sRGB。
/// 法线贴图必须用线性格式，否则 sRGB 解码会破坏法线方向。
///
/// # 示例
///
/// ```rust,no_run
/// use anvilkit_render::renderer::buffer::create_texture_linear;
/// use anvilkit_render::renderer::RenderDevice;
///
/// # async fn example(device: &RenderDevice) {
/// let normal_data = vec![128u8, 128, 255, 255]; // flat normal (0.5, 0.5, 1.0)
/// let (texture, view) = create_texture_linear(device, 1, 1, &normal_data, "Normal Map");
/// # }
/// ```
pub fn create_texture_linear(
    device: &RenderDevice,
    width: u32,
    height: u32,
    data: &[u8],
    label: &str,
) -> (wgpu::Texture, wgpu::TextureView) {
    let size = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };

    let texture = device.device().create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    device.queue().write_texture(
        wgpu::ImageCopyTexture {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        data,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(4 * width),
            rows_per_image: Some(height),
        },
        size,
    );

    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    (texture, view)
}

/// 创建线性过滤纹理采样器
///
/// # 示例
///
/// ```rust,no_run
/// use anvilkit_render::renderer::buffer::create_sampler;
/// use anvilkit_render::renderer::RenderDevice;
///
/// # async fn example(device: &RenderDevice) {
/// let sampler = create_sampler(device, "Linear Sampler");
/// # }
/// ```
pub fn create_sampler(device: &RenderDevice, label: &str) -> wgpu::Sampler {
    device.device().create_sampler(&wgpu::SamplerDescriptor {
        label: Some(label),
        address_mode_u: wgpu::AddressMode::Repeat,
        address_mode_v: wgpu::AddressMode::Repeat,
        address_mode_w: wgpu::AddressMode::Repeat,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_vertex_size() {
        assert_eq!(std::mem::size_of::<ColorVertex>(), 24); // 6 * f32
    }

    #[test]
    fn test_color_vertex_layout() {
        let layout = ColorVertex::layout();
        assert_eq!(layout.array_stride, 24);
        assert_eq!(layout.attributes.len(), 2);
        assert_eq!(layout.attributes[0].offset, 0);
        assert_eq!(layout.attributes[0].shader_location, 0);
        assert_eq!(layout.attributes[1].offset, 12);
        assert_eq!(layout.attributes[1].shader_location, 1);
    }

    #[test]
    fn test_color_vertex_creation() {
        let v = ColorVertex {
            position: [1.0, 2.0, 3.0],
            color: [0.5, 0.5, 0.5],
        };
        assert_eq!(v.position, [1.0, 2.0, 3.0]);
        assert_eq!(v.color, [0.5, 0.5, 0.5]);
    }

    #[test]
    fn test_color_vertex_bytemuck() {
        let vertices = [
            ColorVertex { position: [0.0, 0.5, 0.0], color: [1.0, 0.0, 0.0] },
            ColorVertex { position: [-0.5, -0.5, 0.0], color: [0.0, 1.0, 0.0] },
        ];
        let bytes: &[u8] = bytemuck::cast_slice(&vertices);
        assert_eq!(bytes.len(), 48); // 2 vertices * 24 bytes each
    }

    #[test]
    fn test_depth_format() {
        assert_eq!(DEPTH_FORMAT, wgpu::TextureFormat::Depth32Float);
    }

    #[test]
    fn test_mesh_vertex_size() {
        assert_eq!(std::mem::size_of::<MeshVertex>(), 32); // 8 * f32
    }

    #[test]
    fn test_mesh_vertex_layout() {
        let layout = MeshVertex::layout();
        assert_eq!(layout.array_stride, 32);
        assert_eq!(layout.attributes.len(), 3);
        assert_eq!(layout.attributes[0].offset, 0);   // position
        assert_eq!(layout.attributes[1].offset, 12);  // normal
        assert_eq!(layout.attributes[2].offset, 24);  // texcoord
    }

    #[test]
    fn test_pbr_vertex_size() {
        assert_eq!(std::mem::size_of::<PbrVertex>(), 48); // 12 * f32
    }

    #[test]
    fn test_pbr_vertex_layout() {
        let layout = PbrVertex::layout();
        assert_eq!(layout.array_stride, 48);
        assert_eq!(layout.attributes.len(), 4);
        assert_eq!(layout.attributes[0].offset, 0);   // position
        assert_eq!(layout.attributes[1].offset, 12);  // normal
        assert_eq!(layout.attributes[2].offset, 24);  // texcoord
        assert_eq!(layout.attributes[3].offset, 32);  // tangent
    }
}
