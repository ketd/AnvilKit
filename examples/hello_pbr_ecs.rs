//! # ECS PBR + 法线贴图 + HDR + IBL 环境光
//!
//! AnvilKit M6d 示例：完整 PBR 管线 + IBL 环境光。
//! BRDF LUT + hemisphere irradiance + split-sum specular。
//!
//! 运行: `cargo run -p anvilkit-render --example hello_pbr_ecs`

use anvilkit_render::prelude::*;
use anvilkit_render::renderer::{
    RenderPipelineBuilder, DEPTH_FORMAT, HDR_FORMAT,
    buffer::{PbrVertex, Vertex, create_uniform_buffer,
             create_depth_texture_msaa, create_hdr_render_target, create_hdr_msaa_texture,
             create_texture, create_texture_linear, create_sampler,
             create_shadow_map, create_shadow_sampler, SHADOW_MAP_SIZE, MSAA_SAMPLE_COUNT},
    assets::RenderAssets,
    draw::{ActiveCamera, DrawCommandList, SceneLights, DirectionalLight, PointLight, MaterialParams},
    state::GpuLight,
    ibl::generate_brdf_lut,
};
use anvilkit_render::plugin::CameraComponent;
use anvilkit_assets::gltf_loader::load_gltf_scene;

/// Cook-Torrance PBR + TBN Normal Mapping + IBL Ambient
const SHADER_SOURCE: &str = include_str!("../shaders/pbr.wgsl");

/// 后处理 WGSL 着色器：全屏三角形 + ACES Filmic Tone Mapping
const TONEMAP_SHADER: &str = include_str!("../shaders/tonemap.wgsl");

use anvilkit_render::window::pack_lights;

#[derive(Resource)]
struct FrameTime(std::time::Instant);

fn main() {
    env_logger::init();

    let scene = load_gltf_scene("assets/textured_sphere.glb")
        .expect("加载场景失败");

    println!("PBR ECS 场景: {} 顶点, metallic={}, roughness={}, normal_map={}",
        scene.mesh.vertex_count(),
        scene.material.metallic_factor,
        scene.material.roughness_factor,
        scene.material.normal_texture.is_some());

    let mesh_vertices: Vec<PbrVertex> = (0..scene.mesh.vertex_count())
        .map(|i| PbrVertex {
            position: scene.mesh.positions[i].into(),
            normal: scene.mesh.normals[i].into(),
            texcoord: scene.mesh.texcoords[i].into(),
            tangent: scene.mesh.tangents[i],
        })
        .collect();

    let mut app = App::new();
    app.add_plugins(RenderPlugin::new().with_window_config(
        WindowConfig::new()
            .with_title("AnvilKit - PBR + IBL (M6d)")
            .with_size(800, 600),
    ));
    app.insert_resource(FrameTime(std::time::Instant::now()));

    // Configure scene lights (directional + 2 point lights)
    app.insert_resource(SceneLights {
        directional: DirectionalLight {
            direction: glam::Vec3::new(-0.5, -0.8, 0.3).normalize(),
            color: glam::Vec3::new(1.0, 0.95, 0.9),
            intensity: 3.0,
        },
        point_lights: vec![
            PointLight {
                position: glam::Vec3::new(2.0, 1.5, -1.0),
                color: glam::Vec3::new(1.0, 0.3, 0.1),  // warm orange
                intensity: 8.0,
                range: 8.0,
            },
            PointLight {
                position: glam::Vec3::new(-2.0, 1.0, -1.5),
                color: glam::Vec3::new(0.1, 0.4, 1.0),  // cool blue
                intensity: 6.0,
                range: 8.0,
            },
        ],
        spot_lights: vec![],
    });

    let event_loop = EventLoop::new().unwrap();
    let config = WindowConfig::new()
        .with_title("AnvilKit - PBR + IBL (M6d)")
        .with_size(800, 600);

    event_loop.run_app(&mut PbrEcsApp {
        render_app: RenderApp::new(config),
        app,
        initialized: false,
        scene_uniform_buffer: None,
        scene_bind_group: None,
        depth_texture_view: None,
        hdr_texture_view: None,
        hdr_msaa_texture_view: None,
        tonemap_pipeline: None,
        tonemap_bind_group: None,
        ibl_bind_group: None,
        mesh_vertices,
        mesh_indices: scene.mesh.indices,
        material: scene.material,
    }).unwrap();
}

struct PbrEcsApp {
    render_app: RenderApp,
    app: App,
    initialized: bool,
    scene_uniform_buffer: Option<wgpu::Buffer>,
    scene_bind_group: Option<wgpu::BindGroup>,
    depth_texture_view: Option<wgpu::TextureView>,
    // HDR multi-pass + MSAA
    hdr_texture_view: Option<wgpu::TextureView>,
    hdr_msaa_texture_view: Option<wgpu::TextureView>,
    tonemap_pipeline: Option<wgpu::RenderPipeline>,
    tonemap_bind_group: Option<wgpu::BindGroup>,
    // IBL
    ibl_bind_group: Option<wgpu::BindGroup>,
    mesh_vertices: Vec<PbrVertex>,
    mesh_indices: Vec<u32>,
    material: anvilkit_assets::material::MaterialData,
}

impl PbrEcsApp {
    fn init_scene(&mut self) {
        if self.initialized { return; }
        let Some(device) = self.render_app.render_device() else { return };
        let Some(format) = self.render_app.surface_format() else { return };
        let (w, h) = self.render_app.window_state().size();

        // Scene uniform buffer (256 bytes = PbrSceneUniform)
        let initial = PbrSceneUniform::default();
        let ub = create_uniform_buffer(device, "PBR Scene UB", bytemuck::bytes_of(&initial));
        let scene_bgl = device.device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("PBR Scene BGL"),
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
            label: Some("PBR Scene BG"),
            layout: &scene_bgl,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: ub.as_entire_binding() }],
        });
        let (_, depth_view) = create_depth_texture_msaa(device, w, h, "Depth");

        // Material bind group (group 1: 5 textures + 1 shared sampler)
        let tex_entry = |t: &Option<anvilkit_assets::material::TextureData>, label, fallback: &[u8; 4], srgb: bool| {
            if let Some(ref tex) = t {
                if srgb { create_texture(device, tex.width, tex.height, &tex.data, label).1 }
                else { create_texture_linear(device, tex.width, tex.height, &tex.data, label).1 }
            } else {
                if srgb { create_texture(device, 1, 1, fallback, label).1 }
                else { create_texture_linear(device, 1, 1, fallback, label).1 }
            }
        };

        let base_color_view = tex_entry(&self.material.base_color_texture, "BaseColor", &[255,255,255,255], true);
        let normal_map_view = tex_entry(&self.material.normal_texture, "NormalMap", &[128,128,255,255], false);
        // MR fallback: white = use uniform metallic/roughness factors as-is (multiply by 1.0)
        let mr_view = tex_entry(&self.material.metallic_roughness_texture, "MR Tex", &[255,255,255,255], false);
        // AO fallback: white = no occlusion
        let ao_view = tex_entry(&self.material.occlusion_texture, "AO Tex", &[255,255,255,255], false);
        // Emissive fallback: black = no emission (factor * 0 = 0)
        let emissive_view = tex_entry(&self.material.emissive_texture, "Emissive Tex", &[255,255,255,255], true);

        let sampler = create_sampler(device, "Sampler");

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
                label: Some("Material BGL"),
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
        let mat_bg = device.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Material BG"),
            layout: &mat_bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&base_color_view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&normal_map_view) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&mr_view) },
                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(&ao_view) },
                wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::TextureView(&emissive_view) },
                wgpu::BindGroupEntry { binding: 5, resource: wgpu::BindingResource::Sampler(&sampler) },
            ],
        });

        // IBL + Shadow bind group (group 2: BRDF LUT + shadow map)
        let brdf_lut_data = generate_brdf_lut(256);
        let (_, brdf_lut_view) = create_texture_linear(device, 256, 256, &brdf_lut_data, "BRDF LUT");
        let (_, shadow_map_view) = create_shadow_map(device, SHADOW_MAP_SIZE, "Shadow Map");
        let shadow_sampler = create_shadow_sampler(device, "Shadow Sampler");

        let ibl_bgl = device.device().create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("IBL+Shadow BGL"),
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
                            view_dimension: wgpu::TextureViewDimension::D2,
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
        let ibl_bg = device.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("IBL+Shadow BG"),
            layout: &ibl_bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&brdf_lut_view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sampler) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&shadow_map_view) },
                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::Sampler(&shadow_sampler) },
            ],
        });

        // PBR Pipeline — renders to HDR_FORMAT with MSAA 4x
        let pipeline = RenderPipelineBuilder::new()
            .with_vertex_shader(SHADER_SOURCE)
            .with_fragment_shader(SHADER_SOURCE)
            .with_format(HDR_FORMAT)
            .with_vertex_layouts(vec![PbrVertex::layout()])
            .with_depth_format(DEPTH_FORMAT)
            .with_multisample_count(MSAA_SAMPLE_COUNT)
            .with_bind_group_layouts(vec![scene_bgl, mat_bgl, ibl_bgl])
            .with_label("PBR HDR MSAA Pipeline")
            .build(device)
            .expect("创建 PBR 管线失败");

        // HDR render target
        let (_, hdr_view) = create_hdr_render_target(device, w, h, "HDR RT");
        let (_, hdr_msaa_view) = create_hdr_msaa_texture(device, w, h, "HDR MSAA");

        // Tone mapping post-process pipeline
        let tonemap_bgl = device.device().create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("Tonemap BGL"),
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
            },
        );

        let tonemap_bg = device.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Tonemap BG"),
            layout: &tonemap_bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&hdr_view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sampler) },
            ],
        });

        let tonemap_pipeline = RenderPipelineBuilder::new()
            .with_vertex_shader(TONEMAP_SHADER)
            .with_fragment_shader(TONEMAP_SHADER)
            .with_format(format)
            .with_vertex_layouts(vec![])
            .with_bind_group_layouts(vec![tonemap_bgl])
            .with_label("Tonemap Pipeline")
            .build(device)
            .expect("创建 Tonemap 管线失败");

        // Upload mesh & create material via ECS RenderAssets
        let mut assets = self.app.world.resource_mut::<RenderAssets>();
        let mesh_handle = assets.upload_mesh_u32(device, &self.mesh_vertices, &self.mesh_indices, "Sphere");
        let mat_handle = assets.create_material(pipeline.into_pipeline(), mat_bg);

        // Spawn entity with PBR material params
        self.app.world.spawn((
            mesh_handle,
            mat_handle,
            MaterialParams {
                metallic: self.material.metallic_factor,
                roughness: self.material.roughness_factor,
                normal_scale: self.material.normal_scale,
                emissive_factor: self.material.emissive_factor,
            },
            Transform::default(),
        ));

        // Spawn camera
        let eye = glam::Vec3::new(0.0, 1.0, -3.0);
        let look_dir = (glam::Vec3::ZERO - eye).normalize();
        let cam_rotation = glam::Quat::from_rotation_arc(glam::Vec3::Z, look_dir);

        self.app.world.spawn((
            CameraComponent { fov: 45.0, near: 0.1, far: 100.0, is_active: true, aspect_ratio: w as f32 / h.max(1) as f32 },
            Transform::from_xyz(eye.x, eye.y, eye.z).with_rotation(cam_rotation),
        ));

        self.scene_uniform_buffer = Some(ub);
        self.scene_bind_group = Some(scene_bg);
        self.depth_texture_view = Some(depth_view);
        self.hdr_texture_view = Some(hdr_view);
        self.hdr_msaa_texture_view = Some(hdr_msaa_view);
        self.tonemap_pipeline = Some(tonemap_pipeline.into_pipeline());
        self.tonemap_bind_group = Some(tonemap_bg);
        self.ibl_bind_group = Some(ibl_bg);
        self.initialized = true;
        println!("HDR PBR + IBL 场景初始化完成！");
    }

    fn render_frame(&self) {
        let Some(device) = self.render_app.render_device() else { return };
        let Some(ub) = &self.scene_uniform_buffer else { return };
        let Some(scene_bg) = &self.scene_bind_group else { return };
        let Some(depth_view) = &self.depth_texture_view else { return };
        let Some(hdr_view) = &self.hdr_texture_view else { return };
        let Some(hdr_msaa_view) = &self.hdr_msaa_texture_view else { return };
        let Some(tonemap_pipeline) = &self.tonemap_pipeline else { return };
        let Some(tonemap_bg) = &self.tonemap_bind_group else { return };
        let Some(ibl_bg) = &self.ibl_bind_group else { return };
        let Some(active_camera) = self.app.world.get_resource::<ActiveCamera>() else { return };
        let Some(draw_list) = self.app.world.get_resource::<DrawCommandList>() else { return };
        let Some(render_assets) = self.app.world.get_resource::<RenderAssets>() else { return };

        if draw_list.commands.is_empty() { return; }

        let Some(frame) = self.render_app.get_current_frame() else { return };
        let swapchain_view = frame.texture.create_view(&Default::default());

        let view_proj = active_camera.view_proj;
        let camera_pos = active_camera.camera_pos;

        let default_lights = SceneLights::default();
        let scene_lights = self.app.world.get_resource::<SceneLights>()
            .unwrap_or(&default_lights);
        let light = &scene_lights.directional;

        // Pack all lights into GPU array
        let (gpu_lights, light_count) = pack_lights(scene_lights);

        // Compute light-space matrix for shadow mapping
        let light_dir = light.direction.normalize();
        let light_pos = -light_dir * 15.0;
        let light_view = glam::Mat4::look_at_lh(light_pos, glam::Vec3::ZERO, glam::Vec3::Y);
        let light_proj = glam::Mat4::orthographic_lh(-10.0, 10.0, -10.0, 10.0, 0.1, 30.0);
        let shadow_view_proj = light_proj * light_view;

        // === Pass 1: Scene → HDR render target ===
        for (i, cmd) in draw_list.commands.iter().enumerate() {
            let Some(gpu_mesh) = render_assets.get_mesh(&cmd.mesh) else { continue };
            let Some(gpu_mat) = render_assets.get_material(&cmd.material) else { continue };

            let model = cmd.model_matrix;
            let normal_matrix = model.inverse().transpose();

            let uniform = PbrSceneUniform {
                model: model.to_cols_array_2d(),
                view_proj: view_proj.to_cols_array_2d(),
                normal_matrix: normal_matrix.to_cols_array_2d(),
                camera_pos: [camera_pos.x, camera_pos.y, camera_pos.z, 0.0],
                light_dir: [light.direction.x, light.direction.y, light.direction.z, 0.0],
                light_color: [light.color.x, light.color.y, light.color.z, light.intensity],
                material_params: [cmd.metallic, cmd.roughness, cmd.normal_scale, light_count as f32],
                lights: gpu_lights,
                shadow_view_proj: shadow_view_proj.to_cols_array_2d(),
                emissive_factor: [cmd.emissive_factor[0], cmd.emissive_factor[1], cmd.emissive_factor[2], 0.0],
            };
            device.queue().write_buffer(ub, 0, bytemuck::bytes_of(&uniform));

            let mut encoder = device.device().create_command_encoder(
                &wgpu::CommandEncoderDescriptor { label: Some("HDR Scene Encoder") },
            );

            {
                let color_load = if i == 0 {
                    wgpu::LoadOp::Clear(wgpu::Color { r: 0.15, g: 0.3, b: 0.6, a: 1.0 })
                } else {
                    wgpu::LoadOp::Load
                };
                let depth_load = if i == 0 {
                    wgpu::LoadOp::Clear(1.0)
                } else {
                    wgpu::LoadOp::Load
                };

                let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("HDR MSAA Scene Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: hdr_msaa_view,
                        resolve_target: Some(hdr_view),
                        ops: wgpu::Operations { load: color_load, store: wgpu::StoreOp::Discard },
                    })],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: depth_view,
                        depth_ops: Some(wgpu::Operations { load: depth_load, store: wgpu::StoreOp::Store }),
                        stencil_ops: None,
                    }),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

                let pipeline = render_assets.get_pipeline(&gpu_mat.pipeline_handle).unwrap();
                rp.set_pipeline(pipeline);
                rp.set_bind_group(0, scene_bg, &[]);
                rp.set_bind_group(1, &gpu_mat.bind_group, &[]);
                rp.set_bind_group(2, ibl_bg, &[]);
                rp.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
                rp.set_index_buffer(gpu_mesh.index_buffer.slice(..), gpu_mesh.index_format);
                rp.draw_indexed(0..gpu_mesh.index_count, 0, 0..1);
            }

            device.queue().submit(std::iter::once(encoder.finish()));
        }

        // === Pass 2: Tone mapping HDR → Swapchain ===
        {
            let mut encoder = device.device().create_command_encoder(
                &wgpu::CommandEncoderDescriptor { label: Some("Tonemap Encoder") },
            );

            {
                let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Tonemap Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &swapchain_view,
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

                rp.set_pipeline(tonemap_pipeline);
                rp.set_bind_group(0, tonemap_bg, &[]);
                rp.draw(0..3, 0..1); // Fullscreen triangle, 3 vertices
            }

            device.queue().submit(std::iter::once(encoder.finish()));
        }

        frame.present();
    }
}

impl ApplicationHandler for PbrEcsApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.render_app.resumed(event_loop);
        self.init_scene();
    }

    fn window_event(&mut self, el: &ActiveEventLoop, wid: WindowId, ev: WindowEvent) {
        match &ev {
            WindowEvent::Resized(new_size) if self.initialized && new_size.width > 0 && new_size.height > 0 => {
                if let Some(device) = self.render_app.render_device() {
                    let (_, depth_view) = create_depth_texture_msaa(device, new_size.width, new_size.height, "Depth");
                    self.depth_texture_view = Some(depth_view);

                    // Recreate HDR RT and tonemap bind group at new size
                    let (_, hdr_view) = create_hdr_render_target(device, new_size.width, new_size.height, "HDR RT");
                    let sampler = create_sampler(device, "Sampler");

                    // Need to recreate bind group since HDR texture view changed
                    if let Some(tonemap_bg) = &self.tonemap_bind_group {
                        let layout = device.device().create_bind_group_layout(
                            &wgpu::BindGroupLayoutDescriptor {
                                label: Some("Tonemap BGL"),
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
                            },
                        );
                        let new_bg = device.device().create_bind_group(&wgpu::BindGroupDescriptor {
                            label: Some("Tonemap BG"),
                            layout: &layout,
                            entries: &[
                                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&hdr_view) },
                                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sampler) },
                            ],
                        });
                        let _ = tonemap_bg; // old bind group dropped
                        self.tonemap_bind_group = Some(new_bg);
                    }
                    self.hdr_texture_view = Some(hdr_view);
                    let (_, hdr_msaa_view) = create_hdr_msaa_texture(device, new_size.width, new_size.height, "HDR MSAA");
                    self.hdr_msaa_texture_view = Some(hdr_msaa_view);
                }
            }
            WindowEvent::RedrawRequested if self.initialized => {
                self.render_frame();
                return;
            }
            _ => {}
        }
        self.render_app.window_event(el, wid, ev);
    }

    fn device_event(&mut self, el: &ActiveEventLoop, did: winit::event::DeviceId, ev: winit::event::DeviceEvent) {
        self.render_app.device_event(el, did, ev);
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // Rotate the model
        if let Some(frame_time) = self.app.world.get_resource::<FrameTime>() {
            let t = frame_time.0.elapsed().as_secs_f32();
            // Update all transforms with rotation
            for mut transform in self.app.world.query::<&mut Transform>()
                .iter_mut(&mut self.app.world)
            {
                // Only rotate entities that have MaterialParams (not camera)
                // Simple check: camera is at non-origin position
                if transform.translation.length() < 0.01 {
                    transform.rotation = glam::Quat::from_rotation_y(t * 0.5);
                }
            }
        }

        self.app.update();
        if let Some(window) = self.render_app.window() {
            window.request_redraw();
        }
    }
}
