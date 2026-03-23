// Craft Tone Mapping + Post-Processing Filters
// Extends the base AnvilKit tonemap with post_fx effects.

@group(0) @binding(0) var hdr_texture: texture_2d<f32>;
@group(0) @binding(1) var hdr_sampler: sampler;

struct FilterUniform {
    filter_type: u32,    // 0=none, 1=underwater, 2=vignette, 3=nightvision
    intensity: f32,
    time: f32,
    apply_gamma: f32,    // 1.0=apply manual gamma (linear swapchain), 0.0=skip (sRGB swapchain)
};

@group(0) @binding(2) var<uniform> post_fx: FilterUniform;
@group(0) @binding(3) var bloom_texture: texture_2d<f32>;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) texcoord: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(i32(vi & 1u) * 4 - 1);
    let y = f32(i32(vi & 2u) * 2 - 1);
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.texcoord = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);
    return out;
}

fn aces_filmic(x: vec3<f32>) -> vec3<f32> {
    return clamp(
        (x * (2.51 * x + 0.03)) / (x * (2.43 * x + 0.59) + 0.14),
        vec3<f32>(0.0),
        vec3<f32>(1.0)
    );
}

// --- Vignette ---
fn vignette(c: vec3<f32>, uv: vec2<f32>, strength: f32) -> vec3<f32> {
    let center = vec2<f32>(0.5, 0.5);
    let d = distance(uv, center);
    let v = 1.0 - smoothstep(0.4, 0.8, d) * strength;
    return c * v;
}

// --- Underwater post_fx ---
fn underwater_post_fx(c: vec3<f32>, uv: vec2<f32>, t: f32) -> vec3<f32> {
    // UV distortion (sinusoidal ripple), clamped to [0,1] to prevent OOB sampling
    let distort_uv = clamp(
        uv + vec2<f32>(
            sin(uv.y * 20.0 + t * 3.0) * 0.003,
            cos(uv.x * 15.0 + t * 2.5) * 0.003
        ),
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 1.0)
    );
    // Re-sample with distorted UV
    let distorted = textureSample(hdr_texture, hdr_sampler, distort_uv).rgb;

    // Blue-green tint
    let tinted = distorted * vec3<f32>(0.5, 0.8, 0.9);

    // Caustic highlight (animated bright spots)
    let caustic_uv = uv * 8.0 + vec2<f32>(t * 0.5, t * 0.3);
    let caustic = pow(
        abs(sin(caustic_uv.x * 3.14) * sin(caustic_uv.y * 3.14)),
        4.0
    ) * 0.15;
    let with_caustic = tinted + vec3<f32>(caustic * 0.3, caustic * 0.6, caustic * 0.5);

    // Vignette (stronger for underwater)
    return vignette(with_caustic, uv, 0.8);
}

// --- Night vision post_fx ---
fn nightvision_post_fx(c: vec3<f32>, uv: vec2<f32>, t: f32) -> vec3<f32> {
    // Convert to luminance
    let lum = dot(c, vec3<f32>(0.299, 0.587, 0.114));

    // Amplify brightness
    let amplified = pow(lum, 0.7) * 2.0;

    // Green channel dominant
    let green = vec3<f32>(amplified * 0.2, amplified, amplified * 0.2);

    // Scanline effect
    let scanline = 1.0 - abs(sin(uv.y * 400.0)) * 0.08;

    // Simple noise grain (using screen position + time)
    let noise_seed = uv * 300.0 + vec2<f32>(t * 100.0, t * 73.0);
    let noise = fract(sin(dot(noise_seed, vec2<f32>(12.9898, 78.233))) * 43758.5453) * 0.08;

    let noisy = green * scanline + vec3<f32>(noise * 0.05, noise * 0.1, noise * 0.02);

    // Vignette
    return vignette(noisy, uv, 0.6);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var uv = in.texcoord;
    var c = textureSample(hdr_texture, hdr_sampler, uv).rgb;

    // Bloom composite (in HDR space, before effects and tone mapping)
    let bloom = textureSample(bloom_texture, hdr_sampler, uv).rgb;
    c += bloom;

    // Apply post_fx effects (in HDR space, before tone mapping)
    if (post_fx.filter_type == 1u) {
        c = underwater_post_fx(c, uv, post_fx.time);
    }
    if (post_fx.filter_type == 2u) {
        c = vignette(c, uv, post_fx.intensity);
    }
    if (post_fx.filter_type == 3u) {
        c = nightvision_post_fx(c, uv, post_fx.time);
    }

    // Tone mapping
    c = aces_filmic(c);
    // Gamma correction (skip if swapchain is sRGB — GPU handles it)
    if (post_fx.apply_gamma > 0.5) {
        c = pow(c, vec3<f32>(1.0 / 2.2));
    }
    return vec4<f32>(c, 1.0);
}
