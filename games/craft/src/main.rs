use std::thread;

#[allow(unused_imports)]
use anvilkit::prelude::*;
use anvilkit_app::{AnvilKitApp, GameCallbacks, GameConfig, GameContext, ScreenPlugin};
use anvilkit_ecs::state::{GameState, NextGameState, in_state};

// DefaultPlugins handles: ECS, Render, Camera, Audio, Input, DeltaTime
use anvilkit_render::renderer::{
    buffer::{
        create_depth_texture, create_hdr_render_target, create_sampler,
    },
    draw::{ActiveCamera, Frustum},
    debug::OverlayLineRenderer,
    text::TextRenderer,
    raycast::screen_to_ray,
};
use anvilkit_render::plugin::CameraComponent;
use anvilkit_input::prelude::{InputState, MouseButton, ActionMap, InputBinding, KeyCode as AK};
use anvilkit_ecs::physics::{Velocity, AabbCollider};
use anvilkit_camera::controller::{CameraController, CameraMode};
use anvilkit_camera::effects::CameraEffects;


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
use craft::systems::survival as survival_sys;
use craft::config;
use craft::persistence;
use craft::ui::{CraftScreen, SettingsReturnTo};
use anvilkit_gameplay::health::{Health, DamageEvent, HealEvent, DeathEvent, health_system};
use anvilkit_gameplay::inventory::{SlotInventory, Inventory, ItemStack};

/// Block types selectable with number keys 1-9.
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
    app.add_plugins(DefaultPlugins::new().with_window(window_config()));
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

    // Screen state machine + auto cursor management
    ScreenPlugin::new(CraftScreen::MainMenu)
        .lock_cursor_in(CraftScreen::Playing)
        .build(&mut app);
    app.insert_resource(SettingsReturnTo::default());

    // Load block data table + locale
    {
        use anvilkit_data::{DataTable, Locale};
        use craft::block::{BlockTable, BlockDefCache};

        let blocks_ron = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/data/blocks.ron"));
        let block_table: BlockTable = DataTable::from_ron("blocks", blocks_ron)
            .expect("Failed to parse blocks.ron");
        let cache = BlockDefCache::from_table(&block_table);
        app.insert_resource(block_table);
        app.insert_resource(cache);

        let mut locale = Locale::new("en");
        let locale_ron = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/data/locale_en.ron"));
        locale.load_ron(locale_ron).expect("Failed to parse locale_en.ron");
        app.insert_resource(locale);
    }

    // Register health/damage events
    app.add_event::<DamageEvent>();
    app.add_event::<HealEvent>();
    app.add_event::<DeathEvent>();

    // Ordering: DayNight → Input → Physics → Survival → CameraFX (Update)
    // CameraController runs in PostUpdate via CameraPlugin (from DefaultPlugins)
    let playing = in_state(CraftScreen::Playing);
    app.add_systems(AnvilKitSchedule::Update, (
        day_night_system.run_if(playing.clone()),
        input_sys::player_movement_system.run_if(playing.clone()),
        input_sys::hotbar_selection_system.run_if(playing.clone()),
        input_sys::toggle_actions_system.run_if(playing.clone()),
        physics_sys::player_physics_system
            .after(input_sys::player_movement_system)
            .run_if(playing.clone()),
        survival_sys::fall_damage_system
            .after(physics_sys::player_physics_system)
            .run_if(playing.clone()),
        survival_sys::drowning_system
            .after(physics_sys::player_physics_system)
            .run_if(playing.clone()),
        health_system
            .after(survival_sys::fall_damage_system)
            .after(survival_sys::drowning_system)
            .run_if(playing.clone()),
        survival_sys::health_regen_system
            .after(health_system)
            .run_if(playing.clone()),
        survival_sys::death_respawn_system
            .after(health_system)
            .run_if(playing.clone()),
        camera_effects_system
            .after(physics_sys::player_physics_system)
            .after(survival_sys::fall_damage_system)
            .run_if(playing),
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
            ..Default::default()
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
        anvilkit_camera::orbit::OrbitState::new(spawn_pos, 5.0)
            .with_distance_limits(2.0, 20.0)
            .with_target_offset(glam::Vec3::new(0.0, 1.6, 0.0)),
        Transform::from_xyz(spawn_pos.x, spawn_pos.y, spawn_pos.z),
        Velocity::zero(),
        AabbCollider::new(glam::Vec3::new(
            config::PLAYER_WIDTH * 0.5,
            config::PLAYER_HEIGHT * 0.5,
            config::PLAYER_WIDTH * 0.5,
        )),
        Health::new(20.0).with_regen(0.5),
        {
            let mut inventory = SlotInventory::new(9);
            for (i, &block) in config::BLOCK_PALETTE.iter().enumerate() {
                inventory.set_slot(i, Some(ItemStack::new(block.item_id(), 64)));
            }
            inventory
        },
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

    let config = GameConfig::new("Craft").with_size(1280, 720);

    AnvilKitApp::run(config, app, CraftGame {
        voxel_gpu: None,
        chunks: ChunkManager::new(request_tx, result_rx),
        line_renderer: None,
        text_renderer: None,
        ui_renderer: None,
        current_hit: None,
        frame_count: 0,
        settings_state: craft::ui::settings_menu::SettingsState::default(),
        theme_applied: false,
    });
}

struct CraftGame {
    voxel_gpu: Option<VoxelGpu>,
    chunks: ChunkManager,
    // HUD renderers
    line_renderer: Option<OverlayLineRenderer>,
    text_renderer: Option<TextRenderer>,
    ui_renderer: Option<anvilkit_render::renderer::ui::UiRenderer>,
    // Current raycast hit (updated each frame)
    current_hit: Option<VoxelHit>,
    // Debug frame counter
    frame_count: u64,
    // UI state
    settings_state: craft::ui::settings_menu::SettingsState,
    theme_applied: bool,
}


impl CraftGame {
    /// Register all Craft key bindings into an ActionMap resource.
    ///
    /// The bindings mirror the hardcoded KeyCode checks in `systems/input.rs`
    /// and `on_window_event`. Games can override these via
    /// `ActionMap::apply_overrides()` with user settings.
    fn setup_action_map(&self, ctx: &mut GameContext) {
        let mut map = ActionMap::new();
        // Movement
        map.add_binding("move_forward",  InputBinding::Key(AK::W));
        map.add_binding("move_backward", InputBinding::Key(AK::S));
        map.add_binding("move_left",     InputBinding::Key(AK::A));
        map.add_binding("move_right",    InputBinding::Key(AK::D));
        map.add_binding("jump",          InputBinding::Key(AK::Space));
        map.add_binding("descend",       InputBinding::Key(AK::LShift));
        map.add_binding("sprint",        InputBinding::Key(AK::LControl));
        // Block interaction
        map.add_binding("place_block",   InputBinding::Mouse(MouseButton::Right));
        map.add_binding("break_block",   InputBinding::Mouse(MouseButton::Left));
        // Block palette (1-9)
        map.add_binding("slot_1", InputBinding::Key(AK::Key1));
        map.add_binding("slot_2", InputBinding::Key(AK::Key2));
        map.add_binding("slot_3", InputBinding::Key(AK::Key3));
        map.add_binding("slot_4", InputBinding::Key(AK::Key4));
        map.add_binding("slot_5", InputBinding::Key(AK::Key5));
        map.add_binding("slot_6", InputBinding::Key(AK::Key6));
        map.add_binding("slot_7", InputBinding::Key(AK::Key7));
        map.add_binding("slot_8", InputBinding::Key(AK::Key8));
        map.add_binding("slot_9", InputBinding::Key(AK::Key9));
        // Toggle actions
        map.add_binding("toggle_flying", InputBinding::Key(AK::Tab));
        map.add_binding("cycle_filter",  InputBinding::Key(AK::F1));
        ctx.app.insert_resource(map);
    }

    fn init_scene(&mut self, ctx: &mut GameContext) {

        // Try loading saved world first
        {
            let mut world = ctx.app.world.resource_mut::<VoxelWorld>();
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

        // Load player state
        match persistence::load_player() {
            Ok(Some(data)) => {
                self.apply_player_save_data(ctx, &data);
            }
            Ok(None) => {} // No saved player state
            Err(e) => {
                println!("Could not load player state: {}", e);
            }
        }

        // Request initial chunks asynchronously via the worker thread pool.
        // Chunks that were loaded from save are already in VoxelWorld;
        // the async workers will generate any that are missing.
        // The game starts immediately — chunks arrive via update() polling.
        {
            let world = ctx.app.world.resource::<VoxelWorld>();
            let load_radius = self.chunks.load_radius;
            // Only request chunks not already present from the save file
            for cx in -load_radius..=load_radius {
                for cz in -load_radius..=load_radius {
                    if !world.chunks.contains_key(&(cx, cz)) {
                        let _ = self.chunks.chunk_request_tx.send((cx, cz));
                        self.chunks.pending_chunks.insert((cx, cz));
                    }
                }
            }
        }

        // Now init GPU and upload
        let Some(device) = ctx.render_app.render_device() else {
            return;
        };
        let Some(format) = ctx.render_app.surface_format() else {
            return;
        };
        let (w, h) = ctx.render_app.window_state().size();

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
            let world = ctx.app.world.resource::<VoxelWorld>();
            self.chunks.upload_all(&world, device);
        }

        // Init HUD renderers
        self.line_renderer = Some(OverlayLineRenderer::new(device, format));
        self.text_renderer = Some(TextRenderer::new(device, format));
        self.ui_renderer = Some(anvilkit_render::renderer::ui::UiRenderer::new(device, format));

        self.voxel_gpu = Some(gpu);
        // initialization complete

        // Cursor is managed automatically by ScreenPlugin

        println!(
            "Craft initialized: {} chunks loaded",
            self.chunks.chunk_meshes.len()
        );
    }

    fn update_chunks(&mut self, ctx: &mut GameContext) {
        let cam_pos = {
            let cam = ctx.app.world.get_resource::<ActiveCamera>();
            match cam {
                Some(c) => c.camera_pos,
                None => return,
            }
        };
        let mut world = ctx.app.world.resource_mut::<VoxelWorld>();
        self.chunks.update(&mut world, cam_pos);
    }

    fn remesh_dirty_chunks(&mut self, ctx: &mut GameContext) {
        let Some(device) = ctx.render_app.render_device() else { return };
        let world = ctx.app.world.resource::<VoxelWorld>();
        self.chunks.remesh_dirty(&world, device);
    }

    fn handle_block_interaction(&mut self, ctx: &mut GameContext) {
        let (w, h) = ctx.render_app.window_state().size();
        let (cam_vp, cam_pos) = {
            let Some(cam) = ctx.app.world.get_resource::<ActiveCamera>() else { return };
            (cam.view_proj, cam.camera_pos)
        };

        // Cast ray from screen center
        let screen_center = glam::Vec2::new(w as f32 * 0.5, h as f32 * 0.5);
        let window_size = glam::Vec2::new(w as f32, h as f32);
        let (ray_origin, ray_dir) = screen_to_ray(screen_center, window_size, &cam_vp);

        // Raycast into voxel world
        let world = ctx.app.world.resource::<VoxelWorld>();
        self.current_hit = raycast::raycast_voxels(
            &world,
            [ray_origin.x, ray_origin.y, ray_origin.z],
            [ray_dir.x, ray_dir.y, ray_dir.z],
            config::RAYCAST_MAX_DIST,
        );

        // Check mouse buttons via ActionMap
        let (left_just, right_just) = {
            let actions = ctx.app.world.resource::<ActionMap>();
            (
                actions.is_action_just_pressed("break_block"),
                actions.is_action_just_pressed("place_block"),
            )
        };

        if let Some(ref hit) = self.current_hit {
            if left_just {
                // Break block — add broken block to inventory
                let [bx, by, bz] = hit.block_pos;
                let broken_block = {
                    let world = ctx.app.world.resource::<VoxelWorld>();
                    world.get_block(bx, by, bz)
                };
                if broken_block != BlockType::Air {
                    let mut world = ctx.app.world.resource_mut::<VoxelWorld>();
                    world.set_block(bx, by, bz, BlockType::Air);
                    self.chunks.mark_dirty_with_neighbors(bx, bz);

                    // Add to player inventory
                    let mut q = ctx.app.world.query::<&mut SlotInventory>();
                    for mut inv in q.iter_mut(&mut ctx.app.world) {
                        let _ = inv.add_item(
                            ItemStack::new(broken_block.item_id(), 1),
                            64,
                        );
                    }
                }
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
                    let selected = ctx.app.world.resource::<SelectedBlock>().block_type;
                    let item_id = selected.item_id();

                    // Check inventory has the block and consume one
                    let has_block = {
                        let mut q = ctx.app.world.query::<&mut SlotInventory>();
                        let mut found = false;
                        for mut inv in q.iter_mut(&mut ctx.app.world) {
                            if inv.remove_item(item_id, 1) > 0 {
                                found = true;
                                break;
                            }
                        }
                        found
                    };

                    if has_block {
                        let mut world = ctx.app.world.resource_mut::<VoxelWorld>();
                        world.set_block(px, py, pz, selected);
                        self.chunks.mark_dirty_with_neighbors(px, pz);
                    }
                }
            }
        }
    }

    fn render_3d_scene(&mut self, ctx: &mut GameContext, swapchain: &wgpu::TextureView) {
        let Some(device) = ctx.render_app.render_device() else {
            return;
        };
        let Some(ref gpu) = self.voxel_gpu else {
            return;
        };

        // Camera data
        let (cam_vp, cam_pos) = {
            let Some(cam) = ctx.app.world.get_resource::<ActiveCamera>() else {
                return;
            };
            (cam.view_proj, cam.camera_pos)
        };

        // Day/night cycle data
        let cycle = ctx.app.world.resource::<DayNightCycle>();
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
            let active_filter = ctx.app.world.resource::<ActiveFilter>();
            let cycle = ctx.app.world.resource::<DayNightCycle>();
            let is_srgb = ctx.render_app.surface_format()
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
            let ssao_settings = ctx.app.world.resource::<SsaoSettings>();
            // Build projection matrix (same as camera uses)
            let (w, h) = ctx.render_app.window_state().size();
            let aspect = w as f32 / h as f32;
            let projection = glam::Mat4::perspective_rh(
                config::FOV.to_radians(), aspect, config::NEAR_PLANE, config::FAR_PLANE,
            );
            gpu.ssao.execute(device, &mut enc, &gpu.depth_view, &projection, &ssao_settings);
        }

        // Bloom passes: downsample → upsample
        {
            let bloom_settings = ctx.app.world.resource::<BloomSettings>();
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
        self.render_hud(ctx, swapchain, &cam_vp, cam_pos);
    }

    fn render_hud(
        &mut self,
        ctx: &mut GameContext,
        swapchain: &wgpu::TextureView,
        cam_vp: &glam::Mat4,
        cam_pos: glam::Vec3,
    ) {
        let Some(device) = ctx.render_app.render_device() else { return };
        let (sw, sh) = ctx.render_app.window_state().size();
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
        let selected = ctx.app.world.resource::<SelectedBlock>();
        let player = ctx.app.world.resource::<PlayerState>();
        let cycle = ctx.app.world.resource::<DayNightCycle>();
        let selected_index = selected.index;

        // Use locale for display names
        let locale = ctx.app.world.get_resource::<anvilkit_data::Locale>();
        let fly_text = locale.map_or("FLY", |l| l.t("hud.fly"));
        let walk_text = locale.map_or("WALK", |l| l.t("hud.walk"));
        let block_name = locale.map_or_else(
            || format!("{:?}", selected.block_type),
            |l| l.t(selected.block_type.locale_key()).to_string(),
        );

        if let Some(ref mut tr) = self.text_renderer {
            let coord_text = format!(
                "XYZ: {:.1} {:.1} {:.1}  {}  Time: {:.0}%",
                cam_pos.x, cam_pos.y, cam_pos.z,
                if player.flying { fly_text } else { walk_text },
                cycle.time * 100.0,
            );
            let block_text = format!("Block: {}", block_name);

            tr.draw_text(device, &mut enc, swapchain, &coord_text, 8.0, 8.0, 16.0, white, sw, sh);
            tr.draw_text(device, &mut enc, swapchain, &block_text, 8.0, 28.0, 16.0, white, sw, sh);
        }

        // Health bar: rendered above the hotbar
        {
            use anvilkit_render::renderer::ui::UiNode;

            let hp_fraction = {
                let mut q = ctx.app.world.query::<&Health>();
                q.iter(&ctx.app.world).next().map(|h| h.fraction()).unwrap_or(1.0)
            };
            let hp_text = {
                let mut q = ctx.app.world.query::<&Health>();
                q.iter(&ctx.app.world).next().map(|h| format!("{:.0}/{:.0}", h.current, h.max)).unwrap_or_default()
            };

            let bar_w = 200.0_f32;
            let bar_h = 12.0_f32;
            let bar_x = (sw - bar_w) * 0.5;
            let bar_y = sh - 36.0 - 16.0 - bar_h - 8.0; // above hotbar

            // Background bar (dark)
            let bg_node = UiNode {
                background_color: [0.15, 0.0, 0.0, 0.7],
                corner_radius: 3.0,
                border_width: 1.0,
                border_color: [0.3, 0.0, 0.0, 0.8],
                text: None,
                style: Default::default(),
                visible: true,
                computed_rect: [bar_x, bar_y, bar_w, bar_h],
            };

            // Foreground bar (red/green based on health)
            let fill_w = bar_w * hp_fraction;
            let fill_color = if hp_fraction > 0.5 {
                [0.2, 0.8, 0.2, 0.9] // green
            } else if hp_fraction > 0.25 {
                [0.9, 0.7, 0.1, 0.9] // yellow
            } else {
                [0.9, 0.1, 0.1, 0.9] // red
            };
            let fg_node = UiNode {
                background_color: fill_color,
                corner_radius: 3.0,
                border_width: 0.0,
                border_color: [0.0; 4],
                text: None,
                style: Default::default(),
                visible: true,
                computed_rect: [bar_x, bar_y, fill_w, bar_h],
            };

            let nodes = [&bg_node, &fg_node];
            if let Some(ref mut ui) = self.ui_renderer {
                ui.render(device, &mut enc, swapchain, &nodes, sw, sh);
            }

            // HP text centered on bar
            if let Some(ref mut tr) = self.text_renderer {
                let text_x = bar_x + bar_w * 0.5 - (hp_text.len() as f32 * 3.5);
                let text_y = bar_y - 1.0;
                tr.draw_text(device, &mut enc, swapchain, &hp_text, text_x, text_y, 11.0, white, sw, sh);
            }
        }

        // Hotbar: 9-slot block selection bar at bottom center
        {
            use anvilkit_render::renderer::ui::UiNode;

            let slot_size = 48.0_f32;
            let slot_gap = 4.0_f32;
            let total_w = 9.0 * slot_size + 8.0 * slot_gap;
            let bar_x = (sw - total_w) * 0.5;
            let bar_y = sh - slot_size - 12.0;

            // Slot backgrounds
            let mut hotbar_nodes: Vec<UiNode> = Vec::with_capacity(9);
            for i in 0..9usize {
                let is_selected = i == selected_index;
                let (bg, border_color) = if is_selected {
                    ([0.15, 0.15, 0.1, 0.75], [1.0, 1.0, 0.0, 1.0])
                } else {
                    ([0.08, 0.08, 0.08, 0.6], [0.4, 0.4, 0.4, 0.7])
                };
                let sx = bar_x + i as f32 * (slot_size + slot_gap);
                hotbar_nodes.push(UiNode {
                    background_color: bg,
                    corner_radius: 6.0,
                    border_width: if is_selected { 2.0 } else { 1.0 },
                    border_color,
                    text: None,
                    style: Default::default(),
                    visible: true,
                    computed_rect: [sx, bar_y, slot_size, slot_size],
                });
            }

            let node_refs: Vec<&UiNode> = hotbar_nodes.iter().collect();
            if let Some(ref mut ui) = self.ui_renderer {
                ui.render(device, &mut enc, swapchain, &node_refs, sw, sh);
            }

            // Draw block names inside slots + slot number
            if let Some(ref mut tr) = self.text_renderer {
                let locale = ctx.app.world.get_resource::<anvilkit_data::Locale>();
                let highlight = glam::Vec3::new(1.0, 1.0, 0.0);
                let dim = glam::Vec3::new(0.6, 0.6, 0.6);

                for i in 0..9usize {
                    let block = config::BLOCK_PALETTE[i];
                    let is_selected = i == selected_index;
                    let sx = bar_x + i as f32 * (slot_size + slot_gap);

                    // Slot number (top-left corner, small)
                    let num = format!("{}", i + 1);
                    tr.draw_text(device, &mut enc, swapchain, &num,
                        sx + 3.0, bar_y + 2.0, 10.0, dim, sw, sh);

                    // Block name (centered, larger)
                    let name = locale.map_or_else(
                        || format!("{:?}", block),
                        |l| l.t(block.locale_key()).to_string(),
                    );
                    // Show up to 5 chars to fit in slot
                    let short: String = name.chars().take(5).collect();
                    let text_w = short.len() as f32 * 11.0 * 0.5;
                    let tx = sx + (slot_size - text_w) * 0.5;
                    let ty = bar_y + slot_size * 0.3;
                    let color = if is_selected { highlight } else { white };
                    tr.draw_text(device, &mut enc, swapchain, &short,
                        tx, ty, 11.0, color, sw, sh);

                    // Quantity "x64" (bottom-right)
                    tr.draw_text(device, &mut enc, swapchain, "x64",
                        sx + slot_size - 20.0, bar_y + slot_size - 13.0, 9.0, dim, sw, sh);
                }
            }

            // Selected block full name below hotbar
            if let Some(ref mut tr) = self.text_renderer {
                let locale = ctx.app.world.get_resource::<anvilkit_data::Locale>();
                let full_name = locale.map_or_else(
                    || format!("{:?}", config::BLOCK_PALETTE[selected_index]),
                    |l| l.t(config::BLOCK_PALETTE[selected_index].locale_key()).to_string(),
                );
                let text_w = full_name.len() as f32 * 12.0 * 0.5;
                let tx = (sw - text_w) * 0.5;
                let ty = bar_y + slot_size + 2.0;
                tr.draw_text(device, &mut enc, swapchain, &full_name,
                    tx, ty, 12.0, glam::Vec3::new(0.9, 0.9, 0.9), sw, sh);
            }
        }

        device.queue().submit(std::iter::once(enc.finish()));
    }

    /// Render egui UI overlay on the given swapchain view.
    fn render_egui_ui(&mut self, ctx: &mut GameContext, swapchain: &wgpu::TextureView) {
        let Some(ref mut egui) = ctx.egui else { return; };
        let Some(device) = ctx.render_app.render_device() else { return; };
        let ws = ctx.render_app.window_state();
        let (w, h) = ws.size();

        // Draw UI using egui
        let egui_ctx = egui.ctx.clone();
        let screen = ctx.app.world.resource::<GameState<CraftScreen>>().0;
        let return_to = *ctx.app.world.resource::<SettingsReturnTo>();

        let next_screen = match screen {
            CraftScreen::MainMenu => {
                // For main menu, also draw a dark background
                egui::CentralPanel::default()
                    .frame(egui::Frame::none().fill(egui::Color32::from_rgb(10, 10, 15)))
                    .show(&egui_ctx, |_ui| {});
                craft::ui::main_menu::draw(&egui_ctx)
            }
            CraftScreen::Paused => craft::ui::pause_menu::draw(&egui_ctx),
            CraftScreen::Settings => {
                craft::ui::settings_menu::draw(&egui_ctx, &mut self.settings_state, return_to)
            }
            CraftScreen::Inventory => {
                let mut sel = ctx.app.world.resource::<SelectedBlock>().index;
                let slots: Vec<(String, u32)> = {
                    let locale = ctx.app.world.resource::<anvilkit_data::Locale>();
                    config::BLOCK_PALETTE.iter().map(|bt| {
                        (locale.t(bt.locale_key()).to_string(), 64)
                    }).collect()
                };
                let result = craft::ui::inventory::draw(&egui_ctx, &slots, &mut sel);
                ctx.app.world.resource_mut::<SelectedBlock>().index = sel;
                result
            }
            CraftScreen::Playing => None,
            _ => None,
        };

        // Handle state transitions from UI
        if let Some(target) = next_screen {
            match target {
                CraftScreen::Quit => { ctx.app.exit(); }
                CraftScreen::SaveAndQuit => {
                    let save_seed = ctx.app.world.resource::<WorldSeed>().0;
                    let world = ctx.app.world.resource::<VoxelWorld>();
                    let _ = persistence::save_world(&world, &world.modified_chunks, save_seed);
                    ctx.app.world.resource_mut::<NextGameState<CraftScreen>>()
                        .set(CraftScreen::MainMenu);
                }
                CraftScreen::Settings => {
                    let from = ctx.app.world.resource::<GameState<CraftScreen>>().0;
                    *ctx.app.world.resource_mut::<SettingsReturnTo>() = match from {
                        CraftScreen::MainMenu => SettingsReturnTo::MainMenu,
                        _ => SettingsReturnTo::Paused,
                    };
                    ctx.app.world.resource_mut::<NextGameState<CraftScreen>>()
                        .set(CraftScreen::Settings);
                }
                other => {
                    ctx.app.world.resource_mut::<NextGameState<CraftScreen>>()
                        .set(other);
                }
            }
        }

        // End egui frame and render to swapchain
        if let Some(window) = ctx.render_app.window().cloned() {
            let mut enc = device.device().create_command_encoder(
                &wgpu::CommandEncoderDescriptor { label: Some("egui enc") },
            );
            egui.end_frame_and_render(
                device.device(), device.queue(),
                &mut enc, &swapchain, &window, w, h,
            );
            device.queue().submit(std::iter::once(enc.finish()));
        }
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

impl GameCallbacks for CraftGame {
    fn init(&mut self, ctx: &mut GameContext) {
        self.init_scene(ctx);
        self.setup_action_map(ctx);
    }

    fn on_resize(&mut self, ctx: &mut GameContext, width: u32, height: u32) {
        if let Some(device) = ctx.render_app.render_device() {
            if let Some(ref mut gpu) = self.voxel_gpu {
                let (_, dv) = create_depth_texture(device, width, height, "Voxel Depth");
                gpu.depth_view = dv;
                let (_, hv) =
                    create_hdr_render_target(device, width, height, "Voxel HDR RT");
                let samp = create_sampler(device, "Tonemap Sampler");
                let bloom_mips = ctx.app.world.resource::<BloomSettings>().mip_count;
                gpu.bloom.resize(device, width, height, bloom_mips);
                gpu.ssao.resize(device, width, height);
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

    fn on_window_event(&mut self, ctx: &mut GameContext, ev: &WindowEvent) -> bool {
        match ev {
            WindowEvent::KeyboardInput { event, .. } => {
                use winit::keyboard::{KeyCode as WK, PhysicalKey};
                if let PhysicalKey::Code(code) = event.physical_key {
                    if event.state.is_pressed() {
                        // Ctrl+S: save world
                        if code == WK::KeyS && !event.repeat {
                            let input = ctx.app.world.resource::<InputState>();
                            if input.is_key_pressed(anvilkit_input::prelude::KeyCode::LControl)
                                || input.is_key_pressed(anvilkit_input::prelude::KeyCode::RControl)
                            {
                                let save_seed = ctx.app.world.resource::<WorldSeed>().0;
                                let world = ctx.app.world.resource::<VoxelWorld>();
                                match persistence::save_world(&world, &world.modified_chunks, save_seed) {
                                    Ok(n) => println!("Saved {} modified chunks to {:?}", n, persistence::save_path()),
                                    Err(e) => println!("Save failed: {}", e),
                                }

                                // Save player state
                                let player_data = self.gather_player_save_data(ctx);
                                match persistence::save_player(&player_data) {
                                    Ok(()) => println!("Player state saved"),
                                    Err(e) => println!("Player save failed: {}", e),
                                }
                            }
                        }
                        match code {
                            WK::Escape => {
                                let current = ctx.app.world.resource::<GameState<CraftScreen>>().0;
                                let settings_ret = *ctx.app.world.resource::<SettingsReturnTo>();
                                let target = match current {
                                    CraftScreen::Playing => Some(CraftScreen::Paused),
                                    CraftScreen::Paused => Some(CraftScreen::Playing),
                                    CraftScreen::Inventory => Some(CraftScreen::Playing),
                                    CraftScreen::Settings => Some(match settings_ret {
                                        SettingsReturnTo::MainMenu => CraftScreen::MainMenu,
                                        SettingsReturnTo::Paused => CraftScreen::Paused,
                                    }),
                                    CraftScreen::MainMenu | CraftScreen::Quit | CraftScreen::SaveAndQuit => None,
                                };
                                if let Some(t) = target {
                                    ctx.app.world.resource_mut::<NextGameState<CraftScreen>>().set(t);
                                } else {
                                    ctx.app.exit();
                                }
                                return true;
                            }
                            WK::KeyE if !event.repeat => {
                                let current = ctx.app.world.resource::<GameState<CraftScreen>>().0;
                                let mut next = ctx.app.world.resource_mut::<NextGameState<CraftScreen>>();
                                match current {
                                    CraftScreen::Playing => next.set(CraftScreen::Inventory),
                                    CraftScreen::Inventory => next.set(CraftScreen::Playing),
                                    _ => {}
                                }
                                return true;
                            }
                            WK::F5 => {
                                let mut q = ctx.app.world.query::<&mut CameraController>();
                                for mut ctrl in q.iter_mut(&mut ctx.app.world) {
                                    ctrl.toggle_perspective();
                                    let mode_name = match &ctrl.mode {
                                        CameraMode::FirstPerson => "First Person",
                                        CameraMode::ThirdPerson => "Third Person",
                                        CameraMode::Orbit => "Orbit",
                                        CameraMode::Free => "Free",
                                        CameraMode::Rail => "Rail",
                                    };
                                    println!("Camera: {}", mode_name);
                                }
                            }
                            // Tab, F1, Digit1-9 handled by ECS systems
                            // (toggle_actions_system + hotbar_selection_system)
                            _ => {}
                        }
                    }
                }
                false // let engine also process for InputState forwarding
            }
            _ => false,
        }
    }

    fn render(&mut self, ctx: &mut GameContext) {
        // Apply theme once
        if !self.theme_applied {
            if let Some(ref egui) = ctx.egui {
                craft::ui::theme::apply_craft_theme(&egui.ctx);
                self.theme_applied = true;
            }
        }

        // Acquire ONE frame for both 3D and egui
        if ctx.render_app.render_device().is_none() { return; }
        let Some(frame) = ctx.render_app.get_current_frame() else { return; };
        let swapchain = frame.texture.create_view(&Default::default());

        let screen = ctx.app.world.resource::<GameState<CraftScreen>>().0;

        // Render 3D scene + HUD (skipped for MainMenu)
        if screen != CraftScreen::MainMenu {
            self.render_3d_scene(ctx, &swapchain);
        }

        // Render egui UI on top (menus, settings, inventory)
        self.render_egui_ui(ctx, &swapchain);

        // Present the single frame with both 3D + egui content
        frame.present();
    }

    fn post_update(&mut self, ctx: &mut GameContext) {
        self.frame_count += 1;
        self.do_post_update(ctx);
    }
}

impl CraftGame {
    // Pre-update logic is now handled by ECS systems:
    // - day_night_system: advances DayNightCycle
    // - camera_effects_system: landing shake, sprint FOV, third-person target

    /// Post-update: game logic that depends on ECS system results.
    fn do_post_update(&mut self, ctx: &mut GameContext) {
        // Periodic debug log (every 120 frames ~ 2 sec)
        if self.frame_count % 120 == 0 {
            let flying = ctx.app.world.resource::<PlayerState>().flying;
            let on_ground = ctx.app.world.resource::<PlayerState>().on_ground;
            let vel = {
                let mut q = ctx.app.world.query::<&Velocity>();
                q.iter(&ctx.app.world).next().map(|v| v.linear).unwrap_or(glam::Vec3::ZERO)
            };
            if let Some(cam) = ctx.app.world.get_resource::<ActiveCamera>() {
                let p = cam.camera_pos;
                log::debug!(
                    "[F{}] pos=({:.1},{:.1},{:.1}) vel=({:.1},{:.1},{:.1}) fly={} gnd={}",
                    self.frame_count,
                    p.x, p.y, p.z,
                    vel.x, vel.y, vel.z,
                    flying, on_ground,
                );
            }
        }

        // Block interaction — only when playing
        let screen = ctx.app.world.resource::<GameState<CraftScreen>>().0;
        if screen == CraftScreen::Playing {
            self.handle_block_interaction(ctx);
        }

        // Remesh any dirty chunks
        self.remesh_dirty_chunks(ctx);

        // Dynamic chunk loading
        self.update_chunks(ctx);
    }

    /// Gather all player state into a serializable struct for saving.
    fn gather_player_save_data(&self, ctx: &mut GameContext) -> persistence::PlayerSaveData {
        let cam_pos = ctx.app.world.get_resource::<ActiveCamera>()
            .map(|c| c.camera_pos)
            .unwrap_or(glam::Vec3::ZERO);

        // Copy resource values to avoid borrow conflicts with queries below
        let flying = ctx.app.world.resource::<PlayerState>().flying;
        let day_night_time = ctx.app.world.resource::<DayNightCycle>().time;
        let selected_slot = ctx.app.world.resource::<SelectedBlock>().index;

        let (health, max_health) = {
            let mut q = ctx.app.world.query::<&Health>();
            q.iter(&ctx.app.world).next()
                .map(|h| (h.current, h.max))
                .unwrap_or((20.0, 20.0))
        };

        let inventory = {
            let mut q = ctx.app.world.query::<&SlotInventory>();
            q.iter(&ctx.app.world).next()
                .map(|inv| {
                    (0..9).map(|i| {
                        inv.get_slot(i).map(|s| (s.item_id, s.quantity))
                    }).collect()
                })
                .unwrap_or_else(Vec::new)
        };

        persistence::PlayerSaveData {
            position: [cam_pos.x, cam_pos.y, cam_pos.z],
            health,
            max_health,
            flying,
            day_night_time,
            inventory,
            selected_slot,
        }
    }

    /// Apply loaded player state to the ECS world.
    fn apply_player_save_data(&self, ctx: &mut GameContext, data: &persistence::PlayerSaveData) {
        // Position
        {
            let mut q = ctx.app.world.query::<&mut Transform>();
            for mut transform in q.iter_mut(&mut ctx.app.world) {
                transform.translation = glam::Vec3::new(
                    data.position[0],
                    data.position[1],
                    data.position[2],
                );
            }
        }

        // Health
        {
            let mut q = ctx.app.world.query::<&mut Health>();
            for mut hp in q.iter_mut(&mut ctx.app.world) {
                hp.max = data.max_health;
                hp.current = data.health.min(data.max_health);
            }
        }

        // Inventory
        {
            let mut q = ctx.app.world.query::<&mut SlotInventory>();
            for mut inv in q.iter_mut(&mut ctx.app.world) {
                for (i, slot_data) in data.inventory.iter().enumerate() {
                    if i >= 9 { break; }
                    match slot_data {
                        Some((item_id, qty)) if *qty > 0 => {
                            inv.set_slot(i, Some(ItemStack::new(*item_id, *qty)));
                        }
                        _ => {
                            inv.set_slot(i, None);
                        }
                    }
                }
            }
        }

        // Player state
        {
            let mut player = ctx.app.world.resource_mut::<PlayerState>();
            player.flying = data.flying;
        }

        // Day/night cycle
        {
            let mut cycle = ctx.app.world.resource_mut::<DayNightCycle>();
            cycle.time = data.day_night_time;
        }

        // Selected block
        {
            let mut selected = ctx.app.world.resource_mut::<SelectedBlock>();
            selected.index = data.selected_slot;
            if data.selected_slot < config::BLOCK_PALETTE.len() {
                selected.block_type = config::BLOCK_PALETTE[data.selected_slot];
            }
        }

        println!("Player state restored: pos=({:.1},{:.1},{:.1}) hp={:.0}/{:.0} fly={}",
            data.position[0], data.position[1], data.position[2],
            data.health, data.max_health, data.flying);
    }
}

