use std::num::NonZeroU64;
use log::info;

use super::render_app::RenderApp;
use crate::renderer::{RenderPipelineBuilder, DEPTH_FORMAT};
use crate::renderer::assets::RenderAssets;
use crate::renderer::state::{RenderState, PbrSceneUniform, CSM_CASCADE_COUNT};
use crate::renderer::buffer::{
    create_depth_texture_msaa,
    create_hdr_render_target, create_hdr_msaa_texture,
    create_sampler, create_texture, create_texture_linear, create_shadow_sampler,
    create_csm_shadow_map,
    Vertex, PbrVertex, SHADOW_MAP_SIZE, HDR_FORMAT, MSAA_SAMPLE_COUNT,
};
use crate::renderer::ibl::get_or_generate_brdf_lut;
use crate::renderer::bloom::{BloomResources, BloomSettings};

/// Shadow pass shader (depth-only, reads model + view_proj from scene uniform)
const PBR_SHADER: &str = include_str!("../../shaders/pbr.wgsl");
const SHADOW_SHADER: &str = include_str!("../../shaders/shadow.wgsl");

/// ACES Filmic tone mapping post-process shader (fullscreen triangle)
const TONEMAP_SHADER: &str = include_str!("../../shaders/tonemap.wgsl");

impl RenderApp {
    /// GPU 初始化后，将共享资源注入 ECS World
    pub(super) fn inject_render_state_to_ecs(&mut self) {
        if self.gpu_initialized {
            return;
        }

        let Some(app) = &mut self.app else { return };
        let Some(device) = &self.render_device else { return };
        let Some(surface) = &self.render_surface else { return };

        let format = surface.format();
        let (w, h) = self.window_state.size();

        // 创建动态 Uniform 缓冲区 — 容量 1024 draws × 1024 bytes/draw = 1 MB
        // PbrSceneUniform 为 992 字节，对齐到 256 边界 → 每个 draw 占 1024 字节
        const UNIFORM_ALIGNMENT: u64 = 256;
        let uniform_stride = {
            let raw = std::mem::size_of::<PbrSceneUniform>() as u64;
            ((raw + UNIFORM_ALIGNMENT - 1) / UNIFORM_ALIGNMENT) * UNIFORM_ALIGNMENT
        };
        const MAX_DRAWS: u64 = 1024;
        let dynamic_uniform_buffer_size = uniform_stride * MAX_DRAWS;
        let scene_uniform_buffer = device.device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("ECS Dynamic Uniform Buffer"),
            size: dynamic_uniform_buffer_size,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_binding_size = NonZeroU64::new(uniform_stride);

        let scene_bind_group_layout = device.device().create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("ECS Scene BGL"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: uniform_binding_size,
                    },
                    count: None,
                }],
            },
        );

        let scene_bind_group = device.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ECS Scene BG"),
            layout: &scene_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &scene_uniform_buffer,
                    offset: 0,
                    size: uniform_binding_size,
                }),
            }],
        });

        let (_, depth_texture_view) = create_depth_texture_msaa(device, w, h, "ECS Depth MSAA");

        // HDR render target (resolve target, sample_count=1) + MSAA color attachment
        let (hdr_texture, hdr_texture_view) = create_hdr_render_target(device, w, h, "ECS HDR RT");
        let (_, hdr_msaa_texture_view) = create_hdr_msaa_texture(device, w, h, "ECS HDR MSAA");
        let sampler = create_sampler(device, "ECS Tonemap Sampler");

        // --- Bloom resources ---
        let bloom_settings = BloomSettings::default();
        let bloom = BloomResources::new(device, w, h, bloom_settings.mip_count);

        // Tonemap bind group layout + bind group (3 entries: HDR + sampler + bloom)
        let tonemap_bgl_entries = [
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
            wgpu::BindGroupLayoutEntry {
                binding: 2, visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                }, count: None,
            },
        ];

        let tonemap_bind_group_layout = device.device().create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("ECS Tonemap BGL"),
                entries: &tonemap_bgl_entries,
            },
        );

        let bloom_view_for_tonemap = if bloom.mip_views.is_empty() {
            &hdr_texture_view // fallback — shouldn't happen
        } else {
            &bloom.mip_views[0]
        };
        let tonemap_bind_group = device.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ECS Tonemap BG"),
            layout: &tonemap_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&hdr_texture_view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sampler) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(bloom_view_for_tonemap) },
            ],
        });

        // Tonemap pipeline BGL (consumed by builder — duplicate needed because builder takes ownership)
        let tonemap_pipeline_bgl = device.device().create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("ECS Tonemap Pipeline BGL"),
                entries: &tonemap_bgl_entries,
            },
        );

        let tonemap_pipeline = RenderPipelineBuilder::new()
            .with_vertex_shader(TONEMAP_SHADER)
            .with_fragment_shader(TONEMAP_SHADER)
            .with_format(format)
            .with_vertex_layouts(vec![])
            .with_bind_group_layouts(vec![tonemap_pipeline_bgl])
            .with_label("ECS Tonemap Pipeline")
            .build(device)
            .expect("创建 Tonemap 管线失败")
            .into_pipeline();

        // IBL + Shadow: bind group 2 (BRDF LUT + CSM shadow map array)
        let brdf_lut_data = get_or_generate_brdf_lut(".cache/brdf_lut_256.bin", 256);
        let (_, brdf_lut_view) = create_texture_linear(device, 256, 256, &brdf_lut_data, "ECS BRDF LUT");
        let (_shadow_tex, shadow_map_view, shadow_cascade_views) =
            create_csm_shadow_map(device, SHADOW_MAP_SIZE, CSM_CASCADE_COUNT as u32, "ECS CSM Shadow Map");
        let shadow_sampler = create_shadow_sampler(device, "ECS Shadow Sampler");

        let ibl_shadow_bind_group_layout = device.device().create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("ECS IBL+Shadow BGL"),
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
                    wgpu::BindGroupLayoutEntry {
                        binding: 2, visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Depth,
                            view_dimension: wgpu::TextureViewDimension::D2Array,
                            multisampled: false,
                        }, count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3, visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                        count: None,
                    },
                ],
            },
        );

        let ibl_shadow_bind_group = device.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ECS IBL+Shadow BG"),
            layout: &ibl_shadow_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&brdf_lut_view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sampler) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&shadow_map_view) },
                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::Sampler(&shadow_sampler) },
            ],
        });

        // Shadow pass pipeline (depth-only, uses PbrVertex layout for position)
        let shadow_scene_bgl = device.device().create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("Shadow Scene BGL"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: uniform_binding_size,
                    },
                    count: None,
                }],
            },
        );

        let shadow_pipeline = RenderPipelineBuilder::new()
            .with_vertex_shader(SHADOW_SHADER)
            .with_format(wgpu::TextureFormat::Rgba8Unorm) // dummy, no color output
            .with_vertex_layouts(vec![PbrVertex::layout()])
            .with_depth_format(DEPTH_FORMAT)
            .with_bind_group_layouts(vec![shadow_scene_bgl])
            .with_label("ECS Shadow Pipeline")
            .build_depth_only(device)
            .expect("创建 Shadow 管线失败")
            .into_pipeline();

        app.insert_resource(RenderState {
            surface_format: format,
            surface_size: (w, h),
            scene_uniform_buffer,
            scene_bind_group,
            scene_bind_group_layout,
            depth_texture_view,
            hdr_texture,
            hdr_texture_view,
            tonemap_pipeline,
            tonemap_bind_group,
            tonemap_bind_group_layout,
            ibl_shadow_bind_group,
            ibl_shadow_bind_group_layout,
            shadow_pipeline,
            shadow_map_view,
            shadow_cascade_views,
            hdr_msaa_texture_view,
            bloom: Some(bloom),
            post_process: crate::renderer::post_process::PostProcessResources::new(),
        });
        app.insert_resource(bloom_settings);
        app.insert_resource(crate::renderer::post_process::PostProcessSettings::default());

        // --- 创建默认 PBR 管线 + 默认材质（StandardMaterial 使用） ---
        {
            use crate::renderer::standard_material::DefaultMaterialHandle;

            // 创建 1x1 fallback 纹理
            let white_pixel = [255u8, 255, 255, 255];
            let normal_pixel = [128u8, 128, 255, 255]; // 默认法线 (0,0,1) in tangent space
            let _black_pixel = [0u8, 0, 0, 255];

            let (_, default_base_view) = create_texture(device, 1, 1, &white_pixel, "Default Base Color");
            let (_, default_normal_view) = create_texture_linear(device, 1, 1, &normal_pixel, "Default Normal Map");
            let (_, default_mr_view) = create_texture_linear(device, 1, 1, &white_pixel, "Default MR");
            let (_, default_ao_view) = create_texture_linear(device, 1, 1, &white_pixel, "Default AO");
            let (_, default_emissive_view) = create_texture(device, 1, 1, &white_pixel, "Default Emissive");
            let default_sampler = create_sampler(device, "Default Material Sampler");

            // Material BGL: 5 textures + 1 sampler
            let tex_layout_entry = |binding: u32| -> wgpu::BindGroupLayoutEntry {
                wgpu::BindGroupLayoutEntry {
                    binding, visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    }, count: None,
                }
            };

            let mat_bgl = device.device().create_bind_group_layout(
                &wgpu::BindGroupLayoutDescriptor {
                    label: Some("Default Material BGL"),
                    entries: &[
                        tex_layout_entry(0), // base_color
                        tex_layout_entry(1), // normal_map
                        tex_layout_entry(2), // metallic_roughness
                        tex_layout_entry(3), // ao
                        tex_layout_entry(4), // emissive
                        wgpu::BindGroupLayoutEntry {
                            binding: 5, visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                },
            );

            let default_mat_bg = device.device().create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Default Material BG"),
                layout: &mat_bgl,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&default_base_view) },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&default_normal_view) },
                    wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&default_mr_view) },
                    wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(&default_ao_view) },
                    wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::TextureView(&default_emissive_view) },
                    wgpu::BindGroupEntry { binding: 5, resource: wgpu::BindingResource::Sampler(&default_sampler) },
                ],
            });

            // Scene BGL 和 IBL+Shadow BGL 用于 PBR 管线需要重新创建（builder 取走所有权）
            let pbr_scene_bgl = device.device().create_bind_group_layout(
                &wgpu::BindGroupLayoutDescriptor {
                    label: Some("PBR Scene BGL"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: true,
                            min_binding_size: uniform_binding_size,
                        },
                        count: None,
                    }],
                },
            );
            let pbr_ibl_bgl = device.device().create_bind_group_layout(
                &wgpu::BindGroupLayoutDescriptor {
                    label: Some("PBR IBL+Shadow BGL"),
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
                        wgpu::BindGroupLayoutEntry {
                            binding: 2, visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Depth,
                                view_dimension: wgpu::TextureViewDimension::D2Array,
                                multisampled: false,
                            }, count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 3, visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                            count: None,
                        },
                    ],
                },
            );

            let pbr_pipeline = RenderPipelineBuilder::new()
                .with_vertex_shader(PBR_SHADER)
                .with_fragment_shader(PBR_SHADER)
                .with_format(HDR_FORMAT)
                .with_vertex_layouts(vec![PbrVertex::layout()])
                .with_depth_format(DEPTH_FORMAT)
                .with_bind_group_layouts(vec![pbr_scene_bgl, mat_bgl, pbr_ibl_bgl])
                .with_label("Default PBR Pipeline")
                .with_multisample_count(MSAA_SAMPLE_COUNT)
                .build(device)
                .expect("创建默认 PBR 管线失败")
                .into_pipeline();

            // 注册到 RenderAssets
            let mat_handle = {
                let mut assets = app.world_mut().get_resource_mut::<RenderAssets>().expect("RenderAssets 必须已注册");
                assets.create_material(pbr_pipeline, default_mat_bg)
            };
            app.world_mut().insert_resource(DefaultMaterialHandle(mat_handle));
            info!("默认 PBR 材质已创建: {:?}", mat_handle);
        }

        self.gpu_initialized = true;
        info!("RenderState (HDR + IBL + Shadow + Bloom + Default PBR) 已注入 ECS World");
    }
}
