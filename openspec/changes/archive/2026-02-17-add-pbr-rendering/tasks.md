## M4d: Cook-Torrance 直接光照 ✅
- [x] d.1 `MaterialData` 新增 `metallic_factor`, `roughness_factor`
- [x] d.2 glTF loader 提取 metallic/roughness
- [x] d.3 SceneUniform 含 material_params
- [x] d.4 WGSL Cook-Torrance 着色器 (GGX + Schlick + Smith)
- [x] d.5 `examples/hello_pbr.rs`

## M4e-M4g: 暂缓 → 移至 M6
HDR/IBL/法线贴图暂缓，优先重构渲染架构 (M5)。
在 RenderGraph + ECS 集成完成后，在正确的架构上实现。

## Quality
- [x] q.1 `cargo check --workspace` 零错误零警告
- [x] q.2 `cargo test --workspace` 517 tests 全部通过
- [x] q.3 现有示例向后兼容
