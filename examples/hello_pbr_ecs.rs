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
const SHADER_SOURCE: &str = r#"
const PI: f32 = 3.14159265359;

struct GpuLight {
    position_type: vec4<f32>,      // xyz=pos, w=type (0=dir, 1=point, 2=spot)
    direction_range: vec4<f32>,    // xyz=dir, w=range
    color_intensity: vec4<f32>,    // rgb=color, w=intensity
    params: vec4<f32>,             // x=inner_cone_cos, y=outer_cone_cos
};

struct SceneUniform {
    model: mat4x4<f32>,
    view_proj: mat4x4<f32>,
    normal_matrix: mat4x4<f32>,
    camera_pos: vec4<f32>,
    light_dir: vec4<f32>,          // legacy
    light_color: vec4<f32>,        // legacy
    material_params: vec4<f32>,    // (metallic, roughness, normal_scale, light_count)
    lights: array<GpuLight, 8>,
    shadow_view_proj: mat4x4<f32>,
    emissive_factor: vec4<f32>,      // rgb + 0
};

@group(0) @binding(0)
var<uniform> scene: SceneUniform;

@group(1) @binding(0)
var base_color_texture: texture_2d<f32>;
@group(1) @binding(1)
var normal_map_texture: texture_2d<f32>;
@group(1) @binding(2)
var metallic_roughness_texture: texture_2d<f32>;
@group(1) @binding(3)
var ao_texture: texture_2d<f32>;
@group(1) @binding(4)
var emissive_texture: texture_2d<f32>;
@group(1) @binding(5)
var material_sampler: sampler;

// IBL + Shadow resources (group 2)
@group(2) @binding(0)
var brdf_lut: texture_2d<f32>;
@group(2) @binding(1)
var brdf_lut_sampler: sampler;
@group(2) @binding(2)
var shadow_map: texture_depth_2d;
@group(2) @binding(3)
var shadow_sampler: sampler_comparison;

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

// Shadow mapping: PCF 3x3
fn calculate_shadow(world_pos: vec3<f32>, shadow_vp: mat4x4<f32>) -> f32 {
    let light_clip = shadow_vp * vec4<f32>(world_pos, 1.0);
    let light_ndc = light_clip.xyz / light_clip.w;
    // NDC → shadow UV: x [-1,1]→[0,1], y [-1,1]→[0,1] (flip Y)
    let shadow_uv = vec2<f32>(light_ndc.x * 0.5 + 0.5, -light_ndc.y * 0.5 + 0.5);
    let current_depth = light_ndc.z;

    // Out of shadow frustum → fully lit
    if (shadow_uv.x < 0.0 || shadow_uv.x > 1.0 || shadow_uv.y < 0.0 || shadow_uv.y > 1.0 || current_depth > 1.0) {
        return 1.0;
    }

    // PCF 3x3 (texel size for 2048 shadow map)
    let texel_size = 1.0 / 2048.0;
    var shadow = 0.0;
    for (var x = -1; x <= 1; x = x + 1) {
        for (var y = -1; y <= 1; y = y + 1) {
            let offset = vec2<f32>(f32(x), f32(y)) * texel_size;
            shadow = shadow + textureSampleCompare(shadow_map, shadow_sampler, shadow_uv + offset, current_depth - 0.005);
        }
    }
    return shadow / 9.0;
}

// Fresnel with roughness for IBL (Sébastien Lagarde)
fn fresnel_schlick_roughness(cos_theta: f32, F0: vec3<f32>, roughness: f32) -> vec3<f32> {
    let smooth_val = max(vec3<f32>(1.0 - roughness), F0);
    return F0 + (smooth_val - F0) * pow(clamp(1.0 - cos_theta, 0.0, 1.0), 5.0);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let albedo = textureSample(base_color_texture, material_sampler, in.texcoord).rgb;
    let normal_scale = scene.material_params.z;

    // Sample metallic-roughness texture (glTF: G=roughness, B=metallic)
    let mr_sample = textureSample(metallic_roughness_texture, material_sampler, in.texcoord);
    let metallic = mr_sample.b * scene.material_params.x;
    let roughness = mr_sample.g * scene.material_params.y;

    // Sample AO texture (R channel)
    let ao = textureSample(ao_texture, material_sampler, in.texcoord).r;

    // Sample normal map and transform from tangent space to world space
    let normal_map = textureSample(normal_map_texture, material_sampler, in.texcoord).rgb;
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

    // === Shadow ===
    let shadow = calculate_shadow(in.world_position, scene.shadow_view_proj);

    // === Direct lighting (multi-light loop) ===
    let light_count = u32(scene.material_params.w);
    var Lo = vec3<f32>(0.0);

    for (var li = 0u; li < light_count; li = li + 1u) {
        let light = scene.lights[li];
        let light_type = u32(light.position_type.w);
        let light_color_raw = light.color_intensity.xyz;
        let light_intensity = light.color_intensity.w;

        var L: vec3<f32>;
        var attenuation: f32 = 1.0;

        if (light_type == 0u) {
            // Directional light
            L = normalize(-light.direction_range.xyz);
        } else {
            // Point or spot light
            let light_pos = light.position_type.xyz;
            let to_light = light_pos - in.world_position;
            let dist = length(to_light);
            L = to_light / max(dist, 0.0001);
            let light_range = light.direction_range.w;
            // Smooth distance attenuation
            let dist_ratio = clamp(dist / light_range, 0.0, 1.0);
            attenuation = max(1.0 - dist_ratio * dist_ratio, 0.0);
            attenuation = attenuation * attenuation;

            if (light_type == 2u) {
                // Spot light cone attenuation
                let spot_dir = normalize(light.direction_range.xyz);
                let cos_angle = dot(spot_dir, -L);
                let inner_cos = light.params.x;
                let outer_cos = light.params.y;
                let cone = clamp((cos_angle - outer_cos) / max(inner_cos - outer_cos, 0.0001), 0.0, 1.0);
                attenuation = attenuation * cone;
            }
        }

        let H = normalize(V + L);
        let radiance = light_color_raw * light_intensity * attenuation;

        let D = distribution_ggx(N, H, roughness);
        let G = geometry_smith(N, V, L, roughness);
        let F = fresnel_schlick(max(dot(H, V), 0.0), F0);

        let numerator = D * G * F;
        let denominator = 4.0 * NdotV * max(dot(N, L), 0.0) + 0.0001;
        let specular = numerator / denominator;

        let kD = (vec3<f32>(1.0) - F) * (1.0 - metallic);
        let NdotL = max(dot(N, L), 0.0);

        // Apply shadow only to directional light (index 0)
        var light_shadow = 1.0;
        if (li == 0u && light_type == 0u) {
            light_shadow = shadow;
        }

        Lo = Lo + (kD * albedo / PI + specular) * radiance * NdotL * light_shadow;
    }

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

    let ambient = (diffuse_ibl + specular_ibl) * ao;

    // Emissive
    let emissive_tex = textureSample(emissive_texture, material_sampler, in.texcoord).rgb;
    let emissive = emissive_tex * scene.emissive_factor.xyz;

    let color = ambient + Lo + emissive;

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

/// Pack SceneLights into GpuLight array for the uniform buffer
fn pack_scene_lights(scene_lights: &SceneLights) -> ([GpuLight; 8], u32) {
    let mut lights = [GpuLight::default(); 8];
    let mut count = 0u32;

    let dir = &scene_lights.directional;
    lights[0] = GpuLight {
        position_type: [0.0, 0.0, 0.0, 0.0],
        direction_range: [dir.direction.x, dir.direction.y, dir.direction.z, 0.0],
        color_intensity: [dir.color.x, dir.color.y, dir.color.z, dir.intensity],
        params: [0.0; 4],
    };
    count += 1;

    for pl in &scene_lights.point_lights {
        if count >= 8 { break; }
        lights[count as usize] = GpuLight {
            position_type: [pl.position.x, pl.position.y, pl.position.z, 1.0],
            direction_range: [0.0, 0.0, 0.0, pl.range],
            color_intensity: [pl.color.x, pl.color.y, pl.color.z, pl.intensity],
            params: [0.0; 4],
        };
        count += 1;
    }

    for sl in &scene_lights.spot_lights {
        if count >= 8 { break; }
        lights[count as usize] = GpuLight {
            position_type: [sl.position.x, sl.position.y, sl.position.z, 2.0],
            direction_range: [sl.direction.x, sl.direction.y, sl.direction.z, sl.range],
            color_intensity: [sl.color.x, sl.color.y, sl.color.z, sl.intensity],
            params: [sl.inner_cone_angle.cos(), sl.outer_cone_angle.cos(), 0.0, 0.0],
        };
        count += 1;
    }

    (lights, count)
}

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
        let (gpu_lights, light_count) = pack_scene_lights(scene_lights);

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
                    wgpu::LoadOp::Clear(wgpu::Color { r: 0.05, g: 0.08, b: 0.18, a: 1.0 })
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
