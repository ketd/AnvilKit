//! # ECS 驱动的多物体渲染
//!
//! AnvilKit M5 里程碑示例：使用 ECS 系统渲染 3 个独立旋转的立方体。
//! 用户只需 spawn 实体（MeshHandle + MaterialHandle + Transform），
//! 渲染循环自动从 ECS World 中提取绘制命令并渲染。
//!
//! 运行: `cargo run -p anvilkit-render --example hello_ecs`

use anvilkit_render::prelude::*;
use anvilkit_render::renderer::{
    RenderPipelineBuilder, DEPTH_FORMAT,
    buffer::{ColorVertex, Vertex, create_uniform_buffer, create_depth_texture},
    assets::RenderAssets,
    draw::{ActiveCamera, DrawCommandList},
};
use anvilkit_render::plugin::CameraComponent;

/// WGSL 着色器：PbrSceneUniform 256 字节布局（仅使用 model + view_proj）
const SHADER_SOURCE: &str = r#"
struct SceneUniform {
    model: mat4x4<f32>,
    view_proj: mat4x4<f32>,
    normal_matrix: mat4x4<f32>,
    camera_pos: vec4<f32>,
    light_dir: vec4<f32>,
    light_color: vec4<f32>,
    material_params: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> scene: SceneUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = scene.view_proj * scene.model * vec4<f32>(in.position, 1.0);
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
"#;

/// 立方体顶点
const CUBE_VERTICES: &[ColorVertex] = &[
    ColorVertex { position: [-0.5, -0.5,  0.5], color: [1.0, 0.0, 0.0] },
    ColorVertex { position: [ 0.5, -0.5,  0.5], color: [1.0, 0.0, 0.0] },
    ColorVertex { position: [ 0.5,  0.5,  0.5], color: [1.0, 0.0, 0.0] },
    ColorVertex { position: [-0.5,  0.5,  0.5], color: [1.0, 0.0, 0.0] },
    ColorVertex { position: [-0.5, -0.5, -0.5], color: [0.0, 1.0, 0.0] },
    ColorVertex { position: [-0.5,  0.5, -0.5], color: [0.0, 1.0, 0.0] },
    ColorVertex { position: [ 0.5,  0.5, -0.5], color: [0.0, 1.0, 0.0] },
    ColorVertex { position: [ 0.5, -0.5, -0.5], color: [0.0, 1.0, 0.0] },
    ColorVertex { position: [-0.5,  0.5, -0.5], color: [0.0, 0.0, 1.0] },
    ColorVertex { position: [-0.5,  0.5,  0.5], color: [0.0, 0.0, 1.0] },
    ColorVertex { position: [ 0.5,  0.5,  0.5], color: [0.0, 0.0, 1.0] },
    ColorVertex { position: [ 0.5,  0.5, -0.5], color: [0.0, 0.0, 1.0] },
    ColorVertex { position: [-0.5, -0.5, -0.5], color: [1.0, 1.0, 0.0] },
    ColorVertex { position: [ 0.5, -0.5, -0.5], color: [1.0, 1.0, 0.0] },
    ColorVertex { position: [ 0.5, -0.5,  0.5], color: [1.0, 1.0, 0.0] },
    ColorVertex { position: [-0.5, -0.5,  0.5], color: [1.0, 1.0, 0.0] },
    ColorVertex { position: [ 0.5, -0.5, -0.5], color: [1.0, 0.0, 1.0] },
    ColorVertex { position: [ 0.5,  0.5, -0.5], color: [1.0, 0.0, 1.0] },
    ColorVertex { position: [ 0.5,  0.5,  0.5], color: [1.0, 0.0, 1.0] },
    ColorVertex { position: [ 0.5, -0.5,  0.5], color: [1.0, 0.0, 1.0] },
    ColorVertex { position: [-0.5, -0.5, -0.5], color: [0.0, 1.0, 1.0] },
    ColorVertex { position: [-0.5, -0.5,  0.5], color: [0.0, 1.0, 1.0] },
    ColorVertex { position: [-0.5,  0.5,  0.5], color: [0.0, 1.0, 1.0] },
    ColorVertex { position: [-0.5,  0.5, -0.5], color: [0.0, 1.0, 1.0] },
];

const CUBE_INDICES: &[u16] = &[
     0,  1,  2,   2,  3,  0,   4,  5,  6,   6,  7,  4,
     8,  9, 10,  10, 11,  8,  12, 13, 14,  14, 15, 12,
    16, 17, 18,  18, 19, 16,  20, 21, 22,  22, 23, 20,
];

#[derive(Component)]
struct RotationSpeed(f32);

#[derive(Resource)]
struct FrameTime(std::time::Instant);

fn main() {
    env_logger::init();

    let mut app = App::new();
    app.add_plugins(RenderPlugin::new().with_window_config(
        WindowConfig::new()
            .with_title("AnvilKit - ECS Multi-Object (M6a)")
            .with_size(800, 600),
    ));
    app.insert_resource(FrameTime(std::time::Instant::now()));
    app.add_systems(AnvilKitSchedule::Update, rotate_cubes);

    let event_loop = EventLoop::new().unwrap();
    let config = WindowConfig::new()
        .with_title("AnvilKit - ECS Multi-Object (M6a)")
        .with_size(800, 600);

    event_loop.run_app(&mut EcsApp {
        render_app: RenderApp::new(config),
        app,
        initialized: false,
        scene_uniform_buffer: None,
        scene_bind_group: None,
        depth_texture_view: None,
    }).unwrap();
}

fn rotate_cubes(
    frame_time: Res<FrameTime>,
    mut query: Query<(&mut Transform, &RotationSpeed)>,
) {
    let t = frame_time.0.elapsed().as_secs_f32();
    for (mut transform, speed) in query.iter_mut() {
        transform.rotation = glam::Quat::from_rotation_y(t * speed.0)
            * glam::Quat::from_rotation_x(t * speed.0 * 0.7);
    }
}

struct EcsApp {
    render_app: RenderApp,
    app: App,
    initialized: bool,
    scene_uniform_buffer: Option<wgpu::Buffer>,
    scene_bind_group: Option<wgpu::BindGroup>,
    depth_texture_view: Option<wgpu::TextureView>,
}

impl EcsApp {
    fn init_scene(&mut self) {
        if self.initialized { return; }
        let Some(device) = self.render_app.render_device() else { return };
        let Some(format) = self.render_app.surface_format() else { return };
        let (w, h) = self.render_app.window_state().size();

        // Scene uniform buffer (256 bytes = PbrSceneUniform)
        let initial = PbrSceneUniform::default();
        let ub = create_uniform_buffer(device, "Scene UB", bytemuck::bytes_of(&initial));
        let scene_bgl = device.device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Scene BGL"),
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
            label: Some("Scene BG"),
            layout: &scene_bgl,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: ub.as_entire_binding() }],
        });
        let (_, depth_view) = create_depth_texture(device, w, h, "Depth");

        // Empty material bind group (group 1 unused by this shader)
        let empty_bgl = device.device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Empty BGL"), entries: &[],
        });
        let empty_bg = device.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Empty BG"), layout: &empty_bgl, entries: &[],
        });

        // Pipeline
        let pipeline = RenderPipelineBuilder::new()
            .with_vertex_shader(SHADER_SOURCE)
            .with_fragment_shader(SHADER_SOURCE)
            .with_format(format)
            .with_vertex_layouts(vec![ColorVertex::layout()])
            .with_depth_format(DEPTH_FORMAT)
            .with_bind_group_layouts(vec![scene_bgl, empty_bgl])
            .with_label("ECS Pipeline")
            .build(device)
            .expect("Pipeline 创建失败");

        // Upload mesh & create material via ECS RenderAssets
        let mut assets = self.app.world.resource_mut::<RenderAssets>();
        let mesh = assets.upload_mesh(device, CUBE_VERTICES, CUBE_INDICES, "Cube");
        let mat = assets.create_material(pipeline.into_pipeline(), empty_bg);

        // Spawn 3 cubes + 1 camera
        self.app.world.spawn((mesh, mat, Transform::from_xyz(-2.5, 0.0, 0.0), RotationSpeed(1.0)));
        self.app.world.spawn((mesh, mat, Transform::from_xyz( 0.0, 0.0, 0.0), RotationSpeed(1.5)));
        self.app.world.spawn((mesh, mat, Transform::from_xyz( 2.5, 0.0, 0.0), RotationSpeed(0.7)));
        // 计算相机朝向原点的旋转（LH 坐标系，forward = +Z）
        let eye = glam::Vec3::new(0.0, 2.0, -6.0);
        let look_dir = (glam::Vec3::ZERO - eye).normalize();
        let cam_rotation = glam::Quat::from_rotation_arc(glam::Vec3::Z, look_dir);

        self.app.world.spawn((
            CameraComponent { fov: 45.0, near: 0.1, far: 100.0, is_active: true, aspect_ratio: w as f32 / h.max(1) as f32 },
            Transform::from_xyz(eye.x, eye.y, eye.z).with_rotation(cam_rotation),
        ));

        self.scene_uniform_buffer = Some(ub);
        self.scene_bind_group = Some(scene_bg);
        self.depth_texture_view = Some(depth_view);
        self.initialized = true;
        println!("ECS 场景初始化完成: 3 个旋转立方体");
    }

    /// 多物体渲染：遍历 DrawCommandList，per-object 独立 submit
    fn render_frame(&self) {
        let Some(device) = self.render_app.render_device() else { return };
        let Some(ub) = &self.scene_uniform_buffer else { return };
        let Some(scene_bg) = &self.scene_bind_group else { return };
        let Some(depth_view) = &self.depth_texture_view else { return };
        let Some(active_camera) = self.app.world.get_resource::<ActiveCamera>() else { return };
        let Some(draw_list) = self.app.world.get_resource::<DrawCommandList>() else { return };
        let Some(render_assets) = self.app.world.get_resource::<RenderAssets>() else { return };

        if draw_list.commands.is_empty() { return; }

        let Some(frame) = self.render_app.get_current_frame() else { return };
        let view = frame.texture.create_view(&Default::default());

        let view_proj = active_camera.view_proj;
        let camera_pos = active_camera.camera_pos;
        let clear_color = wgpu::Color { r: 0.1, g: 0.2, b: 0.3, a: 1.0 };

        for (i, cmd) in draw_list.commands.iter().enumerate() {
            let Some(gpu_mesh) = render_assets.get_mesh(&cmd.mesh) else { continue };
            let Some(gpu_mat) = render_assets.get_material(&cmd.material) else { continue };

            let model = cmd.model_matrix;
            let normal_matrix = model.inverse().transpose();

            // Per-object: 更新 PBR uniform → encoder → render pass → submit
            let uniform = PbrSceneUniform {
                model: model.to_cols_array_2d(),
                view_proj: view_proj.to_cols_array_2d(),
                normal_matrix: normal_matrix.to_cols_array_2d(),
                camera_pos: [camera_pos.x, camera_pos.y, camera_pos.z, 0.0],
                light_dir: [0.0, -1.0, 0.0, 0.0],
                light_color: [1.0, 1.0, 1.0, 3.0],
                material_params: [cmd.metallic, cmd.roughness, 0.0, 0.0],
                ..Default::default()
            };
            device.queue().write_buffer(ub, 0, bytemuck::bytes_of(&uniform));

            let mut encoder = device.device().create_command_encoder(
                &wgpu::CommandEncoderDescriptor { label: Some("ECS Encoder") },
            );

            {
                let color_load = if i == 0 {
                    wgpu::LoadOp::Clear(clear_color)
                } else {
                    wgpu::LoadOp::Load
                };
                let depth_load = if i == 0 {
                    wgpu::LoadOp::Clear(1.0)
                } else {
                    wgpu::LoadOp::Load
                };

                let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("ECS Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations { load: color_load, store: wgpu::StoreOp::Store },
                    })],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: depth_view,
                        depth_ops: Some(wgpu::Operations { load: depth_load, store: wgpu::StoreOp::Store }),
                        stencil_ops: None,
                    }),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

                rp.set_pipeline(&gpu_mat.pipeline);
                rp.set_bind_group(0, scene_bg, &[]);
                rp.set_bind_group(1, &gpu_mat.bind_group, &[]);
                rp.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
                rp.set_index_buffer(gpu_mesh.index_buffer.slice(..), gpu_mesh.index_format);
                rp.draw_indexed(0..gpu_mesh.index_count, 0, 0..1);
            }

            device.queue().submit(std::iter::once(encoder.finish()));
        }

        frame.present();
    }
}

impl ApplicationHandler for EcsApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.render_app.resumed(event_loop);
        self.init_scene();
    }

    fn window_event(&mut self, el: &ActiveEventLoop, wid: WindowId, ev: WindowEvent) {
        match &ev {
            WindowEvent::Resized(new_size) if self.initialized && new_size.width > 0 && new_size.height > 0 => {
                if let Some(device) = self.render_app.render_device() {
                    let (_, view) = create_depth_texture(device, new_size.width, new_size.height, "Depth");
                    self.depth_texture_view = Some(view);
                }
            }
            WindowEvent::RedrawRequested if self.initialized => {
                self.render_frame();
                return; // 不传递给 render_app，我们自己处理渲染
            }
            _ => {}
        }
        self.render_app.window_event(el, wid, ev);
    }

    fn device_event(&mut self, el: &ActiveEventLoop, did: winit::event::DeviceId, ev: winit::event::DeviceEvent) {
        self.render_app.device_event(el, did, ev);
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.app.update();
        if let Some(window) = self.render_app.window() {
            window.request_redraw();
        }
    }
}
