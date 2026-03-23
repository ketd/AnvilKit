// Depth of Field shader — Circle of Confusion + disc blur + composite.

struct DofParams {
    focus_distance: f32,
    focus_range: f32,
    bokeh_radius: f32,
    _pad: f32,
};

@group(0) @binding(0) var src_texture: texture_2d<f32>;
@group(0) @binding(1) var depth_texture: texture_depth_2d;
@group(0) @binding(2) var tex_sampler: sampler;
@group(0) @binding(3) var<uniform> params: DofParams;

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

// --- Pass 1: Compute Circle of Confusion (output R16Float) ---
@fragment
fn coc_fs(in: VertexOutput) -> @location(0) f32 {
    let dims = vec2<f32>(textureDimensions(depth_texture));
    let coord = vec2<i32>(in.uv * dims);
    let depth = textureLoad(depth_texture, coord, 0);
    // Linearize: assume reversed-Z NDC (depth=1 near, depth=0 far)
    let linear_depth = 1.0 / max(depth, 0.0001);
    let diff = abs(linear_depth - params.focus_distance);
    let coc = clamp(diff / params.focus_range, 0.0, 1.0);
    return coc;
}

// --- Pass 2: Disc blur using CoC (reads src_texture + CoC from binding 4) ---
@group(0) @binding(4) var coc_texture: texture_2d<f32>;

@fragment
fn blur_fs(in: VertexOutput) -> @location(0) vec4<f32> {
    let texel_size = vec2<f32>(1.0) / vec2<f32>(textureDimensions(src_texture));
    let center_coc = textureSample(coc_texture, tex_sampler, in.uv).r;
    let radius = center_coc * params.bokeh_radius;

    var color = textureSample(src_texture, tex_sampler, in.uv).rgb;
    var total_weight = 1.0;

    // Simple disc kernel: 16 samples in a circle
    let SAMPLE_COUNT = 16u;
    let GOLDEN_ANGLE = 2.39996323;
    for (var i = 0u; i < SAMPLE_COUNT; i++) {
        let angle = f32(i) * GOLDEN_ANGLE;
        let r = sqrt(f32(i + 1u) / f32(SAMPLE_COUNT)) * radius;
        let offset = vec2<f32>(cos(angle), sin(angle)) * r * texel_size;
        let sample_color = textureSample(src_texture, tex_sampler, in.uv + offset).rgb;
        let sample_coc = textureSample(coc_texture, tex_sampler, in.uv + offset).r;
        let w = smoothstep(0.0, 1.0, sample_coc);
        color += sample_color * w;
        total_weight += w;
    }

    return vec4<f32>(color / total_weight, 1.0);
}

// --- Pass 3: Composite sharp + blurred by CoC ---
@group(0) @binding(5) var blurred_texture: texture_2d<f32>;

@fragment
fn composite_fs(in: VertexOutput) -> @location(0) vec4<f32> {
    let sharp = textureSample(src_texture, tex_sampler, in.uv).rgb;
    let blurred = textureSample(blurred_texture, tex_sampler, in.uv).rgb;
    let coc = textureSample(coc_texture, tex_sampler, in.uv).r;
    let result = mix(sharp, blurred, smoothstep(0.1, 0.5, coc));
    return vec4<f32>(result, 1.0);
}
