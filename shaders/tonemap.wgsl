// AnvilKit Tone Mapping 后处理着色器
// 全屏三角形 + ACES Filmic + Gamma 校正

@group(0) @binding(0) var hdr_texture: texture_2d<f32>;
@group(0) @binding(1) var hdr_sampler: sampler;

struct VertexOutput { @builtin(position) position: vec4<f32>, @location(0) texcoord: vec2<f32> };

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
    return clamp((x * (2.51 * x + 0.03)) / (x * (2.43 * x + 0.59) + 0.14), vec3<f32>(0.0), vec3<f32>(1.0));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var c = textureSample(hdr_texture, hdr_sampler, in.texcoord).rgb;
    c = aces_filmic(c);
    c = pow(c, vec3<f32>(1.0 / 2.2));
    return vec4<f32>(c, 1.0);
}
