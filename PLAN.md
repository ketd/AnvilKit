# AnvilKit 开发状态

## 已完成里程碑

所有规划阶段 (M0-M12c) 均已完成。详细路线图见 `openspec/project.md`。

### Phase A: 渲染基础 (M1-M6d)
- M1: 核心地基 — ECS + 数学/时间
- M2: 渲染系统 — wgpu 管线 + 窗口管理
- M3-M3.5: 3D 渲染验证 — 三角形 → 旋转立方体
- M4a-M4d: 资源管线 — glTF 加载 + 纹理 + Blinn-Phong + PBR
- M5: ECS 多物体渲染 — RenderAssets + DrawCommandList
- M6a-M6d: 渲染深化 — 法线贴图 + HDR + IBL

### Phase B: 生产渲染 (M7)
- M7a-M7d: 多光源 + 阴影 + 完整材质 + MSAA 4x

### Phase C: 渲染性能 (M8)
- M8a-M8c: Frustum Culling + GPU Instancing + 多 Submesh

### Phase D: 场景基础设施 (M9)
- M9a-M9c: 场景图 + 资产管线 + 2D 渲染栈

### Phase E: 游戏系统 (M10)
- M10a-M10c: 输入系统 + 物理集成 + 音频集成

### Phase F: 高级功能 (M11)
- M11a-M11c: 骨骼动画 + UI 系统 + 粒子系统

### Phase G: 开发者体验 (M12)
- M12a-M12c: 调试工具 + 性能分析 + 文档/示例

## 示例游戏
- `games/craft` — Minecraft 风格体素沙盒
- `games/billiards` — 台球模拟

## 当前工作
- 引擎质量重构 (`openspec/changes/refactor-engine-quality/`)
