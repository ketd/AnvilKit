//! # Color Grading 后处理
//!
//! 调色管线：曝光、对比度、饱和度、白平衡、3D LUT。

use bevy_ecs::prelude::*;
use crate::renderer::RenderDevice;
use crate::renderer::buffer::HDR_FORMAT;

const COLOR_GRADING_SHADER: &str = include_str!("../shaders/color_grading.wgsl");

/// Color Grading 配置
#[derive(Debug, Clone, Resource)]
pub struct ColorGradingSettings {
    /// Whether color grading is enabled.
    pub enabled: bool,
    /// Exposure multiplier (1.0 = no change).
    pub exposure: f32,
    /// Contrast multiplier (1.0 = no change).
    pub contrast: f32,
    /// Saturation multiplier (1.0 = no change, 0.0 = grayscale).
    pub saturation: f32,
    /// Color temperature offset (-1.0 cool .. 1.0 warm).
    pub temperature: f32,
    /// Green-magenta tint (-1.0 .. 1.0).
    pub tint: f32,
    /// LUT contribution (0.0 = none, 1.0 = full LUT).
    pub lut_contribution: f32,
}

impl Default for ColorGradingSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            exposure: 1.0,
            contrast: 1.0,
            saturation: 1.0,
            temperature: 0.0,
            tint: 0.0,
            lut_contribution: 0.0,
        }
    }
}

/// Color Grading GPU uniform
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ColorGradingUniform {
    /// Exposure.
    pub exposure: f32,
    /// Contrast.
    pub contrast: f32,
    /// Saturation.
    pub saturation: f32,
    /// Temperature.
    pub temperature: f32,
    /// Tint.
    pub tint: f32,
    /// LUT contribution.
    pub lut_contribution: f32,
    /// Padding.
    pub _pad0: f32,
    /// Padding.
    pub _pad1: f32,
}

/// LUT size (32x32x32).
pub const LUT_SIZE: u32 = 32;

/// Color Grading GPU 资源
pub struct ColorGradingResources {
    /// Render pipeline.
    pub pipeline: wgpu::RenderPipeline,
    /// Uniform buffer.
    pub uniform_buffer: wgpu::Buffer,
    /// 3D LUT texture (Rgba8, 32^3).
    pub lut_texture: wgpu::Texture,
    /// LUT texture view.
    pub lut_view: wgpu::TextureView,
    /// Linear sampler.
    pub sampler: wgpu::Sampler,
    /// LUT sampler (linear, clamp).
    pub lut_sampler: wgpu::Sampler,
    /// Bind group layout.
    pub bgl: wgpu::BindGroupLayout,
}

impl ColorGradingResources {
    /// Create Color Grading GPU resources with identity LUT.
    pub fn new(device: &RenderDevice) -> Self {
        let sampler = device.device().create_sampler(&wgpu::SamplerDescriptor {
            label: Some("ColorGrading Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let lut_sampler = device.device().create_sampler(&wgpu::SamplerDescriptor {
            label: Some("ColorGrading LUT Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let uniform_buffer = device.device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("ColorGrading Uniform"),
            size: std::mem::size_of::<ColorGradingUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let (lut_texture, lut_view) = Self::generate_identity_lut(device);

        let bgl = device.device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ColorGrading BGL"),
            entries: &[
                // binding 0: src_texture
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
                // binding 1: sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // binding 2: uniform
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // binding 3: 3D LUT texture
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D3,
                        multisampled: false,
                    },
                    count: None,
                },
                // binding 4: LUT sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let shader = device.device().create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ColorGrading Shader"),
            source: wgpu::ShaderSource::Wgsl(COLOR_GRADING_SHADER.into()),
        });

        let pl = device.device().create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ColorGrading PL"),
            bind_group_layouts: &[&bgl],
            push_constant_ranges: &[],
        });

        let pipeline = device.device().create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ColorGrading Pipeline"),
            layout: Some(&pl),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: HDR_FORMAT,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });

        Self {
            pipeline,
            uniform_buffer,
            lut_texture,
            lut_view,
            sampler,
            lut_sampler,
            bgl,
        }
    }

    /// Generate a 32x32x32 identity LUT (each texel = its own coordinate as color).
    pub fn generate_identity_lut(device: &RenderDevice) -> (wgpu::Texture, wgpu::TextureView) {
        let size = LUT_SIZE;
        let mut data = Vec::with_capacity((size * size * size * 4) as usize);
        for z in 0..size {
            for y in 0..size {
                for x in 0..size {
                    data.push((x as f32 / (size - 1) as f32 * 255.0) as u8);
                    data.push((y as f32 / (size - 1) as f32 * 255.0) as u8);
                    data.push((z as f32 / (size - 1) as f32 * 255.0) as u8);
                    data.push(255);
                }
            }
        }

        let texture = device.device().create_texture(&wgpu::TextureDescriptor {
            label: Some("ColorGrading Identity LUT"),
            size: wgpu::Extent3d { width: size, height: size, depth_or_array_layers: size },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D3,
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
            &data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(size * 4),
                rows_per_image: Some(size),
            },
            wgpu::Extent3d { width: size, height: size, depth_or_array_layers: size },
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D3),
            ..Default::default()
        });

        (texture, view)
    }

    /// No-op resize (color grading has no resolution-dependent textures).
    pub fn resize(&mut self, _device: &RenderDevice, _width: u32, _height: u32) {}

    /// Execute color grading pass.
    pub fn execute(
        &self,
        device: &RenderDevice,
        encoder: &mut wgpu::CommandEncoder,
        src_view: &wgpu::TextureView,
        dst_view: &wgpu::TextureView,
        settings: &ColorGradingSettings,
    ) {
        if !settings.enabled {
            return;
        }

        let uniform = ColorGradingUniform {
            exposure: settings.exposure,
            contrast: settings.contrast,
            saturation: settings.saturation,
            temperature: settings.temperature,
            tint: settings.tint,
            lut_contribution: settings.lut_contribution,
            _pad0: 0.0,
            _pad1: 0.0,
        };
        device.queue().write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniform));

        let bg = device.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ColorGrading BG"),
            layout: &self.bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(src_view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&self.sampler) },
                wgpu::BindGroupEntry { binding: 2, resource: self.uniform_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(&self.lut_view) },
                wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::Sampler(&self.lut_sampler) },
            ],
        });

        let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("ColorGrading Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: dst_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        rp.set_pipeline(&self.pipeline);
        rp.set_bind_group(0, &bg, &[]);
        rp.draw(0..3, 0..1);
    }
}
