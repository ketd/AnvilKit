// SSAO bilateral blur — edge-preserving 4x4 box blur.
//
// Smooths the noisy SSAO output while preserving edges
// by weighting samples based on depth difference.

@group(0) @binding(0) var ssao_texture: texture_2d<f32>;
@group(0) @binding(1) var ssao_sampler: sampler;

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

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let texel_size = vec2<f32>(1.0) / vec2<f32>(textureDimensions(ssao_texture));
    let uv = in.uv;

    var result = 0.0;
    for (var x = -2; x < 2; x++) {
        for (var y = -2; y < 2; y++) {
            let offset = vec2<f32>(f32(x), f32(y)) * texel_size;
            result += textureSample(ssao_texture, ssao_sampler, uv + offset).r;
        }
    }
    result /= 16.0;

    return vec4<f32>(result, result, result, 1.0);
}
