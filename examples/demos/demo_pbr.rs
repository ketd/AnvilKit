//! # Demo: PBR Materials — 5x5 grid of spheres with varying metallic/roughness
//!
//! Demonstrates PBR material parameters by rendering a grid of spheres where
//! metallic varies by column (0.0 to 1.0) and roughness varies by row (0.1 to 1.0).
//!
//! ```bash
//! cargo run -p anvilkit-render --features capture --example demo_pbr -- \
//!     --capture-dir /tmp/frames/pbr --capture-frames 120
//! ```

use std::time::Instant;
use anvilkit_render::prelude::*;
use anvilkit_render::window::pack_lights;
use anvilkit_render::renderer::{
    RenderPipelineBuilder, DEPTH_FORMAT, HDR_FORMAT,
    buffer::{PbrVertex, Vertex, create_uniform_buffer,
             create_depth_texture_msaa, create_hdr_render_target, create_hdr_msaa_texture,
             create_texture, create_texture_linear, create_sampler,
             create_csm_shadow_map, create_shadow_sampler, SHADOW_MAP_SIZE, MSAA_SAMPLE_COUNT},
    assets::RenderAssets,
    draw::{ActiveCamera, DrawCommandList, SceneLights, DirectionalLight, PointLight, MaterialParams},
    ibl::generate_brdf_lut,
    bloom::{BloomResources, BloomSettings},
};
use anvilkit_render::plugin::CameraComponent;
use anvilkit_render::renderer::capture::{CaptureState, CaptureResources, save_png};
use anvilkit_assets::procedural::generate_sphere;

const SHADER_SOURCE: &str = include_str!("../../shaders/pbr.wgsl");
const TONEMAP_SHADER: &str = include_str!("../../shaders/tonemap.wgsl");

#[derive(Resource)]
struct FrameTime(Instant);

/// Per-sphere material info: metallic, roughness, and its bind group.
struct SphereMaterial {
    metallic: f32,
    roughness: f32,
    bind_group: wgpu::BindGroup,
}

fn main() {
    env_logger::init();

    let capture = CaptureState::from_args();

    // Generate sphere mesh data
    let sphere_mesh = generate_sphere(0.4, 24, 16);
    let mesh_vertices: Vec<PbrVertex> = (0..sphere_mesh.vertex_count())
        .map(|i| PbrVertex {
            position: sphere_mesh.positions[i].into(),
            normal: sphere_mesh.normals[i].into(),
            texcoord: sphere_mesh.texcoords[i].into(),
            tangent: sphere_mesh.tangents[i],
        })
        .collect();
    let mesh_indices = sphere_mesh.indices;

    let mut app = App::new();
    app.add_plugins(RenderPlugin::new().with_window_config(
        WindowConfig::new()
            .with_title("Demo: PBR Materials")
            .with_size(640, 480),
    ));
    app.insert_resource(FrameTime(Instant::now()));
    app.insert_resource(SceneLights {
        directional: DirectionalLight {
            direction: glam::Vec3::new(-0.4, -0.7, 0.5).normalize(),
            color: glam::Vec3::new(1.0, 0.95, 0.85),
            intensity: 4.0,
        },
        point_lights: vec![
            PointLight { position: glam::Vec3::new(3.0, 3.0, -3.0), color: glam::Vec3::new(1.0, 0.9, 0.8), intensity: 10.0, range: 15.0 },
        ],
        spot_lights: vec![],
    });

    let event_loop = EventLoop::new().unwrap();
    let config = WindowConfig::new().with_title("Demo: PBR Materials").with_size(640, 480);

    event_loop.run_app(&mut DemoPbrApp {
        render_app: RenderApp::new(config),
        app,
        capture,
        initialized: false,
        capture_resources: None,
        scene_ub: None,
        scene_bg: None,
        depth_view: None,
        hdr_view: None,
        hdr_msaa_view: None,
        tonemap_pipeline: None,
        tonemap_bg: None,
        ibl_bg: None,
        bloom: None,
        mesh_vertices,
        mesh_indices,
    }).unwrap();
}

struct DemoPbrApp {
    render_app: RenderApp,
    app: App,
    capture: CaptureState,
    initialized: bool,
    capture_resources: Option<CaptureResources>,
    scene_ub: Option<wgpu::Buffer>,
    scene_bg: Option<wgpu::BindGroup>,
    depth_view: Option<wgpu::TextureView>,
    hdr_view: Option<wgpu::TextureView>,
    hdr_msaa_view: Option<wgpu::TextureView>,
    tonemap_pipeline: Option<wgpu::RenderPipeline>,
    tonemap_bg: Option<wgpu::BindGroup>,
    ibl_bg: Option<wgpu::BindGroup>,
    bloom: Option<BloomResources>,
    mesh_vertices: Vec<PbrVertex>,
    mesh_indices: Vec<u32>,
}

impl DemoPbrApp {
    fn init_scene(&mut self) {
        if self.initialized { return; }
        let Some(device) = self.render_app.render_device() else { return };
        let Some(format) = self.render_app.surface_format() else { return };
        let (w, h) = self.render_app.window_state().size();

        // Scene uniform
        let initial = PbrSceneUniform::default();
        let ub = create_uniform_buffer(device, "Scene UB", bytemuck::bytes_of(&initial));
        let scene_bgl = device.device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Scene BGL"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0, visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                count: None,
            }],
        });
        let scene_bg = device.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Scene BG"), layout: &scene_bgl,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: ub.as_entire_binding() }],
        });
        let (_, depth_view) = create_depth_texture_msaa(device, w, h, "Depth");

        // Shared textures
        let sampler = create_sampler(device, "Sampler");
        let flat_normal = create_texture_linear(device, 1, 1, &[128, 128, 255, 255], "FlatNormal").1;
        let white_ao = create_texture_linear(device, 1, 1, &[255, 255, 255, 255], "WhiteAO").1;
        let white_bc = create_texture(device, 1, 1, &[255, 255, 255, 255], "WhiteBC").1;
        let black_emissive = create_texture(device, 1, 1, &[0, 0, 0, 255], "BlackEmissive").1;

        let tex_entry = |b: u32| wgpu::BindGroupLayoutEntry {
            binding: b, visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2, multisampled: false,
            }, count: None,
        };
        let mat_bgl = device.device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Mat BGL"),
            entries: &[tex_entry(0), tex_entry(1), tex_entry(2), tex_entry(3), tex_entry(4),
                wgpu::BindGroupLayoutEntry { binding: 5, visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None }],
        });

        // Create 25 material bind groups BEFORE consuming mat_bgl in the pipeline builder
        let mut sphere_materials: Vec<SphereMaterial> = Vec::with_capacity(25);
        for row in 0..5u32 {
            for col in 0..5u32 {
                let metallic = col as f32 / 4.0;
                let roughness = row as f32 / 4.0 * 0.9 + 0.1;

                let mr_view = create_texture_linear(device, 1, 1,
                    &[(metallic * 255.0) as u8, (roughness * 255.0) as u8, 0, 255], "MR").1;

                let bg = device.device().create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Sphere Mat BG"), layout: &mat_bgl,
                    entries: &[
                        wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&white_bc) },
                        wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&flat_normal) },
                        wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&mr_view) },
                        wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(&white_ao) },
                        wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::TextureView(&black_emissive) },
                        wgpu::BindGroupEntry { binding: 5, resource: wgpu::BindingResource::Sampler(&sampler) },
                    ],
                });
                sphere_materials.push(SphereMaterial { metallic, roughness, bind_group: bg });
            }
        }

        // IBL + Shadow
        let brdf_data = generate_brdf_lut(256);
        let (_, brdf_view) = create_texture_linear(device, 256, 256, &brdf_data, "BRDF LUT");
        let (_shadow_tex, shadow_view, _shadow_cascade_views) = create_csm_shadow_map(device, SHADOW_MAP_SIZE, 3, "Shadow Map");
        let shadow_samp = create_shadow_sampler(device, "Shadow Sampler");
        let ibl_bgl = device.device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("IBL+Shadow BGL"),
            entries: &[tex_entry(0),
                wgpu::BindGroupLayoutEntry { binding: 1, visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None },
                wgpu::BindGroupLayoutEntry { binding: 2, visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2Array, multisampled: false }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 3, visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison), count: None }],
        });
        let ibl_bg = device.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("IBL+Shadow BG"), layout: &ibl_bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&brdf_view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sampler) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&shadow_view) },
                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::Sampler(&shadow_samp) },
            ],
        });

        // PBR pipeline (shared by all 25 spheres)
        let pipeline = RenderPipelineBuilder::new()
            .with_vertex_shader(SHADER_SOURCE)
            .with_fragment_shader(SHADER_SOURCE)
            .with_format(HDR_FORMAT)
            .with_vertex_layouts(vec![PbrVertex::layout()])
            .with_depth_format(DEPTH_FORMAT)
            .with_multisample_count(MSAA_SAMPLE_COUNT)
            .with_bind_group_layouts(vec![scene_bgl, mat_bgl, ibl_bgl])
            .with_label("PBR Pipeline")
            .build(device).expect("Failed to create PBR pipeline");

        // HDR + Bloom + Tonemap
        let (_, hdr_view) = create_hdr_render_target(device, w, h, "HDR RT");
        let (_, hdr_msaa_view) = create_hdr_msaa_texture(device, w, h, "HDR MSAA");
        let bloom = BloomResources::new(device, w, h, 5);
        let bloom_view = if bloom.mip_views.is_empty() { &hdr_view } else { &bloom.mip_views[0] };

        let tm_bgl = device.device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Tonemap BGL"),
            entries: &[tex_entry(0),
                wgpu::BindGroupLayoutEntry { binding: 1, visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None },
                tex_entry(2)],
        });
        let tm_bg = device.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Tonemap BG"), layout: &tm_bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&hdr_view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sampler) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(bloom_view) },
            ],
        });
        let tm_bgl2 = device.device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("TM BGL2"),
            entries: &[tex_entry(0),
                wgpu::BindGroupLayoutEntry { binding: 1, visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None },
                tex_entry(2)],
        });
        let tm_pipe = RenderPipelineBuilder::new()
            .with_vertex_shader(TONEMAP_SHADER).with_fragment_shader(TONEMAP_SHADER)
            .with_format(format).with_vertex_layouts(vec![])
            .with_bind_group_layouts(vec![tm_bgl2]).with_label("Tonemap")
            .build(device).expect("Tonemap pipeline");

        // Upload mesh and register pipeline, collect all handles before spawning
        let (mesh_h, sphere_handles) = {
            let mut assets = self.app.world.resource_mut::<RenderAssets>();
            let pipeline_handle = assets.register_pipeline(pipeline.into_pipeline());
            let mesh_h = assets.upload_mesh_u32(device, &self.mesh_vertices, &self.mesh_indices, "Sphere");

            let mut handles = Vec::with_capacity(25);
            for row in 0..5u32 {
                for col in 0..5u32 {
                    let sm = sphere_materials.remove(0);
                    let mat_h = assets.create_material_with_pipeline(pipeline_handle, sm.bind_group);
                    let x = col as f32 * 1.0 - 2.0;
                    let z = row as f32 * 1.0 - 2.0;
                    handles.push((mat_h, sm.metallic, sm.roughness, x, z));
                }
            }
            (mesh_h, handles)
        }; // assets dropped here

        // Spawn 5x5 grid of spheres
        for (mat_h, metallic, roughness, x, z) in sphere_handles {
            self.app.world.spawn((
                mesh_h, mat_h,
                MaterialParams {
                    metallic,
                    roughness,
                    normal_scale: 1.0,
                    emissive_factor: [0.0, 0.0, 0.0],
                },
                Transform::from_xyz(x, 0.0, z),
                GlobalTransform(glam::Mat4::from_translation(glam::Vec3::new(x, 0.0, z))),
            ));
        }

        // Camera
        let eye = glam::Vec3::new(0.0, 2.0, -5.0);
        let look_dir = (glam::Vec3::ZERO - eye).normalize();
        let cam_rot = glam::Quat::from_rotation_arc(glam::Vec3::Z, look_dir);
        self.app.world.spawn((
            CameraComponent { fov: 45.0, near: 0.1, far: 100.0, is_active: true, aspect_ratio: w as f32 / h.max(1) as f32 },
            Transform::from_xyz(eye.x, eye.y, eye.z).with_rotation(cam_rot),
            GlobalTransform::default(),
        ));

        // Capture resources
        self.capture_resources = Some(CaptureResources::new(device.device(), w, h, format));

        self.scene_ub = Some(ub);
        self.scene_bg = Some(scene_bg);
        self.depth_view = Some(depth_view);
        self.hdr_view = Some(hdr_view);
        self.hdr_msaa_view = Some(hdr_msaa_view);
        self.tonemap_pipeline = Some(tm_pipe.into_pipeline());
        self.tonemap_bg = Some(tm_bg);
        self.ibl_bg = Some(ibl_bg);
        self.bloom = Some(bloom);
        self.initialized = true;
        println!("Demo PBR initialized (capture: {})", self.capture.recording);
    }

    fn render_frame(&mut self) {
        let Some(device) = self.render_app.render_device() else { return };
        let Some(ub) = &self.scene_ub else { return };
        let Some(scene_bg) = &self.scene_bg else { return };
        let Some(depth_view) = &self.depth_view else { return };
        let Some(hdr_view) = &self.hdr_view else { return };
        let Some(hdr_msaa_view) = &self.hdr_msaa_view else { return };
        let Some(tm_pipe) = &self.tonemap_pipeline else { return };
        let Some(tm_bg) = &self.tonemap_bg else { return };
        let Some(ibl_bg) = &self.ibl_bg else { return };
        let Some(cam) = self.app.world.get_resource::<ActiveCamera>() else { return };
        let Some(dl) = self.app.world.get_resource::<DrawCommandList>() else { return };
        let Some(ra) = self.app.world.get_resource::<RenderAssets>() else { return };
        let draw_list_empty = dl.commands.is_empty();

        let Some(frame) = self.render_app.get_current_frame() else { return };
        let swapchain = frame.texture.create_view(&Default::default());

        let def_lights = SceneLights::default();
        let lights = self.app.world.get_resource::<SceneLights>().unwrap_or(&def_lights);
        let (gpu_lights, lc) = pack_lights(lights);
        let ld = lights.directional.direction.normalize();
        let lp = -ld * 15.0;
        let lv = glam::Mat4::look_at_lh(lp, glam::Vec3::ZERO, glam::Vec3::Y);
        let lproj = glam::Mat4::orthographic_lh(-10.0, 10.0, -10.0, 10.0, 0.1, 30.0);
        let svp = lproj * lv;

        // Scene pass -> HDR MSAA
        for (i, cmd) in dl.commands.iter().enumerate() {
            let Some(gm) = ra.get_mesh(&cmd.mesh) else { continue };
            let Some(gmat) = ra.get_material(&cmd.material) else { continue };
            let m = cmd.model_matrix;
            let u = PbrSceneUniform {
                model: m.to_cols_array_2d(),
                view_proj: cam.view_proj.to_cols_array_2d(),
                normal_matrix: m.inverse().transpose().to_cols_array_2d(),
                camera_pos: [cam.camera_pos.x, cam.camera_pos.y, cam.camera_pos.z, 0.0],
                light_dir: [ld.x, ld.y, ld.z, 0.0],
                light_color: [lights.directional.color.x, lights.directional.color.y, lights.directional.color.z, lights.directional.intensity],
                material_params: [cmd.metallic, cmd.roughness, cmd.normal_scale, lc as f32],
                lights: gpu_lights,
                cascade_view_projs: [svp.to_cols_array_2d(); 3],
                cascade_splits: [10.0, 30.0, 100.0, 1.0 / 2048.0],
                emissive_factor: [cmd.emissive_factor[0], cmd.emissive_factor[1], cmd.emissive_factor[2], 3.0],
            };
            device.queue().write_buffer(ub, 0, bytemuck::bytes_of(&u));

            let mut enc = device.device().create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Scene") });
            {
                let cl = if i == 0 { wgpu::LoadOp::Clear(wgpu::Color { r: 0.02, g: 0.02, b: 0.04, a: 1.0 }) } else { wgpu::LoadOp::Load };
                let dl_op = if i == 0 { wgpu::LoadOp::Clear(1.0) } else { wgpu::LoadOp::Load };
                let mut rp = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Scene Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: hdr_msaa_view, resolve_target: Some(hdr_view),
                        ops: wgpu::Operations { load: cl, store: wgpu::StoreOp::Store },
                    })],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: depth_view,
                        depth_ops: Some(wgpu::Operations { load: dl_op, store: wgpu::StoreOp::Store }),
                        stencil_ops: None,
                    }),
                    timestamp_writes: None, occlusion_query_set: None,
                });
                let pipeline = ra.get_pipeline(&gmat.pipeline_handle).unwrap();
                rp.set_pipeline(pipeline);
                rp.set_bind_group(0, scene_bg, &[]);
                rp.set_bind_group(1, &gmat.bind_group, &[]);
                rp.set_bind_group(2, ibl_bg, &[]);
                rp.set_vertex_buffer(0, gm.vertex_buffer.slice(..));
                rp.set_index_buffer(gm.index_buffer.slice(..), gm.index_format);
                rp.draw_indexed(0..gm.index_count, 0, 0..1);
            }
            device.queue().submit(std::iter::once(enc.finish()));
        }

        // Bloom
        if let Some(ref bloom) = self.bloom {
            let mut enc = device.device().create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Bloom") });
            bloom.execute(device, &mut enc, hdr_view, &BloomSettings::default());
            device.queue().submit(std::iter::once(enc.finish()));
        }

        // Tonemap -> swapchain + capture
        {
            let mut enc = device.device().create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Tonemap") });
            {
                let mut rp = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Tonemap"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &swapchain, resolve_target: None,
                        ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store },
                    })],
                    depth_stencil_attachment: None, timestamp_writes: None, occlusion_query_set: None,
                });
                rp.set_pipeline(tm_pipe);
                rp.set_bind_group(0, tm_bg, &[]);
                rp.draw(0..3, 0..1);
            }

            if self.capture.should_capture() && !draw_list_empty {
                if let Some(ref cr) = self.capture_resources {
                    {
                        let mut rp = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some("Capture Tonemap"),
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: &cr.capture_view, resolve_target: None,
                                ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store },
                            })],
                            depth_stencil_attachment: None, timestamp_writes: None, occlusion_query_set: None,
                        });
                        rp.set_pipeline(tm_pipe);
                        rp.set_bind_group(0, tm_bg, &[]);
                        rp.draw(0..3, 0..1);
                    }
                    cr.encode_copy(&mut enc);
                }
            }

            device.queue().submit(std::iter::once(enc.finish()));
        }

        if self.capture.should_capture() && !draw_list_empty {
            if let Some(ref cr) = self.capture_resources {
                if let Some(path) = self.capture.current_output_path() {
                    let pixels = cr.read_pixels(device.device());
                    save_png(&pixels, cr.width, cr.height, &path);
                }
            }
            self.capture.on_frame_captured();
        }

        frame.present();
    }
}

impl ApplicationHandler for DemoPbrApp {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        self.render_app.resumed(el);
        self.init_scene();
    }

    fn window_event(&mut self, el: &ActiveEventLoop, wid: WindowId, ev: WindowEvent) {
        match &ev {
            WindowEvent::CloseRequested => el.exit(),
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

    fn about_to_wait(&mut self, el: &ActiveEventLoop) {
        if let Some(ft) = self.app.world.get_resource::<FrameTime>() {
            let t = ft.0.elapsed().as_secs_f32();
            let radius = 5.0;
            let height = 2.0;
            let speed = 0.3;
            let angle = t * speed + 1.2;
            let eye = glam::Vec3::new(
                angle.sin() * radius,
                height + (t * speed * 0.3).sin() * 0.2,
                angle.cos() * radius,
            );
            let target = glam::Vec3::ZERO;
            let look_dir = (target - eye).normalize();
            let cam_rot = glam::Quat::from_rotation_arc(glam::Vec3::Z, look_dir);
            for (cam, mut transform) in self.app.world.query::<(&CameraComponent, &mut Transform)>().iter_mut(&mut self.app.world) {
                if cam.is_active {
                    transform.translation = eye;
                    transform.rotation = cam_rot;
                }
            }
        }
        self.app.update();
        if let Some(w) = self.render_app.window() { w.request_redraw(); }

        if self.capture.exit_requested {
            println!("Capture complete ({} frames)", self.capture.frame_count);
            el.exit();
        }
    }
}
