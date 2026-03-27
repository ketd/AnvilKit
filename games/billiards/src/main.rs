#[allow(unused_imports)]
use anvilkit::prelude::*;
use anvilkit_app::{AnvilKitApp, GameCallbacks, GameConfig, GameContext};

use anvilkit_render::renderer::{
    buffer::{
        create_depth_texture_msaa, create_hdr_render_target,
        create_hdr_msaa_texture, create_sampler,
    },
    assets::RenderAssets,
    draw::{ActiveCamera, DrawCommandList, SceneLights, DirectionalLight, PointLight, MaterialParams},
    state::PbrSceneUniform,
    debug::OverlayLineRenderer,
    text::TextRenderer,
};
use anvilkit_render::plugin::CameraComponent;
use anvilkit_input::prelude::InputState;
use anvilkit_ecs::physics::{Velocity, DeltaTime};

use billiards::components::*;
use billiards::resources::*;
use billiards::physics::{dynamics, collision, pocket};
use billiards::systems::{input as input_sys, game_logic, ui_update};
use billiards::render::{setup, colors};

// ---------------------------------------------------------------------------
//  Helpers
// ---------------------------------------------------------------------------

use anvilkit_render::window::pack_lights;

/// Triangle rack positions for 15 balls.
/// Row 0 (apex, 1 ball) faces the cue ball; row 4 (5 balls) is the back.
/// The rack is placed on the foot end (+Z side) of the table.
fn rack_positions(config: &BilliardConfig) -> [glam::Vec3; 15] {
    let r = config.ball_radius;
    let spacing = r * 2.05; // slight gap between balls
    // Foot spot: the standard rack position is 1/4 from the foot end
    let foot_spot_z = config.table_half_depth * 0.5;

    let mut positions = [glam::Vec3::ZERO; 15];
    let mut idx = 0;
    for row in 0..5 {
        let balls_in_row = row + 1;
        // Rows extend AWAY from cue ball (toward +Z / foot cushion)
        let row_z = foot_spot_z + (row as f32) * spacing * 0.866;
        let row_start_x = -(balls_in_row as f32 - 1.0) * spacing * 0.5;
        for col in 0..balls_in_row {
            positions[idx] = glam::Vec3::new(
                row_start_x + col as f32 * spacing,
                config.ball_radius,
                row_z,
            );
            idx += 1;
        }
    }
    positions
}

/// Ball number assignment for rack positions.
/// 8-ball must be at position index 4 (center of row 2).
fn rack_ball_numbers() -> [u8; 15] {
    // Standard 8-ball rack: 8 in center, rest shuffled
    // Simplified: just place 1-15 with 8 swapped to position 4
    let numbers: [u8; 15] = [1, 2, 3, 4, 8, 6, 7, 5, 9, 10, 11, 12, 13, 14, 15];
    numbers
}

// ---------------------------------------------------------------------------
//  Main
// ---------------------------------------------------------------------------

fn main() {
    env_logger::init();
    println!("AnvilKit Billiards");
    println!("  Mouse aim + Click-hold-release to shoot");
    println!("  ESC = quit");

    let config = BilliardConfig::default();

    let mut app = App::new();
    app.add_plugins(RenderPlugin::new().with_window_config(
        WindowConfig::new()
            .with_title("AnvilKit Billiards")
            .with_size(1280, 720),
    ));

    // Resources
    app.insert_resource(InputState::new());
    app.insert_resource(DeltaTime(1.0 / 60.0));
    app.insert_resource(GameState::default());
    app.insert_resource(ShotState::default());
    app.insert_resource(BallTracker::default());
    app.insert_resource(input_sys::WindowSize::default());

    // Systems
    app.add_systems(AnvilKitSchedule::Update, (
        input_sys::aim_input_system,
        input_sys::shot_execution_system.after(input_sys::aim_input_system),
        dynamics::billiard_velocity_system.after(input_sys::shot_execution_system),
        collision::ball_collision_system.after(dynamics::billiard_velocity_system),
        collision::cushion_collision_system.after(collision::ball_collision_system),
        pocket::pocket_detection_system.after(collision::cushion_collision_system),
        game_logic::game_logic_system.after(pocket::pocket_detection_system),
        game_logic::check_game_over_system.after(game_logic::game_logic_system),
    ));

    // Lighting
    app.insert_resource(SceneLights {
        directional: DirectionalLight {
            direction: glam::Vec3::new(-0.3, -0.8, 0.4).normalize(),
            color: glam::Vec3::new(1.0, 0.97, 0.92),
            intensity: 3.5,
        },
        point_lights: vec![
            PointLight {
                position: glam::Vec3::new(0.0, 6.0, 0.0),
                color: glam::Vec3::new(1.0, 0.95, 0.85),
                intensity: 25.0,
                range: 20.0,
            },
        ],
        spot_lights: vec![],
    });

    // Spawn cue ball — head end of table (negative Z, near camera)
    let cue_z = -config.table_half_depth * 0.5;
    let cue_entity = app.world.spawn((
        CueBall,
        Transform::from_xyz(0.0, config.ball_radius, cue_z),
        Velocity::zero(),
        MaterialParams { metallic: 0.05, roughness: 0.2, normal_scale: 1.0, emissive_factor: [0.0; 3] },
    )).id();

    // Spawn numbered balls in triangle rack
    let rack_pos = rack_positions(&config);
    let ball_nums = rack_ball_numbers();
    let mut ball_entities = vec![cue_entity];
    for i in 0..15 {
        let e = app.world.spawn((
            NumberedBall { number: ball_nums[i], potted: false },
            Transform::from_xyz(rack_pos[i].x, rack_pos[i].y, rack_pos[i].z),
            Velocity::zero(),
            MaterialParams {
                metallic: colors::BALL_METALLIC[ball_nums[i] as usize],
                roughness: 0.2,
                normal_scale: 1.0,
                emissive_factor: [0.0; 3],
            },
        )).id();
        ball_entities.push(e);
    }

    // Store tracker
    if let Some(mut tracker) = app.world.get_resource_mut::<BallTracker>() {
        tracker.ball_entities = ball_entities.clone();
    }

    // Spawn table surface
    let table_entity = app.world.spawn((
        TableSurface,
        Transform::from_xyz(0.0, 0.0, 0.0),
        MaterialParams { metallic: 0.0, roughness: 0.9, normal_scale: 1.0, emissive_factor: [0.0; 3] },
    )).id();

    // Spawn cushions (4 entities at rail positions)
    let hw = config.table_half_width;
    let hd = config.table_half_depth;
    let rail_h = 0.15;
    let rail_thick = 0.15;
    let cushion_positions = [
        glam::Vec3::new(hw + rail_thick, rail_h, 0.0),   // +X
        glam::Vec3::new(-hw - rail_thick, rail_h, 0.0),  // -X
        glam::Vec3::new(0.0, rail_h, -hd - rail_thick),  // -Z
        glam::Vec3::new(0.0, rail_h, hd + rail_thick),   // +Z
    ];
    let mut cushion_entities = Vec::new();
    for pos in &cushion_positions {
        let e = app.world.spawn((
            Transform::from_xyz(pos.x, pos.y, pos.z),
            MaterialParams { metallic: 0.1, roughness: 0.6, normal_scale: 1.0, emissive_factor: [0.0; 3] },
        )).id();
        cushion_entities.push(e);
    }

    // Camera: overhead angled view from -Z side (looking into +Z toward table)
    let eye = glam::Vec3::new(0.0, 12.0, -10.0);
    let look_dir = (glam::Vec3::ZERO - eye).normalize();
    let cam_rot = glam::Quat::from_rotation_arc(glam::Vec3::Z, look_dir);
    app.world.spawn((
        CameraComponent { fov: 55.0, near: 0.1, far: 100.0, is_active: true, aspect_ratio: 1280.0 / 720.0, ..Default::default() },
        Transform::from_xyz(eye.x, eye.y, eye.z).with_rotation(cam_rot),
    ));

    app.insert_resource(config);

    println!("Billiard setup: 16 balls + table + 4 cushions + camera");

    let config = GameConfig::new("AnvilKit Billiards").with_size(1280, 720);

    AnvilKitApp::run(config, app, BilliardGame {
        ball_entities,
        table_entity,
        cushion_entities,
        scene_gpu: None,
        line_renderer: None,
        text_renderer: None,
    });
}

// ---------------------------------------------------------------------------
//  BilliardGame
// ---------------------------------------------------------------------------

struct BilliardGame {
    ball_entities: Vec<Entity>,
    table_entity: Entity,
    cushion_entities: Vec<Entity>,
    scene_gpu: Option<setup::SceneGpu>,
    line_renderer: Option<OverlayLineRenderer>,
    text_renderer: Option<TextRenderer>,
}

impl BilliardGame {
    fn init_scene(&mut self, ctx: &mut GameContext) {
        let Some(device) = ctx.render_app.render_device() else { return };
        let Some(format) = ctx.render_app.surface_format() else { return };
        let (w, h) = ctx.render_app.window_state().size();

        let config = { let c = ctx.app.world.resource::<BilliardConfig>(); c.clone() };
        let mut assets = ctx.app.world.resource_mut::<RenderAssets>();
        let gpu = setup::init_scene(device, format, w, h, &mut assets, &config);

        // Attach mesh/material handles to ball entities
        // Ball 0 = cue ball, 1-15 = numbered (stored as ball_entities[0..16])
        for (i, &e) in self.ball_entities.iter().enumerate() {
            let ball_num = if i == 0 { 0 } else {
                // Look up the actual ball number from the entity's NumberedBall component
                if let Some(nb) = ctx.app.world.get::<NumberedBall>(e) {
                    nb.number as usize
                } else {
                    i
                }
            };
            ctx.app.world.entity_mut(e).insert((gpu.sphere_mesh, gpu.ball_materials[ball_num]));
        }

        // Table
        ctx.app.world.entity_mut(self.table_entity).insert((gpu.plane_mesh, gpu.table_material));

        // Cushions
        for (i, &e) in self.cushion_entities.iter().enumerate() {
            let mesh_idx = i; // 0=+X, 1=-X use X mesh; 2=-Z, 3=+Z use Z mesh
            ctx.app.world.entity_mut(e).insert((gpu.cushion_meshes[mesh_idx], gpu.cushion_material));
        }

        // Line and text renderers
        self.line_renderer = Some(OverlayLineRenderer::new(device, format));
        self.text_renderer = Some(TextRenderer::new(device, format));

        self.scene_gpu = Some(gpu);
        // initialization complete
        println!("Billiard scene initialized!");
    }

    fn render_frame(&mut self, ctx: &mut GameContext) {
        let Some(device) = ctx.render_app.render_device() else { return };
        let Some(ref gpu) = self.scene_gpu else { return };

        // Copy camera data to avoid borrow conflicts later
        let (cam_vp, cam_pos) = {
            let Some(cam) = ctx.app.world.get_resource::<ActiveCamera>() else { return };
            (cam.view_proj, cam.camera_pos)
        };
        let Some(dl) = ctx.app.world.get_resource::<DrawCommandList>() else { return };
        let Some(ra) = ctx.app.world.get_resource::<RenderAssets>() else { return };
        if dl.commands.is_empty() { return; }

        let Some(frame) = ctx.render_app.get_current_frame() else { return };
        let swapchain = frame.texture.create_view(&Default::default());

        let def_lights = SceneLights::default();
        let lights = ctx.app.world.get_resource::<SceneLights>().unwrap_or(&def_lights);
        let (gpu_lights, lc) = pack_lights(lights);
        let ld = lights.directional.direction.normalize();

        // HDR Scene pass — render all objects into MSAA HDR target
        // Each object needs its own uniform update + render pass submission
        // (write_buffer can't happen during a render pass)
        for (i, cmd) in dl.commands.iter().enumerate() {
            let Some(gm) = ra.get_mesh(&cmd.mesh) else { continue };
            let Some(gmat) = ra.get_material(&cmd.material) else { continue };
            let m = cmd.model_matrix;
            let u = PbrSceneUniform {
                model: m.to_cols_array_2d(),
                view_proj: cam_vp.to_cols_array_2d(),
                normal_matrix: m.inverse().transpose().to_cols_array_2d(),
                camera_pos: [cam_pos.x, cam_pos.y, cam_pos.z, 0.0],
                light_dir: [ld.x, ld.y, ld.z, 0.0],
                light_color: [lights.directional.color.x, lights.directional.color.y, lights.directional.color.z, lights.directional.intensity],
                material_params: [cmd.metallic, cmd.roughness, cmd.normal_scale, lc as f32],
                lights: gpu_lights,
                cascade_view_projs: [glam::Mat4::IDENTITY.to_cols_array_2d(); 3],
                cascade_splits: [10.0, 30.0, 100.0, 1.0 / 2048.0],
                emissive_factor: [cmd.emissive_factor[0], cmd.emissive_factor[1], cmd.emissive_factor[2], 3.0],
            };
            device.queue().write_buffer(&gpu.scene_ub, 0, bytemuck::bytes_of(&u));

            let mut enc = device.device().create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Billiard Scene Enc") });
            {
                let cl = if i == 0 { wgpu::LoadOp::Clear(wgpu::Color { r: 0.05, g: 0.12, b: 0.25, a: 1.0 }) } else { wgpu::LoadOp::Load };
                let dl_op = if i == 0 { wgpu::LoadOp::Clear(1.0) } else { wgpu::LoadOp::Load };
                // Always resolve MSAA to HDR target (every pass, not just last)
                let mut rp = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Billiard Scene Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &gpu.hdr_msaa_view,
                        resolve_target: Some(&gpu.hdr_view),
                        ops: wgpu::Operations { load: cl, store: wgpu::StoreOp::Discard },
                    })],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &gpu.depth_view,
                        depth_ops: Some(wgpu::Operations { load: dl_op, store: wgpu::StoreOp::Store }),
                        stencil_ops: None,
                    }),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
                let pipeline = ra.get_pipeline(&gmat.pipeline_handle).unwrap();
                rp.set_pipeline(pipeline);
                rp.set_bind_group(0, &gpu.scene_bg, &[]);
                rp.set_bind_group(1, &gmat.bind_group, &[]);
                rp.set_bind_group(2, &gpu.ibl_bg, &[]);
                rp.set_vertex_buffer(0, gm.vertex_buffer.slice(..));
                rp.set_index_buffer(gm.index_buffer.slice(..), gm.index_format);
                rp.draw_indexed(0..gm.index_count, 0, 0..1);
            }
            device.queue().submit(std::iter::once(enc.finish()));
        }

        // Tonemap → swapchain
        {
            let mut enc = device.device().create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Tonemap Enc") });
            {
                let mut rp = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Tonemap"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &swapchain,
                        resolve_target: None,
                        ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
                rp.set_pipeline(&gpu.tonemap_pipeline);
                rp.set_bind_group(0, &gpu.tonemap_bg, &[]);
                rp.draw(0..3, 0..1);
            }
            device.queue().submit(std::iter::once(enc.finish()));
        }

        // Aim line (only in Aiming phase)
        if let Some(ref mut lr) = self.line_renderer {
            // Copy data out of world to avoid borrow conflicts
            let aim_data: Option<(glam::Vec3, glam::Vec3)> = {
                let gs = ctx.app.world.get_resource::<GameState>();
                let shot = ctx.app.world.get_resource::<ShotState>();
                match (gs, shot) {
                    (Some(gs), Some(shot))
                        if (gs.phase == GamePhase::Aiming || gs.phase == GamePhase::PowerCharging)
                            && shot.aim_valid =>
                    {
                        Some((shot.aim_direction, shot.aim_point))
                    }
                    _ => None,
                }
            };
            if let Some((aim_dir, _aim_pt)) = aim_data {
                let cue_positions: Vec<glam::Vec3> = ctx.app.world.query_filtered::<&Transform, With<CueBall>>()
                    .iter(&ctx.app.world)
                    .map(|t| t.translation)
                    .collect();
                if let Some(&cue_pos) = cue_positions.first() {
                    let line_end = cue_pos + aim_dir * 3.0;
                    let line_y = cue_pos.y;
                    let lines = vec![(
                        glam::Vec3::new(cue_pos.x, line_y, cue_pos.z),
                        glam::Vec3::new(line_end.x, line_y, line_end.z),
                        glam::Vec3::new(1.0, 1.0, 1.0), // white
                    )];
                    let mut enc = device.device().create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Aim Line Enc") });
                    lr.render(device, &mut enc, &swapchain, &lines, &cam_vp);
                    device.queue().submit(std::iter::once(enc.finish()));
                }
            }
        }

        // Text UI
        if let Some(ref mut tr) = self.text_renderer {
            let (sw, sh) = ctx.render_app.window_state().size();
            let status = {
                let gs = ctx.app.world.get_resource::<GameState>().unwrap();
                let shot = ctx.app.world.get_resource::<ShotState>().unwrap();
                ui_update::format_status_text(gs, shot)
            };
            let mut enc = device.device().create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Text Enc") });
            tr.draw_text(device, &mut enc, &swapchain, &status,
                10.0, 10.0, 20.0, glam::Vec3::new(1.0, 1.0, 0.8),
                sw as f32, sh as f32);
            device.queue().submit(std::iter::once(enc.finish()));
        }

        frame.present();
    }
}

impl GameCallbacks for BilliardGame {
    fn init(&mut self, ctx: &mut GameContext) {
        self.init_scene(ctx);
    }

    fn on_resize(&mut self, ctx: &mut GameContext, width: u32, height: u32) {
        if let Some(device) = ctx.render_app.render_device() {
            if let Some(ref mut gpu) = self.scene_gpu {
                let (_, dv) = create_depth_texture_msaa(device, width, height, "Billiard Depth");
                gpu.depth_view = dv;
                let (_, hv) = create_hdr_render_target(device, width, height, "Billiard HDR RT");
                let (_, hmv) = create_hdr_msaa_texture(device, width, height, "Billiard HDR MSAA");
                let samp = create_sampler(device, "Billiard Sampler");

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
                let layout = device.device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                gpu.tonemap_bg = device.device().create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Tonemap BG"),
                    layout: &layout,
                    entries: &[
                        wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&hv) },
                        wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&samp) },
                    ],
                });
                gpu.hdr_view = hv;
                gpu.hdr_msaa_view = hmv;
            }
        }
    }

    fn on_window_event(&mut self, ctx: &mut GameContext, ev: &WindowEvent) -> bool {
        match ev {
            WindowEvent::KeyboardInput { event, .. } => {
                use winit::keyboard::{KeyCode as WK, PhysicalKey};
                if let PhysicalKey::Code(code) = event.physical_key {
                    if event.state.is_pressed() && code == WK::Escape {
                        ctx.app.exit();
                        return true;
                    }
                    if event.state.is_pressed() && code == WK::KeyR {
                        self.reset_game(ctx);
                    }
                }
                false // let engine handle InputState forwarding
            }
            _ => false,
        }
    }

    fn render(&mut self, ctx: &mut GameContext) {
        self.render_frame(ctx);
    }
}

impl BilliardGame {
    fn reset_game(&mut self, ctx: &mut GameContext) {
        let config = { let c = ctx.app.world.resource::<BilliardConfig>(); c.clone() };
        let rack_pos = rack_positions(&config);
        // Reset cue ball
        if let Some(cue_e) = self.ball_entities.first() {
            if let Some(mut t) = ctx.app.world.get_mut::<Transform>(*cue_e) {
                t.translation = glam::Vec3::new(0.0, config.ball_radius, -config.table_half_depth * 0.5);
            }
            if let Some(mut v) = ctx.app.world.get_mut::<Velocity>(*cue_e) {
                v.linear = glam::Vec3::ZERO;
            }
        }

        // Reset numbered balls
        for i in 0..15 {
            let e = self.ball_entities[i + 1];
            if let Some(mut t) = ctx.app.world.get_mut::<Transform>(e) {
                t.translation = rack_pos[i];
            }
            if let Some(mut v) = ctx.app.world.get_mut::<Velocity>(e) {
                v.linear = glam::Vec3::ZERO;
            }
            if let Some(mut nb) = ctx.app.world.get_mut::<NumberedBall>(e) {
                nb.potted = false;
            }
        }

        // Reset resources
        if let Some(mut gs) = ctx.app.world.get_resource_mut::<GameState>() {
            *gs = GameState::default();
        }
        if let Some(mut shot) = ctx.app.world.get_resource_mut::<ShotState>() {
            *shot = ShotState::default();
        }
        if let Some(mut tracker) = ctx.app.world.get_resource_mut::<BallTracker>() {
            tracker.on_table = [true; 16];
        }

        println!("Game reset!");
    }
}
