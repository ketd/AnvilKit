//! # Debug 渲染器
//!
//! 线框调试绘制：线段、包围盒、球体、点。
//! 使用 Line List 拓扑一次性绘制所有调试图元。

use crate::renderer::RenderDevice;
use glam::Vec3;

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

/// Debug 渲染器
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
                    // 8 corners
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
                    // 12 edges
                    let edges: [(usize, usize); 12] = [
                        (0,1),(1,2),(2,3),(3,0), // front
                        (4,5),(5,6),(6,7),(7,4), // back
                        (0,4),(1,5),(2,6),(3,7), // connecting
                    ];
                    for (a, b) in edges {
                        self.line_vertices.push(DebugVertex { position: corners[a].to_array(), color: *color });
                        self.line_vertices.push(DebugVertex { position: corners[b].to_array(), color: *color });
                    }
                }
                DebugDrawCommand::Sphere { center, radius, color, segments } => {
                    let segs = (*segments).max(8) as usize;
                    // 3 axis-aligned circles
                    for axis in 0..3 {
                        for i in 0..segs {
                            let a0 = std::f32::consts::TAU * (i as f32) / (segs as f32);
                            let a1 = std::f32::consts::TAU * ((i + 1) as f32) / (segs as f32);
                            let (p0, p1) = match axis {
                                0 => ( // YZ circle
                                    *center + Vec3::new(0.0, a0.cos() * radius, a0.sin() * radius),
                                    *center + Vec3::new(0.0, a1.cos() * radius, a1.sin() * radius),
                                ),
                                1 => ( // XZ circle
                                    *center + Vec3::new(a0.cos() * radius, 0.0, a0.sin() * radius),
                                    *center + Vec3::new(a1.cos() * radius, 0.0, a1.sin() * radius),
                                ),
                                _ => ( // XY circle
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
                    // 3-axis cross
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

        // Clamp to max
        if self.line_vertices.len() > MAX_DEBUG_VERTICES {
            self.line_vertices.truncate(MAX_DEBUG_VERTICES);
        }
    }

    /// Upload vertices and render debug lines.
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

        // Upload view_proj
        device.queue().write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(view_proj),
        );

        // Upload vertices
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_vertex_layout() {
        let layout = DebugVertex::layout();
        assert_eq!(
            layout.array_stride,
            std::mem::size_of::<DebugVertex>() as u64
        );
        assert_eq!(layout.attributes.len(), 2);
    }

    #[test]
    fn test_debug_draw_commands() {
        // We cannot construct a full DebugRenderer without a GPU device,
        // so test the command types and manual vertex building.
        let mut commands: Vec<DebugDrawCommand> = Vec::new();

        // Draw a line
        commands.push(DebugDrawCommand::Line {
            start: Vec3::ZERO,
            end: Vec3::X,
            color: [1.0, 0.0, 0.0, 1.0],
        });

        // Draw a box
        commands.push(DebugDrawCommand::Box {
            center: Vec3::ZERO,
            half_extents: Vec3::ONE,
            color: [0.0, 1.0, 0.0, 1.0],
        });

        // Draw a sphere
        commands.push(DebugDrawCommand::Sphere {
            center: Vec3::new(5.0, 0.0, 0.0),
            radius: 1.0,
            color: [0.0, 0.0, 1.0, 1.0],
            segments: 16,
        });

        // Draw a point
        commands.push(DebugDrawCommand::Point {
            position: Vec3::Y,
            color: [1.0, 1.0, 1.0, 1.0],
            size: 0.1,
        });

        assert_eq!(commands.len(), 4);

        // Verify that each command variant stores the right data
        match &commands[0] {
            DebugDrawCommand::Line { start, end, .. } => {
                assert_eq!(*start, Vec3::ZERO);
                assert_eq!(*end, Vec3::X);
            }
            _ => panic!("Expected Line command"),
        }
        match &commands[1] {
            DebugDrawCommand::Box { center, half_extents, .. } => {
                assert_eq!(*center, Vec3::ZERO);
                assert_eq!(*half_extents, Vec3::ONE);
            }
            _ => panic!("Expected Box command"),
        }
    }

    #[test]
    fn test_debug_vertex_pod() {
        // Verify that DebugVertex is Pod/Zeroable (cast_slice should work)
        let vertices = vec![
            DebugVertex {
                position: [1.0, 2.0, 3.0],
                color: [1.0, 0.0, 0.0, 1.0],
            },
            DebugVertex {
                position: [4.0, 5.0, 6.0],
                color: [0.0, 1.0, 0.0, 1.0],
            },
        ];
        let bytes: &[u8] = bytemuck::cast_slice(&vertices);
        assert_eq!(
            bytes.len(),
            2 * std::mem::size_of::<DebugVertex>()
        );
    }
}
