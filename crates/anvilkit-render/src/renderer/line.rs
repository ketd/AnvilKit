//! # 线段渲染器
//!
//! 提供 3D 空间中的线段渲染功能，用于瞄准线、调试可视化和辅助线。
//!
//! ## 使用示例
//!
//! ```rust,no_run
//! use anvilkit_render::renderer::line::LineRenderer;
//! use glam::{Vec3, Mat4};
//!
//! // 创建 LineRenderer（需要 GPU device 和 surface format）
//! // let renderer = LineRenderer::new(&device, format);
//!
//! // 渲染线段
//! // let lines = vec![
//! //     (Vec3::ZERO, Vec3::new(5.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0)),
//! // ];
//! // renderer.render(&device, &mut encoder, &target_view, &lines, &view_proj);
//! ```

use glam::{Mat4, Vec3};
use wgpu::{
    BindGroup, Buffer, CommandEncoder, RenderPipeline, TextureView,
};

use crate::renderer::RenderDevice;
use crate::renderer::buffer::{ColorVertex, Vertex, create_uniform_buffer};
use crate::renderer::pipeline::RenderPipelineBuilder;

/// Line shader source
const LINE_SHADER: &str = include_str!("../shaders/line.wgsl");

/// 线段渲染器
///
/// 使用 `PrimitiveTopology::LineList` 渲染 3D 线段。
/// 支持单帧动态线段列表，每帧重新上传顶点数据。
pub struct LineRenderer {
    pipeline: RenderPipeline,
    scene_buffer: Buffer,
    scene_bind_group: BindGroup,
    /// Cached vertex buffer for per-frame reuse
    cached_vb: Option<(Buffer, u64)>,
}

/// 视图-投影 uniform 数据（64 字节，mat4x4<f32>）
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct LineSceneUniform {
    view_proj: [[f32; 4]; 4],
}

impl LineRenderer {
    /// 创建线段渲染器
    ///
    /// # 参数
    ///
    /// - `device`: GPU 渲染设备
    /// - `format`: 渲染目标纹理格式
    pub fn new(device: &RenderDevice, format: wgpu::TextureFormat) -> Self {
        let scene_uniform = LineSceneUniform {
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
        };
        let scene_buffer = create_uniform_buffer(
            device,
            "Line Scene Uniform",
            bytemuck::bytes_of(&scene_uniform),
        );

        let scene_bind_group_layout =
            device
                .device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Line Scene BGL"),
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
            label: Some("Line Scene BG"),
            layout: &scene_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: scene_buffer.as_entire_binding(),
            }],
        });

        // Reuse the bind group layout for the pipeline (identical structure)
        let pipeline = RenderPipelineBuilder::new()
            .with_vertex_shader(LINE_SHADER)
            .with_fragment_shader(LINE_SHADER)
            .with_format(format)
            .with_vertex_layouts(vec![ColorVertex::layout()])
            .with_bind_group_layouts(vec![scene_bind_group_layout])
            .with_topology(wgpu::PrimitiveTopology::LineList)
            .with_label("Line Pipeline")
            .build(device)
            .expect("创建 Line 管线失败")
            .into_pipeline();

        Self {
            pipeline,
            scene_buffer,
            scene_bind_group,
            cached_vb: None,
        }
    }

    /// 渲染线段列表
    ///
    /// # 参数
    ///
    /// - `device`: GPU 渲染设备
    /// - `encoder`: 命令编码器
    /// - `target`: 渲染目标纹理视图
    /// - `lines`: 线段列表，每项为 `(start, end, color)` 的 RGB 颜色
    /// - `view_proj`: 视图-投影矩阵
    pub fn render(
        &mut self,
        device: &RenderDevice,
        encoder: &mut CommandEncoder,
        target: &TextureView,
        lines: &[(Vec3, Vec3, Vec3)],
        view_proj: &Mat4,
    ) {
        if lines.is_empty() {
            return;
        }

        // Update view_proj uniform
        let uniform = LineSceneUniform {
            view_proj: view_proj.to_cols_array_2d(),
        };
        device
            .queue()
            .write_buffer(&self.scene_buffer, 0, bytemuck::bytes_of(&uniform));

        // Build vertex data
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

        // Reuse cached buffer if large enough
        let data: &[u8] = bytemuck::cast_slice(&vertices);
        let needed = data.len() as u64;
        let reuse = self.cached_vb.as_ref().map_or(false, |(_, cap)| *cap >= needed);
        if !reuse {
            self.cached_vb = Some((
                device.device().create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Line VB (cached)"),
                    size: needed,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }),
                needed,
            ));
        }
        let vertex_buffer = &self.cached_vb.as_ref().expect("buffer must be initialized above").0;
        device.queue().write_buffer(vertex_buffer, 0, data);

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Line Pass"),
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

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.scene_bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.draw(0..vertices.len() as u32, 0..1);
        }
    }
}
