// Minimal egui shader — textured triangles with sRGB vertex colors.

struct Uniforms {
    screen_size: vec2<f32>,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(1) @binding(0) var t_texture: texture_2d<f32>;
@group(1) @binding(1) var s_sampler: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
};

// sRGB to linear conversion for a single channel
fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        return c / 12.92;
    } else {
        return pow((c + 0.055) / 1.055, 2.4);
    }
}

@vertex
fn vs_main(
    @location(0) pos: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    // Convert from egui screen coordinates to clip space
    out.position = vec4<f32>(
        2.0 * pos.x / uniforms.screen_size.x - 1.0,
        1.0 - 2.0 * pos.y / uniforms.screen_size.y,
        0.0,
        1.0,
    );
    out.uv = uv;
    // egui vertex colors are sRGB, convert to linear for blending
    out.color = vec4<f32>(
        srgb_to_linear(color.r),
        srgb_to_linear(color.g),
        srgb_to_linear(color.b),
        color.a,
    );
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex = textureSample(t_texture, s_sampler, in.uv);
    // egui uses premultiplied alpha
    return in.color * tex;
}
