// Sprite shader — orthographic projection + texture sampling

struct OrthoUniform {
    projection: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> ortho: OrthoUniform;

@group(1) @binding(0)
var sprite_texture: texture_2d<f32>;
@group(1) @binding(1)
var sprite_sampler: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) texcoord: vec2<f32>,
    @location(2) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) tint: vec3<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = ortho.projection * vec4<f32>(in.position, 1.0);
    out.uv = in.texcoord;
    out.tint = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(sprite_texture, sprite_sampler, in.uv);
    return vec4<f32>(tex_color.rgb * in.tint, tex_color.a);
}
