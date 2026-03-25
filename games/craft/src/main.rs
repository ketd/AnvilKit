use std::thread;

use anvilkit_render::prelude::*;
use anvilkit_render::renderer::{
    buffer::{
        create_depth_texture, create_hdr_render_target, create_sampler,
    },
    draw::{ActiveCamera, Frustum},
    line::LineRenderer,
    text::TextRenderer,
    raycast::screen_to_ray,
};
use anvilkit_render::plugin::CameraComponent;
use anvilkit_input::prelude::{InputState, MouseButton};
use anvilkit_ecs::physics::DeltaTime;
use anvilkit_camera::prelude::*;

use craft::block::BlockType;
use craft::chunk::CHUNK_SIZE;
use craft::chunk_manager::{ChunkManager, ChunkGenResult};
use craft::world_gen::WorldGenerator;
use craft::raycast::{self, VoxelHit};
use craft::render::setup::{self, VoxelGpu, VoxelSceneUniform, SkyUniform};
use craft::render::filters::{ActiveFilter, FilterUniform};
use anvilkit_render::renderer::bloom::BloomSettings;
use anvilkit_render::renderer::ssao::SsaoSettings;
use craft::components::*;
use craft::resources::*;
use craft::systems::input as input_sys;
use craft::systems::physics as physics_sys;
use craft::systems::day_night::day_night_system;
use craft::systems::camera_fx::camera_effects_system;
use craft::config;
use craft::persistence;

/// Block types selectable with number keys 1-9.
const BLOCK_PALETTE: [BlockType; 9] = [
    BlockType::Grass,
    BlockType::Dirt,
    BlockType::Stone,
    BlockType::Sand,
    BlockType::Brick,
    BlockType::Wood,
    BlockType::Glass,
    BlockType::Cobble,
    BlockType::Plank,
];

fn window_config() -> WindowConfig {
    WindowConfig::new().with_title("Craft").with_size(1280, 720)
}

fn main() {
    env_logger::init();
    println!("Craft — powered by AnvilKit");
    println!("  WASD = move, Mouse = look, Space = jump/up, Shift = down");
    println!("  Tab = toggle flying, LMB = break, RMB = place");
    println!("  1-9 = select block, Ctrl+S = save, ESC = quit");
    println!("  F5 = toggle 1st/3rd person, F1 = cycle filter, Ctrl+W = sprint");

    let mut app = App::new();
    app.add_plugins(RenderPlugin::new().with_window_config(window_config()));

    app.insert_resource(InputState::new());
    app.insert_resource(DeltaTime(1.0 / 60.0)); // updated each frame by RenderApp::tick()
    app.insert_resource(WorldSeed::default());
    app.insert_resource(PlayerState::default());
    app.insert_resource(VoxelWorld::default());
    app.insert_resource(SelectedBlock::default());
    app.insert_resource(DayNightCycle::default());
    // Bloom and SSAO disabled for voxel pixel-art — these effects
    // blur the crisp block edges. Available for PBR games (billiards, etc.)
    app.insert_resource(BloomSettings { enabled: false, ..BloomSettings::default() });
    app.insert_resource(SsaoSettings { enabled: false, ..SsaoSettings::default() });
    app.insert_resource(ActiveFilter::default());

    // Explicit ordering: DayNight → Input → Physics → CameraFX → CameraController
    app.add_systems(AnvilKitSchedule::Update, (
        day_night_system,
        input_sys::player_movement_system,
        physics_sys::player_physics_system.after(input_sys::player_movement_system),
        camera_effects_system.after(physics_sys::player_physics_system),
        camera_controller_system.after(camera_effects_system),
    ));

    // FPS Camera — spawn at a reasonable height above terrain
    let spawn_pos = glam::Vec3::new(
        (CHUNK_SIZE as f32) * 3.5,
        50.0,
        (CHUNK_SIZE as f32) * 3.5,
    );
    app.world.spawn((
        FpsCamera,
        CameraComponent {
            fov: config::FOV,
            near: config::NEAR_PLANE,
            far: config::FAR_PLANE,
            is_active: true,
            aspect_ratio: 1280.0 / 720.0,
        },
        {
            let mut cc = CameraController::default();
            cc.pitch_limits = (-1.5, 1.5);
            cc.mouse_sensitivity = 0.003;
            cc.move_speed = 10.0;
            cc
        },
        {
            let mut fx = CameraEffects::default();
            fx.head_bob_enabled = true;
            fx
        },
        Transform::from_xyz(spawn_pos.x, spawn_pos.y, spawn_pos.z),
    ));

    let seed = app.world.resource::<WorldSeed>().0;

    // Async chunk generation channels (crossbeam for multi-consumer request channel)
    let (request_tx, request_rx) = crossbeam_channel::unbounded::<(i32, i32)>();
    let (result_tx, result_rx) = crossbeam_channel::unbounded::<ChunkGenResult>();

    // Spawn worker thread pool for chunk generation
    let num_workers = thread::available_parallelism()
        .map(|n| n.get().saturating_sub(1).clamp(1, config::MAX_WORKER_THREADS))
        .unwrap_or(2);
    for _ in 0..num_workers {
        let rx = request_rx.clone();
        let tx = result_tx.clone();
        let worker_seed = seed;
        thread::spawn(move || {
            let gen = WorldGenerator::new(worker_seed);
            while let Ok((cx, cz)) = rx.recv() {
                let chunk_data = gen.generate_chunk(cx, cz);
                let _ = tx.send(ChunkGenResult { cx, cz, chunk_data });
            }
        });
    }
    // Drop the extra sender/receiver clones so channels close when all workers exit
    drop(request_rx);
    drop(result_tx);

    let event_loop = EventLoop::new().unwrap();
    let wconfig = window_config();
    event_loop
        .run_app(&mut CraftApp {
            render_app: RenderApp::new(wconfig),
            app,
            initialized: false,
            voxel_gpu: None,
            world_gen: WorldGenerator::new(seed),
            chunks: ChunkManager::new(request_tx, result_rx),
            line_renderer: None,
            text_renderer: None,
            current_hit: None,
            frame_count: 0,
        })
        .unwrap();
}

struct CraftApp {
    render_app: RenderApp,
    app: App,
    initialized: bool,
    voxel_gpu: Option<VoxelGpu>,
    world_gen: WorldGenerator,
    chunks: ChunkManager,
    // HUD renderers
    line_renderer: Option<LineRenderer>,
    text_renderer: Option<TextRenderer>,
    // Current raycast hit (updated each frame)
    current_hit: Option<VoxelHit>,
    // Debug frame counter
    frame_count: u64,
}


impl CraftApp {
    fn init_scene(&mut self) {
        if self.initialized {
            return;
        }

        // Try loading saved world first
        {
            let mut world = self.app.world.resource_mut::<VoxelWorld>();
            match persistence::load_world(&mut world) {
                Ok((seed, loaded)) if !loaded.is_empty() => {
                    println!("Loaded {} modified chunks from save (seed={})", loaded.len(), seed);
                }
                Err(e) => {
                    println!("Could not load save: {}", e);
                }
                _ => {}
            }
        }

        // Generate chunks (doesn't need GPU) — fill any not loaded from save
        {
            let mut world = self.app.world.resource_mut::<VoxelWorld>();
            ChunkManager::generate_initial_chunks(&mut world, &self.world_gen, 0, 0, self.chunks.load_radius);
        }

        // Now init GPU and upload
        let Some(device) = self.render_app.render_device() else {
            return;
        };
        let Some(format) = self.render_app.surface_format() else {
            return;
        };
        let (w, h) = self.render_app.window_state().size();

        // Load texture atlas — convert magenta color key (255,0,255) to transparent
        let atlas_path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/textures/texture.png");
        let mut atlas_img = match image::open(atlas_path) {
            Ok(img) => img.to_rgba8(),
            Err(e) => {
                log::error!("Failed to load texture.png: {}. Using fallback.", e);
                // Generate a 256x256 magenta/black checkerboard as fallback
                let mut fallback = image::RgbaImage::new(256, 256);
                for (x, y, pixel) in fallback.enumerate_pixels_mut() {
                    if (x / 16 + y / 16) % 2 == 0 {
                        *pixel = image::Rgba([255, 0, 255, 255]);
                    } else {
                        *pixel = image::Rgba([0, 0, 0, 255]);
                    }
                }
                fallback
            }
        };
        for pixel in atlas_img.pixels_mut() {
            if pixel[0] == 255 && pixel[1] == 0 && pixel[2] == 255 {
                pixel[0] = 0;
                pixel[1] = 0;
                pixel[2] = 0;
                pixel[3] = 0;
            }
        }
        let (aw, ah) = atlas_img.dimensions();

        let gpu = setup::init_voxel_gpu(device, format, w, h, &atlas_img, aw, ah);

        // Upload chunk meshes
        {
            let world = self.app.world.resource::<VoxelWorld>();
            self.chunks.upload_all(&world, device);
        }

        // Init HUD renderers
        self.line_renderer = Some(LineRenderer::new(device, format));
        self.text_renderer = Some(TextRenderer::new(device, format));

        self.voxel_gpu = Some(gpu);
        self.initialized = true;

        // Hide cursor for FPS mode
        if let Some(window) = self.render_app.window() {
            let _ = window.set_cursor_grab(winit::window::CursorGrabMode::Confined)
                .or_else(|_| window.set_cursor_grab(winit::window::CursorGrabMode::Locked));
            window.set_cursor_visible(false);
        }

        println!(
            "Craft initialized: {} chunks loaded",
            self.chunks.chunk_meshes.len()
        );
    }

    fn update_chunks(&mut self) {
        let cam_pos = {
            let cam = self.app.world.get_resource::<ActiveCamera>();
            match cam {
                Some(c) => c.camera_pos,
                None => return,
            }
        };
        let mut world = self.app.world.resource_mut::<VoxelWorld>();
        self.chunks.update(&mut world, cam_pos);
    }

    fn remesh_dirty_chunks(&mut self) {
        let Some(device) = self.render_app.render_device() else { return };
        let world = self.app.world.resource::<VoxelWorld>();
        self.chunks.remesh_dirty(&world, device);
    }

    fn handle_block_interaction(&mut self) {
        let (w, h) = self.render_app.window_state().size();
        let (cam_vp, cam_pos) = {
            let Some(cam) = self.app.world.get_resource::<ActiveCamera>() else { return };
            (cam.view_proj, cam.camera_pos)
        };

        // Cast ray from screen center
        let screen_center = glam::Vec2::new(w as f32 * 0.5, h as f32 * 0.5);
        let window_size = glam::Vec2::new(w as f32, h as f32);
        let (ray_origin, ray_dir) = screen_to_ray(screen_center, window_size, &cam_vp);

        // Raycast into voxel world
        let world = self.app.world.resource::<VoxelWorld>();
        self.current_hit = raycast::raycast_voxels(
            &world,
            [ray_origin.x, ray_origin.y, ray_origin.z],
            [ray_dir.x, ray_dir.y, ray_dir.z],
            config::RAYCAST_MAX_DIST,
        );

        // Check mouse buttons
        let (left_just, right_just) = {
            let input = self.app.world.resource::<InputState>();
            (
                input.is_mouse_just_pressed(MouseButton::Left),
                input.is_mouse_just_pressed(MouseButton::Right),
            )
        };

        if let Some(ref hit) = self.current_hit {
            if left_just {
                // Break block
                let [bx, by, bz] = hit.block_pos;
                let mut world = self.app.world.resource_mut::<VoxelWorld>();
                world.set_block(bx, by, bz, BlockType::Air);
                self.chunks.mark_dirty_with_neighbors(bx, bz);
            } else if right_just {
                // Place block adjacent to hit face
                let [bx, by, bz] = hit.block_pos;
                let [nx, ny, nz] = hit.face_normal;
                let px = bx + nx;
                let py = by + ny;
                let pz = bz + nz;

                // Don't place inside the player
                let cam_pos_arr = [cam_pos.x, cam_pos.y, cam_pos.z];
                let player_min = [cam_pos_arr[0] - 0.3, cam_pos_arr[1] - 1.6, cam_pos_arr[2] - 0.3];
                let player_max = [cam_pos_arr[0] + 0.3, cam_pos_arr[1] + 0.2, cam_pos_arr[2] + 0.3];
                let block_min = [px as f32, py as f32, pz as f32];
                let block_max = [(px + 1) as f32, (py + 1) as f32, (pz + 1) as f32];
                let overlaps = player_max[0] > block_min[0] && player_min[0] < block_max[0]
                    && player_max[1] > block_min[1] && player_min[1] < block_max[1]
                    && player_max[2] > block_min[2] && player_min[2] < block_max[2];

                if !overlaps {
                    let selected = self.app.world.resource::<SelectedBlock>().block_type;
                    let mut world = self.app.world.resource_mut::<VoxelWorld>();
                    world.set_block(px, py, pz, selected);
                    self.chunks.mark_dirty_with_neighbors(px, pz);
                }
            }
        }
    }

    fn render_frame(&mut self) {
        let Some(device) = self.render_app.render_device() else {
            return;
        };
        let Some(ref gpu) = self.voxel_gpu else {
            return;
        };

        // Camera data
        let (cam_vp, cam_pos) = {
            let Some(cam) = self.app.world.get_resource::<ActiveCamera>() else {
                return;
            };
            (cam.view_proj, cam.camera_pos)
        };

        let Some(frame) = self.render_app.get_current_frame() else {
            return;
        };
        let swapchain = frame.texture.create_view(&Default::default());

        // Day/night cycle data
        let cycle = self.app.world.resource::<DayNightCycle>();
        let light_dir = cycle.light_dir();
        let ambient = cycle.ambient();
        let fog_color = cycle.fog_color();
        let sky_top = cycle.sky_top();
        let sky_horizon = cycle.sky_horizon();
        let sky_bottom = cycle.sky_bottom();

        // Update scene uniform
        let uniform = VoxelSceneUniform {
            view_proj: cam_vp.to_cols_array_2d(),
            camera_pos: [cam_pos.x, cam_pos.y, cam_pos.z, 0.0],
            light_dir: [light_dir.x, light_dir.y, light_dir.z, 0.0],
            fog_color,
            time_ambient: [cycle.time, ambient, 80.0, 200.0],
        };
        device
            .queue()
            .write_buffer(&gpu.scene_ub, 0, bytemuck::bytes_of(&uniform));

        // Update sky uniform
        let inv_vp = cam_vp.inverse();
        let sky_uniform = SkyUniform {
            inv_view_proj: inv_vp.to_cols_array_2d(),
            sky_top: [sky_top[0], sky_top[1], sky_top[2], 1.0],
            sky_horizon: [sky_horizon[0], sky_horizon[1], sky_horizon[2], 1.0],
            sky_bottom: [sky_bottom[0], sky_bottom[1], sky_bottom[2], 1.0],
            sun_dir: [light_dir.x, light_dir.y, light_dir.z, 0.0],
        };
        device
            .queue()
            .write_buffer(&gpu.sky_ub, 0, bytemuck::bytes_of(&sky_uniform));

        // Frustum culling
        let frustum = Frustum::from_view_proj(&cam_vp);

        // --- Update filter uniform (before tonemap pass) ---
        {
            let active_filter = self.app.world.resource::<ActiveFilter>();
            let cycle = self.app.world.resource::<DayNightCycle>();
            let is_srgb = self.render_app.surface_format()
                .map(|f| f.is_srgb())
                .unwrap_or(false);
            let filter_uniform = FilterUniform {
                filter_type: active_filter.filter as u32,
                intensity: 1.0,
                time: cycle.time * 600.0,
                apply_gamma: if is_srgb { 0.0 } else { 1.0 },
            };
            device.queue().write_buffer(&gpu.filter_ub, 0, bytemuck::bytes_of(&filter_uniform));
        }

        // --- Batched: single encoder for Sky → Voxel → Water → Tonemap ---
        let mut enc = device.device().create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Craft Frame Enc"),
        });

        // Pass 1: Sky (HDR RT, Clear)
        {
            let mut rp = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Sky Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &gpu.hdr_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: fog_color[0] as f64,
                            g: fog_color[1] as f64,
                            b: fog_color[2] as f64,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            rp.set_pipeline(&gpu.sky_pipeline);
            rp.set_bind_group(0, &gpu.sky_bg, &[]);
            rp.draw(0..3, 0..1);
        }

        // Pass 2: Voxel scene (HDR RT, Load, depth test)
        {
            let mut rp = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Voxel Scene Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &gpu.hdr_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &gpu.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            rp.set_pipeline(&gpu.voxel_pipeline);
            rp.set_bind_group(0, &gpu.scene_bg, &[]);
            rp.set_bind_group(1, &gpu.atlas_bg, &[]);

            for (_key, cm) in &self.chunks.chunk_meshes {
                if !ChunkManager::is_visible(cm, &frustum) {
                    continue;
                }
                rp.set_vertex_buffer(0, cm.vertex_buffer.slice(..));
                rp.set_index_buffer(cm.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                rp.draw_indexed(0..cm.index_count, 0, 0..1);
            }
        }

        // Pass 3: Water (HDR RT, Load, depth read-only, alpha blend)
        {
            let mut rp = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Water Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &gpu.hdr_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &gpu.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            rp.set_pipeline(&gpu.water_pipeline);
            rp.set_bind_group(0, &gpu.scene_bg, &[]);
            rp.set_bind_group(1, &gpu.atlas_bg, &[]);

            for (_key, cm) in &self.chunks.chunk_meshes {
                if cm.water_index_count == 0 {
                    continue;
                }
                if !ChunkManager::is_visible(cm, &frustum) {
                    continue;
                }
                if let (Some(ref wvb), Some(ref wib)) = (&cm.water_vertex_buffer, &cm.water_index_buffer) {
                    rp.set_vertex_buffer(0, wvb.slice(..));
                    rp.set_index_buffer(wib.slice(..), wgpu::IndexFormat::Uint32);
                    rp.draw_indexed(0..cm.water_index_count, 0, 0..1);
                }
            }
        }

        // SSAO pass: depth → half-res AO → blur
        {
            let ssao_settings = self.app.world.resource::<SsaoSettings>();
            // Build projection matrix (same as camera uses)
            let (w, h) = self.render_app.window_state().size();
            let aspect = w as f32 / h as f32;
            let projection = glam::Mat4::perspective_rh(
                config::FOV.to_radians(), aspect, config::NEAR_PLANE, config::FAR_PLANE,
            );
            gpu.ssao.execute(device, &mut enc, &gpu.depth_view, &projection, &ssao_settings);
        }

        // Bloom passes: downsample → upsample
        {
            let bloom_settings = self.app.world.resource::<BloomSettings>();
            gpu.bloom.execute(device, &mut enc, &gpu.hdr_view, &bloom_settings);
        }

        // Pass 4: Tonemap (HDR + Bloom → swapchain)
        {
            let mut rp = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Tonemap"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &swapchain,
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
            rp.set_pipeline(&gpu.tonemap_pipeline);
            rp.set_bind_group(0, &gpu.tonemap_bg, &[]);
            rp.draw(0..3, 0..1);
        }

        // Single submit for all 4 passes
        device.queue().submit(std::iter::once(enc.finish()));

        // --- Pass 5: HUD (crosshair, coordinates, block highlight) ---
        // Re-acquire device ref to avoid borrow conflict with &mut self
        let _ = gpu; // end immutable borrow on self.voxel_gpu
        self.render_hud(&swapchain, &cam_vp, cam_pos);

        frame.present();
    }

    fn render_hud(
        &mut self,
        swapchain: &wgpu::TextureView,
        cam_vp: &glam::Mat4,
        cam_pos: glam::Vec3,
    ) {
        let Some(device) = self.render_app.render_device() else { return };
        let (sw, sh) = self.render_app.window_state().size();
        let sw = sw as f32;
        let sh = sh as f32;

        // Single command encoder for all HUD draws
        let mut enc = device.device().create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("HUD Enc"),
        });

        // Crosshair: use orthographic projection for screen-space lines
        let ortho = glam::Mat4::orthographic_rh(0.0, sw, sh, 0.0, -1.0, 1.0);
        let cx = sw * 0.5;
        let cy = sh * 0.5;
        let cross_size = 10.0;
        let white = glam::Vec3::new(1.0, 1.0, 1.0);

        let lines = vec![
            // Horizontal
            (glam::Vec3::new(cx - cross_size, cy, 0.0), glam::Vec3::new(cx + cross_size, cy, 0.0), white),
            // Vertical
            (glam::Vec3::new(cx, cy - cross_size, 0.0), glam::Vec3::new(cx, cy + cross_size, 0.0), white),
        ];

        // Render crosshair
        if let Some(ref mut lr) = self.line_renderer {
            lr.render(device, &mut enc, swapchain, &lines, &ortho);
        }

        // Block highlight wireframe (3D)
        if let Some(ref hit) = self.current_hit {
            if let Some(ref mut lr) = self.line_renderer {
                let wireframe_lines = block_wireframe(hit.block_pos, glam::Vec3::new(0.2, 0.2, 0.2));
                lr.render(device, &mut enc, swapchain, &wireframe_lines, cam_vp);
            }
        }

        // Text: coordinates and selected block
        let selected = self.app.world.resource::<SelectedBlock>();
        let player = self.app.world.resource::<PlayerState>();
        let cycle = self.app.world.resource::<DayNightCycle>();
        let selected_index = selected.index;

        if let Some(ref mut tr) = self.text_renderer {
            let coord_text = format!(
                "XYZ: {:.1} {:.1} {:.1}  {}  Time: {:.0}%",
                cam_pos.x, cam_pos.y, cam_pos.z,
                if player.flying { "FLY" } else { "WALK" },
                cycle.time * 100.0,
            );
            let block_text = format!("Block: {:?}", selected.block_type);

            tr.draw_text(device, &mut enc, swapchain, &coord_text, 8.0, 8.0, 16.0, white, sw, sh);
            tr.draw_text(device, &mut enc, swapchain, &block_text, 8.0, 28.0, 16.0, white, sw, sh);
        }

        // Hotbar: 9-slot block selection bar at bottom center
        {
            let slot_size = 36.0_f32;
            let slot_gap = 4.0_f32;
            let total_w = 9.0 * slot_size + 8.0 * slot_gap;
            let bar_x = (sw - total_w) * 0.5;
            let bar_y = sh - slot_size - 16.0;

            let mut hotbar_lines = Vec::new();
            let gray = glam::Vec3::new(0.5, 0.5, 0.5);
            let highlight = glam::Vec3::new(1.0, 1.0, 0.0);

            for i in 0..9usize {
                let sx = bar_x + i as f32 * (slot_size + slot_gap);
                let sy = bar_y;
                let color = if i == selected_index { highlight } else { gray };

                // Box corners
                let tl = glam::Vec3::new(sx, sy, 0.0);
                let tr_pt = glam::Vec3::new(sx + slot_size, sy, 0.0);
                let br = glam::Vec3::new(sx + slot_size, sy + slot_size, 0.0);
                let bl = glam::Vec3::new(sx, sy + slot_size, 0.0);

                hotbar_lines.push((tl, tr_pt, color));
                hotbar_lines.push((tr_pt, br, color));
                hotbar_lines.push((br, bl, color));
                hotbar_lines.push((bl, tl, color));
            }

            if let Some(ref mut lr) = self.line_renderer {
                lr.render(device, &mut enc, swapchain, &hotbar_lines, &ortho);
            }

            // Draw block name initials in each slot
            if let Some(ref mut tr) = self.text_renderer {
                let labels = ["G", "D", "S", "Sa", "B", "W", "Gl", "C", "P"];
                for i in 0..9usize {
                    let sx = bar_x + i as f32 * (slot_size + slot_gap) + slot_size * 0.25;
                    let sy = bar_y + slot_size * 0.2;
                    let color = if i == selected_index { highlight } else { white };
                    tr.draw_text(device, &mut enc, swapchain, labels[i], sx, sy, 14.0, color, sw, sh);
                }
            }
        }

        device.queue().submit(std::iter::once(enc.finish()));
    }
}

/// Generate 12 line segments for a wireframe box around a block position.
fn block_wireframe(pos: [i32; 3], color: glam::Vec3) -> Vec<(glam::Vec3, glam::Vec3, glam::Vec3)> {
    let s = 0.005; // slight expansion to avoid z-fighting
    let x0 = pos[0] as f32 - s;
    let y0 = pos[1] as f32 - s;
    let z0 = pos[2] as f32 - s;
    let x1 = (pos[0] + 1) as f32 + s;
    let y1 = (pos[1] + 1) as f32 + s;
    let z1 = (pos[2] + 1) as f32 + s;

    let v = |x: f32, y: f32, z: f32| glam::Vec3::new(x, y, z);

    vec![
        // Bottom face
        (v(x0, y0, z0), v(x1, y0, z0), color),
        (v(x1, y0, z0), v(x1, y0, z1), color),
        (v(x1, y0, z1), v(x0, y0, z1), color),
        (v(x0, y0, z1), v(x0, y0, z0), color),
        // Top face
        (v(x0, y1, z0), v(x1, y1, z0), color),
        (v(x1, y1, z0), v(x1, y1, z1), color),
        (v(x1, y1, z1), v(x0, y1, z1), color),
        (v(x0, y1, z1), v(x0, y1, z0), color),
        // Vertical edges
        (v(x0, y0, z0), v(x0, y1, z0), color),
        (v(x1, y0, z0), v(x1, y1, z0), color),
        (v(x1, y0, z1), v(x1, y1, z1), color),
        (v(x0, y0, z1), v(x0, y1, z1), color),
    ]
}

impl ApplicationHandler for CraftApp {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        self.render_app.resumed(el);
        self.init_scene();
    }

    fn window_event(&mut self, el: &ActiveEventLoop, wid: WindowId, ev: WindowEvent) {
        // Game-specific event handling
        match &ev {
            WindowEvent::Resized(s) if self.initialized && s.width > 0 && s.height > 0 => {
                if let Some(device) = self.render_app.render_device() {
                    if let Some(ref mut gpu) = self.voxel_gpu {
                        let (_, dv) = create_depth_texture(device, s.width, s.height, "Voxel Depth");
                        gpu.depth_view = dv;
                        let (_, hv) =
                            create_hdr_render_target(device, s.width, s.height, "Voxel HDR RT");
                        let samp = create_sampler(device, "Tonemap Sampler");
                        // Resize bloom + SSAO
                        let bloom_mips = self.app.world.resource::<BloomSettings>().mip_count;
                        gpu.bloom.resize(device, s.width, s.height, bloom_mips);
                        gpu.ssao.resize(device, s.width, s.height);
                        let bloom_view = gpu.bloom.mip_views.first().unwrap_or(&hv);
                        gpu.tonemap_bg =
                            device
                                .device()
                                .create_bind_group(&wgpu::BindGroupDescriptor {
                                    label: Some("Tonemap BG"),
                                    layout: &gpu.tonemap_bgl,
                                    entries: &[
                                        wgpu::BindGroupEntry {
                                            binding: 0,
                                            resource: wgpu::BindingResource::TextureView(&hv),
                                        },
                                        wgpu::BindGroupEntry {
                                            binding: 1,
                                            resource: wgpu::BindingResource::Sampler(&samp),
                                        },
                                        wgpu::BindGroupEntry {
                                            binding: 2,
                                            resource: gpu.filter_ub.as_entire_binding(),
                                        },
                                        wgpu::BindGroupEntry {
                                            binding: 3,
                                            resource: wgpu::BindingResource::TextureView(bloom_view),
                                        },
                                        wgpu::BindGroupEntry {
                                            binding: 4,
                                            resource: wgpu::BindingResource::TextureView(&gpu.ssao.blurred_view),
                                        },
                                    ],
                                });
                        gpu.hdr_view = hv;
                    }
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                use winit::keyboard::{KeyCode as WK, PhysicalKey};
                if let PhysicalKey::Code(code) = event.physical_key {
                    if event.state.is_pressed() {
                        // Ctrl+S: save world
                        if code == WK::KeyS && !event.repeat {
                            let input = self.app.world.resource::<InputState>();
                            if input.is_key_pressed(anvilkit_input::prelude::KeyCode::LControl)
                                || input.is_key_pressed(anvilkit_input::prelude::KeyCode::RControl)
                            {
                                let save_seed = self.app.world.resource::<WorldSeed>().0;
                                let world = self.app.world.resource::<VoxelWorld>();
                                match persistence::save_world(&world, &world.modified_chunks, save_seed) {
                                    Ok(n) => println!("Saved {} modified chunks to {:?}", n, persistence::save_path()),
                                    Err(e) => println!("Save failed: {}", e),
                                }
                            }
                        }
                        match code {
                            WK::Escape => { el.exit(); return; }
                            WK::Tab => {
                                if let Some(mut ps) = self.app.world.get_resource_mut::<PlayerState>() {
                                    ps.flying = !ps.flying;
                                    println!("Flying: {}", if ps.flying { "ON" } else { "OFF" });
                                }
                            }
                            WK::F1 => {
                                if let Some(mut af) = self.app.world.get_resource_mut::<ActiveFilter>() {
                                    af.filter = af.filter.cycle();
                                    println!("Filter: {}", af.filter.name());
                                }
                            }
                            WK::F5 => {
                                let pos = self.app.world.get_resource::<ActiveCamera>()
                                    .map(|c| c.camera_pos)
                                    .unwrap_or(glam::Vec3::ZERO);
                                let mut q = self.app.world.query::<&mut CameraController>();
                                for mut ctrl in q.iter_mut(&mut self.app.world) {
                                    ctrl.toggle_perspective(pos);
                                    let mode_name = match &ctrl.mode {
                                        CameraMode::FirstPerson => "First Person",
                                        CameraMode::ThirdPerson { .. } => "Third Person",
                                        CameraMode::Free => "Free",
                                    };
                                    println!("Camera: {}", mode_name);
                                }
                            }
                            WK::Digit1 => self.select_block(0),
                            WK::Digit2 => self.select_block(1),
                            WK::Digit3 => self.select_block(2),
                            WK::Digit4 => self.select_block(3),
                            WK::Digit5 => self.select_block(4),
                            WK::Digit6 => self.select_block(5),
                            WK::Digit7 => self.select_block(6),
                            WK::Digit8 => self.select_block(7),
                            WK::Digit9 => self.select_block(8),
                            _ => {}
                        }
                    }
                }
            }
            WindowEvent::RedrawRequested if self.initialized => {
                self.render_frame();
                return;
            }
            _ => {}
        }
        // Engine handles input forwarding (keyboard→InputState, mouse, cursor, scroll)
        // and window lifecycle (close, resize surface, focus)
        RenderApp::forward_input(&mut self.app, &ev);
        self.render_app.window_event(el, wid, ev);
    }

    fn device_event(
        &mut self,
        el: &ActiveEventLoop,
        did: winit::event::DeviceId,
        ev: winit::event::DeviceEvent,
    ) {
        RenderApp::forward_device_input(&mut self.app, &ev);
        self.render_app.device_event(el, did, ev);
    }

    fn about_to_wait(&mut self, _el: &ActiveEventLoop) {
        self.frame_count += 1;

        // Engine tick: DeltaTime → app.update() (runs all ECS systems) → end_frame → redraw
        self.render_app.tick(&mut self.app);

        // Post-update: game logic that depends on ECS system results
        self.post_update();
    }
}

impl CraftApp {
    // Pre-update logic is now handled by ECS systems:
    // - day_night_system: advances DayNightCycle
    // - camera_effects_system: landing shake, sprint FOV, third-person target

    /// Post-update: game logic that depends on ECS system results.
    fn post_update(&mut self) {
        // Periodic debug log (every 120 frames ~ 2 sec)
        if self.frame_count % 120 == 0 {
            let player = self.app.world.resource::<PlayerState>();
            if let Some(cam) = self.app.world.get_resource::<ActiveCamera>() {
                let p = cam.camera_pos;
                log::debug!(
                    "[F{}] pos=({:.1},{:.1},{:.1}) vel=({:.1},{:.1},{:.1}) fly={} gnd={}",
                    self.frame_count,
                    p.x, p.y, p.z,
                    player.velocity.x, player.velocity.y, player.velocity.z,
                    player.flying, player.on_ground,
                );
            }
        }

        // Block interaction (raycast + place/break)
        self.handle_block_interaction();

        // Remesh any dirty chunks
        self.remesh_dirty_chunks();

        // Dynamic chunk loading
        self.update_chunks();
    }

    fn select_block(&mut self, index: usize) {
        if index < BLOCK_PALETTE.len() {
            if let Some(mut sb) = self.app.world.get_resource_mut::<SelectedBlock>() {
                sb.block_type = BLOCK_PALETTE[index];
                sb.index = index;
                println!("Selected: {:?}", sb.block_type);
            }
        }
    }
}

