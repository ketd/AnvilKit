// Particle point-sprite shader

struct SceneUniform {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> scene: SceneUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) size: f32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
};

// Expand each particle into a camera-facing quad via vertex_index
// Each particle is drawn as 6 vertices (2 triangles)
fn get_corner(vid: u32) -> vec2<f32> {
    let idx = vid % 6u;
    // Triangle 1: TL, BR, TR  Triangle 2: TL, BL, BR
    // 0: (-0.5, -0.5)  1: (0.5, -0.5)  2: (0.5, 0.5)
    // 3: (-0.5, -0.5)  4: (0.5, 0.5)   5: (-0.5, 0.5)
    var x: f32 = -0.5;
    var y: f32 = -0.5;
    if idx == 1u || idx == 2u || idx == 4u {
        x = 0.5;
    }
    if idx == 2u || idx == 4u || idx == 5u {
        y = 0.5;
    }
    return vec2<f32>(x, y);
}

@vertex
fn vs_main(in: VertexInput, @builtin(vertex_index) vid: u32) -> VertexOutput {
    let corner = get_corner(vid);

    var out: VertexOutput;
    // Billboard: offset in clip space after projection
    let clip = scene.view_proj * vec4<f32>(in.position, 1.0);
    let offset = vec2<f32>(corner.x * in.size, corner.y * in.size);
    out.clip_position = vec4<f32>(clip.xy + offset * clip.w, clip.z, clip.w);
    out.color = in.color;
    out.uv = corner + vec2<f32>(0.5, 0.5);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Soft circle: distance from center
    let dist = length(in.uv - vec2<f32>(0.5, 0.5)) * 2.0;
    let alpha = 1.0 - smoothstep(0.8, 1.0, dist);
    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}
