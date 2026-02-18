# Change: Add Blinn-Phong Lighting (M4c)

## Why
M4b 实现了纹理采样，但着色器只有简单的 `ndotl` 漫反射。模型缺少高光和环境光遮蔽，看起来不立体。M4c 添加 Blinn-Phong 光照模型，让模型在方向光下有真实的漫反射和高光效果。

## What Changes
- 新增光照 Uniform 结构体（光源方向、颜色、环境光强度）
- 扩展 MVP Uniform 为包含 model/view/projection 分离矩阵和法线矩阵
- 更新 WGSL 着色器实现 Blinn-Phong 光照（环境光 + 漫反射 + 高光）
- 新增 `examples/hello_lit.rs` — 带光照的纹理模型

## Impact
- Affected specs: `render-system`（新增 Lighting Uniform 能力）
- Affected code: `examples/` 新增 hello_lit.rs
- 不修改 anvilkit-render 库代码 — 光照 Uniform 在示例层面实现，用现有 bind group 能力
