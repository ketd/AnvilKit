// Voxel rendering shader for Craft
// Group 0: scene uniform (view_proj, camera_pos, light_dir, fog, time)
// Group 1: atlas texture + sampler (NEAREST)
//
// UV encoding:
//   Block faces: uv = (tile_index, -1.0) — shader computes atlas UV from world_pos
//   Plants:      uv = (atlas_u, atlas_v)  — direct atlas coordinates (uv.y >= 0)

struct VoxelSceneUniform {
    view_proj: mat4x4<f32>,
    camera_pos: vec4<f32>,
    light_dir: vec4<f32>,   // xyz = direction (toward light), w = unused
    fog_color: vec4<f32>,   // rgb = fog color, a = fog density
    time_ambient: vec4<f32>, // x = time, y = ambient strength, z = fog_start, w = fog_end
};

@group(0) @binding(0) var<uniform> scene: VoxelSceneUniform;

@group(1) @binding(0) var atlas: texture_2d<f32>;
@group(1) @binding(1) var samp: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) ao: f32,
    @location(4) light: f32,  // packed: sky*16 + block
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) ao: f32,
    @location(3) world_pos: vec3<f32>,
    @location(4) light: f32,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = scene.view_proj * vec4<f32>(in.position, 1.0);
    out.uv = in.uv;
    out.normal = in.normal;
    out.ao = in.ao;
    out.light = in.light;
    out.world_pos = in.position;
    return out;
}

// Water vertex shader — displaces Y with multi-layer sine waves (top faces only)
@vertex
fn vs_water(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let t = scene.time_ambient.x * 600.0;
    let p = in.position;

    // Only displace top faces (normal pointing up); side/bottom faces stay flat
    var dy = 0.0;
    if (in.normal.y > 0.5) {
        let wave1 = sin(p.x * 0.8 + t * 1.2) * cos(p.z * 0.6 + t * 0.9) * 0.08;
        let wave2 = sin(p.x * 1.5 - t * 0.7 + p.z * 1.2) * 0.05;
        let wave3 = cos(p.z * 2.0 + t * 1.6 + p.x * 0.5) * 0.03;
        dy = wave1 + wave2 + wave3;
    }

    let displaced = vec3<f32>(p.x, p.y + dy, p.z);
    out.clip_position = scene.view_proj * vec4<f32>(displaced, 1.0);
    out.uv = in.uv;
    out.normal = in.normal;
    out.ao = in.ao;
    out.light = in.light;
    out.world_pos = displaced;
    return out;
}

// Compute atlas UV for a block face from tile index and world position.
// Each block gets its own correctly-tiled UV, even across greedy-merged quads.
fn block_atlas_uv(tile_idx: u32, normal: vec3<f32>, wp: vec3<f32>) -> vec2<f32> {
    // Atlas layout: 32 columns x 16 rows (512x256 pixels)
    let tile_col = f32(tile_idx % 32u);
    let tile_row = f32(tile_idx / 32u);
    let tile_base = vec2<f32>(tile_col / 32.0, tile_row / 16.0);
    let tile_size_x = 1.0 / 32.0;
    let tile_size_y = 1.0 / 16.0;
    let inset = 1.0 / 4096.0;

    // Compute per-block UV from world position based on face normal.
    // Matches the UV orientation of the original per-block emit_face.
    var local_uv: vec2<f32>;
    let n = normal;

    if (abs(n.y) > 0.5) {
        // Top (+Y) or Bottom (-Y): texture axes X, Z
        local_uv = fract(wp.xz);
    } else if (abs(n.x) > 0.5) {
        // Right (+X) or Left (-X): texture axes Z, Y
        if (n.x > 0.0) {
            local_uv = vec2<f32>(1.0 - fract(wp.z), 1.0 - fract(wp.y));
        } else {
            local_uv = vec2<f32>(fract(wp.z), 1.0 - fract(wp.y));
        }
    } else {
        // Front (+Z) or Back (-Z): texture axes X, Y
        if (n.z > 0.0) {
            local_uv = vec2<f32>(fract(wp.x), 1.0 - fract(wp.y));
        } else {
            local_uv = vec2<f32>(1.0 - fract(wp.x), 1.0 - fract(wp.y));
        }
    }

    return tile_base + vec2<f32>(inset, inset) + local_uv * vec2<f32>(tile_size_x - 2.0 * inset, tile_size_y - 2.0 * inset);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Compute atlas UV: block faces encode tile index in uv.x with sentinel uv.y=-1,
    // plants pass direct atlas coordinates. UV is computed conditionally but
    // textureSample is called unconditionally for WGSL uniform control flow.
    var sample_uv: vec2<f32>;
    if (in.uv.y < 0.0) {
        let tile_idx = u32(in.uv.x + 0.5);
        sample_uv = block_atlas_uv(tile_idx, in.normal, in.world_pos);
    } else {
        sample_uv = in.uv;
    }
    let albedo = textureSample(atlas, samp, sample_uv);

    // Alpha test for plants/transparent blocks
    if (albedo.a < 0.5) {
        discard;
    }

    // Directional light (simple NdotL)
    let light_dir = normalize(scene.light_dir.xyz);
    let n = normalize(in.normal);
    let ndotl = max(dot(n, light_dir), 0.0);

    let ambient = scene.time_ambient.y;
    let diffuse = ndotl * (1.0 - ambient);
    let sun_light = ambient + diffuse;

    // Block lighting: unpack sky*16+block
    let sky_light = floor(in.light / 16.0) / 15.0;
    let block_light = (in.light % 16.0) / 15.0;
    // Day factor from sun height (ambient proxy)
    let day_factor = clamp(ambient / 0.4, 0.0, 1.0);
    let block_lighting = max(sky_light * day_factor, block_light);

    // Combine sun + block lighting.
    // Minimum floor (0.06) ensures even unlit areas show faint geometry —
    // pitch-black voids feel broken; torches still matter since 0.06 is very dim.
    let light = max(sun_light * block_lighting, 0.06);

    // AO darkening: remap [0,1] → [0.15,1] so fully occluded corners are
    // very dark but never pure black.
    let ao = 0.15 + 0.85 * in.ao;

    // Apply lighting
    var color = albedo.rgb * light * ao;

    // Distance fog
    let dist = length(in.world_pos - scene.camera_pos.xyz);
    let fog_start = scene.time_ambient.z;
    let fog_end = scene.time_ambient.w;
    let fog_factor = clamp((dist - fog_start) / (fog_end - fog_start), 0.0, 1.0);
    color = mix(color, scene.fog_color.rgb, fog_factor);

    return vec4<f32>(color, 1.0);
}

// ---------------------------------------------------------------------------
// Water shader — animated surface with Fresnel, specular, and caustics
// ---------------------------------------------------------------------------

// Hash-based pseudo-noise for caustic pattern (no texture needed)
fn hash2(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3<f32>(p.x, p.y, p.x) * 0.1031);
    p3 = p3 + vec3<f32>(dot(p3, vec3<f32>(p3.y + 33.33, p3.z + 33.33, p3.x + 33.33)));
    return fract((p3.x + p3.y) * p3.z);
}

// Smooth value noise
fn vnoise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f); // smoothstep

    let a = hash2(i);
    let b = hash2(i + vec2<f32>(1.0, 0.0));
    let c = hash2(i + vec2<f32>(0.0, 1.0));
    let d = hash2(i + vec2<f32>(1.0, 1.0));

    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

// Fractional Brownian Motion — 3 octaves of animated noise
fn fbm_water(p: vec2<f32>, t: f32) -> f32 {
    var val = 0.0;
    var amp = 0.5;
    var pos = p;
    // Octave 1: large slow waves
    val += amp * vnoise(pos * 0.8 + vec2<f32>(t * 0.4, t * 0.3));
    amp *= 0.5; pos = pos * 2.1 + vec2<f32>(1.7, 3.2);
    // Octave 2: medium ripples
    val += amp * vnoise(pos * 1.0 + vec2<f32>(-t * 0.6, t * 0.5));
    amp *= 0.5; pos = pos * 2.0 + vec2<f32>(5.1, 1.3);
    // Octave 3: fine detail
    val += amp * vnoise(pos * 1.2 + vec2<f32>(t * 0.8, -t * 0.4));
    return val;
}

@fragment
fn fs_water(in: VertexOutput) -> @location(0) vec4<f32> {
    let t = scene.time_ambient.x * 600.0; // convert normalized cycle time to seconds
    let wp = in.world_pos;
    let n = normalize(in.normal);
    let is_top = n.y > 0.5;

    // --- View direction ---
    let view_dir = normalize(scene.camera_pos.xyz - wp);
    let light_dir = normalize(scene.light_dir.xyz);
    let ambient = scene.time_ambient.y;

    // --- Shading normal: animated waves for top faces, vertex normal for sides ---
    var shade_normal: vec3<f32>;
    var water_base: vec3<f32>;
    var fresnel: f32;

    if (is_top) {
        // Animated surface normal via height-field gradient
        let eps = 0.15;
        let h_c = fbm_water(wp.xz, t);
        let h_r = fbm_water(wp.xz + vec2<f32>(eps, 0.0), t);
        let h_u = fbm_water(wp.xz + vec2<f32>(0.0, eps), t);
        let wave_strength = 0.35;
        let ddx = (h_r - h_c) / eps * wave_strength;
        let ddz = (h_u - h_c) / eps * wave_strength;
        shade_normal = normalize(vec3<f32>(-ddx, 1.0, -ddz));

        // Deep vs shallow color
        let deep_color  = vec3<f32>(0.05, 0.15, 0.35);
        let shallow_color = vec3<f32>(0.15, 0.45, 0.55);
        let depth_factor = clamp(h_c * 1.5, 0.0, 1.0);
        water_base = mix(deep_color, shallow_color, depth_factor);

        // Fresnel
        let fresnel_base = 0.02;
        let cos_theta = max(dot(shade_normal, view_dir), 0.0);
        fresnel = fresnel_base + (1.0 - fresnel_base) * pow(1.0 - cos_theta, 5.0);
    } else {
        // Side/bottom faces: use vertex normal, darker underwater color
        shade_normal = n;
        water_base = vec3<f32>(0.04, 0.10, 0.28); // darker for underwater sides
        fresnel = 0.02;
    }

    // --- Directional lighting ---
    let ndotl = max(dot(shade_normal, light_dir), 0.0);
    let diffuse = ndotl * (1.0 - ambient);
    let light = ambient + diffuse;

    var color = water_base * light;

    // --- Sun specular (top faces only) ---
    if (is_top) {
        let half_vec = normalize(light_dir + view_dir);
        let spec = pow(max(dot(shade_normal, half_vec), 0.0), 128.0);
        let sun_visible = clamp(light_dir.y * 4.0, 0.0, 1.0);
        let spec_color = vec3<f32>(1.0, 0.95, 0.8) * spec * 1.2 * sun_visible;
        color += spec_color;

        // Sky reflection tint
        let sky_reflect = mix(
            vec3<f32>(0.15, 0.25, 0.40),
            scene.fog_color.rgb * 0.6,
            clamp(light_dir.y + 0.3, 0.0, 1.0)
        );
        color = mix(color, sky_reflect, fresnel * 0.6);

        // Caustic pattern
        let caustic1 = vnoise(wp.xz * 3.0 + vec2<f32>(t * 1.2, t * 0.9));
        let caustic2 = vnoise(wp.xz * 3.5 + vec2<f32>(-t * 0.8, t * 1.1));
        let caustic = pow(clamp(caustic1 * caustic2 * 2.5, 0.0, 1.0), 2.0) * 0.15;
        color += vec3<f32>(caustic) * light;
    }

    // --- Distance fog ---
    let dist = length(wp - scene.camera_pos.xyz);
    let fog_start = scene.time_ambient.z;
    let fog_end = scene.time_ambient.w;
    let fog_factor = clamp((dist - fog_start) / (fog_end - fog_start), 0.0, 1.0);
    color = mix(color, scene.fog_color.rgb, fog_factor);

    // --- Alpha: top faces use Fresnel-based alpha, sides are more opaque ---
    var alpha: f32;
    if (is_top) {
        alpha = mix(0.55, 0.85, fresnel) * (1.0 - fog_factor * 0.5);
    } else {
        alpha = 0.75 * (1.0 - fog_factor * 0.5);
    }

    return vec4<f32>(color, alpha);
}
