//! # 调试渲染和性能分析工具
//!
//! 统一的调试模块，提供：
//! - [`DebugMode`] / [`RenderStats`] / [`DebugOverlay`]: 调试状态和统计
//! - [`DebugRenderer`]: 3D 调试图元渲染（线段、包围盒、球体、点）
//! - [`OverlayLineRenderer`]: 2D/3D 叠加线段渲染（十字准星、瞄准线等）

use bevy_ecs::prelude::*;
use anvilkit_describe::Describe;
use glam::{Vec3, Mat4};
use crate::renderer::RenderDevice;
use crate::renderer::buffer::{ColorVertex, Vertex, create_uniform_buffer};
use crate::renderer::pipeline::RenderPipelineBuilder;
use super::shared::{CachedBuffer, MatrixUniform};

// ---------------------------------------------------------------------------
//  Debug mode / stats / overlay (ECS types)
// ---------------------------------------------------------------------------

/// 调试渲染模式
///
/// # 示例
///
/// ```rust
/// use anvilkit_render::renderer::debug::DebugMode;
///
/// let mode = DebugMode::Wireframe;
/// assert!(!mode.is_normal());
/// assert!(DebugMode::None.is_normal());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Describe)]
/// Debug rendering visualization mode.
pub enum DebugMode {
    /// 正常渲染
    None,
    /// 线框叠加
    Wireframe,
    /// 仅漫反射
    DiffuseOnly,
    /// 仅镜面反射
    SpecularOnly,
}

impl DebugMode {
    /// 是否为正常渲染模式
    pub fn is_normal(&self) -> bool {
        matches!(self, DebugMode::None)
    }
}

impl Default for DebugMode {
    fn default() -> Self {
        DebugMode::None
    }
}

/// 每帧渲染统计
///
/// # 示例
///
/// ```rust
/// use anvilkit_render::renderer::debug::RenderStats;
///
/// let mut stats = RenderStats::new();
/// stats.record_draw_call(100);
/// stats.record_draw_call(200);
/// assert_eq!(stats.draw_calls, 2);
/// assert_eq!(stats.triangles, 300);
/// ```
#[derive(Debug, Clone, Resource, Describe)]
/// Per-frame rendering performance statistics.
pub struct RenderStats {
    /// 绘制调用次数
    #[describe(hint = "Number of GPU draw calls this frame", range = "0..100000", default = "0")]
    pub draw_calls: u32,
    /// 渲染的三角形总数
    #[describe(hint = "Total triangles rendered this frame", range = "0..10000000", default = "0")]
    pub triangles: u32,
    /// 渲染的顶点总数
    #[describe(hint = "Total vertices processed this frame", range = "0..30000000", default = "0")]
    pub vertices: u32,
    /// 活跃的光源数
    #[describe(hint = "Number of active lights in the scene", range = "0..256", default = "0")]
    pub active_lights: u32,
    /// 视锥体剔除掉的物体数
    #[describe(hint = "Objects culled by frustum this frame", range = "0..100000", default = "0")]
    pub culled_objects: u32,
    /// 可见物体数
    #[describe(hint = "Objects visible after culling", range = "0..100000", default = "0")]
    pub visible_objects: u32,
    /// 帧时间（毫秒）
    #[describe(hint = "Frame duration in milliseconds", range = "0.0..1000.0", default = "0.0")]
    pub frame_time_ms: f32,
    /// FPS（基于帧时间计算）
    #[describe(hint = "Frames per second (1/frame_time)", range = "0.0..10000.0", default = "0.0")]
    pub fps: f32,
    /// GPU 内存使用估计（字节）
    #[describe(hint = "Estimated GPU memory usage in bytes", default = "0")]
    pub gpu_memory_bytes: u64,
}

impl RenderStats {
    /// Creates a new `RenderStats` with all counters at zero.
    pub fn new() -> Self {
        Self {
            draw_calls: 0,
            triangles: 0,
            vertices: 0,
            active_lights: 0,
            culled_objects: 0,
            visible_objects: 0,
            frame_time_ms: 0.0,
            fps: 0.0,
            gpu_memory_bytes: 0,
        }
    }

    /// 记录一次绘制调用
    pub fn record_draw_call(&mut self, triangle_count: u32) {
        self.draw_calls += 1;
        self.triangles += triangle_count;
    }

    /// 更新帧时间
    pub fn update_frame_time(&mut self, dt_seconds: f32) {
        self.frame_time_ms = dt_seconds * 1000.0;
        self.fps = if dt_seconds > 0.0 { 1.0 / dt_seconds } else { 0.0 };
    }

    /// 帧开始时重置计数器
    pub fn reset_frame(&mut self) {
        self.draw_calls = 0;
        self.triangles = 0;
        self.vertices = 0;
        self.culled_objects = 0;
        self.visible_objects = 0;
    }

    /// 格式化为摘要字符串
    pub fn summary(&self) -> String {
        format!(
            "FPS: {:.0} | {:.1}ms | Draw: {} | Tri: {} | Vis: {}/{}",
            self.fps, self.frame_time_ms,
            self.draw_calls, self.triangles,
            self.visible_objects, self.visible_objects + self.culled_objects,
        )
    }
}

impl Default for RenderStats {
    fn default() -> Self {
        Self::new()
    }
}

/// 调试叠加层配置
///
/// # 示例
///
/// ```rust
/// use anvilkit_render::renderer::debug::DebugOverlay;
///
/// let overlay = DebugOverlay::default();
/// assert!(!overlay.show_stats);
/// ```
#[derive(Debug, Clone, Resource, Describe)]
/// Debug overlay configuration.
pub struct DebugOverlay {
    /// 是否显示统计信息
    #[describe(hint = "Show performance stats overlay", default = "false")]
    pub show_stats: bool,
    /// 当前调试模式
    pub debug_mode: DebugMode,
}

impl Default for DebugOverlay {
    fn default() -> Self {
        Self {
            show_stats: false,
            debug_mode: DebugMode::None,
        }
    }
}

// ---------------------------------------------------------------------------
//  DebugRenderer — 3D debug primitives (renders to HDR with depth)
// ---------------------------------------------------------------------------

const DEBUG_LINES_SHADER: &str = include_str!("../shaders/debug_lines.wgsl");

/// Maximum number of debug vertices per frame.
pub const MAX_DEBUG_VERTICES: usize = 65536;

/// Debug 顶点
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DebugVertex {
    /// World-space position.
    pub position: [f32; 3],
    /// RGBA color.
    pub color: [f32; 4],
}

impl DebugVertex {
    /// Vertex buffer layout for the debug vertex.
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 12,
                    shader_location: 1,
                },
            ],
        }
    }
}

/// Debug draw command.
#[derive(Debug, Clone)]
pub enum DebugDrawCommand {
    /// Draw a line segment.
    Line {
        /// Start point.
        start: Vec3,
        /// End point.
        end: Vec3,
        /// RGBA color.
        color: [f32; 4],
    },
    /// Draw a wireframe sphere (3 axis circles).
    Sphere {
        /// Center.
        center: Vec3,
        /// Radius.
        radius: f32,
        /// RGBA color.
        color: [f32; 4],
        /// Segments per circle.
        segments: u32,
    },
    /// Draw a wireframe box (12 edges).
    Box {
        /// Center.
        center: Vec3,
        /// Half-extents.
        half_extents: Vec3,
        /// RGBA color.
        color: [f32; 4],
    },
    /// Draw a point (rendered as small cross).
    Point {
        /// Position.
        position: Vec3,
        /// RGBA color.
        color: [f32; 4],
        /// Size (half-length of cross arms).
        size: f32,
    },
}

/// Debug 渲染器 — 3D scene debug primitives
pub struct DebugRenderer {
    /// Accumulated draw commands.
    pub commands: Vec<DebugDrawCommand>,
    /// Built line vertices (from commands).
    line_vertices: Vec<DebugVertex>,
    /// GPU vertex buffer (dynamic).
    vertex_buffer: wgpu::Buffer,
    /// Render pipeline (line list).
    pipeline: wgpu::RenderPipeline,
    /// View-projection uniform buffer.
    uniform_buffer: wgpu::Buffer,
    /// Bind group layout.
    bgl: wgpu::BindGroupLayout,
}

impl DebugRenderer {
    /// Create a new debug renderer.
    pub fn new(device: &RenderDevice) -> Self {
        let vertex_buffer = device.device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("Debug Vertex Buffer"),
            size: (MAX_DEBUG_VERTICES * std::mem::size_of::<DebugVertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_buffer = device.device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("Debug Uniform"),
            size: 64, // mat4x4<f32>
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bgl = device.device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Debug BGL"),
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

        let shader = device.device().create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Debug Lines Shader"),
            source: wgpu::ShaderSource::Wgsl(DEBUG_LINES_SHADER.into()),
        });

        let pl = device.device().create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Debug PL"),
            bind_group_layouts: &[&bgl],
            push_constant_ranges: &[],
        });

        let pipeline = device.device().create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Debug Lines Pipeline"),
            layout: Some(&pl),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[DebugVertex::layout()],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: crate::renderer::buffer::DEPTH_FORMAT,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: crate::renderer::buffer::HDR_FORMAT,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });

        Self {
            commands: Vec::new(),
            line_vertices: Vec::new(),
            vertex_buffer,
            pipeline,
            uniform_buffer,
            bgl,
        }
    }

    /// Draw a line segment.
    pub fn draw_line(&mut self, start: Vec3, end: Vec3, color: [f32; 4]) {
        self.commands.push(DebugDrawCommand::Line { start, end, color });
    }

    /// Draw a wireframe box.
    pub fn draw_box(&mut self, center: Vec3, half_extents: Vec3, color: [f32; 4]) {
        self.commands.push(DebugDrawCommand::Box { center, half_extents, color });
    }

    /// Draw a wireframe sphere.
    pub fn draw_sphere(&mut self, center: Vec3, radius: f32, color: [f32; 4], segments: u32) {
        self.commands.push(DebugDrawCommand::Sphere { center, radius, color, segments });
    }

    /// Draw a point (as small cross).
    pub fn draw_point(&mut self, position: Vec3, color: [f32; 4], size: f32) {
        self.commands.push(DebugDrawCommand::Point { position, color, size });
    }

    /// Convert commands to line vertices.
    pub fn prepare(&mut self) {
        self.line_vertices.clear();

        for cmd in &self.commands {
            match cmd {
                DebugDrawCommand::Line { start, end, color } => {
                    self.line_vertices.push(DebugVertex { position: start.to_array(), color: *color });
                    self.line_vertices.push(DebugVertex { position: end.to_array(), color: *color });
                }
                DebugDrawCommand::Box { center, half_extents, color } => {
                    let c = *center;
                    let h = *half_extents;
                    let corners = [
                        c + Vec3::new(-h.x, -h.y, -h.z),
                        c + Vec3::new( h.x, -h.y, -h.z),
                        c + Vec3::new( h.x,  h.y, -h.z),
                        c + Vec3::new(-h.x,  h.y, -h.z),
                        c + Vec3::new(-h.x, -h.y,  h.z),
                        c + Vec3::new( h.x, -h.y,  h.z),
                        c + Vec3::new( h.x,  h.y,  h.z),
                        c + Vec3::new(-h.x,  h.y,  h.z),
                    ];
                    let edges: [(usize, usize); 12] = [
                        (0,1),(1,2),(2,3),(3,0),
                        (4,5),(5,6),(6,7),(7,4),
                        (0,4),(1,5),(2,6),(3,7),
                    ];
                    for (a, b) in edges {
                        self.line_vertices.push(DebugVertex { position: corners[a].to_array(), color: *color });
                        self.line_vertices.push(DebugVertex { position: corners[b].to_array(), color: *color });
                    }
                }
                DebugDrawCommand::Sphere { center, radius, color, segments } => {
                    let segs = (*segments).max(8) as usize;
                    for axis in 0..3 {
                        for i in 0..segs {
                            let a0 = std::f32::consts::TAU * (i as f32) / (segs as f32);
                            let a1 = std::f32::consts::TAU * ((i + 1) as f32) / (segs as f32);
                            let (p0, p1) = match axis {
                                0 => (
                                    *center + Vec3::new(0.0, a0.cos() * radius, a0.sin() * radius),
                                    *center + Vec3::new(0.0, a1.cos() * radius, a1.sin() * radius),
                                ),
                                1 => (
                                    *center + Vec3::new(a0.cos() * radius, 0.0, a0.sin() * radius),
                                    *center + Vec3::new(a1.cos() * radius, 0.0, a1.sin() * radius),
                                ),
                                _ => (
                                    *center + Vec3::new(a0.cos() * radius, a0.sin() * radius, 0.0),
                                    *center + Vec3::new(a1.cos() * radius, a1.sin() * radius, 0.0),
                                ),
                            };
                            self.line_vertices.push(DebugVertex { position: p0.to_array(), color: *color });
                            self.line_vertices.push(DebugVertex { position: p1.to_array(), color: *color });
                        }
                    }
                }
                DebugDrawCommand::Point { position, color, size } => {
                    let s = *size;
                    let p = *position;
                    for axis in 0..3 {
                        let mut offset = Vec3::ZERO;
                        match axis {
                            0 => offset.x = s,
                            1 => offset.y = s,
                            _ => offset.z = s,
                        }
                        self.line_vertices.push(DebugVertex { position: (p - offset).to_array(), color: *color });
                        self.line_vertices.push(DebugVertex { position: (p + offset).to_array(), color: *color });
                    }
                }
            }
        }

        if self.line_vertices.len() > MAX_DEBUG_VERTICES {
            self.line_vertices.truncate(MAX_DEBUG_VERTICES);
        }
    }

    /// Upload vertices and render debug lines into an HDR target with depth.
    pub fn render(
        &self,
        device: &RenderDevice,
        encoder: &mut wgpu::CommandEncoder,
        target_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        view_proj: &[[f32; 4]; 4],
    ) {
        if self.line_vertices.is_empty() {
            return;
        }

        device.queue().write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(view_proj),
        );

        device.queue().write_buffer(
            &self.vertex_buffer,
            0,
            bytemuck::cast_slice(&self.line_vertices),
        );

        let bg = device.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Debug BG"),
            layout: &self.bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: self.uniform_buffer.as_entire_binding(),
            }],
        });

        let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Debug Lines Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        rp.set_pipeline(&self.pipeline);
        rp.set_bind_group(0, &bg, &[]);
        rp.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        rp.draw(0..self.line_vertices.len() as u32, 0..1);
    }

    /// Clear all commands and vertices.
    pub fn clear(&mut self) {
        self.commands.clear();
        self.line_vertices.clear();
    }

    /// Return the number of accumulated draw commands.
    pub fn command_count(&self) -> usize {
        self.commands.len()
    }

    /// Return the number of line vertices after `prepare()`.
    pub fn vertex_count(&self) -> usize {
        self.line_vertices.len()
    }
}

// ---------------------------------------------------------------------------
//  OverlayLineRenderer — 2D/3D overlay lines (replaces LineRenderer)
// ---------------------------------------------------------------------------

/// Line shader source
const LINE_SHADER: &str = include_str!("../shaders/line.wgsl");

/// Overlay line renderer — renders colored line segments without depth testing.
///
/// Used for crosshairs, aim lines, block highlight wireframes, and other
/// overlay visuals that should always be visible on top of the scene.
///
/// This replaces the former standalone `LineRenderer`.
pub struct OverlayLineRenderer {
    pipeline: wgpu::RenderPipeline,
    scene_buffer: wgpu::Buffer,
    scene_bind_group: wgpu::BindGroup,
    cached_vb: CachedBuffer,
}

impl OverlayLineRenderer {
    /// Create a new overlay line renderer.
    ///
    /// - `device`: GPU render device
    /// - `format`: Swapchain texture format (e.g., Bgra8UnormSrgb)
    pub fn new(device: &RenderDevice, format: wgpu::TextureFormat) -> Self {
        let scene_uniform = MatrixUniform::identity();
        let scene_buffer = create_uniform_buffer(
            device,
            "Overlay Line Uniform",
            bytemuck::bytes_of(&scene_uniform),
        );

        let scene_bgl = device.device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Overlay Line BGL"),
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

        let scene_bind_group = device.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Overlay Line BG"),
            layout: &scene_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: scene_buffer.as_entire_binding(),
            }],
        });

        let pipeline = RenderPipelineBuilder::new()
            .with_vertex_shader(LINE_SHADER)
            .with_fragment_shader(LINE_SHADER)
            .with_format(format)
            .with_vertex_layouts(vec![ColorVertex::layout()])
            .with_bind_group_layouts(vec![scene_bgl])
            .with_topology(wgpu::PrimitiveTopology::LineList)
            .with_label("Overlay Line Pipeline")
            .build(device)
            .expect("Failed to create overlay line pipeline")
            .into_pipeline();

        Self {
            pipeline,
            scene_buffer,
            scene_bind_group,
            cached_vb: CachedBuffer::vertex("Overlay Line VB (cached)"),
        }
    }

    /// Render line segments onto the target (no depth testing).
    ///
    /// Each line is `(start, end, color_rgb)` where color is a Vec3 RGB.
    pub fn render(
        &mut self,
        device: &RenderDevice,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        lines: &[(Vec3, Vec3, Vec3)],
        view_proj: &Mat4,
    ) {
        if lines.is_empty() {
            return;
        }

        let uniform = MatrixUniform::from_mat4(view_proj);
        device.queue().write_buffer(&self.scene_buffer, 0, bytemuck::bytes_of(&uniform));

        let mut vertices = Vec::with_capacity(lines.len() * 2);
        for (start, end, color) in lines {
            vertices.push(ColorVertex {
                position: [start.x, start.y, start.z],
                color: [color.x, color.y, color.z],
            });
            vertices.push(ColorVertex {
                position: [end.x, end.y, end.z],
                color: [color.x, color.y, color.z],
            });
        }

        let data: &[u8] = bytemuck::cast_slice(&vertices);
        let vertex_buffer = self.cached_vb.ensure_and_write(device.device(), device.queue(), data);

        {
            let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Overlay Line Pass"),
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
            rp.set_bind_group(0, &self.scene_bind_group, &[]);
            rp.set_vertex_buffer(0, vertex_buffer.slice(..));
            rp.draw(0..vertices.len() as u32, 0..1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_mode() {
        assert!(DebugMode::None.is_normal());
        assert!(!DebugMode::Wireframe.is_normal());
        assert!(!DebugMode::DiffuseOnly.is_normal());
    }

    #[test]
    fn test_render_stats() {
        let mut stats = RenderStats::new();
        stats.record_draw_call(100);
        stats.record_draw_call(50);
        assert_eq!(stats.draw_calls, 2);
        assert_eq!(stats.triangles, 150);

        stats.update_frame_time(1.0 / 60.0);
        assert!((stats.fps - 60.0).abs() < 1.0);

        let summary = stats.summary();
        assert!(summary.contains("FPS:"));
        assert!(summary.contains("Draw: 2"));
    }

    #[test]
    fn test_render_stats_reset() {
        let mut stats = RenderStats::new();
        stats.record_draw_call(100);
        stats.visible_objects = 5;
        stats.culled_objects = 3;

        stats.reset_frame();
        assert_eq!(stats.draw_calls, 0);
        assert_eq!(stats.triangles, 0);
        assert_eq!(stats.visible_objects, 0);
    }

    #[test]
    fn test_debug_overlay_default() {
        let overlay = DebugOverlay::default();
        assert!(!overlay.show_stats);
        assert!(overlay.debug_mode.is_normal());
    }

    #[test]
    fn test_debug_vertex_layout() {
        let layout = DebugVertex::layout();
        assert_eq!(layout.array_stride, std::mem::size_of::<DebugVertex>() as u64);
        assert_eq!(layout.attributes.len(), 2);
    }

    #[test]
    fn test_debug_draw_commands() {
        let mut commands: Vec<DebugDrawCommand> = Vec::new();
        commands.push(DebugDrawCommand::Line {
            start: Vec3::ZERO, end: Vec3::X, color: [1.0, 0.0, 0.0, 1.0],
        });
        commands.push(DebugDrawCommand::Box {
            center: Vec3::ZERO, half_extents: Vec3::ONE, color: [0.0, 1.0, 0.0, 1.0],
        });
        commands.push(DebugDrawCommand::Sphere {
            center: Vec3::new(5.0, 0.0, 0.0), radius: 1.0, color: [0.0, 0.0, 1.0, 1.0], segments: 16,
        });
        commands.push(DebugDrawCommand::Point {
            position: Vec3::Y, color: [1.0, 1.0, 1.0, 1.0], size: 0.1,
        });
        assert_eq!(commands.len(), 4);
    }

    #[test]
    fn test_debug_vertex_pod() {
        let vertices = vec![
            DebugVertex { position: [1.0, 2.0, 3.0], color: [1.0, 0.0, 0.0, 1.0] },
            DebugVertex { position: [4.0, 5.0, 6.0], color: [0.0, 1.0, 0.0, 1.0] },
        ];
        let bytes: &[u8] = bytemuck::cast_slice(&vertices);
        assert_eq!(bytes.len(), 2 * std::mem::size_of::<DebugVertex>());
    }
}
