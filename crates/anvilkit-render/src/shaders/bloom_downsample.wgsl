// Bloom downsample shader — 13-tap bilinear filter with brightness threshold.
//
// First pass (mip 0 → mip 1): applies threshold to extract bright pixels.
// Subsequent passes (mip N → mip N+1): pure downsample without threshold.

struct BloomParams {
    threshold: f32,
    knee: f32,      // soft knee range
    intensity: f32,
    mip_level: f32, // 0 = threshold pass, >0 = pure downsample
};

@group(0) @binding(0) var src_texture: texture_2d<f32>;
@group(0) @binding(1) var src_sampler: sampler;
@group(0) @binding(2) var<uniform> params: BloomParams;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

// Fullscreen triangle (no vertex buffer needed)
@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VertexOutput {
    var out: VertexOutput;
    let uv = vec2<f32>(f32((vi << 1u) & 2u), f32(vi & 2u));
    out.position = vec4<f32>(uv * 2.0 - 1.0, 0.0, 1.0);
    out.uv = vec2<f32>(uv.x, 1.0 - uv.y);
    return out;
}

// Soft threshold: smoothstep-like knee curve
fn soft_threshold(color: vec3<f32>, threshold: f32, knee: f32) -> vec3<f32> {
    let brightness = max(color.r, max(color.g, color.b));
    var soft = brightness - threshold + knee;
    soft = clamp(soft, 0.0, 2.0 * knee);
    soft = soft * soft / (4.0 * knee + 0.00001);
    let contribution = max(soft, brightness - threshold) / max(brightness, 0.00001);
    return color * max(contribution, 0.0);
}

// 13-tap downsample filter (from Call of Duty: Advanced Warfare presentation)
// Produces a better result than naive bilinear by sampling in a cross pattern
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let texel_size = vec2<f32>(1.0) / vec2<f32>(textureDimensions(src_texture));
    let uv = in.uv;

    // 13-tap pattern: center + 4 diagonal + 4 axis + 4 far diagonal
    let a = textureSample(src_texture, src_sampler, uv + texel_size * vec2<f32>(-2.0, 2.0)).rgb;
    let b = textureSample(src_texture, src_sampler, uv + texel_size * vec2<f32>(0.0, 2.0)).rgb;
    let c = textureSample(src_texture, src_sampler, uv + texel_size * vec2<f32>(2.0, 2.0)).rgb;

    let d = textureSample(src_texture, src_sampler, uv + texel_size * vec2<f32>(-2.0, 0.0)).rgb;
    let e = textureSample(src_texture, src_sampler, uv).rgb;
    let f = textureSample(src_texture, src_sampler, uv + texel_size * vec2<f32>(2.0, 0.0)).rgb;

    let g = textureSample(src_texture, src_sampler, uv + texel_size * vec2<f32>(-2.0, -2.0)).rgb;
    let h = textureSample(src_texture, src_sampler, uv + texel_size * vec2<f32>(0.0, -2.0)).rgb;
    let i = textureSample(src_texture, src_sampler, uv + texel_size * vec2<f32>(2.0, -2.0)).rgb;

    let j = textureSample(src_texture, src_sampler, uv + texel_size * vec2<f32>(-1.0, 1.0)).rgb;
    let k = textureSample(src_texture, src_sampler, uv + texel_size * vec2<f32>(1.0, 1.0)).rgb;
    let l = textureSample(src_texture, src_sampler, uv + texel_size * vec2<f32>(-1.0, -1.0)).rgb;
    let m = textureSample(src_texture, src_sampler, uv + texel_size * vec2<f32>(1.0, -1.0)).rgb;

    // Weighted combination (prevents firefly artifacts from bright spots)
    var color = e * 0.125;
    color += (a + c + g + i) * 0.03125;
    color += (b + d + f + h) * 0.0625;
    color += (j + k + l + m) * 0.125;

    // Apply threshold only on the first downsample pass
    if params.mip_level < 0.5 {
        color = soft_threshold(color, params.threshold, params.knee);
    }

    return vec4<f32>(color, 1.0);
}
