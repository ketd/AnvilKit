// AnvilKit PBR 着色器
// Cook-Torrance BRDF + TBN 法线贴图 + 多光源 + 阴影 + IBL + 完整材质

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
    cascade_view_projs: array<mat4x4<f32>, 3>,
    cascade_splits: vec4<f32>,
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
@group(2) @binding(2) var shadow_map: texture_depth_2d_array;
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

fn calculate_shadow(world_pos: vec3<f32>) -> f32 {
    // Compute view-space depth for cascade selection
    let view_pos = scene.view_proj * vec4<f32>(world_pos, 1.0);
    let view_z = view_pos.w; // w contains the linear view-space depth after perspective

    // Determine cascade index from split distances
    let cascade_count = u32(scene.emissive_factor.w);
    var cascade_idx = 0u;
    if (view_z > scene.cascade_splits.x) { cascade_idx = 1u; }
    if (view_z > scene.cascade_splits.y) { cascade_idx = 2u; }
    if (cascade_idx >= cascade_count) { return 1.0; }

    let shadow_vp = scene.cascade_view_projs[cascade_idx];
    let clip = shadow_vp * vec4<f32>(world_pos, 1.0);
    let ndc = clip.xyz / clip.w;
    let uv = vec2<f32>(ndc.x * 0.5 + 0.5, -ndc.y * 0.5 + 0.5);
    let depth = ndc.z;
    if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0 || depth > 1.0) { return 1.0; }

    let ts = scene.cascade_splits.w; // shadow texel size
    var s = 0.0;
    for (var x = -1; x <= 1; x++) { for (var y = -1; y <= 1; y++) {
        s += textureSampleCompare(shadow_map, shadow_sampler, uv + vec2<f32>(f32(x), f32(y)) * ts, cascade_idx, depth - 0.005);
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

    let shadow = calculate_shadow(in.world_position);
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
