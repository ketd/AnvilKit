// Motion Blur shader — velocity buffer reconstruction + directional blur.

struct MotionBlurParams {
    intensity: f32,
    samples: f32,
    _pad0: f32,
    _pad1: f32,
    prev_view_proj_0: vec4<f32>,
    prev_view_proj_1: vec4<f32>,
    prev_view_proj_2: vec4<f32>,
    prev_view_proj_3: vec4<f32>,
    curr_inv_view_proj_0: vec4<f32>,
    curr_inv_view_proj_1: vec4<f32>,
    curr_inv_view_proj_2: vec4<f32>,
    curr_inv_view_proj_3: vec4<f32>,
};

@group(0) @binding(0) var src_texture: texture_2d<f32>;
@group(0) @binding(1) var depth_texture: texture_depth_2d;
@group(0) @binding(2) var tex_sampler: sampler;
@group(0) @binding(3) var<uniform> params: MotionBlurParams;

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

fn get_prev_view_proj() -> mat4x4<f32> {
    return mat4x4<f32>(
        params.prev_view_proj_0,
        params.prev_view_proj_1,
        params.prev_view_proj_2,
        params.prev_view_proj_3,
    );
}

fn get_curr_inv_view_proj() -> mat4x4<f32> {
    return mat4x4<f32>(
        params.curr_inv_view_proj_0,
        params.curr_inv_view_proj_1,
        params.curr_inv_view_proj_2,
        params.curr_inv_view_proj_3,
    );
}

// Compute screen-space velocity by reprojecting with previous frame's VP
@fragment
fn velocity_fs(in: VertexOutput) -> @location(0) vec2<f32> {
    let dims = vec2<f32>(textureDimensions(depth_texture));
    let coord = vec2<i32>(in.uv * dims);
    let depth = textureLoad(depth_texture, coord, 0);

    // NDC position (current frame)
    let ndc = vec4<f32>(in.uv * 2.0 - 1.0, depth, 1.0);

    // Reconstruct world position
    let inv_vp = get_curr_inv_view_proj();
    let world_h = inv_vp * ndc;
    let world_pos = world_h.xyz / world_h.w;

    // Reproject to previous frame
    let prev_vp = get_prev_view_proj();
    let prev_clip = prev_vp * vec4<f32>(world_pos, 1.0);
    let prev_ndc = prev_clip.xy / prev_clip.w;

    // Screen-space velocity (in UV space)
    let velocity = (in.uv - (prev_ndc * 0.5 + 0.5)) * params.intensity;
    return velocity;
}

// Directional blur along velocity vector
@group(0) @binding(4) var velocity_texture: texture_2d<f32>;

@fragment
fn blur_fs(in: VertexOutput) -> @location(0) vec4<f32> {
    let velocity = textureSample(velocity_texture, tex_sampler, in.uv).rg;
    let num_samples = i32(params.samples);

    var color = textureSample(src_texture, tex_sampler, in.uv).rgb;

    for (var i = 1; i < num_samples; i++) {
        let t = f32(i) / f32(num_samples - 1) - 0.5;
        let offset = velocity * t;
        color += textureSample(src_texture, tex_sampler, in.uv + offset).rgb;
    }

    color /= f32(num_samples);
    return vec4<f32>(color, 1.0);
}
