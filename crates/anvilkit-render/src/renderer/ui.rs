//! # UI 渲染
//!
//! GPU rendering for UI nodes. Data model types (UiNode, UiStyle, UiText, etc.)
//! live in `anvilkit-ui` and are re-exported here for backward compatibility.

use bytemuck::{Pod, Zeroable};
use super::shared::MatrixUniform;
use wgpu::util::DeviceExt;

// Re-export all UI data model types from anvilkit-ui for backward compatibility
pub use anvilkit_ui::{
    FlexDirection, Align, Val, UiStyle, UiText, UiNode, UiInteraction,
    UiLayoutEngine,
    UiEventKind, UiEvent, UiEvents, ui_hit_test, process_ui_interactions,
    Widget,
    UiTheme, UiFocus,
};

// ---------------------------------------------------------------------------
//  UiRenderer — GPU pipeline for UI rectangles
// ---------------------------------------------------------------------------

const UI_SHADER: &str = include_str!("../shaders/ui.wgsl");

/// UI 矩形 GPU 顶点
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct UiVertex {
    /// Normalized corner position within the quad (0..1).
    pub position: [f32; 2],
    /// Top-left corner of the rectangle in screen pixels.
    pub rect_min: [f32; 2],
    /// Width and height of the rectangle in screen pixels.
    pub rect_size: [f32; 2],
    /// Background fill color [R, G, B, A].
    pub color: [f32; 4],
    /// Border stroke color [R, G, B, A].
    pub border_color: [f32; 4],
    /// Packed parameters: [border_radius, border_width, 0, 0].
    pub params: [f32; 4],
}

impl UiVertex {
    fn layout() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: &[wgpu::VertexAttribute] = &[
            wgpu::VertexAttribute { offset: 0, shader_location: 0, format: wgpu::VertexFormat::Float32x2 },
            wgpu::VertexAttribute { offset: 8, shader_location: 1, format: wgpu::VertexFormat::Float32x2 },
            wgpu::VertexAttribute { offset: 16, shader_location: 2, format: wgpu::VertexFormat::Float32x2 },
            wgpu::VertexAttribute { offset: 24, shader_location: 3, format: wgpu::VertexFormat::Float32x4 },
            wgpu::VertexAttribute { offset: 40, shader_location: 4, format: wgpu::VertexFormat::Float32x4 },
            wgpu::VertexAttribute { offset: 56, shader_location: 5, format: wgpu::VertexFormat::Float32x4 },
        ];
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<UiVertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: ATTRIBUTES,
        }
    }
}

/// GPU UI 渲染器
pub struct UiRenderer {
    /// The wgpu render pipeline for UI rectangles.
    pub pipeline: wgpu::RenderPipeline,
    /// Uniform buffer holding the orthographic projection matrix.
    pub ortho_buffer: wgpu::Buffer,
    /// Bind group for the orthographic projection uniform.
    pub ortho_bind_group: wgpu::BindGroup,
    /// Cached vertex buffer for per-frame reuse.
    cached_vb: super::shared::CachedBuffer,
}

impl UiRenderer {
    /// Creates the UI render pipeline, uniform buffer, and bind group.
    pub fn new(device: &super::RenderDevice, format: wgpu::TextureFormat) -> Self {
        let shader = device.device().create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("UI Shader"),
            source: wgpu::ShaderSource::Wgsl(UI_SHADER.into()),
        });

        let ortho_bgl = device.device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("UI Ortho BGL"),
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

        let pipeline_layout = device.device().create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("UI Pipeline Layout"),
            bind_group_layouts: &[&ortho_bgl],
            push_constant_ranges: &[],
        });

        let pipeline = device.device().create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("UI Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[UiVertex::layout()],
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

        let initial = MatrixUniform::identity();
        let ortho_buffer = device.device().create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("UI Ortho UB"),
            contents: bytemuck::bytes_of(&initial),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let ortho_bg = device.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("UI Ortho BG"),
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
            cached_vb: super::shared::CachedBuffer::vertex("UI VB (cached)"),
        }
    }

    /// 从 computed_rect 列表渲染 UI 矩形
    pub fn render(
        &mut self,
        device: &super::RenderDevice,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        nodes: &[&UiNode],
        screen_width: f32,
        screen_height: f32,
    ) {
        if nodes.is_empty() {
            return;
        }

        // Update ortho
        let ortho = glam::Mat4::orthographic_lh(0.0, screen_width, screen_height, 0.0, -1.0, 1.0);
        let uniform = MatrixUniform::from_mat4(&ortho);
        device.queue().write_buffer(&self.ortho_buffer, 0, bytemuck::bytes_of(&uniform));

        // Build vertices
        let mut vertices = Vec::new();
        for node in nodes {
            if !node.visible || node.computed_rect[2] <= 0.0 || node.computed_rect[3] <= 0.0 {
                continue;
            }
            let [x, y, w, h] = node.computed_rect;
            let params = [node.corner_radius, node.border_width, 0.0, 0.0];

            // 6 vertices (2 triangles)
            let corners = [
                [0.0f32, 0.0], [1.0, 0.0], [1.0, 1.0],
                [0.0, 0.0], [1.0, 1.0], [0.0, 1.0],
            ];
            for corner in &corners {
                vertices.push(UiVertex {
                    position: *corner,
                    rect_min: [x, y],
                    rect_size: [w, h],
                    color: node.background_color,
                    border_color: node.border_color,
                    params,
                });
            }
        }

        if vertices.is_empty() {
            return;
        }

        let data = bytemuck::cast_slice(&vertices);
        let vb = self.cached_vb.ensure_and_write(device.device(), device.queue(), data);

        {
            let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("UI Pass"),
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
            rp.set_vertex_buffer(0, vb.slice(..));
            rp.draw(0..vertices.len() as u32, 0..1);
        }
    }
}
