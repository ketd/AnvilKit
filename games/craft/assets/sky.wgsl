// Sky dome shader: fullscreen triangle with gradient sky + sun disc.

struct SkyUniform {
    inv_view_proj: mat4x4<f32>,
    sky_top: vec4<f32>,
    sky_horizon: vec4<f32>,
    sky_bottom: vec4<f32>,
    sun_dir: vec4<f32>,
};

@group(0) @binding(0) var<uniform> sky: SkyUniform;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VertexOutput {
    // Fullscreen triangle (vertices 0,1,2)
    var out: VertexOutput;
    let x = f32(i32(vi & 1u)) * 4.0 - 1.0;
    let y = f32(i32(vi >> 1u)) * 4.0 - 1.0;
    out.clip_position = vec4<f32>(x, y, 0.999, 1.0);
    out.uv = vec2<f32>(x, y);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Reconstruct world direction from screen coords
    let ndc = vec4<f32>(in.uv.x, in.uv.y, 1.0, 1.0);
    let world_pos = sky.inv_view_proj * ndc;
    let dir = normalize(world_pos.xyz / world_pos.w);

    // Vertical gradient: bottom -> horizon -> top based on y component
    let y = dir.y;
    var color: vec3<f32>;
    if y > 0.0 {
        // Above horizon: blend horizon -> top
        let t = clamp(y, 0.0, 1.0);
        let t_smooth = sqrt(t); // faster transition near horizon
        color = mix(sky.sky_horizon.xyz, sky.sky_top.xyz, t_smooth);
    } else {
        // Below horizon: blend horizon -> bottom (ground fog)
        let t = clamp(-y, 0.0, 1.0);
        let t_smooth = sqrt(t);
        color = mix(sky.sky_horizon.xyz, sky.sky_bottom.xyz, t_smooth);
    }

    // Sun disc
    let sun_dir = normalize(sky.sun_dir.xyz);
    let sun_dot = dot(dir, sun_dir);
    // Hard sun disc with soft glow
    let sun_core = pow(max(sun_dot, 0.0), 256.0);  // tight core
    let sun_glow = pow(max(sun_dot, 0.0), 32.0) * 0.15;  // soft glow
    let sun_intensity = sun_core + sun_glow;
    let sun_color = vec3<f32>(1.4, 1.3, 1.0); // warm sun
    color = color + sun_color * sun_intensity;

    return vec4<f32>(color, 1.0);
}
