// Bloom upsample shader — 9-tap tent filter with additive blending.
//
// Reads from the lower (smaller) mip level and additively combines
// with the current (larger) mip level during the upsample chain.

struct BloomParams {
    threshold: f32,
    knee: f32,
    intensity: f32,
    mip_level: f32,
};

@group(0) @binding(0) var src_texture: texture_2d<f32>;
@group(0) @binding(1) var src_sampler: sampler;
@group(0) @binding(2) var<uniform> params: BloomParams;

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

// 9-tap tent filter for smooth upsample
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let texel_size = vec2<f32>(1.0) / vec2<f32>(textureDimensions(src_texture));
    let uv = in.uv;

    // 3x3 tent filter kernel (weights sum to 1.0)
    var color = vec3<f32>(0.0);
    color += textureSample(src_texture, src_sampler, uv + texel_size * vec2<f32>(-1.0, 1.0)).rgb * (1.0 / 16.0);
    color += textureSample(src_texture, src_sampler, uv + texel_size * vec2<f32>(0.0, 1.0)).rgb * (2.0 / 16.0);
    color += textureSample(src_texture, src_sampler, uv + texel_size * vec2<f32>(1.0, 1.0)).rgb * (1.0 / 16.0);

    color += textureSample(src_texture, src_sampler, uv + texel_size * vec2<f32>(-1.0, 0.0)).rgb * (2.0 / 16.0);
    color += textureSample(src_texture, src_sampler, uv).rgb * (4.0 / 16.0);
    color += textureSample(src_texture, src_sampler, uv + texel_size * vec2<f32>(1.0, 0.0)).rgb * (2.0 / 16.0);

    color += textureSample(src_texture, src_sampler, uv + texel_size * vec2<f32>(-1.0, -1.0)).rgb * (1.0 / 16.0);
    color += textureSample(src_texture, src_sampler, uv + texel_size * vec2<f32>(0.0, -1.0)).rgb * (2.0 / 16.0);
    color += textureSample(src_texture, src_sampler, uv + texel_size * vec2<f32>(1.0, -1.0)).rgb * (1.0 / 16.0);

    // Scale by bloom intensity
    color *= params.intensity;

    return vec4<f32>(color, 1.0);
}
