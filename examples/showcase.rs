//! # AnvilKit PBR Showcase
//!
//! Demonstrates the complete rendering pipeline:
//! - glTF model loading (DamagedHelmet with full PBR textures)
//! - Cook-Torrance PBR with metallic-roughness workflow
//! - Normal mapping (TBN matrix)
//! - Metallic-roughness / AO / Emissive texture maps
//! - Multi-light system (directional + point lights)
//! - Shadow mapping with PCF 3x3
//! - IBL ambient lighting (BRDF LUT + hemisphere irradiance)
//! - HDR rendering with ACES Filmic tone mapping
//! - MSAA 4x antialiasing
//! - Orbit camera animation
//!
//! Run: `cargo run -p anvilkit-render --example showcase`

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

// ---------------------------------------------------------------------------
//  PBR Shader (same as hello_pbr_ecs — full pipeline)
// ---------------------------------------------------------------------------
const SHADER_SOURCE: &str = r#"
const PI: f32 = 3.14159265359;

struct GpuLight {
    position_type: vec4<f32>,
    direction_range: vec4<f32>,
    color_intensity: vec4<f32>,
    params: vec4<f32>,
};

struct SceneUniform {
    model: mat4x4<f32>,
    view_proj: mat4x4<f32>,
    normal_matrix: mat4x4<f32>,
    camera_pos: vec4<f32>,
    light_dir: vec4<f32>,
    light_color: vec4<f32>,
    material_params: vec4<f32>,
    lights: array<GpuLight, 8>,
    shadow_view_proj: mat4x4<f32>,
    emissive_factor: vec4<f32>,
};

@group(0) @binding(0) var<uniform> scene: SceneUniform;
@group(1) @binding(0) var base_color_texture: texture_2d<f32>;
@group(1) @binding(1) var normal_map_texture: texture_2d<f32>;
@group(1) @binding(2) var metallic_roughness_texture: texture_2d<f32>;
@group(1) @binding(3) var ao_texture: texture_2d<f32>;
@group(1) @binding(4) var emissive_texture: texture_2d<f32>;
@group(1) @binding(5) var material_sampler: sampler;
@group(2) @binding(0) var brdf_lut: texture_2d<f32>;
@group(2) @binding(1) var brdf_lut_sampler: sampler;
@group(2) @binding(2) var shadow_map: texture_depth_2d;
@group(2) @binding(3) var shadow_sampler: sampler_comparison;

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

fn distribution_ggx(N: vec3<f32>, H: vec3<f32>, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let NdotH = max(dot(N, H), 0.0);
    let denom = NdotH * NdotH * (a2 - 1.0) + 1.0;
    return a2 / (PI * denom * denom);
}

fn fresnel_schlick(cos_theta: f32, F0: vec3<f32>) -> vec3<f32> {
    return F0 + (1.0 - F0) * pow(clamp(1.0 - cos_theta, 0.0, 1.0), 5.0);
}

fn geometry_schlick_ggx(NdotV: f32, roughness: f32) -> f32 {
    let r = roughness + 1.0;
    let k = (r * r) / 8.0;
    return NdotV / (NdotV * (1.0 - k) + k);
}

fn geometry_smith(N: vec3<f32>, V: vec3<f32>, L: vec3<f32>, roughness: f32) -> f32 {
    return geometry_schlick_ggx(max(dot(N, V), 0.0), roughness) *
           geometry_schlick_ggx(max(dot(N, L), 0.0), roughness);
}

fn hemisphere_irradiance(N: vec3<f32>) -> vec3<f32> {
    let sky = vec3<f32>(0.30, 0.50, 0.90);
    let ground = vec3<f32>(0.10, 0.08, 0.05);
    return mix(ground, sky, N.y * 0.5 + 0.5);
}

fn hemisphere_specular(R: vec3<f32>, roughness: f32) -> vec3<f32> {
    let sky = vec3<f32>(0.50, 0.70, 1.00);
    let ground = vec3<f32>(0.10, 0.08, 0.05);
    let avg = (sky + ground) * 0.5;
    let sharp = mix(ground, sky, R.y * 0.5 + 0.5);
    return mix(avg, sharp, 1.0 - roughness * roughness);
}

fn calculate_shadow(world_pos: vec3<f32>, shadow_vp: mat4x4<f32>) -> f32 {
    let clip = shadow_vp * vec4<f32>(world_pos, 1.0);
    let ndc = clip.xyz / clip.w;
    let uv = vec2<f32>(ndc.x * 0.5 + 0.5, -ndc.y * 0.5 + 0.5);
    let depth = ndc.z;
    if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0 || depth > 1.0) { return 1.0; }
    let ts = 1.0 / 2048.0;
    var s = 0.0;
    for (var x = -1; x <= 1; x++) { for (var y = -1; y <= 1; y++) {
        s += textureSampleCompare(shadow_map, shadow_sampler, uv + vec2<f32>(f32(x), f32(y)) * ts, depth - 0.005);
    }}
    return s / 9.0;
}

fn fresnel_schlick_roughness(cos_theta: f32, F0: vec3<f32>, roughness: f32) -> vec3<f32> {
    return F0 + (max(vec3<f32>(1.0 - roughness), F0) - F0) * pow(clamp(1.0 - cos_theta, 0.0, 1.0), 5.0);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let albedo = textureSample(base_color_texture, material_sampler, in.texcoord).rgb;
    let normal_scale = scene.material_params.z;
    let mr = textureSample(metallic_roughness_texture, material_sampler, in.texcoord);
    let metallic = mr.b * scene.material_params.x;
    let roughness = mr.g * scene.material_params.y;
    let ao = textureSample(ao_texture, material_sampler, in.texcoord).r;

    let nm = textureSample(normal_map_texture, material_sampler, in.texcoord).rgb;
    var tn = nm * 2.0 - vec3<f32>(1.0);
    tn.x *= normal_scale; tn.y *= normal_scale;
    tn = normalize(tn);
    let T = normalize(in.world_tangent);
    let B = normalize(in.world_bitangent);
    let Ng = normalize(in.world_normal);
    let N = normalize(T * tn.x + B * tn.y + Ng * tn.z);

    let V = normalize(scene.camera_pos.xyz - in.world_position);
    let NdotV = max(dot(N, V), 0.0);
    let F0 = mix(vec3<f32>(0.04), albedo, metallic);

    let shadow = calculate_shadow(in.world_position, scene.shadow_view_proj);
    let light_count = u32(scene.material_params.w);
    var Lo = vec3<f32>(0.0);

    for (var li = 0u; li < light_count; li++) {
        let light = scene.lights[li];
        let lt = u32(light.position_type.w);
        var L: vec3<f32>; var atten: f32 = 1.0;
        if (lt == 0u) { L = normalize(-light.direction_range.xyz); }
        else {
            let d = light.position_type.xyz - in.world_position;
            let dist = length(d); L = d / max(dist, 0.0001);
            let r = clamp(dist / light.direction_range.w, 0.0, 1.0);
            atten = max(1.0 - r * r, 0.0); atten *= atten;
            if (lt == 2u) {
                let ca = dot(normalize(light.direction_range.xyz), -L);
                atten *= clamp((ca - light.params.y) / max(light.params.x - light.params.y, 0.0001), 0.0, 1.0);
            }
        }
        let H = normalize(V + L);
        let rad = light.color_intensity.xyz * light.color_intensity.w * atten;
        let D = distribution_ggx(N, H, roughness);
        let G = geometry_smith(N, V, L, roughness);
        let F = fresnel_schlick(max(dot(H, V), 0.0), F0);
        let spec = D * G * F / (4.0 * NdotV * max(dot(N, L), 0.0) + 0.0001);
        let kD = (vec3<f32>(1.0) - F) * (1.0 - metallic);
        var ls = 1.0;
        if (li == 0u && lt == 0u) { ls = shadow; }
        Lo += (kD * albedo / PI + spec) * rad * max(dot(N, L), 0.0) * ls;
    }

    let Fi = fresnel_schlick_roughness(NdotV, F0, roughness);
    let kDi = (vec3<f32>(1.0) - Fi) * (1.0 - metallic);
    let diff_ibl = hemisphere_irradiance(N) * albedo * kDi;
    let R = reflect(-V, N);
    let brdf = textureSample(brdf_lut, brdf_lut_sampler, vec2<f32>(NdotV, roughness)).rg;
    let spec_ibl = hemisphere_specular(R, roughness) * (F0 * brdf.x + brdf.y);
    let ambient = (diff_ibl + spec_ibl) * ao;

    let emissive_tex = textureSample(emissive_texture, material_sampler, in.texcoord).rgb;
    let emissive = emissive_tex * scene.emissive_factor.xyz;

    return vec4<f32>(ambient + Lo + emissive, 1.0);
}
"#;

const TONEMAP_SHADER: &str = r#"
@group(0) @binding(0) var hdr_texture: texture_2d<f32>;
@group(0) @binding(1) var hdr_sampler: sampler;

struct VertexOutput { @builtin(position) position: vec4<f32>, @location(0) texcoord: vec2<f32> };

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(i32(vi & 1u) * 4 - 1);
    let y = f32(i32(vi & 2u) * 2 - 1);
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.texcoord = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);
    return out;
}

fn aces_filmic(x: vec3<f32>) -> vec3<f32> {
    return clamp((x * (2.51 * x + 0.03)) / (x * (2.43 * x + 0.59) + 0.14), vec3<f32>(0.0), vec3<f32>(1.0));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var c = textureSample(hdr_texture, hdr_sampler, in.texcoord).rgb;
    c = aces_filmic(c);
    c = pow(c, vec3<f32>(1.0 / 2.2));
    return vec4<f32>(c, 1.0);
}
"#;

// ---------------------------------------------------------------------------
//  Light packing helper
// ---------------------------------------------------------------------------
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
//  Application
// ---------------------------------------------------------------------------

#[derive(Resource)]
struct FrameTime(std::time::Instant);

fn main() {
    env_logger::init();

    let scene = load_gltf_scene("assets/damaged_helmet.glb")
        .expect("Failed to load DamagedHelmet.glb");

    println!("Showcase: {} vertices, {} indices",
        scene.mesh.vertex_count(), scene.mesh.index_count());
    println!("  base_color: {}, normal: {}, MR: {}, AO: {}, emissive: {}",
        scene.material.base_color_texture.is_some(),
        scene.material.normal_texture.is_some(),
        scene.material.metallic_roughness_texture.is_some(),
        scene.material.occlusion_texture.is_some(),
        scene.material.emissive_texture.is_some());

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
            .with_title("AnvilKit Showcase - DamagedHelmet PBR")
            .with_size(1024, 768),
    ));
    app.insert_resource(FrameTime(std::time::Instant::now()));

    // Warm sunlight + cool fill + rim accent
    app.insert_resource(SceneLights {
        directional: DirectionalLight {
            direction: glam::Vec3::new(-0.4, -0.7, 0.5).normalize(),
            color: glam::Vec3::new(1.0, 0.95, 0.85),
            intensity: 4.0,
        },
        point_lights: vec![
            PointLight {
                position: glam::Vec3::new(3.0, 2.0, -2.0),
                color: glam::Vec3::new(1.0, 0.6, 0.3),
                intensity: 12.0,
                range: 10.0,
            },
            PointLight {
                position: glam::Vec3::new(-3.0, 1.0, -1.0),
                color: glam::Vec3::new(0.3, 0.5, 1.0),
                intensity: 8.0,
                range: 10.0,
            },
        ],
        spot_lights: vec![],
    });

    let event_loop = EventLoop::new().unwrap();
    let config = WindowConfig::new()
        .with_title("AnvilKit Showcase - DamagedHelmet PBR")
        .with_size(1024, 768);

    event_loop.run_app(&mut ShowcaseApp {
        render_app: RenderApp::new(config),
        app,
        initialized: false,
        scene_ub: None,
        scene_bg: None,
        depth_view: None,
        hdr_view: None,
        hdr_msaa_view: None,
        tonemap_pipeline: None,
        tonemap_bg: None,
        ibl_bg: None,
        mesh_vertices,
        mesh_indices: scene.mesh.indices,
        material: scene.material,
    }).unwrap();
}

struct ShowcaseApp {
    render_app: RenderApp,
    app: App,
    initialized: bool,
    scene_ub: Option<wgpu::Buffer>,
    scene_bg: Option<wgpu::BindGroup>,
    depth_view: Option<wgpu::TextureView>,
    hdr_view: Option<wgpu::TextureView>,
    hdr_msaa_view: Option<wgpu::TextureView>,
    tonemap_pipeline: Option<wgpu::RenderPipeline>,
    tonemap_bg: Option<wgpu::BindGroup>,
    ibl_bg: Option<wgpu::BindGroup>,
    mesh_vertices: Vec<PbrVertex>,
    mesh_indices: Vec<u32>,
    material: anvilkit_assets::material::MaterialData,
}

impl ShowcaseApp {
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
            label: Some("Scene BG"), layout: &scene_bgl,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: ub.as_entire_binding() }],
        });
        let (_, depth_view) = create_depth_texture_msaa(device, w, h, "Depth");

        // Material textures (group 1: 5 textures + 1 sampler)
        let tex = |t: &Option<anvilkit_assets::material::TextureData>, label, fb: &[u8; 4], srgb: bool| {
            if let Some(ref tex) = t {
                if srgb { create_texture(device, tex.width, tex.height, &tex.data, label).1 }
                else { create_texture_linear(device, tex.width, tex.height, &tex.data, label).1 }
            } else {
                if srgb { create_texture(device, 1, 1, fb, label).1 }
                else { create_texture_linear(device, 1, 1, fb, label).1 }
            }
        };
        let bc = tex(&self.material.base_color_texture, "BaseColor", &[255,255,255,255], true);
        let nm = tex(&self.material.normal_texture, "Normal", &[128,128,255,255], false);
        let mr = tex(&self.material.metallic_roughness_texture, "MR", &[255,255,255,255], false);
        let ao = tex(&self.material.occlusion_texture, "AO", &[255,255,255,255], false);
        let em = tex(&self.material.emissive_texture, "Emissive", &[255,255,255,255], true);
        let sampler = create_sampler(device, "Sampler");

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
                wgpu::BindGroupLayoutEntry {
                    binding: 5, visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None,
                }],
        });
        let mat_bg = device.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Mat BG"), layout: &mat_bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&bc) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&nm) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&mr) },
                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(&ao) },
                wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::TextureView(&em) },
                wgpu::BindGroupEntry { binding: 5, resource: wgpu::BindingResource::Sampler(&sampler) },
            ],
        });

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

        // PBR pipeline (HDR + MSAA)
        let pipeline = RenderPipelineBuilder::new()
            .with_vertex_shader(SHADER_SOURCE)
            .with_fragment_shader(SHADER_SOURCE)
            .with_format(HDR_FORMAT)
            .with_vertex_layouts(vec![PbrVertex::layout()])
            .with_depth_format(DEPTH_FORMAT)
            .with_multisample_count(MSAA_SAMPLE_COUNT)
            .with_bind_group_layouts(vec![scene_bgl, mat_bgl, ibl_bgl])
            .with_label("PBR Pipeline")
            .build(device)
            .expect("Failed to create PBR pipeline");

        // HDR + tonemap
        let (_, hdr_view) = create_hdr_render_target(device, w, h, "HDR RT");
        let (_, hdr_msaa_view) = create_hdr_msaa_texture(device, w, h, "HDR MSAA");
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
            .with_label("Tonemap Pipeline")
            .build(device).expect("Failed to create tonemap pipeline");

        // Upload mesh
        let mut assets = self.app.world.resource_mut::<RenderAssets>();
        let mesh_h = assets.upload_mesh_u32(device, &self.mesh_vertices, &self.mesh_indices, "Helmet");
        let mat_h = assets.create_material(pipeline.into_pipeline(), mat_bg);

        self.app.world.spawn((
            mesh_h, mat_h,
            MaterialParams {
                metallic: self.material.metallic_factor,
                roughness: self.material.roughness_factor,
                normal_scale: self.material.normal_scale,
                emissive_factor: self.material.emissive_factor,
            },
            Transform::default(),
        ));

        // Camera (will orbit)
        let eye = glam::Vec3::new(0.0, 0.5, -3.0);
        let look_dir = (glam::Vec3::ZERO - eye).normalize();
        let cam_rot = glam::Quat::from_rotation_arc(glam::Vec3::Z, look_dir);
        self.app.world.spawn((
            CameraComponent { fov: 45.0, near: 0.1, far: 100.0, is_active: true, aspect_ratio: w as f32 / h.max(1) as f32 },
            Transform::from_xyz(eye.x, eye.y, eye.z).with_rotation(cam_rot),
        ));

        self.scene_ub = Some(ub);
        self.scene_bg = Some(scene_bg);
        self.depth_view = Some(depth_view);
        self.hdr_view = Some(hdr_view);
        self.hdr_msaa_view = Some(hdr_msaa_view);
        self.tonemap_pipeline = Some(tm_pipe.into_pipeline());
        self.tonemap_bg = Some(tm_bg);
        self.ibl_bg = Some(ibl_bg);
        self.initialized = true;
        println!("Showcase initialized!");
    }

    fn render_frame(&self) {
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
        let lp = -ld * 15.0;
        let lv = glam::Mat4::look_at_lh(lp, glam::Vec3::ZERO, glam::Vec3::Y);
        let lproj = glam::Mat4::orthographic_lh(-10.0, 10.0, -10.0, 10.0, 0.1, 30.0);
        let svp = lproj * lv;

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
                shadow_view_proj: svp.to_cols_array_2d(),
                emissive_factor: [cmd.emissive_factor[0], cmd.emissive_factor[1], cmd.emissive_factor[2], 0.0],
            };
            device.queue().write_buffer(ub, 0, bytemuck::bytes_of(&u));

            let mut enc = device.device().create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Scene Enc") });
            {
                let cl = if i == 0 { wgpu::LoadOp::Clear(wgpu::Color { r: 0.05, g: 0.08, b: 0.18, a: 1.0 }) } else { wgpu::LoadOp::Load };
                let dl_op = if i == 0 { wgpu::LoadOp::Clear(1.0) } else { wgpu::LoadOp::Load };
                let mut rp = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Scene Pass"),
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
            let mut enc = device.device().create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Tonemap Enc") });
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

        frame.present();
    }
}

impl ApplicationHandler for ShowcaseApp {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        self.render_app.resumed(el);
        self.init_scene();
    }

    fn window_event(&mut self, el: &ActiveEventLoop, wid: WindowId, ev: WindowEvent) {
        match &ev {
            WindowEvent::Resized(s) if self.initialized && s.width > 0 && s.height > 0 => {
                if let Some(device) = self.render_app.render_device() {
                    let (_, dv) = create_depth_texture_msaa(device, s.width, s.height, "Depth");
                    self.depth_view = Some(dv);
                    let (_, hv) = create_hdr_render_target(device, s.width, s.height, "HDR RT");
                    let (_, hmv) = create_hdr_msaa_texture(device, s.width, s.height, "HDR MSAA");
                    let samp = create_sampler(device, "Sampler");
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
        // Orbit camera
        if let Some(ft) = self.app.world.get_resource::<FrameTime>() {
            let t = ft.0.elapsed().as_secs_f32();
            let radius = 3.5;
            let height = 0.8;
            let speed = 0.3;
            let eye = glam::Vec3::new(
                (t * speed).sin() * radius,
                height + (t * speed * 0.5).sin() * 0.3,
                (t * speed).cos() * radius,
            );
            let target = glam::Vec3::new(0.0, 0.0, 0.0);
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
    }
}
