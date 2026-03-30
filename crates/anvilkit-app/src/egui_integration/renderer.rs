//! Minimal egui renderer for wgpu 0.19.
//!
//! Replaces `egui-wgpu` to avoid wgpu version conflicts. Renders egui's
//! ClippedPrimitive output (textured triangles with scissor rects) directly
//! to a wgpu TextureView.

use std::collections::HashMap;

use bytemuck::{Pod, Zeroable};

/// Screen descriptor for egui rendering.
pub struct ScreenDescriptor {
    pub size_in_pixels: [u32; 2],
    pub pixels_per_point: f32,
}

impl ScreenDescriptor {
    fn size_in_points(&self) -> [f32; 2] {
        [
            self.size_in_pixels[0] as f32 / self.pixels_per_point,
            self.size_in_pixels[1] as f32 / self.pixels_per_point,
        ]
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct EguiUniforms {
    screen_size: [f32; 2],
    _pad: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct EguiVertex {
    pos: [f32; 2],
    uv: [f32; 2],
    color: [u8; 4],
}

/// Minimal egui GPU renderer using wgpu 0.19.
pub struct EguiRenderer {
    pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    textures: HashMap<egui::TextureId, (wgpu::Texture, wgpu::BindGroup)>,
    sampler: wgpu::Sampler,
    next_user_id: u64,
}

impl EguiRenderer {
    pub fn new(device: &wgpu::Device, output_format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("egui shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("egui_shader.wgsl").into()),
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("egui uniforms"),
            size: std::mem::size_of::<EguiUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("egui uniform bgl"),
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

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("egui uniform bg"),
            layout: &uniform_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let texture_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("egui texture bgl"),
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

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("egui pipeline layout"),
            bind_group_layouts: &[&uniform_bgl, &texture_bgl],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("egui pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<EguiVertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 8,
                            shader_location: 1,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Unorm8x4,
                            offset: 16,
                            shader_location: 2,
                        },
                    ],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: output_format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::OneMinusDstAlpha,
                            dst_factor: wgpu::BlendFactor::One,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
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

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("egui sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        Self {
            pipeline,
            uniform_buffer,
            uniform_bind_group,
            texture_bind_group_layout: texture_bgl,
            textures: HashMap::new(),
            sampler,
            next_user_id: 0,
        }
    }

    /// Update or create a texture from egui's TexturesDelta.
    pub fn update_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        id: egui::TextureId,
        delta: &egui::epaint::ImageDelta,
    ) {
        let pixels: Vec<u8> = match &delta.image {
            egui::ImageData::Color(img) => img
                .pixels
                .iter()
                .flat_map(|c| c.to_array())
                .collect(),
            egui::ImageData::Font(img) => img
                .srgba_pixels(None)
                .flat_map(|c| c.to_array())
                .collect(),
        };

        let [w, h] = delta.image.size().map(|x| x as u32);

        if let Some(pos) = delta.pos {
            // Partial update
            if let Some((_tex, _bg)) = self.textures.get(&id) {
                queue.write_texture(
                    wgpu::ImageCopyTexture {
                        texture: &self.textures[&id].0,
                        mip_level: 0,
                        origin: wgpu::Origin3d {
                            x: pos[0] as u32,
                            y: pos[1] as u32,
                            z: 0,
                        },
                        aspect: wgpu::TextureAspect::All,
                    },
                    &pixels,
                    wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(w * 4),
                        rows_per_image: None,
                    },
                    wgpu::Extent3d {
                        width: w,
                        height: h,
                        depth_or_array_layers: 1,
                    },
                );
            }
        } else {
            // Full texture create/replace
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("egui texture"),
                size: wgpu::Extent3d {
                    width: w,
                    height: h,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });

            queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &pixels,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(w * 4),
                    rows_per_image: None,
                },
                wgpu::Extent3d {
                    width: w,
                    height: h,
                    depth_or_array_layers: 1,
                },
            );

            let view = texture.create_view(&Default::default());
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("egui texture bg"),
                layout: &self.texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                ],
            });

            self.textures.insert(id, (texture, bind_group));
        }
    }

    /// Register a native wgpu TextureView as an egui texture.
    pub fn register_native_texture(
        &mut self,
        device: &wgpu::Device,
        view: &wgpu::TextureView,
    ) -> egui::TextureId {
        let id = egui::TextureId::User(self.next_user_id);
        self.next_user_id += 1;

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("egui user texture bg"),
            layout: &self.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });

        // We don't own the texture, so store a dummy. The bind group holds the view ref.
        // Since we can't create a dummy texture cheaply, we'll use a 1x1 placeholder.
        let dummy = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("egui user dummy"),
            size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        self.textures.insert(id, (dummy, bind_group));
        id
    }

    /// Render egui output.
    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        primitives: &[egui::ClippedPrimitive],
        textures_delta: &egui::TexturesDelta,
        screen: &ScreenDescriptor,
    ) {
        // Update textures
        for (id, delta) in &textures_delta.set {
            self.update_texture(device, queue, *id, delta);
        }

        // Update uniforms
        let size = screen.size_in_points();
        let uniforms = EguiUniforms {
            screen_size: size,
            _pad: [0.0; 2],
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));

        // Collect vertices and indices
        let mut all_vertices: Vec<EguiVertex> = Vec::new();
        let mut all_indices: Vec<u32> = Vec::new();

        struct DrawCall {
            texture_id: egui::TextureId,
            scissor: [u32; 4], // x, y, w, h
            index_start: u32,
            index_count: u32,
            vertex_offset: i32,
        }

        let mut draws: Vec<DrawCall> = Vec::new();
        let ppp = screen.pixels_per_point;
        let [sw, sh] = screen.size_in_pixels;

        for prim in primitives {
            let egui::ClippedPrimitive { clip_rect, primitive } = prim;

            if let egui::epaint::Primitive::Mesh(mesh) = primitive {
                if mesh.vertices.is_empty() || mesh.indices.is_empty() {
                    continue;
                }

                // Clip rect in pixels
                let x = (clip_rect.min.x * ppp).round().max(0.0) as u32;
                let y = (clip_rect.min.y * ppp).round().max(0.0) as u32;
                let w = ((clip_rect.max.x * ppp).round() as u32).saturating_sub(x).min(sw);
                let h = ((clip_rect.max.y * ppp).round() as u32).saturating_sub(y).min(sh);

                if w == 0 || h == 0 {
                    continue;
                }

                let vertex_offset = all_vertices.len() as i32;
                let index_start = all_indices.len() as u32;

                for v in &mesh.vertices {
                    all_vertices.push(EguiVertex {
                        pos: [v.pos.x, v.pos.y],
                        uv: [v.uv.x, v.uv.y],
                        color: v.color.to_array(),
                    });
                }
                all_indices.extend_from_slice(&mesh.indices);

                draws.push(DrawCall {
                    texture_id: mesh.texture_id,
                    scissor: [x, y, w, h],
                    index_start,
                    index_count: mesh.indices.len() as u32,
                    vertex_offset,
                });
            }
        }

        if draws.is_empty() {
            // Free textures even when nothing to draw
            for id in &textures_delta.free {
                self.textures.remove(id);
            }
            return;
        }

        // Create GPU buffers
        use wgpu::util::DeviceExt;
        let vb = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("egui vb"),
            contents: bytemuck::cast_slice(&all_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let ib = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("egui ib"),
            contents: bytemuck::cast_slice(&all_indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        // Render pass
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });

            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            pass.set_vertex_buffer(0, vb.slice(..));
            pass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint32);

            for draw in &draws {
                if let Some((_tex, bg)) = self.textures.get(&draw.texture_id) {
                    pass.set_bind_group(1, bg, &[]);
                    pass.set_scissor_rect(
                        draw.scissor[0],
                        draw.scissor[1],
                        draw.scissor[2],
                        draw.scissor[3],
                    );
                    pass.draw_indexed(
                        draw.index_start..(draw.index_start + draw.index_count),
                        draw.vertex_offset,
                        0..1,
                    );
                }
            }
        }

        // Free old textures
        for id in &textures_delta.free {
            self.textures.remove(id);
        }
    }
}
