// AnvilKit Skinned PBR Shader
// Extends PBR shader with bone matrix palette for skeletal animation
// Vertex shader applies Linear Blend Skinning (LBS) before world transform

const PI: f32 = 3.14159265359;
const MAX_JOINTS: u32 = 128u;

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

struct JointMatrices {
    matrices: array<mat4x4<f32>, 128>,
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
@group(3) @binding(0) var<storage, read> joints: JointMatrices;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) texcoord: vec2<f32>,
    @location(3) tangent: vec4<f32>,
    @location(4) joint_indices: vec4<u32>,
    @location(5) joint_weights: vec4<f32>,
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
    // Linear Blend Skinning
    let skin_matrix = joints.matrices[in.joint_indices.x] * in.joint_weights.x
                    + joints.matrices[in.joint_indices.y] * in.joint_weights.y
                    + joints.matrices[in.joint_indices.z] * in.joint_weights.z
                    + joints.matrices[in.joint_indices.w] * in.joint_weights.w;

    let skinned_pos = skin_matrix * vec4<f32>(in.position, 1.0);
    let skinned_normal = (skin_matrix * vec4<f32>(in.normal, 0.0)).xyz;
    let skinned_tangent = (skin_matrix * vec4<f32>(in.tangent.xyz, 0.0)).xyz;

    var out: VertexOutput;
    let world_pos = scene.model * skinned_pos;
    out.clip_position = scene.view_proj * world_pos;
    out.world_position = world_pos.xyz;
    let N = normalize((scene.normal_matrix * vec4<f32>(skinned_normal, 0.0)).xyz);
    let T = normalize((scene.model * vec4<f32>(skinned_tangent, 0.0)).xyz);
    let B = cross(N, T) * in.tangent.w;
    out.world_normal = N;
    out.world_tangent = T;
    out.world_bitangent = B;
    out.texcoord = in.texcoord;
    return out;
}

// ---------- PBR functions (unified with pbr.wgsl) ----------

fn distribution_ggx(N: vec3<f32>, H: vec3<f32>, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let NdotH = max(dot(N, H), 0.0);
    let denom = NdotH * NdotH * (a2 - 1.0) + 1.0;
    return a2 / (PI * denom * denom);
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

fn fresnel_schlick(cos_theta: f32, F0: vec3<f32>) -> vec3<f32> {
    return F0 + (1.0 - F0) * pow(clamp(1.0 - cos_theta, 0.0, 1.0), 5.0);
}

fn fresnel_schlick_roughness(cos_theta: f32, F0: vec3<f32>, roughness: f32) -> vec3<f32> {
    return F0 + (max(vec3<f32>(1.0 - roughness), F0) - F0) * pow(1.0 - cos_theta, 5.0);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let base_color = textureSample(base_color_texture, material_sampler, in.texcoord);
    let albedo = base_color.rgb;
    let alpha = base_color.a;

    let mr = textureSample(metallic_roughness_texture, material_sampler, in.texcoord);
    let metallic = mr.b * scene.material_params.x;
    let roughness = clamp(mr.g * scene.material_params.y, 0.04, 1.0);

    let normal_scale = scene.material_params.z;
    let normal_map = textureSample(normal_map_texture, material_sampler, in.texcoord).xyz * 2.0 - 1.0;
    let scaled_normal = vec3<f32>(normal_map.x * normal_scale, normal_map.y * normal_scale, normal_map.z);
    let TBN = mat3x3<f32>(normalize(in.world_tangent), normalize(in.world_bitangent), normalize(in.world_normal));
    let N = normalize(TBN * scaled_normal);

    let ao = textureSample(ao_texture, material_sampler, in.texcoord).r;
    let emissive_tex = textureSample(emissive_texture, material_sampler, in.texcoord).rgb;
    let emissive = emissive_tex * scene.emissive_factor.xyz;

    let V = normalize(scene.camera_pos.xyz - in.world_position);
    let NdotV = max(dot(N, V), 0.0);
    let F0 = mix(vec3<f32>(0.04), albedo, metallic);

    var Lo = vec3<f32>(0.0);
    let light_count = u32(scene.material_params.w);

    for (var i = 0u; i < light_count; i++) {
        let light = scene.lights[i];
        let light_type = u32(light.position_type.w);
        var L: vec3<f32>;
        var attenuation: f32 = 1.0;
        let light_color = light.color_intensity.xyz * light.color_intensity.w;

        if (light_type == 0u) {
            L = normalize(-light.direction_range.xyz);
        } else {
            let light_pos = light.position_type.xyz;
            let to_light = light_pos - in.world_position;
            let dist = length(to_light);
            L = normalize(to_light);
            let range = light.direction_range.w;
            attenuation = max(1.0 - dist / range, 0.0);
            attenuation = attenuation * attenuation;
        }

        let H = normalize(V + L);
        let NdotL = max(dot(N, L), 0.0);
        let NdotH = max(dot(N, H), 0.0);
        let HdotV = max(dot(H, V), 0.0);

        let D = distribution_ggx(N, H, roughness);
        let G = geometry_smith(N, V, L, roughness);
        let F = fresnel_schlick(HdotV, F0);

        let spec = (D * G * F) / (4.0 * NdotV * NdotL + 0.0001);
        let kD = (vec3<f32>(1.0) - F) * (1.0 - metallic);
        Lo += (kD * albedo / PI + spec) * light_color * NdotL * attenuation;
    }

    // IBL ambient
    let F_env = fresnel_schlick_roughness(NdotV, F0, roughness);
    let kD_env = (vec3<f32>(1.0) - F_env) * (1.0 - metallic);
    let irradiance = vec3<f32>(0.15, 0.18, 0.25);
    let diffuse_env = kD_env * albedo * irradiance;
    let brdf = textureSample(brdf_lut, brdf_lut_sampler, vec2<f32>(NdotV, roughness)).rg;
    let spec_env = F_env * brdf.x + brdf.y;
    let ambient = (diffuse_env + spec_env * 0.3) * ao;

    let color = Lo + ambient + emissive;
    return vec4<f32>(color, alpha);
}
