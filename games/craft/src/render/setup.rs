use anvilkit_render::prelude::*;
use anvilkit_render::renderer::{
    RenderPipelineBuilder, DEPTH_FORMAT, HDR_FORMAT,
    buffer::{
        Vertex, create_uniform_buffer,
        create_depth_texture, create_hdr_render_target,
        create_sampler,
    },
};
use bytemuck::{Pod, Zeroable};

use crate::vertex::BlockVertex;

const VOXEL_SHADER: &str = include_str!("../../assets/voxel.wgsl");
const SKY_SHADER: &str = include_str!("../../assets/sky.wgsl");
const TONEMAP_SHADER: &str = include_str!("../../../../shaders/tonemap.wgsl");

/// GPU uniform matching VoxelSceneUniform in voxel.wgsl.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct VoxelSceneUniform {
    pub view_proj: [[f32; 4]; 4],
    pub camera_pos: [f32; 4],
    pub light_dir: [f32; 4],
    pub fog_color: [f32; 4],
    pub time_ambient: [f32; 4], // x=time, y=ambient, z=fog_start, w=fog_end
}

impl Default for VoxelSceneUniform {
    fn default() -> Self {
        Self {
            view_proj: glam::Mat4::IDENTITY.to_cols_array_2d(),
            camera_pos: [0.0; 4],
            light_dir: [0.3, 0.8, 0.5, 0.0],
            fog_color: [0.53, 0.71, 0.92, 1.0],
            time_ambient: [0.0, 0.35, 80.0, 200.0],
        }
    }
}

/// GPU uniform for sky shader.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct SkyUniform {
    pub inv_view_proj: [[f32; 4]; 4],
    pub sky_top: [f32; 4],
    pub sky_horizon: [f32; 4],
    pub sky_bottom: [f32; 4],
    pub sun_dir: [f32; 4],
}

impl Default for SkyUniform {
    fn default() -> Self {
        Self {
            inv_view_proj: glam::Mat4::IDENTITY.to_cols_array_2d(),
            sky_top: [0.25, 0.47, 0.85, 1.0],
            sky_horizon: [0.55, 0.73, 0.94, 1.0],
            sky_bottom: [0.37, 0.50, 0.65, 1.0],
            sun_dir: [0.2, 0.8, 0.5, 0.0],
        }
    }
}

/// All GPU state for voxel rendering.
pub struct VoxelGpu {
    pub scene_ub: wgpu::Buffer,
    pub scene_bg: wgpu::BindGroup,
    pub atlas_bg: wgpu::BindGroup,
    pub voxel_pipeline: wgpu::RenderPipeline,
    pub depth_view: wgpu::TextureView,
    pub hdr_view: wgpu::TextureView,
    pub tonemap_pipeline: wgpu::RenderPipeline,
    pub tonemap_bg: wgpu::BindGroup,
    pub scene_bgl: wgpu::BindGroupLayout,
    pub atlas_bgl: wgpu::BindGroupLayout,
    // Sky
    pub sky_pipeline: wgpu::RenderPipeline,
    pub sky_ub: wgpu::Buffer,
    pub sky_bg: wgpu::BindGroup,
}

pub fn init_voxel_gpu(
    device: &RenderDevice,
    surface_format: wgpu::TextureFormat,
    w: u32,
    h: u32,
    atlas_rgba: &[u8],
    atlas_width: u32,
    atlas_height: u32,
) -> VoxelGpu {
    // --- Scene uniform (group 0) ---
    let initial = VoxelSceneUniform::default();
    let scene_ub = create_uniform_buffer(device, "Voxel Scene UB", bytemuck::bytes_of(&initial));

    let scene_bgl = device.device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Voxel Scene BGL"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });

    let scene_bg = device.device().create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Voxel Scene BG"),
        layout: &scene_bgl,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: scene_ub.as_entire_binding(),
        }],
    });

    // --- Atlas texture (group 1) — NEAREST sampling for pixel art ---
    let atlas_tex = device.device().create_texture(&wgpu::TextureDescriptor {
        label: Some("Atlas Texture"),
        size: wgpu::Extent3d {
            width: atlas_width,
            height: atlas_height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    device.queue().write_texture(
        wgpu::ImageCopyTexture {
            texture: &atlas_tex,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        atlas_rgba,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(4 * atlas_width),
            rows_per_image: Some(atlas_height),
        },
        wgpu::Extent3d {
            width: atlas_width,
            height: atlas_height,
            depth_or_array_layers: 1,
        },
    );
    let atlas_view = atlas_tex.create_view(&wgpu::TextureViewDescriptor::default());

    let nearest_sampler = device.device().create_sampler(&wgpu::SamplerDescriptor {
        label: Some("Nearest Sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });

    let atlas_bgl = device.device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Atlas BGL"),
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

    let atlas_bg = device.device().create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Atlas BG"),
        layout: &atlas_bgl,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&atlas_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&nearest_sampler),
            },
        ],
    });

    // --- Voxel render pipeline (no MSAA) ---
    let voxel_layout = device.device().create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Voxel Pipeline Layout"),
        bind_group_layouts: &[&scene_bgl, &atlas_bgl],
        push_constant_ranges: &[],
    });
    let voxel_shader = device.device().create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Voxel Shader"),
        source: wgpu::ShaderSource::Wgsl(VOXEL_SHADER.into()),
    });
    let voxel_pipeline = device.device().create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Voxel Pipeline"),
        layout: Some(&voxel_layout),
        vertex: wgpu::VertexState {
            module: &voxel_shader,
            entry_point: "vs_main",
            buffers: &[BlockVertex::layout()],
        },
        fragment: Some(wgpu::FragmentState {
            module: &voxel_shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: HDR_FORMAT,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            cull_mode: Some(wgpu::Face::Back),
            ..Default::default()
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: Default::default(),
            bias: Default::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    });

    // --- Sky pipeline (fullscreen triangle, no depth, writes to HDR) ---
    let sky_initial = SkyUniform::default();
    let sky_ub = create_uniform_buffer(device, "Sky UB", bytemuck::bytes_of(&sky_initial));

    let sky_bgl = device.device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Sky BGL"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });

    let sky_bg = device.device().create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Sky BG"),
        layout: &sky_bgl,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: sky_ub.as_entire_binding(),
        }],
    });

    let sky_shader = device.device().create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Sky Shader"),
        source: wgpu::ShaderSource::Wgsl(SKY_SHADER.into()),
    });
    let sky_layout = device.device().create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Sky Pipeline Layout"),
        bind_group_layouts: &[&sky_bgl],
        push_constant_ranges: &[],
    });
    let sky_pipeline = device.device().create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Sky Pipeline"),
        layout: Some(&sky_layout),
        vertex: wgpu::VertexState {
            module: &sky_shader,
            entry_point: "vs_main",
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &sky_shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: HDR_FORMAT,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            ..Default::default()
        },
        depth_stencil: None, // sky has no depth
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    });

    // --- Depth + HDR RT ---
    let (_, depth_view) = create_depth_texture(device, w, h, "Voxel Depth");
    let (_, hdr_view) = create_hdr_render_target(device, w, h, "Voxel HDR RT");

    // --- Tonemap pipeline ---
    let linear_sampler = create_sampler(device, "Tonemap Sampler");
    let tex_entry = |b: u32| wgpu::BindGroupLayoutEntry {
        binding: b,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    };
    let tm_bgl = device.device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Tonemap BGL"),
        entries: &[
            tex_entry(0),
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    });
    let tonemap_bg = device.device().create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Tonemap BG"),
        layout: &tm_bgl,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&hdr_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&linear_sampler),
            },
        ],
    });
    let tonemap_pipeline = RenderPipelineBuilder::new()
        .with_vertex_shader(TONEMAP_SHADER)
        .with_fragment_shader(TONEMAP_SHADER)
        .with_format(surface_format)
        .with_vertex_layouts(vec![])
        .with_bind_group_layouts(vec![tm_bgl])
        .with_label("Voxel Tonemap Pipeline")
        .build(device)
        .expect("Failed to create tonemap pipeline")
        .into_pipeline();

    VoxelGpu {
        scene_ub,
        scene_bg,
        atlas_bg,
        voxel_pipeline,
        depth_view,
        hdr_view,
        tonemap_pipeline,
        tonemap_bg,
        scene_bgl,
        atlas_bgl,
        sky_pipeline,
        sky_ub,
        sky_bg,
    }
}
