# Change: Add Texture System (M4b)

## Why
M4a 验证了 glTF 网格加载和法线着色渲染。但模型缺少纹理，看起来不真实。M4b 添加纹理加载和采样能力，从 glTF 提取基础色贴图并应用到模型上。

## What Changes
- `anvilkit-assets`: 从 glTF 提取纹理图像数据，新增 `TextureData` 结构体和 `MaterialData`
- `anvilkit-render/buffer.rs`: 新增 `create_texture()` 和 `create_sampler()` GPU 资源辅助函数
- `RenderApp`: 支持第二个 bind group（材质纹理 + 采样器）
- 更新 WGSL 着色器支持纹理采样
- 新增 `examples/hello_textured.rs` — 带纹理的旋转模型

## Impact
- Affected specs: `render-system` (新增纹理创建、采样器、多 bind group), `asset-system` (新增纹理/材质提取)
- Affected code: `crates/anvilkit-assets/` (gltf_loader, 新增 texture/material), `crates/anvilkit-render/` (buffer.rs, events.rs)
- New files: `examples/hello_textured.rs`
