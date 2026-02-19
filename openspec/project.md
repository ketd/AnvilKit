# Project Context

## Purpose
**AnvilKit** 是一个使用 Rust 构建的模块化游戏基础设施框架，旨在提供统一的、数据驱动的游戏开发工具集，支持 2D 和 3D 游戏开发，具有高性能、可组合性和优秀的开发者体验。

核心设计理念：
- **统一但非均一架构** — 提供统一的顶层 API 处理 2D/3D，底层各自优化
- **模块化设计** — 通过 Cargo feature flags 实现按需编译
- **生态集成** — 基于 bevy_ecs, wgpu, rapier 等顶级库构建
- **开发者体验** — 追求 API 简洁性、清晰错误信息、快速编译

GitHub: https://github.com/ketd/AnvilKit

## Tech Stack
- **Language**: Rust (Edition 2021)
- **ECS**: `bevy_ecs 0.14` — 高性能实体组件系统
- **Rendering**: `wgpu 0.19` — 现代跨平台图形 API (Vulkan/Metal/D3D12/WebGL)
- **Windowing**: `winit 0.30` — 跨平台窗口管理
- **Math**: `glam` — 游戏优化的 SIMD 数学库
- **Physics (planned)**: `rapier2d` / `rapier3d` — 高性能物理引擎
- **Audio (planned)**: `kira` — 游戏音频引擎
- **3D Models (planned)**: `gltf` — glTF 2.0 格式支持
- **Error Handling**: `thiserror`
- **Serialization**: `serde` (optional)
- **Shaders**: WGSL (WebGPU Shading Language)

## Project Conventions

### Code Style
- 所有公共 API 使用中文文档注释
- 每个公共类型和函数都必须有文档和使用示例
- 零错误零警告标准 — 代码必须通过 `cargo check` 无警告
- 使用 `rustfmt` 和 `clippy` 保持代码质量
- 深度实现标准 — 避免简化结构，提供完整功能实现

### Architecture Patterns
- **ECS 数据驱动架构** — 基于 Bevy ECS 的 Component/System/Resource 模式
- **Plugin 模块化** — 每个子系统通过 Plugin trait 集成
- **Builder 模式** — 复杂配置使用 fluent builder API
- **分层模块化** — 按功能拆分 crate：core → ecs → render/physics/audio
- **Workspace 结构** — Cargo workspace 管理多个 crate

### Workspace Structure
```
anvilkit/
├── Cargo.toml              # Workspace 配置
├── crates/
│   ├── anvilkit-core/      # 核心类型、数学、时间系统
│   ├── anvilkit-ecs/       # Bevy ECS 封装和扩展
│   ├── anvilkit-render/    # wgpu 渲染引擎 (2D/3D)
│   ├── anvilkit-physics/   # (planned) Rapier 物理引擎集成
│   ├── anvilkit-assets/    # (planned) 资源系统
│   ├── anvilkit-audio/     # (planned) Kira 音频引擎集成
│   ├── anvilkit-input/     # (planned) 跨平台输入系统
│   └── anvilkit/           # (planned) 主 crate 和插件系统
├── examples/               # 示例项目
└── docs/                   # 文档和教程
```

### Testing Strategy
- 每个模块需要完整的单元测试覆盖
- 每个公共 API 需要文档测试 (doc tests)
- 性能基准测试放在 `benches/` 目录
- 集成测试验证跨模块功能

### Git Workflow
- 主分支: `master`
- 提交信息格式: `type(scope): description` (中文描述)
  - 例: `feat(anvilkit-core): 完成核心模块深度架构实现`
- License: MIT OR Apache-2.0

## Domain Context
AnvilKit 是一个游戏引擎基础设施层，介于底层图形/物理库和完整游戏引擎之间。
关键领域概念：
- **ECS (Entity Component System)**: 数据驱动的游戏对象架构
- **Transform Hierarchy**: 父子变换传播系统
- **Render Pipeline**: GPU 渲染管线 (顶点处理 → 片元着色 → 输出合并)
- **PBR (Physically Based Rendering)**: 基于物理的渲染材质系统

## Important Constraints
- 目标平台: Desktop (Windows/macOS/Linux)，未来扩展到 Web (WASM) 和移动端
- 性能目标: >1M entities/frame, 60FPS @ 1080p, <100MB 基础内存占用
- bevy_ecs 版本锁定在 0.14，API 需与之兼容
- wgpu 0.19 / winit 0.30 版本需确保 API 兼容性

## External Dependencies
- Bevy ECS: https://bevyengine.org/
- wgpu: https://wgpu.rs/
- winit: https://github.com/rust-windowing/winit
- glam: https://github.com/bitshifter/glam-rs
- Rapier Physics: https://rapier.rs/ (planned)
- Kira Audio: https://github.com/tesselode/kira (planned)

## Milestone Roadmap

### 已完成

- **M0**: 项目初始化和规划 — **已完成**
- **M1**: 核心地基 (ECS + 数学/时间) — **已完成**
- **M2**: 渲染系统 (wgpu 渲染管线 + 窗口管理) — **已完成**
- **M3**: 3D 渲染验证 ("你好，三角形！") — **已完成**
- **M3.5**: 3D 渲染深化 ("旋转的立方体" — Uniform/深度/索引绘制) — **已完成**
- **M4a**: glTF 网格加载 + 法线可视化 — **已完成**
- **M4b**: 纹理系统 + 基础色贴图 — **已完成**
- **M4c**: Blinn-Phong 光照 — **已完成**
- **M4d**: Cook-Torrance 直接光照 PBR — **已完成**
- **M5**: ECS 多物体渲染架构 (RenderAssets + DrawCommandList + 自动提取) — **已完成**
- **M6a**: ECS PBR 统一 + Legacy 清理 (PbrSceneUniform 256B, SceneLights, MaterialParams; 删除 RenderContext/legacy 组件/6 个旧示例) — **已完成**
- **M6b**: 法线贴图 — PbrVertex (48B, tangent), TBN 矩阵, normal map 采样, glTF tangent 提取, create_texture_linear — **已完成**
- **M6c**: HDR 渲染管线 — Rgba16Float offscreen RT, ACES Filmic tone mapping, multi-pass 渲染 (scene pass → post-process pass) — **已完成**

- **M6d**: IBL 环境光 — BRDF LUT (importance sampling GGX), hemisphere irradiance, split-sum specular, 3-group pipeline — **已完成**

### Phase B: 生产渲染 (M7) — 能渲染"像样的"游戏场景

- **M7a**: 多光源 — 点光源 + 聚光灯, GpuLight[8] uniform 数组, shader 光源循环, 距离衰减 + 锥形衰减 — **已完成**
- **M7b**: 阴影系统 — 方向光 shadow pass (depth-only pipeline) + shadow map 2048x2048 + PCF 3x3 软阴影 + comparison sampler — **已完成**
- **M7c**: 完整材质系统 — MR 纹理 (G=roughness B=metallic), AO 贴图, Emissive 纹理+因子, 6-binding material group, glTF 全属性提取 — **已完成**
- **M7d**: 抗锯齿 — MSAA 4x (HDR scene pass: MSAA color + resolve target, MSAA depth, pipeline multisample_count=4) — **已完成**

### Phase C: 渲染性能 (M8) — 能处理 100+ 物体的场景

- **M8a**: Frustum Culling — AABB 组件, Frustum (Gribb/Hartmann 6 平面提取), render_extract_system 自动剔除不可见物体 — **已完成**
- **M8b**: GPU Instancing + 批处理 — DrawCommandList.sort_for_batching(), InstanceData (128B), MeshHandle/MaterialHandle.index(), 自动按 material→mesh 排序减少状态切换 — **已完成**
- **M8c**: 多 Submesh — Submesh + MultiMeshScene, load_gltf_scene_multi() 遍历所有 mesh/primitive, 每个 primitive 独立材质 — **已完成**

### Phase D: 场景基础设施 (M9) — 游戏级别的内容管理

- **M9a**: 场景图 — Parent/Children/GlobalTransform, TransformPlugin 自动传播, TransformHierarchy 工具, serde 序列化 — **已完成** (在 ECS 模块中实现)
- **M9b**: 资产管线 — AssetServer (load/dedup/state), AssetHandle<T> (Arc refcount), AssetStorage<T>, LoadState 追踪 — **已完成**
- **M9c**: 2D 渲染栈 — Sprite 组件, SpriteVertex (32B), TextureAtlas (grid/rect), SpriteBatch (z-order sort), AtlasRect UV — **已完成**

### Phase E: 游戏系统 (M10) — 交互式游戏所需的核心系统

- **M10a**: 输入系统 — InputState (key/mouse pressed/just_pressed/released), ActionMap (逻辑动作→物理绑定), KeyCode/MouseButton 枚举 — **已完成**
- **M10b**: 物理集成 — RigidBody/Collider/Velocity 组件, RigidBodyType, ColliderShape (Sphere/Cuboid/Capsule/TriMesh), 无运行时依赖 — **已完成**
- **M10c**: 音频集成 — AudioSource/AudioListener 组件, PlaybackState, spatial audio 参数, 无运行时依赖 — **已完成**

### Phase F: 高级功能 (M11) — 丰富游戏表现力

- **M11a**: 骨骼动画 — Skeleton/Joint/SkinData, AnimationClip/Channel/Keyframe (线性/阶梯/三次插值), AnimationPlayer (循环/速度控制) — **已完成**
- **M11b**: UI 系统 — UiNode 组件, UiStyle (Flexbox 属性), UiText, Val (Px/Percent/Auto), Align/FlexDirection — **已完成**
- **M11c**: 粒子系统 — ParticleEmitter 组件, Particle 生命周期, ParticleSystem (池化/回收), EmitShape (Point/Sphere/Cone/Box) — **已完成**

### Phase G: 开发者体验 (M12) — 完善工具链和文档

- **M12a**: 调试工具 — wireframe 模式, 光照/法线调试可视化, bounds/collider 显示
- **M12b**: 性能分析 — GPU timing queries, draw call/三角形统计, frame graph 可视化
- **M12c**: 文档和教程 — 完整 API 文档, 入门教程系列, 示例项目集 (Pong/Platformer/3D Scene)
