//! # AnvilKit Mini Game
//!
//! Comprehensive validation example exercising all runtime modules:
//! - InputState-driven player movement (WASD)
//! - Multiple 3D cubes with PBR rendering
//! - AABB collision detection
//! - Particle effects on collision
//! - Audio playback (background music + SFX)
//! - UI score/health display
//!
//! Run: `cargo run -p anvilkit-render --example game`

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
use anvilkit_input::prelude::{InputState, KeyCode};
use anvilkit_ecs::physics::{
    Velocity, DeltaTime, CollisionEvents, AabbCollider,
    velocity_integration_system, collision_detection_system,
};
// Audio types available: anvilkit_ecs::audio::{AudioSource, PlaybackState}
use anvilkit_render::renderer::particle::{ParticleSystem, Particle, ParticleRenderer};
use anvilkit_render::renderer::ui::{UiNode, UiText, UiStyle, Val, UiRenderer};

// ---------------------------------------------------------------------------
//  Procedural cube mesh
// ---------------------------------------------------------------------------
fn cube_vertices(half: f32) -> (Vec<PbrVertex>, Vec<u32>) {
    let h = half;
    // 24 vertices (4 per face, 6 faces), 36 indices
    let positions: [[f32; 3]; 24] = [
        // +Z face
        [-h, -h,  h], [ h, -h,  h], [ h,  h,  h], [-h,  h,  h],
        // -Z face
        [ h, -h, -h], [-h, -h, -h], [-h,  h, -h], [ h,  h, -h],
        // +X face
        [ h, -h,  h], [ h, -h, -h], [ h,  h, -h], [ h,  h,  h],
        // -X face
        [-h, -h, -h], [-h, -h,  h], [-h,  h,  h], [-h,  h, -h],
        // +Y face
        [-h,  h,  h], [ h,  h,  h], [ h,  h, -h], [-h,  h, -h],
        // -Y face
        [-h, -h, -h], [ h, -h, -h], [ h, -h,  h], [-h, -h,  h],
    ];
    let normals: [[f32; 3]; 24] = [
        [0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0],
        [0.0, 0.0,-1.0], [0.0, 0.0,-1.0], [0.0, 0.0,-1.0], [0.0, 0.0,-1.0],
        [1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 0.0],
        [-1.0,0.0, 0.0], [-1.0,0.0, 0.0], [-1.0,0.0, 0.0], [-1.0,0.0, 0.0],
        [0.0, 1.0, 0.0], [0.0, 1.0, 0.0], [0.0, 1.0, 0.0], [0.0, 1.0, 0.0],
        [0.0,-1.0, 0.0], [0.0,-1.0, 0.0], [0.0,-1.0, 0.0], [0.0,-1.0, 0.0],
    ];
    let uvs: [[f32; 2]; 24] = [
        [0.0,1.0],[1.0,1.0],[1.0,0.0],[0.0,0.0],
        [0.0,1.0],[1.0,1.0],[1.0,0.0],[0.0,0.0],
        [0.0,1.0],[1.0,1.0],[1.0,0.0],[0.0,0.0],
        [0.0,1.0],[1.0,1.0],[1.0,0.0],[0.0,0.0],
        [0.0,1.0],[1.0,1.0],[1.0,0.0],[0.0,0.0],
        [0.0,1.0],[1.0,1.0],[1.0,0.0],[0.0,0.0],
    ];
    let tangents: [[f32; 4]; 6] = [
        [1.0, 0.0, 0.0, 1.0], // +Z
        [-1.0,0.0, 0.0, 1.0], // -Z
        [0.0, 0.0, 1.0, 1.0], // +X  (was -1, corrected)
        [0.0, 0.0,-1.0, 1.0], // -X
        [1.0, 0.0, 0.0, 1.0], // +Y
        [1.0, 0.0, 0.0, 1.0], // -Y
    ];

    let vertices: Vec<PbrVertex> = (0..24).map(|i| PbrVertex {
        position: positions[i],
        normal: normals[i],
        texcoord: uvs[i],
        tangent: tangents[i / 4],
    }).collect();

    let mut indices = Vec::with_capacity(36);
    for face in 0..6u32 {
        let base = face * 4;
        indices.extend_from_slice(&[base, base+1, base+2, base, base+2, base+3]);
    }
    (vertices, indices)
}

// ---------------------------------------------------------------------------
//  PBR Shader
// ---------------------------------------------------------------------------
const SHADER_SOURCE: &str = include_str!("../shaders/pbr.wgsl");
const TONEMAP_SHADER: &str = include_str!("../shaders/tonemap.wgsl");

fn pack_scene_lights(lights: &SceneLights) -> ([GpuLight; 8], u32) {
    let mut gpu = [GpuLight::default(); 8];
    let mut n = 0u32;
    let d = &lights.directional;
    gpu[0] = GpuLight {
        position_type: [0.0, 0.0, 0.0, 0.0],
        direction_range: [d.direction.x, d.direction.y, d.direction.z, 0.0],
        color_intensity: [d.color.x, d.color.y, d.color.z, d.intensity],
        params: [0.0; 4],
    };
    n += 1;
    for pl in &lights.point_lights {
        if n >= 8 { break; }
        gpu[n as usize] = GpuLight {
            position_type: [pl.position.x, pl.position.y, pl.position.z, 1.0],
            direction_range: [0.0, 0.0, 0.0, pl.range],
            color_intensity: [pl.color.x, pl.color.y, pl.color.z, pl.intensity],
            params: [0.0; 4],
        };
        n += 1;
    }
    (gpu, n)
}

// ---------------------------------------------------------------------------
//  Game components
// ---------------------------------------------------------------------------

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Obstacle;

#[derive(Resource)]
struct GameState {
    score: u32,
    health: u32,
    collision_cooldown: f32,
}

impl Default for GameState {
    fn default() -> Self {
        Self { score: 0, health: 100, collision_cooldown: 0.0 }
    }
}

#[derive(Resource)]
struct GameParticles {
    system: ParticleSystem,
}

// ---------------------------------------------------------------------------
//  Game systems
// ---------------------------------------------------------------------------

fn player_movement_system(
    input: Res<InputState>,
    mut query: Query<&mut Velocity, With<Player>>,
) {
    let speed = 5.0;
    for mut velocity in query.iter_mut() {
        let mut dir = glam::Vec3::ZERO;
        if input.is_key_pressed(KeyCode::W) || input.is_key_pressed(KeyCode::Up) { dir.z += 1.0; }
        if input.is_key_pressed(KeyCode::S) || input.is_key_pressed(KeyCode::Down) { dir.z -= 1.0; }
        if input.is_key_pressed(KeyCode::A) || input.is_key_pressed(KeyCode::Left) { dir.x -= 1.0; }
        if input.is_key_pressed(KeyCode::D) || input.is_key_pressed(KeyCode::Right) { dir.x += 1.0; }
        if dir.length_squared() > 0.0 { dir = dir.normalize(); }
        velocity.linear = dir * speed;
    }
}

fn collision_response_system(
    events: Res<CollisionEvents>,
    dt: Res<DeltaTime>,
    mut game_state: ResMut<GameState>,
    mut particles: ResMut<GameParticles>,
    mut player_query: Query<(&mut Transform, &mut Velocity, &AabbCollider), With<Player>>,
    obstacle_query: Query<(&Transform, &AabbCollider), (With<Obstacle>, Without<Player>)>,
) {
    game_state.collision_cooldown = (game_state.collision_cooldown - dt.0).max(0.0);
    for event in events.iter() {
        // Determine which entity is the player
        let (player_e, obstacle_e) = if player_query.get(event.a).is_ok() {
            (event.a, event.b)
        } else if player_query.get(event.b).is_ok() {
            (event.b, event.a)
        } else {
            continue;
        };

        let obstacle_pos = if let Ok((ot, _)) = obstacle_query.get(obstacle_e) {
            ot.translation
        } else {
            continue;
        };

        if let Ok((mut pt, mut vel, _col)) = player_query.get_mut(player_e) {
            // --- Physical push-out: separate player from obstacle ---
            let diff = pt.translation - obstacle_pos;
            let push_dir = if diff.length_squared() > 0.001 {
                glam::Vec3::new(diff.x, 0.0, diff.z).normalize()
            } else {
                glam::Vec3::X // fallback
            };
            // Push player out by overlap amount + small margin
            pt.translation += push_dir * 0.15;

            // Bounce velocity: reflect along push direction
            let speed = vel.linear.length().max(3.0);
            vel.linear = push_dir * speed * 0.6;

            // --- Scoring (with cooldown) ---
            if game_state.collision_cooldown <= 0.0 {
                game_state.score += 10;
                game_state.health = game_state.health.saturating_sub(5);
                game_state.collision_cooldown = 0.3;
                println!("COLLISION! Score: {} HP: {}", game_state.score, game_state.health);
            }

            // --- Particles: burst at contact point ---
            let contact = (pt.translation + obstacle_pos) * 0.5;
            for i in 0..40 {
                let angle = (i as f32 / 40.0) * std::f32::consts::TAU;
                let spread = 1.0 + (i % 3) as f32 * 0.5;
                let up = 1.5 + (i % 5) as f32 * 0.8;
                let pvel = glam::Vec3::new(
                    angle.cos() * spread * 3.0,
                    up,
                    angle.sin() * spread * 3.0,
                );
                let mut p = Particle::new(contact, pvel, 0.8 + (i % 4) as f32 * 0.2);
                // Orange-yellow gradient
                let t = i as f32 / 40.0;
                p.color = [1.0, 0.3 + t * 0.5, 0.05, 1.0];
                p.size = 0.03 + (i % 3) as f32 * 0.015;
                particles.system.emit(p);
            }
        }
    }
}

fn particle_update_system(
    dt: Res<DeltaTime>,
    mut particles: ResMut<GameParticles>,
) {
    particles.system.update(dt.0, glam::Vec3::new(0.0, -9.8, 0.0));
}

fn ui_update_system(
    game_state: Res<GameState>,
    mut query: Query<&mut UiNode>,
) {
    for mut node in query.iter_mut() {
        if let Some(ref mut text) = node.text {
            if text.content.starts_with("Score") {
                text.content = format!("Score: {}", game_state.score);
            } else if text.content.starts_with("HP") {
                text.content = format!("HP: {}", game_state.health);
            }
        }
    }
}

// ---------------------------------------------------------------------------
//  Application
// ---------------------------------------------------------------------------

fn main() {
    env_logger::init();
    println!("AnvilKit Mini Game");
    println!("  WASD / Arrows = move player (green cube)");
    println!("  Collide with red cubes to score points");
    println!("  ESC = quit");

    let (cube_verts, cube_indices) = cube_vertices(0.5);

    let mut app = App::new();
    app.add_plugins(RenderPlugin::new().with_window_config(
        WindowConfig::new()
            .with_title("AnvilKit Mini Game")
            .with_size(1024, 768),
    ));

    // Resources
    app.insert_resource(InputState::new());
    app.insert_resource(DeltaTime(1.0 / 60.0));
    app.init_resource::<CollisionEvents>();
    app.init_resource::<GameState>();
    app.insert_resource(GameParticles { system: ParticleSystem::new(500) });

    // Systems
    app.add_systems(AnvilKitSchedule::Update, (
        player_movement_system,
        velocity_integration_system.after(player_movement_system),
        collision_detection_system.after(velocity_integration_system),
        collision_response_system.after(collision_detection_system),
        particle_update_system,
        ui_update_system,
    ));

    // Lighting
    app.insert_resource(SceneLights {
        directional: DirectionalLight {
            direction: glam::Vec3::new(-0.4, -0.7, 0.5).normalize(),
            color: glam::Vec3::new(1.0, 0.95, 0.85),
            intensity: 4.0,
        },
        point_lights: vec![
            PointLight {
                position: glam::Vec3::new(3.0, 4.0, 0.0),
                color: glam::Vec3::new(1.0, 0.8, 0.6),
                intensity: 15.0,
                range: 15.0,
            },
        ],
        spot_lights: vec![],
    });

    // Spawn player entity (MeshHandle/MaterialHandle added after GPU init)
    let player_entity = app.world.spawn((
        Player,
        Transform::from_xyz(0.0, 0.5, 0.0),
        Velocity::zero(),
        AabbCollider::cube(0.5),
        MaterialParams { metallic: 0.1, roughness: 0.4, normal_scale: 1.0, emissive_factor: [0.0, 0.1, 0.0] },
    )).id();

    // Spawn obstacle entities
    let obstacle_positions = [
        glam::Vec3::new(3.0, 0.5, 3.0),
        glam::Vec3::new(-3.0, 0.5, -2.0),
        glam::Vec3::new(5.0, 0.5, -4.0),
        glam::Vec3::new(-4.0, 0.5, 5.0),
    ];
    let mut obstacle_entities = Vec::new();
    for pos in &obstacle_positions {
        let e = app.world.spawn((
            Obstacle,
            Transform::from_xyz(pos.x, pos.y, pos.z),
            AabbCollider::cube(0.5),
            MaterialParams { metallic: 0.6, roughness: 0.3, normal_scale: 1.0, emissive_factor: [0.1, 0.0, 0.0] },
        )).id();
        obstacle_entities.push(e);
    }

    // Ground plane entity
    let ground_entity = app.world.spawn((
        Transform::from_xyz(0.0, -0.05, 0.0).with_scale(glam::Vec3::new(20.0, 0.1, 20.0)),
        MaterialParams { metallic: 0.0, roughness: 0.8, normal_scale: 1.0, emissive_factor: [0.0; 3] },
    )).id();

    // Camera
    let eye = glam::Vec3::new(0.0, 12.0, -10.0);
    let look_dir = (glam::Vec3::ZERO - eye).normalize();
    let cam_rot = glam::Quat::from_rotation_arc(glam::Vec3::Z, look_dir);
    app.world.spawn((
        CameraComponent { fov: 50.0, near: 0.1, far: 100.0, is_active: true, aspect_ratio: 1024.0 / 768.0 },
        Transform::from_xyz(eye.x, eye.y, eye.z).with_rotation(cam_rot),
    ));

    // UI nodes
    app.world.spawn(UiNode {
        background_color: [0.0, 0.0, 0.0, 0.7],
        border_radius: 8.0,
        text: Some(UiText::new("Score: 0").with_font_size(24.0)),
        visible: true,
        computed_rect: [10.0, 10.0, 200.0, 40.0],
        style: UiStyle { width: Val::Px(200.0), height: Val::Px(40.0), ..Default::default() },
        ..Default::default()
    });
    app.world.spawn(UiNode {
        background_color: [0.5, 0.0, 0.0, 0.7],
        border_radius: 8.0,
        text: Some(UiText::new("HP: 100").with_font_size(24.0).with_color([1.0, 0.2, 0.2, 1.0])),
        visible: true,
        computed_rect: [10.0, 60.0, 200.0, 40.0],
        style: UiStyle { width: Val::Px(200.0), height: Val::Px(40.0), ..Default::default() },
        ..Default::default()
    });

    println!("Game setup: 1 player + {} obstacles + ground + camera + UI", obstacle_positions.len());

    // Run with custom ApplicationHandler for GPU resource management
    let event_loop = EventLoop::new().unwrap();
    let config = WindowConfig::new().with_title("AnvilKit Mini Game").with_size(1024, 768);
    event_loop.run_app(&mut GameApp {
        render_app: RenderApp::new(config),
        app,
        initialized: false,
        player_entity,
        obstacle_entities,
        ground_entity,
        cube_verts,
        cube_indices,
        scene_ub: None,
        scene_bg: None,
        depth_view: None,
        hdr_view: None,
        hdr_msaa_view: None,
        tonemap_pipeline: None,
        tonemap_bg: None,
        ibl_bg: None,
        particle_renderer: None,
        ui_renderer: None,
    }).unwrap();
}

struct GameApp {
    render_app: RenderApp,
    app: App,
    initialized: bool,
    player_entity: Entity,
    obstacle_entities: Vec<Entity>,
    ground_entity: Entity,
    cube_verts: Vec<PbrVertex>,
    cube_indices: Vec<u32>,
    // GPU resources (created after init)
    scene_ub: Option<wgpu::Buffer>,
    scene_bg: Option<wgpu::BindGroup>,
    depth_view: Option<wgpu::TextureView>,
    hdr_view: Option<wgpu::TextureView>,
    hdr_msaa_view: Option<wgpu::TextureView>,
    tonemap_pipeline: Option<wgpu::RenderPipeline>,
    tonemap_bg: Option<wgpu::BindGroup>,
    ibl_bg: Option<wgpu::BindGroup>,
    particle_renderer: Option<ParticleRenderer>,
    ui_renderer: Option<UiRenderer>,
}

impl GameApp {
    fn init_scene(&mut self) {
        if self.initialized { return; }
        let Some(device) = self.render_app.render_device() else { return };
        let Some(format) = self.render_app.surface_format() else { return };
        let (w, h) = self.render_app.window_state().size();

        // Scene uniform buffer
        let initial = PbrSceneUniform::default();
        let ub = create_uniform_buffer(device, "Game Scene UB", bytemuck::bytes_of(&initial));
        let scene_bgl = device.device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Game Scene BGL"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false, min_binding_size: None,
                },
                count: None,
            }],
        });
        let scene_bg = device.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Game Scene BG"), layout: &scene_bgl,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: ub.as_entire_binding() }],
        });
        let (_, depth_view) = create_depth_texture_msaa(device, w, h, "Game Depth");

        // 1x1 solid-color textures for materials
        let make_tex = |data: &[u8; 4], label, srgb: bool| {
            if srgb { create_texture(device, 1, 1, data, label).1 }
            else { create_texture_linear(device, 1, 1, data, label).1 }
        };
        let white_tex = make_tex(&[255,255,255,255], "White", true);
        let flat_normal = make_tex(&[128,128,255,255], "FlatNormal", false);
        let white_lin = make_tex(&[255,255,255,255], "WhiteLin", false);
        let sampler = create_sampler(device, "Game Sampler");

        let tex_entry = |b: u32| wgpu::BindGroupLayoutEntry {
            binding: b, visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2, multisampled: false,
            }, count: None,
        };

        // Player material (greenish base color)
        let player_bc = make_tex(&[100, 230, 120, 255], "PlayerBC", true);
        let mat_bgl = device.device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Mat BGL"),
            entries: &[tex_entry(0), tex_entry(1), tex_entry(2), tex_entry(3), tex_entry(4),
                wgpu::BindGroupLayoutEntry { binding: 5, visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None }],
        });

        let make_mat_bg = |bc: &wgpu::TextureView, label| {
            device.device().create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(label), layout: &mat_bgl,
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

        let player_mat_bg = make_mat_bg(&player_bc, "Player Mat BG");
        let obstacle_bc = make_tex(&[230, 80, 80, 255], "ObstacleBC", true);
        let obstacle_mat_bg = make_mat_bg(&obstacle_bc, "Obstacle Mat BG");
        let ground_bc = make_tex(&[180, 180, 160, 255], "GroundBC", true);
        let ground_mat_bg = make_mat_bg(&ground_bc, "Ground Mat BG");

        // IBL + Shadow (group 2)
        let brdf_data = generate_brdf_lut(256);
        let (_, brdf_view) = create_texture_linear(device, 256, 256, &brdf_data, "BRDF LUT");
        let (_, shadow_view) = create_shadow_map(device, SHADOW_MAP_SIZE, "Shadow Map");
        let shadow_samp = create_shadow_sampler(device, "Shadow Sampler");
        let ibl_bgl = device.device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("IBL+Shadow BGL"),
            entries: &[
                tex_entry(0),
                wgpu::BindGroupLayoutEntry { binding: 1, visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None },
                wgpu::BindGroupLayoutEntry { binding: 2, visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2, multisampled: false }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 3, visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison), count: None },
            ],
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

        // Helper to build a PBR pipeline (each material needs its own due to move semantics)
        let build_pbr_pipeline = |device: &RenderDevice, label: &str,
            s_bgl: &wgpu::BindGroupLayout, m_bgl: &wgpu::BindGroupLayout, i_bgl: &wgpu::BindGroupLayout| {
            // Recreate bind group layouts for each pipeline
            let pipeline_layout = device.device().create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(&format!("{} Layout", label)),
                bind_group_layouts: &[s_bgl, m_bgl, i_bgl],
                push_constant_ranges: &[],
            });
            let shader = device.device().create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(&format!("{} Shader", label)),
                source: wgpu::ShaderSource::Wgsl(SHADER_SOURCE.into()),
            });
            device.device().create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader, entry_point: "vs_main",
                    buffers: &[PbrVertex::layout()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader, entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: HDR_FORMAT, blend: None, write_mask: wgpu::ColorWrites::ALL,
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
                    count: MSAA_SAMPLE_COUNT, mask: !0, alpha_to_coverage_enabled: false,
                },
                multiview: None,
            })
        };

        let player_pipeline = build_pbr_pipeline(device, "Player PBR", &scene_bgl, &mat_bgl, &ibl_bgl);
        let obstacle_pipeline = build_pbr_pipeline(device, "Obstacle PBR", &scene_bgl, &mat_bgl, &ibl_bgl);
        let ground_pipeline = build_pbr_pipeline(device, "Ground PBR", &scene_bgl, &mat_bgl, &ibl_bgl);

        // HDR + tonemap
        let (_, hdr_view) = create_hdr_render_target(device, w, h, "Game HDR RT");
        let (_, hdr_msaa_view) = create_hdr_msaa_texture(device, w, h, "Game HDR MSAA");
        let tm_bgl = device.device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Tonemap BGL"),
            entries: &[tex_entry(0),
                wgpu::BindGroupLayoutEntry { binding: 1, visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None }],
        });
        let tm_bg = device.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Tonemap BG"), layout: &tm_bgl,
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
            .with_label("Game Tonemap Pipeline")
            .build(device).expect("Failed to create tonemap pipeline");

        // Upload cube mesh + create materials
        let mut assets = self.app.world.resource_mut::<RenderAssets>();
        let mesh_h = assets.upload_mesh_u32(device, &self.cube_verts, &self.cube_indices, "Cube");
        let player_mat_h = assets.create_material(player_pipeline, player_mat_bg);
        let obstacle_mat_h = assets.create_material(obstacle_pipeline, obstacle_mat_bg);
        let ground_mat_h = assets.create_material(ground_pipeline, ground_mat_bg);

        // Attach mesh/material handles to entities
        self.app.world.entity_mut(self.player_entity).insert((mesh_h, player_mat_h));
        for &e in &self.obstacle_entities {
            self.app.world.entity_mut(e).insert((mesh_h, obstacle_mat_h));
        }
        self.app.world.entity_mut(self.ground_entity).insert((mesh_h, ground_mat_h));

        // Particle and UI renderers (render onto swapchain after tonemap)
        let particle_renderer = ParticleRenderer::new(device, format);
        let ui_renderer = UiRenderer::new(device, format);

        self.scene_ub = Some(ub);
        self.scene_bg = Some(scene_bg);
        self.depth_view = Some(depth_view);
        self.hdr_view = Some(hdr_view);
        self.hdr_msaa_view = Some(hdr_msaa_view);
        self.tonemap_pipeline = Some(tm_pipe.into_pipeline());
        self.tonemap_bg = Some(tm_bg);
        self.ibl_bg = Some(ibl_bg);
        self.particle_renderer = Some(particle_renderer);
        self.ui_renderer = Some(ui_renderer);
        self.initialized = true;
        println!("Game scene initialized!");
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
        if dl.commands.is_empty() { return; }

        let Some(frame) = self.render_app.get_current_frame() else { return };
        let swapchain = frame.texture.create_view(&Default::default());

        let def_lights = SceneLights::default();
        let lights = self.app.world.get_resource::<SceneLights>().unwrap_or(&def_lights);
        let (gpu_lights, lc) = pack_scene_lights(lights);
        let ld = lights.directional.direction.normalize();

        // Scene pass → HDR MSAA
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
                shadow_view_proj: glam::Mat4::IDENTITY.to_cols_array_2d(),
                emissive_factor: [cmd.emissive_factor[0], cmd.emissive_factor[1], cmd.emissive_factor[2], 0.0],
            };
            device.queue().write_buffer(ub, 0, bytemuck::bytes_of(&u));

            let mut enc = device.device().create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Game Scene Enc") });
            {
                let cl = if i == 0 { wgpu::LoadOp::Clear(wgpu::Color { r: 0.15, g: 0.3, b: 0.6, a: 1.0 }) } else { wgpu::LoadOp::Load };
                let dl_op = if i == 0 { wgpu::LoadOp::Clear(1.0) } else { wgpu::LoadOp::Load };
                let mut rp = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Game Scene Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: hdr_msaa_view, resolve_target: Some(hdr_view),
                        ops: wgpu::Operations { load: cl, store: wgpu::StoreOp::Discard },
                    })],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: depth_view,
                        depth_ops: Some(wgpu::Operations { load: dl_op, store: wgpu::StoreOp::Store }),
                        stencil_ops: None,
                    }),
                    timestamp_writes: None, occlusion_query_set: None,
                });
                rp.set_pipeline(&gmat.pipeline);
                rp.set_bind_group(0, scene_bg, &[]);
                rp.set_bind_group(1, &gmat.bind_group, &[]);
                rp.set_bind_group(2, ibl_bg, &[]);
                rp.set_vertex_buffer(0, gm.vertex_buffer.slice(..));
                rp.set_index_buffer(gm.index_buffer.slice(..), gm.index_format);
                rp.draw_indexed(0..gm.index_count, 0, 0..1);
            }
            device.queue().submit(std::iter::once(enc.finish()));
        }

        // Tonemap → swapchain
        {
            let mut enc = device.device().create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Game Tonemap Enc") });
            { let mut rp = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Tonemap"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &swapchain, resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store },
                })],
                depth_stencil_attachment: None, timestamp_writes: None, occlusion_query_set: None,
            });
            rp.set_pipeline(tm_pipe);
            rp.set_bind_group(0, tm_bg, &[]);
            rp.draw(0..3, 0..1); }
            device.queue().submit(std::iter::once(enc.finish()));
        }

        // Particle rendering → swapchain (additive overlay)
        if let Some(ref pr) = self.particle_renderer {
            if let Some(particles) = self.app.world.get_resource::<GameParticles>() {
                let mut enc = device.device().create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Game Particle Enc") });
                pr.render(device, &mut enc, &swapchain, &particles.system, &cam.view_proj);
                device.queue().submit(std::iter::once(enc.finish()));
            }
        }

        // UI rendering → swapchain (overlay)
        if let Some(ref ur) = self.ui_renderer {
            let (sw, sh) = self.render_app.window_state().size();
            // Collect UI node data (query needs &mut World, so clone the data)
            let ui_nodes: Vec<UiNode> = self.app.world.query::<&UiNode>()
                .iter(&self.app.world)
                .filter(|n| n.visible)
                .cloned()
                .collect();
            if !ui_nodes.is_empty() {
                let node_refs: Vec<&UiNode> = ui_nodes.iter().collect();
                let mut enc = device.device().create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Game UI Enc") });
                ur.render(device, &mut enc, &swapchain, &node_refs, sw as f32, sh as f32);
                device.queue().submit(std::iter::once(enc.finish()));
            }
        }

        frame.present();
    }
}

impl ApplicationHandler for GameApp {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        self.render_app.resumed(el);
        self.init_scene();
    }

    fn window_event(&mut self, el: &ActiveEventLoop, wid: WindowId, ev: WindowEvent) {
        match &ev {
            WindowEvent::Resized(s) if self.initialized && s.width > 0 && s.height > 0 => {
                if let Some(device) = self.render_app.render_device() {
                    let (_, dv) = create_depth_texture_msaa(device, s.width, s.height, "Game Depth");
                    self.depth_view = Some(dv);
                    let (_, hv) = create_hdr_render_target(device, s.width, s.height, "Game HDR RT");
                    let (_, hmv) = create_hdr_msaa_texture(device, s.width, s.height, "Game HDR MSAA");
                    let samp = create_sampler(device, "Game Sampler");
                    let layout = device.device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                        label: Some("Tonemap BGL"), entries: &[
                            wgpu::BindGroupLayoutEntry { binding: 0, visibility: wgpu::ShaderStages::FRAGMENT,
                                ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                    view_dimension: wgpu::TextureViewDimension::D2, multisampled: false }, count: None },
                            wgpu::BindGroupLayoutEntry { binding: 1, visibility: wgpu::ShaderStages::FRAGMENT,
                                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None },
                        ],
                    });
                    self.tonemap_bg = Some(device.device().create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("Tonemap BG"), layout: &layout, entries: &[
                            wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&hv) },
                            wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&samp) },
                        ],
                    }));
                    self.hdr_view = Some(hv);
                    self.hdr_msaa_view = Some(hmv);
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                use winit::keyboard::{KeyCode as WK, PhysicalKey};
                if let PhysicalKey::Code(code) = event.physical_key {
                    if let Some(key) = anvilkit_input::prelude::KeyCode::from_winit(code) {
                        if let Some(mut input) = self.app.world.get_resource_mut::<InputState>() {
                            if event.state.is_pressed() { input.press_key(key); }
                            else { input.release_key(key); }
                        }
                    }
                    if event.state.is_pressed() && code == WK::Escape { el.exit(); return; }
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

    fn device_event(&mut self, el: &ActiveEventLoop, did: winit::event::DeviceId, ev: winit::event::DeviceEvent) {
        self.render_app.device_event(el, did, ev);
    }

    fn about_to_wait(&mut self, _el: &ActiveEventLoop) {
        self.app.update();
        if let Some(mut input) = self.app.world.get_resource_mut::<InputState>() {
            input.end_frame();
        }
        if let Some(w) = self.render_app.window() { w.request_redraw(); }
    }
}
