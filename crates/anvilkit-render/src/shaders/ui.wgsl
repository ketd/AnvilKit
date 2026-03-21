// UI rectangle shader — colored rectangles with rounded corners (SDF)

struct OrthoUniform {
    projection: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> ortho: OrthoUniform;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) rect_min: vec2<f32>,
    @location(2) rect_size: vec2<f32>,
    @location(3) color: vec4<f32>,
    @location(4) border_color: vec4<f32>,
    @location(5) params: vec4<f32>,  // border_radius, border_width, 0, 0
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) local_pos: vec2<f32>,
    @location(1) rect_size: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) border_color: vec4<f32>,
    @location(4) params: vec4<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let world_pos = in.rect_min + in.position * in.rect_size;
    out.clip_position = ortho.projection * vec4<f32>(world_pos, 0.0, 1.0);
    out.local_pos = in.position * in.rect_size;
    out.rect_size = in.rect_size;
    out.color = in.color;
    out.border_color = in.border_color;
    out.params = in.params;
    return out;
}

// Rounded rectangle SDF
fn rounded_rect_sdf(pos: vec2<f32>, size: vec2<f32>, radius: f32) -> f32 {
    let half = size * 0.5;
    let r = min(radius, min(half.x, half.y));
    let d = abs(pos - half) - half + vec2<f32>(r, r);
    return length(max(d, vec2<f32>(0.0, 0.0))) + min(max(d.x, d.y), 0.0) - r;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let radius = in.params.x;
    let border_width = in.params.y;

    let dist = rounded_rect_sdf(in.local_pos, in.rect_size, radius);

    // Smooth edge antialiasing
    let aa = fwidth(dist);
    let alpha = 1.0 - smoothstep(-aa, aa, dist);

    if alpha < 0.001 {
        discard;
    }

    // Border
    if border_width > 0.0 {
        let inner_offset = border_width;
        let inner = rounded_rect_sdf(
            in.local_pos - vec2<f32>(inner_offset, inner_offset),
            in.rect_size - vec2<f32>(inner_offset * 2.0, inner_offset * 2.0),
            max(radius - border_width, 0.0),
        );
        let border_mask = smoothstep(-aa, aa, -inner);
        let fill_color = mix(in.color, in.border_color, border_mask);
        return vec4<f32>(fill_color.rgb, fill_color.a * alpha);
    }

    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}
