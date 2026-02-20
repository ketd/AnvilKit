use anvilkit_render::prelude::*;
use anvilkit_render::renderer::{
    RenderPipelineBuilder, DEPTH_FORMAT, HDR_FORMAT,
    buffer::{
        PbrVertex, Vertex, create_uniform_buffer,
        create_depth_texture_msaa, create_hdr_render_target, create_hdr_msaa_texture,
        create_texture, create_texture_linear, create_sampler,
        create_shadow_map, create_shadow_sampler, SHADOW_MAP_SIZE, MSAA_SAMPLE_COUNT,
    },
    assets::RenderAssets,
    ibl::generate_brdf_lut,
};
use anvilkit_assets::procedural::{generate_sphere, generate_plane, generate_box};

use crate::render::colors::*;
use crate::resources::BilliardConfig;

const SHADER_SOURCE: &str = include_str!("../../../../shaders/pbr.wgsl");
const TONEMAP_SHADER: &str = include_str!("../../../../shaders/tonemap.wgsl");

/// Converts MeshData to PbrVertex + u32 indices.
fn mesh_data_to_pbr(mesh: &anvilkit_assets::mesh::MeshData) -> (Vec<PbrVertex>, Vec<u32>) {
    let verts: Vec<PbrVertex> = (0..mesh.positions.len())
        .map(|i| PbrVertex {
            position: mesh.positions[i].into(),
            normal: mesh.normals[i].into(),
            texcoord: mesh.texcoords[i].into(),
            tangent: mesh.tangents[i],
        })
        .collect();
    (verts, mesh.indices.clone())
}

/// All GPU handles produced by scene init.
pub struct SceneGpu {
    pub sphere_mesh: MeshHandle,
    pub plane_mesh: MeshHandle,
    pub cushion_meshes: [MeshHandle; 4], // +x, -x, +z, -z
    pub ball_materials: [MaterialHandle; 16],
    pub table_material: MaterialHandle,
    pub cushion_material: MaterialHandle,
    pub pipeline_handle: PipelineHandle,
    pub scene_ub: wgpu::Buffer,
    pub scene_bg: wgpu::BindGroup,
    pub depth_view: wgpu::TextureView,
    pub hdr_view: wgpu::TextureView,
    pub hdr_msaa_view: wgpu::TextureView,
    pub tonemap_pipeline: wgpu::RenderPipeline,
    pub tonemap_bg: wgpu::BindGroup,
    pub ibl_bg: wgpu::BindGroup,
}

pub fn init_scene(device: &RenderDevice, format: wgpu::TextureFormat, w: u32, h: u32, assets: &mut RenderAssets, config: &BilliardConfig) -> SceneGpu {
    // Scene uniform buffer
    let initial = PbrSceneUniform::default();
    let ub = create_uniform_buffer(device, "Billiard Scene UB", bytemuck::bytes_of(&initial));
    let scene_bgl = device.device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Billiard Scene BGL"),
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
        label: Some("Billiard Scene BG"),
        layout: &scene_bgl,
        entries: &[wgpu::BindGroupEntry { binding: 0, resource: ub.as_entire_binding() }],
    });

    let (_, depth_view) = create_depth_texture_msaa(device, w, h, "Billiard Depth");

    // Textures helpers
    let make_tex = |data: &[u8; 4], label: &str, srgb: bool| -> wgpu::TextureView {
        if srgb {
            create_texture(device, 1, 1, data, label).1
        } else {
            create_texture_linear(device, 1, 1, data, label).1
        }
    };
    let flat_normal = make_tex(&[128, 128, 255, 255], "FlatNormal", false);
    let white_lin = make_tex(&[255, 255, 255, 255], "WhiteLin", false);
    let white_tex = make_tex(&[255, 255, 255, 255], "WhiteTex", true);
    let sampler = create_sampler(device, "Billiard Sampler");

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

    let mat_bgl = device.device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Mat BGL"),
        entries: &[
            tex_entry(0), tex_entry(1), tex_entry(2), tex_entry(3), tex_entry(4),
            wgpu::BindGroupLayoutEntry {
                binding: 5,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    });

    let make_mat_bg = |bc: &wgpu::TextureView, label: &str| -> wgpu::BindGroup {
        device.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(label),
            layout: &mat_bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(bc) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&flat_normal) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&white_lin) },
                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(&white_lin) },
                wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::TextureView(&white_tex) },
                wgpu::BindGroupEntry { binding: 5, resource: wgpu::BindingResource::Sampler(&sampler) },
            ],
        })
    };

    // IBL + Shadow (group 2)
    let brdf_data = generate_brdf_lut(256);
    let (_, brdf_view) = create_texture_linear(device, 256, 256, &brdf_data, "BRDF LUT");
    let (_, shadow_view) = create_shadow_map(device, SHADOW_MAP_SIZE, "Shadow Map");
    let shadow_samp = create_shadow_sampler(device, "Shadow Sampler");
    let ibl_bgl = device.device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("IBL+Shadow BGL"),
        entries: &[
            tex_entry(0),
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Depth,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                count: None,
            },
        ],
    });
    let ibl_bg = device.device().create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("IBL+Shadow BG"),
        layout: &ibl_bgl,
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&brdf_view) },
            wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sampler) },
            wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&shadow_view) },
            wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::Sampler(&shadow_samp) },
        ],
    });

    // PBR pipeline (shared)
    let build_pbr_pipeline = |device: &RenderDevice,
                               s_bgl: &wgpu::BindGroupLayout,
                               m_bgl: &wgpu::BindGroupLayout,
                               i_bgl: &wgpu::BindGroupLayout|
        -> wgpu::RenderPipeline {
        let pipeline_layout = device.device().create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Billiard PBR Layout"),
            bind_group_layouts: &[s_bgl, m_bgl, i_bgl],
            push_constant_ranges: &[],
        });
        let shader = device.device().create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Billiard PBR Shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER_SOURCE.into()),
        });
        device.device().create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Billiard PBR Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[PbrVertex::layout()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
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
                count: MSAA_SAMPLE_COUNT,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        })
    };

    let pbr_pipeline = build_pbr_pipeline(device, &scene_bgl, &mat_bgl, &ibl_bgl);
    let pipeline_handle = assets.register_pipeline(pbr_pipeline);

    // Meshes — sizes from config
    let hw = config.table_half_width;
    let hd = config.table_half_depth;
    let ball_r = config.ball_radius;

    let sphere_data = generate_sphere(ball_r, 32, 24);
    let (sv, si) = mesh_data_to_pbr(&sphere_data);
    let sphere_mesh = assets.upload_mesh_u32(device, &sv, &si, "Sphere");

    let plane_data = generate_plane(hw * 2.0, hd * 2.0);
    let (pv, pi) = mesh_data_to_pbr(&plane_data);
    let plane_mesh = assets.upload_mesh_u32(device, &pv, &pi, "Table Plane");

    // Cushion boxes: 4 rails around the table
    let rail_h = 0.15;
    let rail_thick = 0.15;
    // X rails (left/right): run along Z axis
    let box_x = generate_box([rail_thick, rail_h, hd + rail_thick]);
    let (bxv, bxi) = mesh_data_to_pbr(&box_x);
    let cushion_mesh_x = assets.upload_mesh_u32(device, &bxv, &bxi, "Cushion X");
    // Z rails (near/far): run along X axis
    let box_z = generate_box([hw + rail_thick, rail_h, rail_thick]);
    let (bzv, bzi) = mesh_data_to_pbr(&box_z);
    let cushion_mesh_z = assets.upload_mesh_u32(device, &bzv, &bzi, "Cushion Z");

    let cushion_meshes = [cushion_mesh_x, cushion_mesh_x, cushion_mesh_z, cushion_mesh_z];

    // Ball materials (16)
    let mut ball_materials = [MaterialHandle(0); 16];
    for i in 0..16 {
        let bc_tex = make_tex(&BALL_COLORS[i], &format!("Ball{} BC", i), true);
        let bg = make_mat_bg(&bc_tex, &format!("Ball{} Mat", i));
        ball_materials[i] = assets.create_material_with_pipeline(pipeline_handle, bg);
    }

    // Table material
    let table_bc = make_tex(&TABLE_COLOR, "Table BC", true);
    let table_bg = make_mat_bg(&table_bc, "Table Mat");
    let table_material = assets.create_material_with_pipeline(pipeline_handle, table_bg);

    // Cushion material
    let cushion_bc = make_tex(&CUSHION_COLOR, "Cushion BC", true);
    let cushion_bg = make_mat_bg(&cushion_bc, "Cushion Mat");
    let cushion_material = assets.create_material_with_pipeline(pipeline_handle, cushion_bg);

    // HDR + Tonemap
    let (_, hdr_view) = create_hdr_render_target(device, w, h, "Billiard HDR RT");
    let (_, hdr_msaa_view) = create_hdr_msaa_texture(device, w, h, "Billiard HDR MSAA");
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
    let tm_bg = device.device().create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Tonemap BG"),
        layout: &tm_bgl,
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&hdr_view) },
            wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sampler) },
        ],
    });
    let tm_pipe = RenderPipelineBuilder::new()
        .with_vertex_shader(TONEMAP_SHADER)
        .with_fragment_shader(TONEMAP_SHADER)
        .with_format(format)
        .with_vertex_layouts(vec![])
        .with_bind_group_layouts(vec![tm_bgl])
        .with_label("Billiard Tonemap Pipeline")
        .build(device)
        .expect("Failed to create tonemap pipeline");

    SceneGpu {
        sphere_mesh,
        plane_mesh,
        cushion_meshes,
        ball_materials,
        table_material,
        cushion_material,
        pipeline_handle,
        scene_ub: ub,
        scene_bg,
        depth_view,
        hdr_view,
        hdr_msaa_view,
        tonemap_pipeline: tm_pipe.into_pipeline(),
        tonemap_bg: tm_bg,
        ibl_bg,
    }
}
