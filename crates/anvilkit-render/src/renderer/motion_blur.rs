//! # Motion Blur 后处理
//!
//! 基于速度 buffer 的方向模糊。
//! 两 pass 流程：速度重建 → 方向模糊。

use bevy_ecs::prelude::*;
use crate::renderer::RenderDevice;
use crate::renderer::buffer::HDR_FORMAT;

const MOTION_BLUR_SHADER: &str = include_str!("../shaders/motion_blur.wgsl");

/// Motion Blur 配置
#[derive(Debug, Clone, Resource)]
pub struct MotionBlurSettings {
    /// Whether motion blur is enabled.
    pub enabled: bool,
    /// Blur intensity multiplier.
    pub intensity: f32,
    /// Number of samples along the velocity vector.
    pub samples: u32,
}

impl Default for MotionBlurSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            intensity: 0.5,
            samples: 8,
        }
    }
}

/// Motion Blur GPU uniform
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MotionBlurUniform {
    /// Intensity.
    pub intensity: f32,
    /// Samples as f32.
    pub samples: f32,
    /// Padding.
    pub _pad0: f32,
    /// Padding.
    pub _pad1: f32,
    /// Previous frame view-projection matrix (column-major, 4 vec4s).
    pub prev_view_proj: [[f32; 4]; 4],
    /// Current frame inverse view-projection matrix.
    pub curr_inv_view_proj: [[f32; 4]; 4],
}

/// Motion Blur GPU 资源
pub struct MotionBlurResources {
    /// Velocity buffer (Rg16Float, full-res).
    pub velocity_texture: wgpu::Texture,
    /// Velocity texture view.
    pub velocity_view: wgpu::TextureView,
    /// Blurred output (Rgba16Float).
    pub output_texture: wgpu::Texture,
    /// Output texture view.
    pub output_view: wgpu::TextureView,
    /// Velocity pass pipeline.
    pub velocity_pipeline: wgpu::RenderPipeline,
    /// Blur pass pipeline.
    pub blur_pipeline: wgpu::RenderPipeline,
    /// Uniform buffer.
    pub uniform_buffer: wgpu::Buffer,
    /// Linear sampler.
    pub sampler: wgpu::Sampler,
    /// Non-filtering sampler for depth.
    pub depth_sampler: wgpu::Sampler,
    /// Velocity pass BGL.
    pub velocity_bgl: wgpu::BindGroupLayout,
    /// Blur pass BGL.
    pub blur_bgl: wgpu::BindGroupLayout,
}

impl MotionBlurResources {
    /// Create Motion Blur GPU resources.
    pub fn new(device: &RenderDevice, width: u32, height: u32) -> Self {
        let (velocity_texture, velocity_view) = Self::create_velocity_texture(device, width, height);
        let (output_texture, output_view) = Self::create_output_texture(device, width, height);

        let sampler = device.device().create_sampler(&wgpu::SamplerDescriptor {
            label: Some("MotionBlur Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let depth_sampler = device.device().create_sampler(&wgpu::SamplerDescriptor {
            label: Some("MotionBlur Depth Sampler"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let uniform_buffer = device.device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("MotionBlur Uniform"),
            size: std::mem::size_of::<MotionBlurUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let shader = device.device().create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("MotionBlur Shader"),
            source: wgpu::ShaderSource::Wgsl(MOTION_BLUR_SHADER.into()),
        });

        // Velocity BGL: src, depth, sampler, uniform
        let velocity_bgl = device.device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("MotionBlur Velocity BGL"),
            entries: &[
                bgl_texture_2d(0),
                bgl_depth_texture(1),
                bgl_sampler_non_filtering(2),
                bgl_uniform(3),
            ],
        });

        // Blur BGL: src, depth (unused), sampler, uniform, velocity
        let blur_bgl = device.device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("MotionBlur Blur BGL"),
            entries: &[
                bgl_texture_2d(0),
                bgl_depth_texture(1),
                bgl_sampler_filtering(2),
                bgl_uniform(3),
                bgl_texture_2d_rg(4),
            ],
        });

        let velocity_pipeline = Self::build_pipeline(device, &shader, "vs_main", "velocity_fs", &velocity_bgl, wgpu::TextureFormat::Rg16Float, "MotionBlur Velocity");
        let blur_pipeline = Self::build_pipeline(device, &shader, "vs_main", "blur_fs", &blur_bgl, HDR_FORMAT, "MotionBlur Blur");

        Self {
            velocity_texture,
            velocity_view,
            output_texture,
            output_view,
            velocity_pipeline,
            blur_pipeline,
            uniform_buffer,
            sampler,
            depth_sampler,
            velocity_bgl,
            blur_bgl,
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
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        })
    }

    fn create_velocity_texture(device: &RenderDevice, w: u32, h: u32) -> (wgpu::Texture, wgpu::TextureView) {
        let tex = device.device().create_texture(&wgpu::TextureDescriptor {
            label: Some("MotionBlur Velocity"),
            size: wgpu::Extent3d { width: w.max(1), height: h.max(1), depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rg16Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
        (tex, view)
    }

    fn create_output_texture(device: &RenderDevice, w: u32, h: u32) -> (wgpu::Texture, wgpu::TextureView) {
        let tex = device.device().create_texture(&wgpu::TextureDescriptor {
            label: Some("MotionBlur Output"),
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
        let (vt, vv) = Self::create_velocity_texture(device, width, height);
        self.velocity_texture = vt;
        self.velocity_view = vv;
        let (ot, ov) = Self::create_output_texture(device, width, height);
        self.output_texture = ot;
        self.output_view = ov;
    }

    /// Execute motion blur passes.
    pub fn execute(
        &self,
        device: &RenderDevice,
        encoder: &mut wgpu::CommandEncoder,
        hdr_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        settings: &MotionBlurSettings,
        prev_view_proj: [[f32; 4]; 4],
        curr_inv_view_proj: [[f32; 4]; 4],
    ) {
        if !settings.enabled {
            return;
        }

        let uniform = MotionBlurUniform {
            intensity: settings.intensity,
            samples: settings.samples as f32,
            _pad0: 0.0,
            _pad1: 0.0,
            prev_view_proj,
            curr_inv_view_proj,
        };
        device.queue().write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniform));

        // Pass 1: Velocity
        {
            let bg = device.device().create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("MotionBlur Velocity BG"),
                layout: &self.velocity_bgl,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(hdr_view) },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(depth_view) },
                    wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&self.depth_sampler) },
                    wgpu::BindGroupEntry { binding: 3, resource: self.uniform_buffer.as_entire_binding() },
                ],
            });
            let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("MotionBlur Velocity Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.velocity_view,
                    resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            rp.set_pipeline(&self.velocity_pipeline);
            rp.set_bind_group(0, &bg, &[]);
            rp.draw(0..3, 0..1);
        }

        // Pass 2: Blur
        {
            let bg = device.device().create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("MotionBlur Blur BG"),
                layout: &self.blur_bgl,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(hdr_view) },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(depth_view) },
                    wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&self.sampler) },
                    wgpu::BindGroupEntry { binding: 3, resource: self.uniform_buffer.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::TextureView(&self.velocity_view) },
                ],
            });
            let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("MotionBlur Blur Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.output_view,
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
    }
}

// --- BGL helpers ---

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

fn bgl_texture_2d_rg(binding: u32) -> wgpu::BindGroupLayoutEntry {
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
