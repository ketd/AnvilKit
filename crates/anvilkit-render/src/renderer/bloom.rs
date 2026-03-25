//! # Bloom 后处理
//!
//! 基于 HDR 渲染管线的物理 Bloom 效果。
//! 使用 13-tap downsample + 9-tap tent filter upsample 的 mip chain 方案。

use bevy_ecs::prelude::*;
use crate::renderer::RenderDevice;
use crate::renderer::buffer::{HDR_FORMAT, BLOOM_MIP_COUNT, create_bloom_mip_chain};

const BLOOM_DOWNSAMPLE_SHADER: &str = include_str!("../shaders/bloom_downsample.wgsl");
const BLOOM_UPSAMPLE_SHADER: &str = include_str!("../shaders/bloom_upsample.wgsl");

/// Bloom 配置参数
///
/// 作为 ECS Resource 插入 World，运行时可调。
///
/// # 示例
///
/// ```rust
/// use anvilkit_render::renderer::bloom::BloomSettings;
///
/// let settings = BloomSettings {
///     enabled: true,
///     threshold: 1.0,
///     knee: 0.1,
///     intensity: 0.3,
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, Resource)]
pub struct BloomSettings {
    /// Whether bloom is enabled.
    pub enabled: bool,
    /// HDR brightness threshold for bloom extraction.
    pub threshold: f32,
    /// Soft knee range for smooth threshold transition.
    pub knee: f32,
    /// Bloom intensity (multiplied during upsample composite).
    pub intensity: f32,
    /// Number of downsample mip levels (default 5).
    pub mip_count: u32,
}

impl Default for BloomSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            threshold: 1.0,
            knee: 0.1,
            intensity: 0.3,
            mip_count: BLOOM_MIP_COUNT,
        }
    }
}

/// Bloom GPU 参数（上传到 uniform buffer）
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BloomUniform {
    /// Brightness threshold.
    pub threshold: f32,
    /// Soft knee.
    pub knee: f32,
    /// Intensity multiplier.
    pub intensity: f32,
    /// Current mip level (0 = threshold pass).
    pub mip_level: f32,
}

/// Bloom GPU 资源集合
pub struct BloomResources {
    /// Bloom mip chain texture (half-res, N mip levels).
    pub mip_texture: wgpu::Texture,
    /// Per-mip texture views for rendering into.
    pub mip_views: Vec<wgpu::TextureView>,
    /// Full mip chain view for sampling the final bloom result.
    pub full_view: wgpu::TextureView,
    /// Downsample render pipeline.
    pub downsample_pipeline: wgpu::RenderPipeline,
    /// Upsample render pipeline (additive blending).
    pub upsample_pipeline: wgpu::RenderPipeline,
    /// Bind group layout shared by both pipelines.
    pub bind_group_layout: wgpu::BindGroupLayout,
    /// Uniform buffer for bloom parameters.
    pub uniform_buffer: wgpu::Buffer,
    /// Linear sampler for bloom texture sampling.
    pub sampler: wgpu::Sampler,
}

impl BloomResources {
    /// 创建 Bloom GPU 资源
    pub fn new(device: &RenderDevice, width: u32, height: u32, mip_count: u32) -> Self {
        let (mip_texture, mip_views) =
            create_bloom_mip_chain(device, width, height, mip_count, "Bloom Mip Chain");

        let full_view = mip_texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Bloom Full View"),
            ..Default::default()
        });

        let sampler = device.device().create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Bloom Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let uniform_buffer = device.device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("Bloom Uniform"),
            size: std::mem::size_of::<BloomUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout =
            device
                .device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Bloom BGL"),
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
                    ],
                });

        // Downsample pipeline — writes to mip N+1 (no blending)
        let downsample_pipeline = Self::build_downsample_pipeline(device, &bind_group_layout);

        // Upsample pipeline — additive blending (src + dst)
        let upsample_pipeline = Self::build_upsample_pipeline(device, &bind_group_layout);

        Self {
            mip_texture,
            mip_views,
            full_view,
            downsample_pipeline,
            upsample_pipeline,
            bind_group_layout,
            uniform_buffer,
            sampler,
        }
    }

    fn build_downsample_pipeline(
        device: &RenderDevice,
        layout: &wgpu::BindGroupLayout,
    ) -> wgpu::RenderPipeline {
        let shader = device
            .device()
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Bloom Downsample Shader"),
                source: wgpu::ShaderSource::Wgsl(BLOOM_DOWNSAMPLE_SHADER.into()),
            });

        let pipeline_layout =
            device
                .device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Bloom Downsample PL"),
                    bind_group_layouts: &[layout],
                    push_constant_ranges: &[],
                });

        device
            .device()
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Bloom Downsample Pipeline"),
                layout: Some(&pipeline_layout),
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
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview: None,
            })
    }

    fn build_upsample_pipeline(
        device: &RenderDevice,
        layout: &wgpu::BindGroupLayout,
    ) -> wgpu::RenderPipeline {
        let shader = device
            .device()
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Bloom Upsample Shader"),
                source: wgpu::ShaderSource::Wgsl(BLOOM_UPSAMPLE_SHADER.into()),
            });

        let pipeline_layout =
            device
                .device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Bloom Upsample PL"),
                    bind_group_layouts: &[layout],
                    push_constant_ranges: &[],
                });

        device
            .device()
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Bloom Upsample Pipeline"),
                layout: Some(&pipeline_layout),
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
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::One,
                                dst_factor: wgpu::BlendFactor::One,
                                operation: wgpu::BlendOperation::Add,
                            },
                            alpha: wgpu::BlendComponent::OVER,
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview: None,
            })
    }

    /// 创建一个绑定指定纹理 view 的 bind group
    pub fn create_bind_group(
        &self,
        device: &RenderDevice,
        texture_view: &wgpu::TextureView,
        label: &str,
    ) -> wgpu::BindGroup {
        device
            .device()
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(label),
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: self.uniform_buffer.as_entire_binding(),
                    },
                ],
            })
    }

    /// 在 resize 时重建 mip chain
    pub fn resize(&mut self, device: &RenderDevice, width: u32, height: u32, mip_count: u32) {
        let (mip_texture, mip_views) =
            create_bloom_mip_chain(device, width, height, mip_count, "Bloom Mip Chain");
        let full_view = mip_texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Bloom Full View"),
            ..Default::default()
        });
        self.mip_texture = mip_texture;
        self.mip_views = mip_views;
        self.full_view = full_view;
    }

    /// 执行完整的 Bloom pass：downsample chain → upsample chain
    ///
    /// `hdr_view` 是场景渲染完成后的 HDR 纹理视图。
    /// 执行后，`self.mip_views[0]` 包含最终的 bloom 结果。
    pub fn execute(
        &self,
        device: &RenderDevice,
        encoder: &mut wgpu::CommandEncoder,
        hdr_view: &wgpu::TextureView,
        settings: &BloomSettings,
    ) {
        if self.mip_views.is_empty() {
            return;
        }

        // When disabled, clear mip_views[0] to BLACK so tonemap's `c += bloom` is identity.
        if !settings.enabled {
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Bloom Clear"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.mip_views[0],
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
            return;
        }

        let mip_count = self.mip_views.len();

        // --- Downsample chain: HDR → mip0, mip0 → mip1, ... ---
        for i in 0..mip_count {
            let src_view = if i == 0 { hdr_view } else { &self.mip_views[i - 1] };
            let dst_view = &self.mip_views[i];

            let uniform = BloomUniform {
                threshold: settings.threshold,
                knee: settings.knee,
                intensity: settings.intensity,
                mip_level: i as f32,
            };
            device
                .queue()
                .write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniform));

            let bg = self.create_bind_group(device, src_view, &format!("Bloom Down BG {}", i));

            let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Bloom Downsample"),
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
            rp.set_pipeline(&self.downsample_pipeline);
            rp.set_bind_group(0, &bg, &[]);
            rp.draw(0..3, 0..1);
        }

        // --- Upsample chain: mip(N-1) → mip(N-2) → ... → mip0 (additive blend) ---
        for i in (0..mip_count - 1).rev() {
            let src_view = &self.mip_views[i + 1];
            let dst_view = &self.mip_views[i];

            let uniform = BloomUniform {
                threshold: settings.threshold,
                knee: settings.knee,
                intensity: settings.intensity,
                mip_level: (i + 1) as f32,
            };
            device
                .queue()
                .write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniform));

            let bg = self.create_bind_group(device, src_view, &format!("Bloom Up BG {}", i));

            let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Bloom Upsample"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: dst_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load, // Additive: load existing content
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            rp.set_pipeline(&self.upsample_pipeline);
            rp.set_bind_group(0, &bg, &[]);
            rp.draw(0..3, 0..1);
        }
    }
}
