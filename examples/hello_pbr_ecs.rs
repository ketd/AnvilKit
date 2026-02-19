//! # ECS PBR + 法线贴图 + HDR + IBL 环境光
//!
//! AnvilKit M6d 示例：完整 PBR 管线 + IBL 环境光。
//! BRDF LUT + hemisphere irradiance + split-sum specular。
//!
//! 运行: `cargo run -p anvilkit-render --example hello_pbr_ecs`

use anvilkit_render::prelude::*;
use anvilkit_render::renderer::{
    RenderPipelineBuilder, DEPTH_FORMAT, HDR_FORMAT,
    buffer::{PbrVertex, Vertex, create_uniform_buffer, create_depth_texture,
             create_hdr_render_target, create_texture, create_texture_linear, create_sampler},
    assets::RenderAssets,
    draw::{ActiveCamera, DrawCommandList, SceneLights, DirectionalLight, MaterialParams},
    ibl::generate_brdf_lut,
};
use anvilkit_render::plugin::CameraComponent;
use anvilkit_assets::gltf_loader::load_gltf_scene;

/// Cook-Torrance PBR + TBN Normal Mapping + IBL Ambient
const SHADER_SOURCE: &str = r#"
const PI: f32 = 3.14159265359;

struct SceneUniform {
    model: mat4x4<f32>,
    view_proj: mat4x4<f32>,
    normal_matrix: mat4x4<f32>,
    camera_pos: vec4<f32>,
    light_dir: vec4<f32>,
    light_color: vec4<f32>,
    material_params: vec4<f32>,  // (metallic, roughness, normal_scale, 0)
};

@group(0) @binding(0)
var<uniform> scene: SceneUniform;

@group(1) @binding(0)
var base_color_texture: texture_2d<f32>;
@group(1) @binding(1)
var base_color_sampler: sampler;
@group(1) @binding(2)
var normal_map_texture: texture_2d<f32>;
@group(1) @binding(3)
var normal_map_sampler: sampler;

// IBL resources (group 2)
@group(2) @binding(0)
var brdf_lut: texture_2d<f32>;
@group(2) @binding(1)
var brdf_lut_sampler: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) texcoord: vec2<f32>,
    @location(3) tangent: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) texcoord: vec2<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) world_position: vec3<f32>,
    @location(3) world_tangent: vec3<f32>,
    @location(4) world_bitangent: vec3<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let world_pos = scene.model * vec4<f32>(in.position, 1.0);
    out.clip_position = scene.view_proj * world_pos;
    out.world_position = world_pos.xyz;

    let N = normalize((scene.normal_matrix * vec4<f32>(in.normal, 0.0)).xyz);
    let T = normalize((scene.model * vec4<f32>(in.tangent.xyz, 0.0)).xyz);
    let B = cross(N, T) * in.tangent.w;

    out.world_normal = N;
    out.world_tangent = T;
    out.world_bitangent = B;
    out.texcoord = in.texcoord;
    return out;
}

// GGX/Trowbridge-Reitz Normal Distribution Function
fn distribution_ggx(N: vec3<f32>, H: vec3<f32>, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let NdotH = max(dot(N, H), 0.0);
    let NdotH2 = NdotH * NdotH;
    let denom = NdotH2 * (a2 - 1.0) + 1.0;
    return a2 / (PI * denom * denom);
}

// Schlick Fresnel approximation
fn fresnel_schlick(cos_theta: f32, F0: vec3<f32>) -> vec3<f32> {
    return F0 + (1.0 - F0) * pow(clamp(1.0 - cos_theta, 0.0, 1.0), 5.0);
}

// Smith GGX Geometry (single direction)
fn geometry_schlick_ggx(NdotV: f32, roughness: f32) -> f32 {
    let r = roughness + 1.0;
    let k = (r * r) / 8.0;
    return NdotV / (NdotV * (1.0 - k) + k);
}

// Smith GGX Geometry (combined)
fn geometry_smith(N: vec3<f32>, V: vec3<f32>, L: vec3<f32>, roughness: f32) -> f32 {
    let NdotV = max(dot(N, V), 0.0);
    let NdotL = max(dot(N, L), 0.0);
    return geometry_schlick_ggx(NdotV, roughness) * geometry_schlick_ggx(NdotL, roughness);
}

// Hemisphere irradiance — simple sky/ground ambient model
fn hemisphere_irradiance(N: vec3<f32>) -> vec3<f32> {
    let sky_color = vec3<f32>(0.30, 0.50, 0.90);    // blue sky
    let ground_color = vec3<f32>(0.10, 0.08, 0.05);  // dark ground
    let t = N.y * 0.5 + 0.5;  // remap [-1,1] → [0,1]
    return mix(ground_color, sky_color, t);
}

// Approximate prefiltered specular from hemisphere
fn hemisphere_specular(R: vec3<f32>, roughness: f32) -> vec3<f32> {
    let sky_color = vec3<f32>(0.50, 0.70, 1.00);
    let ground_color = vec3<f32>(0.10, 0.08, 0.05);
    let t = R.y * 0.5 + 0.5;
    // Rougher surfaces see blurred environment → blend toward average
    let avg = (sky_color + ground_color) * 0.5;
    let sharp = mix(ground_color, sky_color, t);
    let blur = 1.0 - roughness * roughness;
    return mix(avg, sharp, blur);
}

// Fresnel with roughness for IBL (Sébastien Lagarde)
fn fresnel_schlick_roughness(cos_theta: f32, F0: vec3<f32>, roughness: f32) -> vec3<f32> {
    let smooth_val = max(vec3<f32>(1.0 - roughness), F0);
    return F0 + (smooth_val - F0) * pow(clamp(1.0 - cos_theta, 0.0, 1.0), 5.0);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let albedo = textureSample(base_color_texture, base_color_sampler, in.texcoord).rgb;
    let metallic = scene.material_params.x;
    let roughness = scene.material_params.y;
    let normal_scale = scene.material_params.z;

    // Sample normal map and transform from tangent space to world space
    let normal_map = textureSample(normal_map_texture, normal_map_sampler, in.texcoord).rgb;
    var tangent_normal = normal_map * 2.0 - vec3<f32>(1.0);
    tangent_normal.x = tangent_normal.x * normal_scale;
    tangent_normal.y = tangent_normal.y * normal_scale;
    tangent_normal = normalize(tangent_normal);

    let T = normalize(in.world_tangent);
    let B = normalize(in.world_bitangent);
    let Ng = normalize(in.world_normal);
    let N = normalize(T * tangent_normal.x + B * tangent_normal.y + Ng * tangent_normal.z);

    let V = normalize(scene.camera_pos.xyz - in.world_position);
    let L = normalize(-scene.light_dir.xyz);
    let H = normalize(V + L);
    let NdotV = max(dot(N, V), 0.0);

    // F0: non-metal 0.04, metal uses albedo
    let F0 = mix(vec3<f32>(0.04), albedo, metallic);

    // === Direct lighting (Cook-Torrance) ===
    let D = distribution_ggx(N, H, roughness);
    let G = geometry_smith(N, V, L, roughness);
    let F = fresnel_schlick(max(dot(H, V), 0.0), F0);

    let numerator = D * G * F;
    let denominator = 4.0 * NdotV * max(dot(N, L), 0.0) + 0.0001;
    let specular = numerator / denominator;

    let kD_direct = (vec3<f32>(1.0) - F) * (1.0 - metallic);
    let NdotL = max(dot(N, L), 0.0);
    let light_radiance = scene.light_color.xyz * scene.light_color.w;
    let Lo = (kD_direct * albedo / PI + specular) * light_radiance * NdotL;

    // === Indirect lighting (IBL) ===
    let F_ibl = fresnel_schlick_roughness(NdotV, F0, roughness);
    let kD_ibl = (vec3<f32>(1.0) - F_ibl) * (1.0 - metallic);

    // Diffuse IBL: hemisphere irradiance
    let irradiance = hemisphere_irradiance(N);
    let diffuse_ibl = irradiance * albedo * kD_ibl;

    // Specular IBL: prefiltered color * (F0 * scale + bias)
    let R = reflect(-V, N);
    let prefiltered_color = hemisphere_specular(R, roughness);
    let brdf = textureSample(brdf_lut, brdf_lut_sampler, vec2<f32>(NdotV, roughness)).rg;
    let specular_ibl = prefiltered_color * (F0 * brdf.x + brdf.y);

    let ambient = diffuse_ibl + specular_ibl;

    let color = ambient + Lo;

    // Output linear HDR — tone mapping in post-process pass
    return vec4<f32>(color, 1.0);
}
"#;

/// 后处理 WGSL 着色器：全屏三角形 + ACES Filmic Tone Mapping
const TONEMAP_SHADER: &str = r#"
@group(0) @binding(0)
var hdr_texture: texture_2d<f32>;
@group(0) @binding(1)
var hdr_sampler: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) texcoord: vec2<f32>,
};

// Fullscreen triangle — no vertex buffer needed
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    // Generate fullscreen triangle covering [-1,1] NDC
    let x = f32(i32(vertex_index & 1u) * 4 - 1);
    let y = f32(i32(vertex_index & 2u) * 2 - 1);
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    // Map from NDC [-1,1] to UV [0,1], flip Y for texture coordinates
    out.texcoord = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);
    return out;
}

// ACES Filmic Tone Mapping (Narkowicz 2015 approximation)
fn aces_filmic(x: vec3<f32>) -> vec3<f32> {
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    return clamp((x * (a * x + b)) / (x * (c * x + d) + e), vec3<f32>(0.0), vec3<f32>(1.0));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(hdr_texture, hdr_sampler, in.texcoord).rgb;

    // ACES Filmic tone mapping
    color = aces_filmic(color);

    // Gamma correction (linear → sRGB)
    color = pow(color, vec3<f32>(1.0 / 2.2));

    return vec4<f32>(color, 1.0);
}
"#;

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

    // Configure scene lights
    app.insert_resource(SceneLights {
        directional: DirectionalLight {
            direction: glam::Vec3::new(-0.5, -0.8, 0.3).normalize(),
            color: glam::Vec3::new(1.0, 0.95, 0.9),
            intensity: 5.0,
        },
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
    // HDR multi-pass
    hdr_texture_view: Option<wgpu::TextureView>,
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
        let (_, depth_view) = create_depth_texture(device, w, h, "Depth");

        // Material bind group (group 1: base_color + normal_map, 4 bindings)
        let base_color_view = if let Some(ref tex) = self.material.base_color_texture {
            let (_, v) = create_texture(device, tex.width, tex.height, &tex.data, "BaseColor");
            v
        } else {
            let (_, v) = create_texture(device, 1, 1, &[255, 255, 255, 255], "Fallback BaseColor");
            v
        };

        // Normal map: use linear texture format (Rgba8Unorm, not sRGB)
        let normal_map_view = if let Some(ref tex) = self.material.normal_texture {
            let (_, v) = create_texture_linear(device, tex.width, tex.height, &tex.data, "NormalMap");
            v
        } else {
            // Flat normal fallback: RGB=(0.5, 0.5, 1.0) = (128, 128, 255) in [0,255]
            let (_, v) = create_texture_linear(device, 1, 1, &[128, 128, 255, 255], "Flat Normal");
            v
        };

        let sampler = create_sampler(device, "Sampler");

        let mat_bgl = device.device().create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("Material BGL"),
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
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        }, count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3, visibility: wgpu::ShaderStages::FRAGMENT,
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
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sampler) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&normal_map_view) },
                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::Sampler(&sampler) },
            ],
        });

        // IBL bind group (group 2: BRDF LUT texture + sampler)
        let brdf_lut_data = generate_brdf_lut(256);
        let (_, brdf_lut_view) = create_texture_linear(device, 256, 256, &brdf_lut_data, "BRDF LUT");

        let ibl_bgl = device.device().create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("IBL BGL"),
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
        let ibl_bg = device.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("IBL BG"),
            layout: &ibl_bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&brdf_lut_view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sampler) },
            ],
        });

        // PBR Pipeline — renders to HDR_FORMAT, 3 bind groups (scene, material, IBL)
        let pipeline = RenderPipelineBuilder::new()
            .with_vertex_shader(SHADER_SOURCE)
            .with_fragment_shader(SHADER_SOURCE)
            .with_format(HDR_FORMAT)
            .with_vertex_layouts(vec![PbrVertex::layout()])
            .with_depth_format(DEPTH_FORMAT)
            .with_bind_group_layouts(vec![scene_bgl, mat_bgl, ibl_bgl])
            .with_label("PBR HDR IBL Pipeline")
            .build(device)
            .expect("创建 PBR 管线失败");

        // HDR render target
        let (_, hdr_view) = create_hdr_render_target(device, w, h, "HDR RT");

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
                material_params: [cmd.metallic, cmd.roughness, cmd.normal_scale, 0.0],
            };
            device.queue().write_buffer(ub, 0, bytemuck::bytes_of(&uniform));

            let mut encoder = device.device().create_command_encoder(
                &wgpu::CommandEncoderDescriptor { label: Some("HDR Scene Encoder") },
            );

            {
                let color_load = if i == 0 {
                    wgpu::LoadOp::Clear(wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 })
                } else {
                    wgpu::LoadOp::Load
                };
                let depth_load = if i == 0 {
                    wgpu::LoadOp::Clear(1.0)
                } else {
                    wgpu::LoadOp::Load
                };

                let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("HDR Scene Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: hdr_view,
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
                    let (_, depth_view) = create_depth_texture(device, new_size.width, new_size.height, "Depth");
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
