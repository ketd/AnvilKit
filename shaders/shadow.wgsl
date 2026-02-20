// AnvilKit 阴影 Pass 着色器 (depth-only)
// 仅需要 model + view_proj，无片段着色器输出

struct SceneUniform {
    model: mat4x4<f32>,
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> scene: SceneUniform;

@vertex
fn vs_main(@location(0) position: vec3<f32>) -> @builtin(position) vec4<f32> {
    return scene.view_proj * scene.model * vec4<f32>(position, 1.0);
}
