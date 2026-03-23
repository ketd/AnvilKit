// Screen-Space Ambient Occlusion (SSAO)
//
// Hemisphere sampling with depth-reconstructed normals.
// Runs at half resolution for performance.

struct SsaoParams {
    projection: mat4x4<f32>,     // camera projection matrix
    inv_projection: mat4x4<f32>, // inverse projection matrix
    radius: f32,                  // sampling radius in view-space
    bias: f32,                    // depth bias to prevent self-occlusion
    intensity: f32,               // AO strength multiplier
    sample_count: f32,            // number of kernel samples (as float)
};

@group(0) @binding(0) var depth_texture: texture_depth_2d;
@group(0) @binding(1) var depth_sampler: sampler;
@group(0) @binding(2) var noise_texture: texture_2d<f32>;
@group(0) @binding(3) var noise_sampler: sampler;
@group(0) @binding(4) var<uniform> params: SsaoParams;
@group(0) @binding(5) var<storage, read> kernel_samples: array<vec4<f32>>;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VertexOutput {
    var out: VertexOutput;
    let uv = vec2<f32>(f32((vi << 1u) & 2u), f32(vi & 2u));
    out.position = vec4<f32>(uv * 2.0 - 1.0, 0.0, 1.0);
    out.uv = vec2<f32>(uv.x, 1.0 - uv.y);
    return out;
}

// Sample depth texture (returns f32 directly for texture_depth_2d)
fn sample_depth(uv: vec2<f32>) -> f32 {
    return textureSampleLevel(depth_texture, depth_sampler, uv, 0.0);
}

// Reconstruct view-space position from depth and UV
fn view_pos_from_depth(uv: vec2<f32>, depth: f32) -> vec3<f32> {
    let ndc = vec4<f32>(uv * 2.0 - 1.0, depth, 1.0);
    var view_pos = params.inv_projection * ndc;
    view_pos /= view_pos.w;
    return view_pos.xyz;
}

// Reconstruct normal from depth buffer using cross product of screen-space derivatives
fn reconstruct_normal(uv: vec2<f32>, texel_size: vec2<f32>) -> vec3<f32> {
    let depth_c = sample_depth(uv);
    let depth_r = sample_depth(uv + vec2<f32>(texel_size.x, 0.0));
    let depth_u = sample_depth(uv + vec2<f32>(0.0, -texel_size.y));

    let pos_c = view_pos_from_depth(uv, depth_c);
    let pos_r = view_pos_from_depth(uv + vec2<f32>(texel_size.x, 0.0), depth_r);
    let pos_u = view_pos_from_depth(uv + vec2<f32>(0.0, -texel_size.y), depth_u);

    let normal = normalize(cross(pos_r - pos_c, pos_u - pos_c));
    return normal;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let depth_dims = vec2<f32>(textureDimensions(depth_texture));
    let texel_size = vec2<f32>(1.0) / depth_dims;
    let uv = in.uv;

    let depth = sample_depth(uv);

    // Skip sky (depth == 1.0 or very close)
    if depth > 0.9999 {
        return vec4<f32>(1.0, 1.0, 1.0, 1.0);
    }

    let frag_pos = view_pos_from_depth(uv, depth);
    let normal = reconstruct_normal(uv, texel_size);

    // Sample noise for random rotation of the kernel
    let noise_dims = vec2<f32>(textureDimensions(noise_texture));
    let noise_uv = uv * depth_dims / noise_dims;
    let random_vec = normalize(textureSample(noise_texture, noise_sampler, noise_uv).xyz * 2.0 - 1.0);

    // Gram-Schmidt to create TBN from normal + random_vec
    let tangent = normalize(random_vec - normal * dot(random_vec, normal));
    let bitangent = cross(normal, tangent);
    let tbn = mat3x3<f32>(tangent, bitangent, normal);

    let sample_count = i32(params.sample_count);
    var occlusion = 0.0;

    for (var i = 0; i < sample_count; i++) {
        // Orient sample in view space
        let sample_dir = tbn * kernel_samples[i].xyz;
        let sample_pos = frag_pos + sample_dir * params.radius;

        // Project sample to screen space
        var offset = params.projection * vec4<f32>(sample_pos, 1.0);
        offset /= offset.w;
        let sample_uv = offset.xy * 0.5 + 0.5;

        // Sample depth at projected position
        let sample_depth_val = sample_depth(vec2<f32>(sample_uv.x, 1.0 - sample_uv.y));
        let sample_view_z = view_pos_from_depth(vec2<f32>(sample_uv.x, 1.0 - sample_uv.y), sample_depth_val).z;

        // Range check: only occlude within a reasonable distance
        let range_check = smoothstep(0.0, 1.0, params.radius / abs(frag_pos.z - sample_view_z));

        // Check if sample point is occluded
        if sample_view_z >= sample_pos.z + params.bias {
            occlusion += range_check;
        }
    }

    occlusion = 1.0 - (occlusion / f32(sample_count)) * params.intensity;
    return vec4<f32>(occlusion, occlusion, occlusion, 1.0);
}
