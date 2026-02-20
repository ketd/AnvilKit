use std::collections::{HashMap, HashSet};


use anvilkit_render::prelude::*;
use anvilkit_render::renderer::{
    buffer::{
        create_vertex_buffer, create_index_buffer_u32,
        create_depth_texture, create_hdr_render_target, create_sampler,
    },
    draw::ActiveCamera,
    line::LineRenderer,
    text::TextRenderer,
    raycast::screen_to_ray,
};
use anvilkit_render::plugin::CameraComponent;
use anvilkit_input::prelude::{InputState, MouseButton};
use anvilkit_ecs::physics::DeltaTime;

use craft::block::BlockType;
use craft::chunk::CHUNK_SIZE;
use craft::world_gen::WorldGenerator;
use craft::mesh;
use craft::raycast::{self, VoxelHit};
use craft::render::setup::{self, VoxelGpu, VoxelSceneUniform, SkyUniform};
use craft::components::*;
use craft::resources::*;
use craft::systems::input as input_sys;
use craft::systems::physics as physics_sys;

/// Per-chunk GPU buffers (not managed by RenderAssets since we do custom draw).
struct ChunkGpuMesh {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
    /// Chunk center for frustum culling (world space).
    center: glam::Vec3,
}

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

fn main() {
    env_logger::init();
    println!("Craft — powered by AnvilKit");
    println!("  WASD = move, Mouse = look, Space = jump/up, Shift = down");
    println!("  Tab = toggle flying, LMB = break, RMB = place");
    println!("  1-9 = select block, ESC = quit");

    let mut app = App::new();
    app.add_plugins(RenderPlugin::new().with_window_config(
        WindowConfig::new()
            .with_title("Craft")
            .with_size(1280, 720),
    ));

    app.insert_resource(InputState::new());
    app.insert_resource(DeltaTime(1.0 / 60.0));
    app.insert_resource(PlayerState::default());
    app.insert_resource(MouseDelta::default());
    app.insert_resource(VoxelWorld::default());
    app.insert_resource(SelectedBlock::default());
    app.insert_resource(DayNightCycle::default());

    app.add_systems(AnvilKitSchedule::Update, input_sys::fps_camera_system);
    app.add_systems(AnvilKitSchedule::Update, physics_sys::player_physics_system);

    // FPS Camera — spawn at a reasonable height above terrain
    let spawn_pos = glam::Vec3::new(
        (CHUNK_SIZE as f32) * 3.5,
        50.0,
        (CHUNK_SIZE as f32) * 3.5,
    );
    app.world.spawn((
        FpsCamera,
        CameraComponent {
            fov: 70.0,
            near: 0.1,
            far: 500.0,
            is_active: true,
            aspect_ratio: 1280.0 / 720.0,
        },
        Transform::from_xyz(spawn_pos.x, spawn_pos.y, spawn_pos.z),
    ));

    let event_loop = EventLoop::new().unwrap();
    let wconfig = WindowConfig::new().with_title("Craft").with_size(1280, 720);
    event_loop
        .run_app(&mut CraftApp {
            render_app: RenderApp::new(wconfig),
            app,
            initialized: false,
            voxel_gpu: None,
            chunk_meshes: HashMap::new(),
            world_gen: WorldGenerator::new(42),
            load_radius: 7,
            last_chunk_pos: (i32::MAX, i32::MAX),
            line_renderer: None,
            text_renderer: None,
            dirty_chunks: HashSet::new(),
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
    chunk_meshes: HashMap<(i32, i32), ChunkGpuMesh>,
    world_gen: WorldGenerator,
    load_radius: i32,
    last_chunk_pos: (i32, i32),
    // HUD renderers
    line_renderer: Option<LineRenderer>,
    text_renderer: Option<TextRenderer>,
    // Dirty chunks needing remesh
    dirty_chunks: HashSet<(i32, i32)>,
    // Current raycast hit (updated each frame)
    current_hit: Option<VoxelHit>,
    // Debug frame counter
    frame_count: u64,
}

fn generate_chunks(
    world: &mut VoxelWorld,
    world_gen: &WorldGenerator,
    center_cx: i32,
    center_cz: i32,
    radius: i32,
) {
    for cx in (center_cx - radius)..=(center_cx + radius) {
        for cz in (center_cz - radius)..=(center_cz + radius) {
            if !world.chunks.contains_key(&(cx, cz)) {
                let chunk = world_gen.generate_chunk(cx, cz);
                world.chunks.insert((cx, cz), chunk);
            }
        }
    }
}

fn upload_new_chunks(
    world: &VoxelWorld,
    chunk_meshes: &mut HashMap<(i32, i32), ChunkGpuMesh>,
    device: &RenderDevice,
) {
    let keys: Vec<(i32, i32)> = world
        .chunks
        .keys()
        .copied()
        .filter(|k| !chunk_meshes.contains_key(k))
        .collect();

    for (cx, cz) in keys {
        mesh_and_upload_chunk(world, chunk_meshes, device, cx, cz);
    }
}

fn mesh_and_upload_chunk(
    world: &VoxelWorld,
    chunk_meshes: &mut HashMap<(i32, i32), ChunkGpuMesh>,
    device: &RenderDevice,
    cx: i32,
    cz: i32,
) {
    let Some(chunk) = world.chunks.get(&(cx, cz)) else { return };
    let neighbors = [
        world.chunks.get(&(cx + 1, cz)),
        world.chunks.get(&(cx - 1, cz)),
        world.chunks.get(&(cx, cz + 1)),
        world.chunks.get(&(cx, cz - 1)),
    ];
    let ox = (cx * CHUNK_SIZE as i32) as f32;
    let oz = (cz * CHUNK_SIZE as i32) as f32;
    let cm = mesh::mesh_chunk(chunk, &neighbors, ox, oz);

    if cm.indices.is_empty() {
        chunk_meshes.remove(&(cx, cz));
        return;
    }

    let vb = create_vertex_buffer(device, &format!("Chunk({},{}) VB", cx, cz), &cm.vertices);
    let ib = create_index_buffer_u32(device, &format!("Chunk({},{}) IB", cx, cz), &cm.indices);
    let center = glam::Vec3::new(
        ox + CHUNK_SIZE as f32 * 0.5,
        128.0,
        oz + CHUNK_SIZE as f32 * 0.5,
    );

    chunk_meshes.insert(
        (cx, cz),
        ChunkGpuMesh {
            vertex_buffer: vb,
            index_buffer: ib,
            index_count: cm.indices.len() as u32,
            center,
        },
    );
}

impl CraftApp {
    fn init_scene(&mut self) {
        if self.initialized {
            return;
        }

        // Generate chunks first (doesn't need GPU)
        {
            let mut world = self.app.world.resource_mut::<VoxelWorld>();
            generate_chunks(&mut world, &self.world_gen, 0, 0, self.load_radius);
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
        let mut atlas_img = image::open(atlas_path)
            .expect("Failed to load texture.png")
            .to_rgba8();
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
            upload_new_chunks(&world, &mut self.chunk_meshes, device);
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
            self.chunk_meshes.len()
        );
    }

    fn update_chunks(&mut self) {
        // Determine which chunk the camera is in
        let cam_pos = {
            let cam = self.app.world.get_resource::<ActiveCamera>();
            match cam {
                Some(c) => c.camera_pos,
                None => return,
            }
        };
        let cx = (cam_pos.x / CHUNK_SIZE as f32).floor() as i32;
        let cz = (cam_pos.z / CHUNK_SIZE as f32).floor() as i32;

        if (cx, cz) == self.last_chunk_pos {
            return;
        }
        self.last_chunk_pos = (cx, cz);

        // Generate new chunks (no GPU needed)
        {
            let mut world = self.app.world.resource_mut::<VoxelWorld>();
            generate_chunks(&mut world, &self.world_gen, cx, cz, self.load_radius);
        }

        // Upload new meshes (needs GPU)
        {
            let Some(device) = self.render_app.render_device() else {
                return;
            };
            let world = self.app.world.resource::<VoxelWorld>();
            upload_new_chunks(&world, &mut self.chunk_meshes, device);
        }

        // Unload far chunks
        let r = self.load_radius + 2;
        let far_keys: Vec<(i32, i32)> = self
            .chunk_meshes
            .keys()
            .copied()
            .filter(|&(kx, kz)| (kx - cx).abs() > r || (kz - cz).abs() > r)
            .collect();
        for key in &far_keys {
            self.chunk_meshes.remove(key);
        }
        let mut world = self.app.world.resource_mut::<VoxelWorld>();
        for key in far_keys {
            world.chunks.remove(&key);
        }
    }

    fn remesh_dirty_chunks(&mut self) {
        if self.dirty_chunks.is_empty() {
            return;
        }
        let Some(device) = self.render_app.render_device() else { return };
        let dirty: Vec<(i32, i32)> = self.dirty_chunks.drain().collect();
        let world = self.app.world.resource::<VoxelWorld>();
        for (cx, cz) in dirty {
            mesh_and_upload_chunk(&world, &mut self.chunk_meshes, device, cx, cz);
        }
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
            10.0,
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
                let chunk_cx = bx.div_euclid(CHUNK_SIZE as i32);
                let chunk_cz = bz.div_euclid(CHUNK_SIZE as i32);
                self.dirty_chunks.insert((chunk_cx, chunk_cz));
                // Mark neighbor chunks dirty if on boundary
                let lx = bx.rem_euclid(CHUNK_SIZE as i32);
                let lz = bz.rem_euclid(CHUNK_SIZE as i32);
                if lx == 0 { self.dirty_chunks.insert((chunk_cx - 1, chunk_cz)); }
                if lx == CHUNK_SIZE as i32 - 1 { self.dirty_chunks.insert((chunk_cx + 1, chunk_cz)); }
                if lz == 0 { self.dirty_chunks.insert((chunk_cx, chunk_cz - 1)); }
                if lz == CHUNK_SIZE as i32 - 1 { self.dirty_chunks.insert((chunk_cx, chunk_cz + 1)); }
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
                    let chunk_cx = px.div_euclid(CHUNK_SIZE as i32);
                    let chunk_cz = pz.div_euclid(CHUNK_SIZE as i32);
                    self.dirty_chunks.insert((chunk_cx, chunk_cz));
                    let lx = px.rem_euclid(CHUNK_SIZE as i32);
                    let lz = pz.rem_euclid(CHUNK_SIZE as i32);
                    if lx == 0 { self.dirty_chunks.insert((chunk_cx - 1, chunk_cz)); }
                    if lx == CHUNK_SIZE as i32 - 1 { self.dirty_chunks.insert((chunk_cx + 1, chunk_cz)); }
                    if lz == 0 { self.dirty_chunks.insert((chunk_cx, chunk_cz - 1)); }
                    if lz == CHUNK_SIZE as i32 - 1 { self.dirty_chunks.insert((chunk_cx, chunk_cz + 1)); }
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
        let frustum_planes = extract_frustum_planes(&cam_vp);

        // --- Pass 1: Sky (HDR RT, Clear) ---
        {
            let mut enc = device
                .device()
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Sky Enc"),
                });
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
            device.queue().submit(std::iter::once(enc.finish()));
        }

        // --- Pass 2: Voxel scene (HDR RT, Load, depth test) ---
        {
            let mut enc = device
                .device()
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Voxel Scene Enc"),
                });
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

                for (_key, cm) in &self.chunk_meshes {
                    if !sphere_in_frustum(&frustum_planes, cm.center, 130.0) {
                        continue;
                    }
                    rp.set_vertex_buffer(0, cm.vertex_buffer.slice(..));
                    rp.set_index_buffer(cm.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                    rp.draw_indexed(0..cm.index_count, 0, 0..1);
                }
            }
            device.queue().submit(std::iter::once(enc.finish()));
        }

        // --- Pass 3: Tonemap (HDR → swapchain) ---
        {
            let mut enc = device
                .device()
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Tonemap Enc"),
                });
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
            device.queue().submit(std::iter::once(enc.finish()));
        }

        // --- Pass 4: HUD (crosshair, coordinates, block highlight) ---
        self.render_hud(device, &swapchain, &cam_vp, cam_pos);

        frame.present();
    }

    fn render_hud(
        &self,
        device: &RenderDevice,
        swapchain: &wgpu::TextureView,
        cam_vp: &glam::Mat4,
        cam_pos: glam::Vec3,
    ) {
        let (sw, sh) = self.render_app.window_state().size();
        let sw = sw as f32;
        let sh = sh as f32;

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
        if let Some(ref lr) = self.line_renderer {
            let mut enc = device.device().create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("HUD Lines Enc"),
            });
            lr.render(device, &mut enc, swapchain, &lines, &ortho);
            device.queue().submit(std::iter::once(enc.finish()));
        }

        // Block highlight wireframe (3D)
        if let Some(ref hit) = self.current_hit {
            if let Some(ref lr) = self.line_renderer {
                let wireframe_lines = block_wireframe(hit.block_pos, glam::Vec3::new(0.2, 0.2, 0.2));
                let mut enc = device.device().create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Block Highlight Enc"),
                });
                lr.render(device, &mut enc, swapchain, &wireframe_lines, cam_vp);
                device.queue().submit(std::iter::once(enc.finish()));
            }
        }

        // Text: coordinates and selected block
        if let Some(ref tr) = self.text_renderer {
            let selected = self.app.world.resource::<SelectedBlock>();
            let player = self.app.world.resource::<PlayerState>();
            let cycle = self.app.world.resource::<DayNightCycle>();

            let coord_text = format!(
                "XYZ: {:.1} {:.1} {:.1}  {}  Time: {:.0}%",
                cam_pos.x, cam_pos.y, cam_pos.z,
                if player.flying { "FLY" } else { "WALK" },
                cycle.time * 100.0,
            );
            let block_text = format!("Block: {:?}", selected.block_type);

            let mut enc = device.device().create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("HUD Text Enc"),
            });
            tr.draw_text(device, &mut enc, swapchain, &coord_text, 8.0, 8.0, 16.0, white, sw, sh);
            tr.draw_text(device, &mut enc, swapchain, &block_text, 8.0, 28.0, 16.0, white, sw, sh);
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

impl ApplicationHandler for CraftApp {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        self.render_app.resumed(el);
        self.init_scene();
    }

    fn window_event(&mut self, el: &ActiveEventLoop, wid: WindowId, ev: WindowEvent) {
        match &ev {
            WindowEvent::Resized(s) if self.initialized && s.width > 0 && s.height > 0 => {
                if let Some(device) = self.render_app.render_device() {
                    if let Some(ref mut gpu) = self.voxel_gpu {
                        let (_, dv) = create_depth_texture(device, s.width, s.height, "Voxel Depth");
                        gpu.depth_view = dv;
                        let (_, hv) =
                            create_hdr_render_target(device, s.width, s.height, "Voxel HDR RT");
                        let samp = create_sampler(device, "Tonemap Sampler");
                        // Recreate tonemap bind group
                        gpu.tonemap_bg =
                            device
                                .device()
                                .create_bind_group(&wgpu::BindGroupDescriptor {
                                    label: Some("Tonemap BG"),
                                    layout: &device.device().create_bind_group_layout(
                                        &wgpu::BindGroupLayoutDescriptor {
                                            label: Some("Tonemap BGL"),
                                            entries: &[
                                                wgpu::BindGroupLayoutEntry {
                                                    binding: 0,
                                                    visibility: wgpu::ShaderStages::FRAGMENT,
                                                    ty: wgpu::BindingType::Texture {
                                                        sample_type:
                                                            wgpu::TextureSampleType::Float {
                                                                filterable: true,
                                                            },
                                                        view_dimension:
                                                            wgpu::TextureViewDimension::D2,
                                                        multisampled: false,
                                                    },
                                                    count: None,
                                                },
                                                wgpu::BindGroupLayoutEntry {
                                                    binding: 1,
                                                    visibility: wgpu::ShaderStages::FRAGMENT,
                                                    ty: wgpu::BindingType::Sampler(
                                                        wgpu::SamplerBindingType::Filtering,
                                                    ),
                                                    count: None,
                                                },
                                            ],
                                        },
                                    ),
                                    entries: &[
                                        wgpu::BindGroupEntry {
                                            binding: 0,
                                            resource: wgpu::BindingResource::TextureView(&hv),
                                        },
                                        wgpu::BindGroupEntry {
                                            binding: 1,
                                            resource: wgpu::BindingResource::Sampler(&samp),
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
                    if let Some(key) = anvilkit_input::prelude::KeyCode::from_winit(code) {
                        if let Some(mut input) = self.app.world.get_resource_mut::<InputState>() {
                            if event.state.is_pressed() {
                                input.press_key(key);
                            } else {
                                input.release_key(key);
                            }
                        }
                    }
                    if event.state.is_pressed() {
                        match code {
                            WK::Escape => {
                                el.exit();
                                return;
                            }
                            WK::Tab => {
                                if let Some(mut ps) =
                                    self.app.world.get_resource_mut::<PlayerState>()
                                {
                                    ps.flying = !ps.flying;
                                    println!(
                                        "Flying: {}",
                                        if ps.flying { "ON" } else { "OFF" }
                                    );
                                }
                            }
                            // Block selection: number keys 1-9
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
                return;
            }
            WindowEvent::CursorMoved { position, .. } => {
                if let Some(mut input) = self.app.world.get_resource_mut::<InputState>() {
                    input.set_mouse_position(glam::Vec2::new(position.x as f32, position.y as f32));
                }
                return;
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if let Some(mb) = anvilkit_input::prelude::MouseButton::from_winit(*button) {
                    if let Some(mut input) = self.app.world.get_resource_mut::<InputState>() {
                        if state.is_pressed() {
                            input.press_mouse(mb);
                        } else {
                            input.release_mouse(mb);
                        }
                    }
                }
                return;
            }
            WindowEvent::RedrawRequested if self.initialized => {
                self.render_frame();
                return;
            }
            _ => {}
        }
        self.render_app.window_event(el, wid, ev);
    }

    fn device_event(
        &mut self,
        el: &ActiveEventLoop,
        did: winit::event::DeviceId,
        ev: winit::event::DeviceEvent,
    ) {
        if let winit::event::DeviceEvent::MouseMotion { delta } = ev {
            if let Some(mut md) = self.app.world.get_resource_mut::<MouseDelta>() {
                md.dx += delta.0 as f32;
                md.dy += delta.1 as f32;
            }
        }
        self.render_app.device_event(el, did, ev);
    }

    fn about_to_wait(&mut self, _el: &ActiveEventLoop) {
        self.frame_count += 1;

        // Advance day/night cycle
        {
            let dt = self.app.world.resource::<DeltaTime>().0;
            let mut cycle = self.app.world.resource_mut::<DayNightCycle>();
            cycle.advance(dt);
        }

        self.app.update();

        // Periodic debug log (every 120 frames ~ 2 sec)
        if self.frame_count % 120 == 0 {
            let player = self.app.world.resource::<PlayerState>();
            let input = self.app.world.resource::<InputState>();
            let md = self.app.world.resource::<MouseDelta>();
            if let Some(cam) = self.app.world.get_resource::<ActiveCamera>() {
                let p = cam.camera_pos;
                println!(
                    "[F{}] pos=({:.1},{:.1},{:.1}) vel=({:.1},{:.1},{:.1}) fly={} gnd={} W={} dx={:.0} dy={:.0}",
                    self.frame_count,
                    p.x, p.y, p.z,
                    player.velocity.x, player.velocity.y, player.velocity.z,
                    player.flying, player.on_ground,
                    input.is_key_pressed(anvilkit_input::prelude::KeyCode::W),
                    md.dx, md.dy,
                );
            }
        }

        // Block interaction (raycast + place/break)
        self.handle_block_interaction();

        // Remesh any dirty chunks
        self.remesh_dirty_chunks();

        // Dynamic chunk loading
        self.update_chunks();

        // Clear mouse delta
        if let Some(mut md) = self.app.world.get_resource_mut::<MouseDelta>() {
            md.dx = 0.0;
            md.dy = 0.0;
        }
        if let Some(mut input) = self.app.world.get_resource_mut::<InputState>() {
            input.end_frame();
        }

        if let Some(w) = self.render_app.window() {
            w.request_redraw();
        }
    }
}

impl CraftApp {
    fn select_block(&mut self, index: usize) {
        if index < BLOCK_PALETTE.len() {
            if let Some(mut sb) = self.app.world.get_resource_mut::<SelectedBlock>() {
                sb.block_type = BLOCK_PALETTE[index];
                println!("Selected: {:?}", sb.block_type);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Frustum culling helpers
// ---------------------------------------------------------------------------

type Plane = glam::Vec4;

fn extract_frustum_planes(vp: &glam::Mat4) -> [Plane; 6] {
    let m = vp.to_cols_array_2d();
    // Rows of the transposed VP matrix
    let row = |r: usize| glam::Vec4::new(m[0][r], m[1][r], m[2][r], m[3][r]);
    let r0 = row(0);
    let r1 = row(1);
    let r2 = row(2);
    let r3 = row(3);

    [
        r3 + r0, // left
        r3 - r0, // right
        r3 + r1, // bottom
        r3 - r1, // top
        r3 + r2, // near
        r3 - r2, // far
    ]
}

fn sphere_in_frustum(planes: &[Plane; 6], center: glam::Vec3, radius: f32) -> bool {
    for p in planes {
        let dist = p.x * center.x + p.y * center.y + p.z * center.z + p.w;
        let len = (p.x * p.x + p.y * p.y + p.z * p.z).sqrt();
        if dist < -radius * len {
            return false;
        }
    }
    true
}
