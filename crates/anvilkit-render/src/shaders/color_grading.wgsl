// Color Grading shader — exposure, contrast, saturation, white balance, LUT.

struct ColorGradingParams {
    exposure: f32,
    contrast: f32,
    saturation: f32,
    temperature: f32,
    tint: f32,
    lut_contribution: f32,
    _pad0: f32,
    _pad1: f32,
};

@group(0) @binding(0) var src_texture: texture_2d<f32>;
@group(0) @binding(1) var tex_sampler: sampler;
@group(0) @binding(2) var<uniform> params: ColorGradingParams;
@group(0) @binding(3) var lut_texture: texture_3d<f32>;
@group(0) @binding(4) var lut_sampler: sampler;

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

// Convert color temperature offset to RGB multipliers
fn temperature_to_rgb(temp: f32) -> vec3<f32> {
    // Approximation: positive = warm (more red), negative = cool (more blue)
    return vec3<f32>(
        1.0 + temp * 0.1,
        1.0,
        1.0 - temp * 0.1,
    );
}

// Apply tint (green-magenta axis)
fn tint_to_rgb(t: f32) -> vec3<f32> {
    return vec3<f32>(
        1.0 + t * 0.05,
        1.0 - abs(t) * 0.05,
        1.0 - t * 0.05,
    );
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(src_texture, tex_sampler, in.uv).rgb;

    // 1. Exposure
    color *= params.exposure;

    // 2. White balance
    color *= temperature_to_rgb(params.temperature);
    color *= tint_to_rgb(params.tint);

    // 3. Contrast (around mid-gray 0.18)
    let mid = vec3<f32>(0.18);
    color = mid + (color - mid) * params.contrast;

    // 4. Saturation
    let luminance = dot(color, vec3<f32>(0.2126, 0.7152, 0.0722));
    color = mix(vec3<f32>(luminance), color, params.saturation);

    // 5. LUT lookup (if contribution > 0)
    if params.lut_contribution > 0.001 {
        // LUT coordinates: clamp to [0.5/32, 1-0.5/32] for half-texel offset
        let lut_size = 32.0;
        let half_texel = 0.5 / lut_size;
        let lut_coord = clamp(color, vec3<f32>(0.0), vec3<f32>(1.0)) * (1.0 - 1.0 / lut_size) + half_texel;
        let lut_color = textureSample(lut_texture, lut_sampler, lut_coord).rgb;
        color = mix(color, lut_color, params.lut_contribution);
    }

    // Clamp to prevent negative values
    color = max(color, vec3<f32>(0.0));

    return vec4<f32>(color, 1.0);
}
