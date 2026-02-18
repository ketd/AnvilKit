# Change: Add PBR Rendering (M4d → M4g)

## Why
M4c 实现了 Blinn-Phong 光照，但这是经验模型，不符合物理规律。现代游戏引擎标配 PBR (Physically Based Rendering)，包括 Cook-Torrance BRDF、IBL、HDR 管线和法线贴图。AnvilKit 需要完整 PBR 能力来达到产品级渲染质量。

## What Changes — 分 4 个阶段

### M4d: Cook-Torrance 直接光照
- Cook-Torrance BRDF (GGX NDF + Schlick Fresnel + Smith GGX Geometry)
- metallic/roughness 参数从 glTF 材质提取
- 替换 Blinn-Phong 着色器
- `examples/hello_pbr.rs`

### M4e: HDR 管线 + Tone Mapping
- 浮点渲染目标 (Rgba16Float)
- 全屏后处理 pass (blit shader)
- ACES Filmic tone mapping
- Gamma 校正
- `RenderApp` 支持多 pass 渲染

### M4f: IBL (基于图像的光照)
- 立方体纹理加载 (HDR equirectangular → cubemap)
- 漫反射辐照度图 (irradiance convolution)
- 镜面预滤波环境图 (split-sum prefilter)
- BRDF LUT 预计算
- 环境光 = 漫反射 IBL + 镜面 IBL

### M4g: 法线贴图 + 完整材质
- 切线空间计算 (tangent/bitangent)
- TBN 矩阵传递到片元着色器
- 法线贴图采样和扰动
- AO 贴图 (可选)
- Emissive 贴图 (可选)
- 多光源支持

## Impact
- Affected specs: `render-system` (大量新增渲染能力), `asset-system` (HDR/cubemap 加载)
- Affected code: `crates/anvilkit-render/` (buffer.rs, pipeline.rs, events.rs 新增多 pass), `crates/anvilkit-assets/` (HDR/cubemap loader)
- New files: 4 个新示例, HDR/cubemap 辅助模块

## Architecture Notes
- M4d 只改着色器 + 材质参数，不需要新基础设施
- M4e 需要离屏渲染目标和后处理 pass — 是架构跳跃最大的阶段
- M4f 需要立方体纹理和 GPU 预计算 — 计算着色器或多 pass 预处理
- M4g 需要扩展顶点格式 — 加 tangent 属性
- 每个阶段可独立验证，前一阶段是后一阶段的前置条件
