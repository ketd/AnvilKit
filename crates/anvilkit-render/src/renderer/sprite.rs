//! # 2D 精灵渲染系统
//!
//! 提供 Sprite 组件、2D 顶点格式、纹理图集和 z-order 排序。
//!
//! ## 设计
//!
//! - `Sprite`: ECS 组件，定义精灵的纹理、颜色、翻转
//! - `SpriteVertex`: 2D 顶点 (position + texcoord + color)
//! - `TextureAtlas`: 精灵图集，将大纹理划分为矩形子区域
//! - `SpriteBatch`: 收集同纹理的精灵并按 z-order 排序

use bevy_ecs::prelude::*;
use glam::{Vec2, Vec3};
use bytemuck::{Pod, Zeroable};
use wgpu::{self, VertexBufferLayout, VertexAttribute, VertexFormat, VertexStepMode};
use wgpu::util::DeviceExt;

use super::buffer::Vertex;

/// 2D 精灵顶点 (32 字节)
///
/// | 偏移 | 属性 | 格式 |
/// |------|------|------|
/// | 0 | position | Float32x3 (x, y, z-order) |
/// | 12 | texcoord | Float32x2 |
/// | 20 | color | Float32x3 (tint RGB) |
///
/// # 示例
///
/// ```rust
/// use anvilkit_render::renderer::sprite::SpriteVertex;
///
/// let vertex = SpriteVertex {
///     position: [100.0, 200.0, 0.0],
///     texcoord: [0.0, 0.0],
///     color: [1.0, 1.0, 1.0],
/// };
/// assert_eq!(std::mem::size_of::<SpriteVertex>(), 32);
/// ```
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct SpriteVertex {
    pub position: [f32; 3],  // x, y, z-order
    pub texcoord: [f32; 2],
    pub color: [f32; 3],     // tint
}

impl Vertex for SpriteVertex {
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
                format: VertexFormat::Float32x2,
            },
            VertexAttribute {
                offset: 20,
                shader_location: 2,
                format: VertexFormat::Float32x3,
            },
        ];

        VertexBufferLayout {
            array_stride: std::mem::size_of::<SpriteVertex>() as u64,
            step_mode: VertexStepMode::Vertex,
            attributes: ATTRIBUTES,
        }
    }
}

/// 纹理图集中的矩形区域
///
/// UV 坐标范围 [0, 1]，表示图集纹理中的子区域。
///
/// # 示例
///
/// ```rust
/// use anvilkit_render::renderer::sprite::AtlasRect;
///
/// let rect = AtlasRect::new(0.0, 0.0, 0.25, 0.25); // 左上角 1/4 区域
/// assert_eq!(rect.width(), 0.25);
/// assert_eq!(rect.height(), 0.25);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AtlasRect {
    /// 左上角 U
    pub u_min: f32,
    /// 左上角 V
    pub v_min: f32,
    /// 右下角 U
    pub u_max: f32,
    /// 右下角 V
    pub v_max: f32,
}

impl AtlasRect {
    pub fn new(u_min: f32, v_min: f32, u_max: f32, v_max: f32) -> Self {
        Self { u_min, v_min, u_max, v_max }
    }

    /// 全纹理区域
    pub fn full() -> Self {
        Self { u_min: 0.0, v_min: 0.0, u_max: 1.0, v_max: 1.0 }
    }

    pub fn width(&self) -> f32 { self.u_max - self.u_min }
    pub fn height(&self) -> f32 { self.v_max - self.v_min }
}

impl Default for AtlasRect {
    fn default() -> Self {
        Self::full()
    }
}

/// 纹理图集
///
/// 将一张大纹理划分为多个命名的矩形子区域。
///
/// # 示例
///
/// ```rust
/// use anvilkit_render::renderer::sprite::{TextureAtlas, AtlasRect};
///
/// let mut atlas = TextureAtlas::new(512, 512);
/// atlas.add_rect("player_idle", AtlasRect::new(0.0, 0.0, 0.25, 0.5));
/// assert!(atlas.get_rect("player_idle").is_some());
/// assert_eq!(atlas.rect_count(), 1);
/// ```
pub struct TextureAtlas {
    /// 图集纹理宽度（像素）
    pub width: u32,
    /// 图集纹理高度（像素）
    pub height: u32,
    /// 命名子区域
    rects: std::collections::HashMap<String, AtlasRect>,
}

impl TextureAtlas {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            rects: std::collections::HashMap::new(),
        }
    }

    /// 添加命名矩形区域
    pub fn add_rect(&mut self, name: &str, rect: AtlasRect) {
        self.rects.insert(name.to_string(), rect);
    }

    /// 从像素坐标添加矩形区域
    pub fn add_rect_pixels(&mut self, name: &str, x: u32, y: u32, w: u32, h: u32) {
        let rect = AtlasRect::new(
            x as f32 / self.width as f32,
            y as f32 / self.height as f32,
            (x + w) as f32 / self.width as f32,
            (y + h) as f32 / self.height as f32,
        );
        self.rects.insert(name.to_string(), rect);
    }

    /// 获取命名矩形区域
    pub fn get_rect(&self, name: &str) -> Option<&AtlasRect> {
        self.rects.get(name)
    }

    /// 子区域数量
    pub fn rect_count(&self) -> usize {
        self.rects.len()
    }

    /// 生成均匀网格图集（cols × rows）
    pub fn from_grid(width: u32, height: u32, cols: u32, rows: u32) -> Self {
        let mut atlas = Self::new(width, height);
        let cell_w = 1.0 / cols as f32;
        let cell_h = 1.0 / rows as f32;
        for row in 0..rows {
            for col in 0..cols {
                let name = format!("{}_{}", col, row);
                atlas.add_rect(&name, AtlasRect::new(
                    col as f32 * cell_w,
                    row as f32 * cell_h,
                    (col + 1) as f32 * cell_w,
                    (row + 1) as f32 * cell_h,
                ));
            }
        }
        atlas
    }
}

/// 精灵组件
///
/// 附加到 ECS 实体上，表示一个 2D 精灵。
///
/// # 示例
///
/// ```rust
/// use anvilkit_render::renderer::sprite::Sprite;
/// use glam::Vec2;
///
/// let sprite = Sprite {
///     size: Vec2::new(64.0, 64.0),
///     color: [1.0, 1.0, 1.0],
///     atlas_rect: Default::default(),
///     flip_x: false,
///     flip_y: false,
///     z_order: 0.0,
/// };
/// ```
#[derive(Debug, Clone, Component)]
pub struct Sprite {
    /// 精灵大小（像素）
    pub size: Vec2,
    /// 着色颜色 (linear RGB)
    pub color: [f32; 3],
    /// 图集矩形区域
    pub atlas_rect: AtlasRect,
    /// 水平翻转
    pub flip_x: bool,
    /// 垂直翻转
    pub flip_y: bool,
    /// Z 排序值（越小越先绘制）
    pub z_order: f32,
}

impl Default for Sprite {
    fn default() -> Self {
        Self {
            size: Vec2::new(64.0, 64.0),
            color: [1.0, 1.0, 1.0],
            atlas_rect: AtlasRect::full(),
            flip_x: false,
            flip_y: false,
            z_order: 0.0,
        }
    }
}

/// 精灵批次命令
///
/// 收集同一纹理的精灵，按 z-order 排序后批量绘制。
#[derive(Default)]
pub struct SpriteBatch {
    /// 精灵顶点数据（6 个顶点 = 2 三角形 / 精灵）
    pub vertices: Vec<SpriteVertex>,
}

impl SpriteBatch {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
    }

    /// 添加一个精灵到批次
    pub fn add_sprite(&mut self, position: Vec3, sprite: &Sprite) {
        let half = sprite.size * 0.5;
        let r = &sprite.atlas_rect;

        let (u_min, u_max) = if sprite.flip_x { (r.u_max, r.u_min) } else { (r.u_min, r.u_max) };
        let (v_min, v_max) = if sprite.flip_y { (r.v_max, r.v_min) } else { (r.v_min, r.v_max) };

        let z = sprite.z_order;
        let c = sprite.color;

        // 两个三角形组成四边形 (CCW)
        let tl = SpriteVertex { position: [position.x - half.x, position.y + half.y, z], texcoord: [u_min, v_min], color: c };
        let bl = SpriteVertex { position: [position.x - half.x, position.y - half.y, z], texcoord: [u_min, v_max], color: c };
        let br = SpriteVertex { position: [position.x + half.x, position.y - half.y, z], texcoord: [u_max, v_max], color: c };
        let tr = SpriteVertex { position: [position.x + half.x, position.y + half.y, z], texcoord: [u_max, v_min], color: c };

        self.vertices.extend_from_slice(&[tl, bl, br, tl, br, tr]);
    }

    /// 精灵数量
    pub fn sprite_count(&self) -> usize {
        self.vertices.len() / 6
    }

    /// 按 z-order 排序（使用精灵第一个顶点的 z 值）
    pub fn sort_by_z_order(&mut self) {
        // 每 6 个顶点为一个精灵，按第一个顶点的 z 排序
        let sprite_count = self.sprite_count();
        if sprite_count <= 1 { return; }

        let mut sprites: Vec<[SpriteVertex; 6]> = Vec::with_capacity(sprite_count);
        for chunk in self.vertices.chunks_exact(6) {
            sprites.push([chunk[0], chunk[1], chunk[2], chunk[3], chunk[4], chunk[5]]);
        }

        sprites.sort_by(|a, b| a[0].position[2].partial_cmp(&b[0].position[2]).unwrap_or(std::cmp::Ordering::Equal));

        self.vertices.clear();
        for sprite in sprites {
            self.vertices.extend_from_slice(&sprite);
        }
    }
}

// ---------------------------------------------------------------------------
//  SpriteRenderer — GPU pipeline for 2D sprite rendering
// ---------------------------------------------------------------------------

const SPRITE_SHADER: &str = include_str!("../../../../shaders/sprite.wgsl");

/// 正交投影 uniform (64 bytes)
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct OrthoUniform {
    pub projection: [[f32; 4]; 4],
}

/// GPU 2D 精灵渲染器
pub struct SpriteRenderer {
    pub pipeline: wgpu::RenderPipeline,
    pub ortho_buffer: wgpu::Buffer,
    pub ortho_bind_group: wgpu::BindGroup,
    pub ortho_bind_group_layout: wgpu::BindGroupLayout,
    pub texture_bind_group_layout: wgpu::BindGroupLayout,
    /// Cached vertex buffer for per-frame reuse (grows as needed, never shrinks)
    cached_vb: Option<(wgpu::Buffer, u64)>,
}

impl SpriteRenderer {
    /// 创建精灵渲染器
    pub fn new(device: &super::RenderDevice, format: wgpu::TextureFormat) -> Self {
        let shader = device.device().create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Sprite Shader"),
            source: wgpu::ShaderSource::Wgsl(SPRITE_SHADER.into()),
        });

        // Ortho uniform bind group layout (group 0)
        let ortho_bgl = device.device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Sprite Ortho BGL"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        // Texture bind group layout (group 1)
        let tex_bgl = device.device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Sprite Texture BGL"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.device().create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Sprite Pipeline Layout"),
            bind_group_layouts: &[&ortho_bgl, &tex_bgl],
            push_constant_ranges: &[],
        });

        let pipeline = device.device().create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Sprite Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[SpriteVertex::layout()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        // Create ortho uniform buffer
        let initial = OrthoUniform {
            projection: glam::Mat4::IDENTITY.to_cols_array_2d(),
        };
        let ortho_buffer = device.device().create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sprite Ortho UB"),
            contents: bytemuck::bytes_of(&initial),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let ortho_bg = device.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Sprite Ortho BG"),
            layout: &ortho_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: ortho_buffer.as_entire_binding(),
            }],
        });

        Self {
            pipeline,
            ortho_buffer,
            ortho_bind_group: ortho_bg,
            ortho_bind_group_layout: ortho_bgl,
            texture_bind_group_layout: tex_bgl,
            cached_vb: None,
        }
    }

    /// 渲染精灵批次
    pub fn render(
        &mut self,
        device: &super::RenderDevice,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        batch: &SpriteBatch,
        texture_bind_group: &wgpu::BindGroup,
        screen_width: f32,
        screen_height: f32,
    ) {
        if batch.vertices.is_empty() {
            return;
        }

        // Update ortho projection
        let ortho = glam::Mat4::orthographic_lh(0.0, screen_width, screen_height, 0.0, -1.0, 1.0);
        let uniform = OrthoUniform {
            projection: ortho.to_cols_array_2d(),
        };
        device.queue().write_buffer(&self.ortho_buffer, 0, bytemuck::bytes_of(&uniform));

        // Upload vertices — reuse cached buffer if large enough, otherwise reallocate
        let data = bytemuck::cast_slice(&batch.vertices);
        let needed = data.len() as u64;
        let reuse = self.cached_vb.as_ref().map_or(false, |(_, cap)| *cap >= needed);
        if !reuse {
            self.cached_vb = Some((
                device.device().create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Sprite VB (cached)"),
                    size: needed,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }),
                needed,
            ));
        }
        let vb = &self.cached_vb.as_ref().unwrap().0;
        device.queue().write_buffer(vb, 0, data);

        {
            let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Sprite Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            rp.set_pipeline(&self.pipeline);
            rp.set_bind_group(0, &self.ortho_bind_group, &[]);
            rp.set_bind_group(1, texture_bind_group, &[]);
            rp.set_vertex_buffer(0, vb.slice(..));
            rp.draw(0..batch.vertices.len() as u32, 0..1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sprite_vertex_size() {
        assert_eq!(std::mem::size_of::<SpriteVertex>(), 32);
    }

    #[test]
    fn test_atlas_rect() {
        let full = AtlasRect::full();
        assert_eq!(full.width(), 1.0);
        assert_eq!(full.height(), 1.0);
    }

    #[test]
    fn test_texture_atlas_grid() {
        let atlas = TextureAtlas::from_grid(256, 256, 4, 4);
        assert_eq!(atlas.rect_count(), 16);
        let r = atlas.get_rect("0_0").unwrap();
        assert!((r.u_min - 0.0).abs() < 0.001);
        assert!((r.u_max - 0.25).abs() < 0.001);
    }

    #[test]
    fn test_sprite_batch() {
        let mut batch = SpriteBatch::new();
        let sprite = Sprite::default();

        batch.add_sprite(Vec3::new(100.0, 200.0, 0.0), &sprite);
        assert_eq!(batch.sprite_count(), 1);
        assert_eq!(batch.vertices.len(), 6);

        batch.add_sprite(Vec3::new(300.0, 200.0, 1.0), &sprite);
        assert_eq!(batch.sprite_count(), 2);
    }

    #[test]
    fn test_sprite_batch_z_sort() {
        let mut batch = SpriteBatch::new();
        let s1 = Sprite { z_order: 2.0, ..Default::default() };
        let s2 = Sprite { z_order: 0.0, ..Default::default() };
        let s3 = Sprite { z_order: 1.0, ..Default::default() };

        batch.add_sprite(Vec3::ZERO, &s1);
        batch.add_sprite(Vec3::ZERO, &s2);
        batch.add_sprite(Vec3::ZERO, &s3);

        batch.sort_by_z_order();

        // After sorting: z=0, z=1, z=2
        assert_eq!(batch.vertices[0].position[2], 0.0);
        assert_eq!(batch.vertices[6].position[2], 1.0);
        assert_eq!(batch.vertices[12].position[2], 2.0);
    }
}
