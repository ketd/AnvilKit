//! # Depth of Field 后处理
//!
//! 基于 Circle of Confusion 的散焦模糊效果。
//! 三 pass 流程：CoC 计算 → 圆盘模糊 → 合成。

use bevy_ecs::prelude::*;
use anvilkit_describe::Describe;
use crate::renderer::RenderDevice;
use crate::renderer::buffer::HDR_FORMAT;

const DOF_SHADER: &str = include_str!("../shaders/dof.wgsl");

/// DOF 配置参数
#[derive(Debug, Clone, Resource, Describe)]
/// Depth-of-field post-process settings.
pub struct DofSettings {
    /// Whether DOF is enabled.
    #[describe(hint = "Enable depth of field", default = "false")]
    pub enabled: bool,
    /// Distance to the focus plane (world units).
    #[describe(hint = "Focus plane distance", range = "0.1..1000.0", default = "10.0")]
    pub focus_distance: f32,
    /// Range around focus_distance that is in sharp focus.
    #[describe(hint = "Sharp focus range around focus distance", range = "0.1..100.0", default = "5.0")]
    pub focus_range: f32,
    /// Maximum blur radius in pixels.
    #[describe(hint = "Max bokeh blur radius in pixels", range = "0.0..16.0", default = "4.0")]
    pub bokeh_radius: f32,
}

impl Default for DofSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            focus_distance: 10.0,
            focus_range: 5.0,
            bokeh_radius: 4.0,
        }
    }
}

/// DOF GPU uniform
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DofUniform {
    /// Focus distance.
    pub focus_distance: f32,
    /// Focus range.
    pub focus_range: f32,
    /// Bokeh radius.
    pub bokeh_radius: f32,
    /// Padding.
    pub _pad: f32,
}

/// DOF GPU 资源
pub struct DofResources {
    /// CoC texture (R16Float, full-res).
    pub coc_texture: wgpu::Texture,
    /// CoC texture view.
    pub coc_view: wgpu::TextureView,
    /// Blurred texture (Rgba16Float, half-res).
    pub blurred_texture: wgpu::Texture,
    /// Blurred texture view.
    pub blurred_view: wgpu::TextureView,
    /// CoC render pipeline.
    pub coc_pipeline: wgpu::RenderPipeline,
    /// Blur render pipeline.
    pub blur_pipeline: wgpu::RenderPipeline,
    /// Composite render pipeline.
    pub composite_pipeline: wgpu::RenderPipeline,
    /// Uniform buffer.
    pub uniform_buffer: wgpu::Buffer,
    /// Linear sampler.
    pub sampler: wgpu::Sampler,
    /// Non-filtering sampler for depth.
    pub depth_sampler: wgpu::Sampler,
    /// CoC pass bind group layout.
    pub coc_bgl: wgpu::BindGroupLayout,
    /// Blur pass bind group layout.
    pub blur_bgl: wgpu::BindGroupLayout,
    /// Composite pass bind group layout.
    pub composite_bgl: wgpu::BindGroupLayout,
}

impl DofResources {
    /// Create DOF GPU resources.
    pub fn new(device: &RenderDevice, width: u32, height: u32) -> Self {
        let (coc_texture, coc_view) = Self::create_coc_texture(device, width, height);
        let half_w = (width / 2).max(1);
        let half_h = (height / 2).max(1);
        let (blurred_texture, blurred_view) = Self::create_blurred_texture(device, half_w, half_h);

        let sampler = device.device().create_sampler(&wgpu::SamplerDescriptor {
            label: Some("DOF Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let depth_sampler = device.device().create_sampler(&wgpu::SamplerDescriptor {
            label: Some("DOF Depth Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let uniform_buffer = device.device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("DOF Uniform"),
            size: std::mem::size_of::<DofUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let shader = device.device().create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("DOF Shader"),
            source: wgpu::ShaderSource::Wgsl(DOF_SHADER.into()),
        });

        // --- CoC BGL: src (HDR), depth, sampler, uniform ---
        let coc_bgl = device.device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("DOF CoC BGL"),
            entries: &[
                bgl_texture_2d(0),      // src_texture (unused in coc, but required by shader)
                bgl_depth_texture(1),    // depth_texture
                bgl_sampler_non_filtering(2), // sampler
                bgl_uniform(3),          // params
            ],
        });

        // --- Blur BGL: src, depth (unused), sampler, uniform, coc ---
        let blur_bgl = device.device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("DOF Blur BGL"),
            entries: &[
                bgl_texture_2d(0),       // src_texture
                bgl_depth_texture(1),    // depth_texture (unused)
                bgl_sampler_filtering(2),// sampler
                bgl_uniform(3),          // params
                bgl_texture_2d_at(4),    // coc_texture
            ],
        });

        // --- Composite BGL: src, depth (unused), sampler, uniform, coc, blurred ---
        let composite_bgl = device.device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("DOF Composite BGL"),
            entries: &[
                bgl_texture_2d(0),       // src_texture (sharp)
                bgl_depth_texture(1),    // depth (unused)
                bgl_sampler_filtering(2),// sampler
                bgl_uniform(3),          // params
                bgl_texture_2d_at(4),    // coc_texture
                bgl_texture_2d_at(5),    // blurred_texture
            ],
        });

        let coc_pipeline = Self::build_pipeline(device, &shader, "vs_main", "coc_fs", &coc_bgl, wgpu::TextureFormat::R16Float, "DOF CoC", None);
        let blur_pipeline = Self::build_pipeline(device, &shader, "vs_main", "blur_fs", &blur_bgl, HDR_FORMAT, "DOF Blur", None);
        let composite_pipeline = Self::build_pipeline(device, &shader, "vs_main", "composite_fs", &composite_bgl, HDR_FORMAT, "DOF Composite", None);

        Self {
            coc_texture,
            coc_view,
            blurred_texture,
            blurred_view,
            coc_pipeline,
            blur_pipeline,
            composite_pipeline,
            uniform_buffer,
            sampler,
            depth_sampler,
            coc_bgl,
            blur_bgl,
            composite_bgl,
        }
    }

    fn build_pipeline(
        device: &RenderDevice,
        shader: &wgpu::ShaderModule,
        vs_entry: &str,
        fs_entry: &str,
        bgl: &wgpu::BindGroupLayout,
        format: wgpu::TextureFormat,
        label: &str,
        blend: Option<wgpu::BlendState>,
    ) -> wgpu::RenderPipeline {
        let pl = device.device().create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(label),
            bind_group_layouts: &[bgl],
            push_constant_ranges: &[],
        });
        device.device().create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(label),
            layout: Some(&pl),
            vertex: wgpu::VertexState { module: shader, entry_point: vs_entry, buffers: &[] },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: fs_entry,
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        })
    }

    fn create_coc_texture(device: &RenderDevice, w: u32, h: u32) -> (wgpu::Texture, wgpu::TextureView) {
        let tex = device.device().create_texture(&wgpu::TextureDescriptor {
            label: Some("DOF CoC Texture"),
            size: wgpu::Extent3d { width: w.max(1), height: h.max(1), depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R16Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
        (tex, view)
    }

    fn create_blurred_texture(device: &RenderDevice, w: u32, h: u32) -> (wgpu::Texture, wgpu::TextureView) {
        let tex = device.device().create_texture(&wgpu::TextureDescriptor {
            label: Some("DOF Blurred Texture"),
            size: wgpu::Extent3d { width: w.max(1), height: h.max(1), depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: HDR_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
        (tex, view)
    }

    /// Rebuild textures on resize.
    pub fn resize(&mut self, device: &RenderDevice, width: u32, height: u32) {
        let (coc_texture, coc_view) = Self::create_coc_texture(device, width, height);
        self.coc_texture = coc_texture;
        self.coc_view = coc_view;
        let half_w = (width / 2).max(1);
        let half_h = (height / 2).max(1);
        let (blurred_texture, blurred_view) = Self::create_blurred_texture(device, half_w, half_h);
        self.blurred_texture = blurred_texture;
        self.blurred_view = blurred_view;
    }

    /// Execute DOF passes.
    pub fn execute(
        &self,
        device: &RenderDevice,
        encoder: &mut wgpu::CommandEncoder,
        hdr_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        settings: &DofSettings,
    ) {
        if !settings.enabled {
            return;
        }

        let uniform = DofUniform {
            focus_distance: settings.focus_distance,
            focus_range: settings.focus_range,
            bokeh_radius: settings.bokeh_radius,
            _pad: 0.0,
        };
        device.queue().write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniform));

        // Pass 1: CoC
        {
            let bg = device.device().create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("DOF CoC BG"),
                layout: &self.coc_bgl,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(hdr_view) },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(depth_view) },
                    wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&self.depth_sampler) },
                    wgpu::BindGroupEntry { binding: 3, resource: self.uniform_buffer.as_entire_binding() },
                ],
            });
            let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("DOF CoC Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.coc_view,
                    resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            rp.set_pipeline(&self.coc_pipeline);
            rp.set_bind_group(0, &bg, &[]);
            rp.draw(0..3, 0..1);
        }

        // Pass 2: Blur (half-res)
        {
            let bg = device.device().create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("DOF Blur BG"),
                layout: &self.blur_bgl,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(hdr_view) },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(depth_view) },
                    wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&self.sampler) },
                    wgpu::BindGroupEntry { binding: 3, resource: self.uniform_buffer.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::TextureView(&self.coc_view) },
                ],
            });
            let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("DOF Blur Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.blurred_view,
                    resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            rp.set_pipeline(&self.blur_pipeline);
            rp.set_bind_group(0, &bg, &[]);
            rp.draw(0..3, 0..1);
        }

        // Pass 3: Composite (blend sharp + blurred based on CoC → write back to HDR)
        {
            let bg = device.device().create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("DOF Composite BG"),
                layout: &self.composite_bgl,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(hdr_view) },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(depth_view) },
                    wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&self.sampler) },
                    wgpu::BindGroupEntry { binding: 3, resource: self.uniform_buffer.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::TextureView(&self.coc_view) },
                    wgpu::BindGroupEntry { binding: 5, resource: wgpu::BindingResource::TextureView(&self.blurred_view) },
                ],
            });
            let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("DOF Composite Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: hdr_view,
                    resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            rp.set_pipeline(&self.composite_pipeline);
            rp.set_bind_group(0, &bg, &[]);
            rp.draw(0..3, 0..1);
        }
    }
}

// --- BGL entry helpers ---

fn bgl_texture_2d(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    }
}

fn bgl_texture_2d_at(binding: u32) -> wgpu::BindGroupLayoutEntry {
    bgl_texture_2d(binding)
}

fn bgl_depth_texture(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Depth,
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    }
}

fn bgl_sampler_filtering(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
        count: None,
    }
}

fn bgl_sampler_non_filtering(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
        count: None,
    }
}

fn bgl_uniform(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}
