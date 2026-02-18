## Context
AnvilKit 已具备基础渲染能力（窗口、GPU 管线、纹理、Blinn-Phong 光照），需要升级到物理正确的 PBR 渲染。目标是达到与 Khronos glTF Sample Viewer 相当的渲染质量。

## Goals / Non-Goals
- Goals: Cook-Torrance BRDF、HDR 管线、IBL 环境光照、法线贴图、完整 glTF PBR 材质
- Non-Goals: 全局光照 (GI)、实时阴影、骨骼动画、后处理特效（Bloom/SSR/SSAO）

## Decisions

### Cook-Torrance 实现
- **NDF**: GGX/Trowbridge-Reitz — 业界标准，和 glTF spec 一致
- **Fresnel**: Schlick 近似 — 精度足够，性能好
- **Geometry**: Smith GGX (height-correlated) — Epic Games UE4 方案
- **能量守恒**: diffuse 项乘以 `(1 - F) * (1 - metallic)` 确保能量守恒

### HDR 管线方案
- 离屏渲染到 `Rgba16Float` 纹理（半精度浮点，精度和带宽平衡）
- 全屏三角形后处理（不用全屏四边形，避免对角线 artifact）
- ACES Filmic tone mapping（比 Reinhard 更好的高光压缩和色彩保真）
- 在后处理 pass 中做 sRGB 转换（`pow(color, 1/2.2)` 或硬件 sRGB）

### IBL 方案
- Equirectangular HDR → Cubemap：GPU render 6 面
- Irradiance map：从 cubemap 卷积，32x32 per face 即可
- Prefiltered specular：5 个 mip level (roughness 0.0→1.0)，128x128 per face
- BRDF LUT：512x512 预计算 2D 纹理（可内嵌或运行时生成）

### 法线贴图方案
- 从 glTF TANGENT 属性读取（如有）
- 无 tangent 时使用 MikkTSpace 算法计算（mikktspace crate）
- TBN 矩阵在顶点着色器中构建，传递到片元着色器
- 法线贴图采样后在切线空间转世界空间

## Risks / Trade-offs
- IBL 预计算需要 render-to-texture 能力 — 当前 RenderApp 只支持渲染到交换链
  → 需要抽象离屏渲染目标
- HDR 多 pass 渲染增加复杂性 — 帧缓冲管理、纹理绑定
  → M4e 是架构跳跃最大的阶段，建议重点设计
- MikkTSpace 引入外部依赖
  → 可先只支持 glTF 内置 tangent，后续补 MikkTSpace
- 没有 IBL 的 PBR 金属表面会全黑
  → M4d 先用强方向光验证 BRDF 正确性，M4f 补 IBL 后效果完整

## Open Questions
- 是否需要 compute shader 做 IBL 预计算？（替代方案：多 pass fragment shader）
- BRDF LUT 是预计算嵌入还是运行时生成？
- 是否支持多个 HDR 环境贴图切换（天空盒轮换）？
