//! # Screen-Space Ambient Occlusion (SSAO)
//!
//! 基于深度 buffer 的 hemisphere sampling SSAO。
//! 半分辨率渲染 + box blur 上采样。

use bevy_ecs::prelude::*;
use crate::renderer::RenderDevice;
use crate::renderer::RenderPipelineBuilder;

const SSAO_SHADER: &str = include_str!("../shaders/ssao.wgsl");
const SSAO_BLUR_SHADER: &str = include_str!("../shaders/ssao_blur.wgsl");

/// SSAO output format — single-channel red (R8Unorm)
const SSAO_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R8Unorm;

/// SSAO 配置参数
#[derive(Debug, Clone, Resource)]
pub struct SsaoSettings {
    /// Whether SSAO is enabled.
    pub enabled: bool,
    /// Sampling quality (number of kernel samples).
    pub quality: SsaoQuality,
    /// Sampling hemisphere radius in view space.
    pub radius: f32,
    /// Depth bias to prevent self-occlusion.
    pub bias: f32,
    /// AO intensity multiplier.
    pub intensity: f32,
}

impl Default for SsaoSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            quality: SsaoQuality::Medium,
            radius: 0.5,
            bias: 0.025,
            intensity: 1.0,
        }
    }
}

/// SSAO 采样质量
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SsaoQuality {
    /// 16 samples.
    Low,
    /// 32 samples.
    Medium,
    /// 64 samples.
    High,
}

impl SsaoQuality {
    /// Sample count for this quality level.
    pub fn sample_count(self) -> u32 {
        match self {
            SsaoQuality::Low => 16,
            SsaoQuality::Medium => 32,
            SsaoQuality::High => 64,
        }
    }
}

/// SSAO GPU uniform
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SsaoUniform {
    /// Camera projection matrix.
    pub projection: [[f32; 4]; 4],
    /// Inverse projection matrix.
    pub inv_projection: [[f32; 4]; 4],
    /// Sampling radius.
    pub radius: f32,
    /// Depth bias.
    pub bias: f32,
    /// Intensity.
    pub intensity: f32,
    /// Sample count (as f32).
    pub sample_count: f32,
}

/// Generate hemisphere kernel samples (tangent-space, z > 0).
fn generate_kernel(count: u32) -> Vec<[f32; 4]> {
    use std::f32::consts::PI;
    let mut kernel = Vec::with_capacity(count as usize);

    // Simple deterministic low-discrepancy distribution
    for i in 0..count {
        let xi1 = (i as f32 + 0.5) / count as f32;
        let xi2 = {
            // Van der Corput sequence in base 2
            let mut bits = i;
            bits = (bits << 16) | (bits >> 16);
            bits = ((bits & 0x55555555) << 1) | ((bits & 0xAAAAAAAA) >> 1);
            bits = ((bits & 0x33333333) << 2) | ((bits & 0xCCCCCCCC) >> 2);
            bits = ((bits & 0x0F0F0F0F) << 4) | ((bits & 0xF0F0F0F0) >> 4);
            bits = ((bits & 0x00FF00FF) << 8) | ((bits & 0xFF00FF00) >> 8);
            bits as f32 / 0x100000000u64 as f32
        };

        // Cosine-weighted hemisphere sampling
        let phi = 2.0 * PI * xi2;
        let cos_theta = (1.0 - xi1).sqrt();
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

        let mut sample = [
            sin_theta * phi.cos(),
            sin_theta * phi.sin(),
            cos_theta,
            0.0,
        ];

        // Accelerating distribution: closer samples have more weight
        let scale = (i as f32 / count as f32).powi(2).max(0.1);
        sample[0] *= scale;
        sample[1] *= scale;
        sample[2] *= scale;

        kernel.push(sample);
    }
    kernel
}

/// Generate 4x4 noise texture (random rotation vectors in tangent space).
fn generate_noise_texture(device: &RenderDevice) -> (wgpu::Texture, wgpu::TextureView) {
    let size = 4u32;
    let mut data = vec![0u8; (size * size * 4) as usize];

    // Deterministic noise using simple hash
    for i in 0..(size * size) {
        let hash = |x: u32| -> f32 {
            let h = x.wrapping_mul(2654435761);
            (h as f32 / u32::MAX as f32) * 2.0 - 1.0
        };
        let x = hash(i * 3 + 0);
        let y = hash(i * 3 + 1);
        let len = (x * x + y * y).sqrt().max(0.001);
        let nx = x / len;
        let ny = y / len;
        let idx = (i * 4) as usize;
        data[idx] = ((nx * 0.5 + 0.5) * 255.0) as u8;
        data[idx + 1] = ((ny * 0.5 + 0.5) * 255.0) as u8;
        data[idx + 2] = 0;
        data[idx + 3] = 255;
    }

    let texture = device.device().create_texture(&wgpu::TextureDescriptor {
        label: Some("SSAO Noise"),
        size: wgpu::Extent3d { width: size, height: size, depth_or_array_layers: 1 },
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
        &data,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(size * 4),
            rows_per_image: Some(size),
        },
        wgpu::Extent3d { width: size, height: size, depth_or_array_layers: 1 },
    );

    let view = texture.create_view(&Default::default());
    (texture, view)
}

/// SSAO GPU 资源集合
pub struct SsaoResources {
    /// Half-resolution SSAO render target (R8Unorm).
    pub ssao_texture: wgpu::Texture,
    /// View for the raw SSAO output.
    pub ssao_view: wgpu::TextureView,
    /// Blurred SSAO result (same size as ssao_texture).
    pub blurred_texture: wgpu::Texture,
    /// View for the blurred SSAO output (used by tonemap).
    pub blurred_view: wgpu::TextureView,
    /// SSAO render pipeline.
    pub ssao_pipeline: wgpu::RenderPipeline,
    /// Blur render pipeline.
    pub blur_pipeline: wgpu::RenderPipeline,
    /// Bind group layout for SSAO pass.
    pub ssao_bgl: wgpu::BindGroupLayout,
    /// Bind group layout for blur pass.
    pub blur_bgl: wgpu::BindGroupLayout,
    /// Uniform buffer for SSAO parameters.
    pub uniform_buffer: wgpu::Buffer,
    /// Storage buffer for kernel samples.
    pub kernel_buffer: wgpu::Buffer,
    /// Noise texture and view.
    pub noise_view: wgpu::TextureView,
    /// Samplers.
    /// Nearest-neighbor sampler for depth texture.
    pub nearest_sampler: wgpu::Sampler,
    /// Linear sampler for noise and blur textures.
    pub linear_sampler: wgpu::Sampler,
    /// Half-resolution width.
    pub half_width: u32,
    /// Half-resolution height.
    pub half_height: u32,
}

impl SsaoResources {
    /// 创建 SSAO GPU 资源
    pub fn new(device: &RenderDevice, width: u32, height: u32, sample_count: u32) -> Self {
        let half_width = (width / 2).max(1);
        let half_height = (height / 2).max(1);

        let ssao_texture = device.device().create_texture(&wgpu::TextureDescriptor {
            label: Some("SSAO RT"),
            size: wgpu::Extent3d { width: half_width, height: half_height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: SSAO_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let ssao_view = ssao_texture.create_view(&Default::default());

        let blurred_texture = device.device().create_texture(&wgpu::TextureDescriptor {
            label: Some("SSAO Blurred"),
            size: wgpu::Extent3d { width: half_width, height: half_height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: SSAO_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let blurred_view = blurred_texture.create_view(&Default::default());

        let (_, noise_view) = generate_noise_texture(device);

        let kernel = generate_kernel(sample_count);
        let kernel_bytes: Vec<u8> = kernel.iter()
            .flat_map(|s| bytemuck::bytes_of(s).to_vec())
            .collect();
        let kernel_buffer = device.device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("SSAO Kernel"),
            size: (sample_count as usize * 16).max(16) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        device.queue().write_buffer(&kernel_buffer, 0, &kernel_bytes);

        let uniform_buffer = device.device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("SSAO Uniform"),
            size: std::mem::size_of::<SsaoUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let nearest_sampler = device.device().create_sampler(&wgpu::SamplerDescriptor {
            label: Some("SSAO Nearest Sampler"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });

        let linear_sampler = device.device().create_sampler(&wgpu::SamplerDescriptor {
            label: Some("SSAO Linear Sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            ..Default::default()
        });

        // SSAO pass BGL: depth + depth_sampler + noise + noise_sampler + uniform + kernel
        let ssao_bgl = device.device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("SSAO BGL"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0, visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    }, count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1, visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2, visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    }, count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3, visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4, visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    }, count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5, visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    }, count: None,
                },
            ],
        });

        // Blur pass BGL: ssao_texture + sampler
        let blur_bgl = device.device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("SSAO Blur BGL"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0, visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    }, count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1, visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        // Build pipelines
        let ssao_bgl_for_pipeline = device.device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("SSAO Pipeline BGL"),
            entries: &[
                wgpu::BindGroupLayoutEntry { binding: 0, visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Depth, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 1, visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering), count: None },
                wgpu::BindGroupLayoutEntry { binding: 2, visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 3, visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None },
                wgpu::BindGroupLayoutEntry { binding: 4, visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 5, visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: true }, has_dynamic_offset: false, min_binding_size: None }, count: None },
            ],
        });

        let ssao_pipeline = RenderPipelineBuilder::new()
            .with_vertex_shader(SSAO_SHADER)
            .with_fragment_shader(SSAO_SHADER)
            .with_format(SSAO_FORMAT)
            .with_vertex_layouts(vec![])
            .with_bind_group_layouts(vec![ssao_bgl_for_pipeline])
            .with_label("SSAO Pipeline")
            .build(device)
            .expect("Failed to build SSAO pipeline")
            .into_pipeline();

        let blur_bgl_for_pipeline = device.device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("SSAO Blur Pipeline BGL"),
            entries: &[
                wgpu::BindGroupLayoutEntry { binding: 0, visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 1, visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None },
            ],
        });

        let blur_pipeline = RenderPipelineBuilder::new()
            .with_vertex_shader(SSAO_BLUR_SHADER)
            .with_fragment_shader(SSAO_BLUR_SHADER)
            .with_format(SSAO_FORMAT)
            .with_vertex_layouts(vec![])
            .with_bind_group_layouts(vec![blur_bgl_for_pipeline])
            .with_label("SSAO Blur Pipeline")
            .build(device)
            .expect("Failed to build SSAO blur pipeline")
            .into_pipeline();

        Self {
            ssao_texture,
            ssao_view,
            blurred_texture,
            blurred_view,
            ssao_pipeline,
            blur_pipeline,
            ssao_bgl,
            blur_bgl,
            uniform_buffer,
            kernel_buffer,
            noise_view,
            nearest_sampler,
            linear_sampler,
            half_width,
            half_height,
        }
    }

    /// Resize SSAO render targets.
    pub fn resize(&mut self, device: &RenderDevice, width: u32, height: u32) {
        let hw = (width / 2).max(1);
        let hh = (height / 2).max(1);
        self.half_width = hw;
        self.half_height = hh;

        self.ssao_texture = device.device().create_texture(&wgpu::TextureDescriptor {
            label: Some("SSAO RT"),
            size: wgpu::Extent3d { width: hw, height: hh, depth_or_array_layers: 1 },
            mip_level_count: 1, sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: SSAO_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        self.ssao_view = self.ssao_texture.create_view(&Default::default());

        self.blurred_texture = device.device().create_texture(&wgpu::TextureDescriptor {
            label: Some("SSAO Blurred"),
            size: wgpu::Extent3d { width: hw, height: hh, depth_or_array_layers: 1 },
            mip_level_count: 1, sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: SSAO_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        self.blurred_view = self.blurred_texture.create_view(&Default::default());
    }

    /// Execute SSAO + blur passes. Returns the blurred AO view for compositing.
    pub fn execute(
        &self,
        device: &RenderDevice,
        encoder: &mut wgpu::CommandEncoder,
        depth_view: &wgpu::TextureView,
        projection: &glam::Mat4,
        settings: &SsaoSettings,
    ) {
        if !settings.enabled {
            // Clear blurred_view to white (1.0 = no occlusion) so tonemap's c *= ao is identity
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("SSAO Clear"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.blurred_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            return;
        }

        // Update uniform
        let inv_proj = projection.inverse();
        let uniform = SsaoUniform {
            projection: projection.to_cols_array_2d(),
            inv_projection: inv_proj.to_cols_array_2d(),
            radius: settings.radius,
            bias: settings.bias,
            intensity: settings.intensity,
            sample_count: settings.quality.sample_count() as f32,
        };
        device.queue().write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniform));

        // SSAO pass bind group
        let ssao_bg = device.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("SSAO BG"),
            layout: &self.ssao_bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(depth_view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&self.nearest_sampler) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&self.noise_view) },
                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::Sampler(&self.linear_sampler) },
                wgpu::BindGroupEntry { binding: 4, resource: self.uniform_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 5, resource: self.kernel_buffer.as_entire_binding() },
            ],
        });

        // Pass 1: SSAO
        {
            let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("SSAO Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.ssao_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            rp.set_pipeline(&self.ssao_pipeline);
            rp.set_bind_group(0, &ssao_bg, &[]);
            rp.draw(0..3, 0..1);
        }

        // Pass 2: Blur
        let blur_bg = device.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("SSAO Blur BG"),
            layout: &self.blur_bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&self.ssao_view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&self.linear_sampler) },
            ],
        });

        {
            let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("SSAO Blur Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.blurred_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            rp.set_pipeline(&self.blur_pipeline);
            rp.set_bind_group(0, &blur_bg, &[]);
            rp.draw(0..3, 0..1);
        }
    }
}
