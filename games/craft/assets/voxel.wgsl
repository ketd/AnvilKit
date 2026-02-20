// Voxel rendering shader for Craft
// Group 0: scene uniform (view_proj, camera_pos, light_dir, fog, time)
// Group 1: atlas texture + sampler (NEAREST)

struct VoxelSceneUniform {
    view_proj: mat4x4<f32>,
    camera_pos: vec4<f32>,
    light_dir: vec4<f32>,   // xyz = direction (toward light), w = unused
    fog_color: vec4<f32>,   // rgb = fog color, a = fog density
    time_ambient: vec4<f32>, // x = time, y = ambient strength, z = fog_start, w = fog_end
};

@group(0) @binding(0) var<uniform> scene: VoxelSceneUniform;

@group(1) @binding(0) var atlas: texture_2d<f32>;
@group(1) @binding(1) var samp: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) ao: f32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) ao: f32,
    @location(3) world_pos: vec3<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = scene.view_proj * vec4<f32>(in.position, 1.0);
    out.uv = in.uv;
    out.normal = in.normal;
    out.ao = in.ao;
    out.world_pos = in.position;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample atlas texture
    let albedo = textureSample(atlas, samp, in.uv);

    // Alpha test for plants/transparent blocks
    if (albedo.a < 0.5) {
        discard;
    }

    // Directional light (simple NdotL)
    let light_dir = normalize(scene.light_dir.xyz);
    let n = normalize(in.normal);
    let ndotl = max(dot(n, light_dir), 0.0);

    let ambient = scene.time_ambient.y;
    let diffuse = ndotl * (1.0 - ambient);
    let light = ambient + diffuse;

    // AO darkening: remap [0,1] → [0.15,1] so fully occluded corners are
    // very dark but never pure black.
    let ao = 0.15 + 0.85 * in.ao;

    // Apply lighting
    var color = albedo.rgb * light * ao;

    // Distance fog
    let dist = length(in.world_pos - scene.camera_pos.xyz);
    let fog_start = scene.time_ambient.z;
    let fog_end = scene.time_ambient.w;
    let fog_factor = clamp((dist - fog_start) / (fog_end - fog_start), 0.0, 1.0);
    color = mix(color, scene.fog_color.rgb, fog_factor);

    return vec4<f32>(color, 1.0);
}
