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
- **Physics**: `rapier3d` — 高性能 3D 物理引擎 (feature-gated)
- **Audio**: `rodio` — 跨平台音频播放
- **3D Models**: `gltf` — glTF 2.0 格式支持
- **UI Layout**: `taffy` — Flexbox 布局引擎
- **File Watching**: `notify` — 资源热重载 (feature-gated)
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
│   ├── anvilkit-core/      # 核心类型、数学、时间、持久化系统
│   ├── anvilkit-ecs/       # Bevy ECS 封装 + 物理/导航/网络/UI/场景系统
│   ├── anvilkit-render/    # wgpu 渲染引擎 (2D/3D/PBR/后处理)
│   ├── anvilkit-assets/    # 资源系统 (异步加载/热重载/缓存/依赖追踪)
│   ├── anvilkit-audio/     # 音频引擎 (rodio 集成)
│   ├── anvilkit-input/     # 跨平台输入系统 (键鼠/手柄/ActionMap)
│   ├── anvilkit-camera/    # 相机系统 (5模式/Trauma抖动/SpringArm/Rail/过渡混合)
│   ├── anvilkit-app/       # App Runner (GameCallbacks + 事件循环 + 窗口管理)
│   ├── anvilkit-ui/        # 独立 UI 框架 (布局/事件/控件/主题)
│   ├── anvilkit-gameplay/  # 游戏性系统 (属性/生命值/物品/冷却/状态效果/对象池)
│   ├── anvilkit-data/      # 数据表 + i18n 本地化
│   └── anvilkit/           # 主 crate (prelude + DefaultPlugins)
├── games/
│   ├── craft/              # Minecraft 风格体素游戏
│   └── billiards/          # PBR 台球游戏
├── examples/               # 示例项目 (game.rs, demo.rs, showcase.rs)
├── tools/anvilkit-cli/     # CLI 工具 (new/codegen/doctor)
└── docs/                   # Fumadocs 文档站 (en/zh 双语)
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

- **M12a**: 调试工具 — DebugMode (Wireframe/Normals/Metallic/Roughness/AO/UV/Depth 等 10 种模式), DebugOverlay 配置 — **已完成**
- **M12b**: 性能分析 — RenderStats (draw_calls/triangles/culled/visible/fps/frame_time), summary() 格式化输出 — **已完成**
- **M12c**: 文档和教程 — 完整 API doc-tests, showcase 示例 (DamagedHelmet PBR 全功能演示) — **已完成**

### Phase H: v0.2 — 现代引擎特性补齐

#### Tier 1: 视觉质量飞跃 — **已完成**

- **Bloom**: 13-tap downsample + 9-tap tent upsample, 5 级 mip chain, BloomSettings (threshold/knee/intensity) — **已完成**
- **SSAO**: hemisphere sampling + 4x4 noise texture + box blur, 半分辨率, SsaoSettings (quality/radius/bias) — **已完成**
- **Cascade Shadow Maps**: 3 级 CSM, 视锥体分割, D2Array shadow map, PBR shader 按 view-Z 选 cascade — **已完成**

#### Tier 2: 架构基础设施 — **已完成**

- **Transform 层级运行时**: render_extract_system 修复使用 GlobalTransform（传播系统 M9a 已有） — **已完成**
- **场景序列化**: SceneSerializer (RON), Serializable marker, Parent/Children serde — **已完成**
- **持久化系统**: SaveManager (多槽位 + 元数据), Settings (RON 类型化分区), WorldStorage (文件系统 KV) — **已完成**
- **异步资源加载**: AssetServer.load_async() + mpsc 通道 + process_completed — **已完成**
- **资源热重载**: FileWatcher (notify crate, feature-gated "hot-reload") — **已完成**

#### Tier 3: 游戏性核心 — **已完成**

- **物理运行时**: RapierContext::raycast() + extract_collision_events_system（rapier 集成 M10b 已有） — **已完成**
- **UI 框架**: UiEvents/hit test/process_interactions + Widget 工厂（布局引擎+渲染器 M11b 已有） — **已完成**
- **骨骼动画管线**: Skeleton/AnimationPlayer 升级为 ECS Component + BoneMatrices（shader M11a 已有） — **已完成**

#### Tier 4: 高级功能 — **已完成**

- **AI / 寻路**: NavMesh (顶点+三角形+邻接图), A* 路径规划 (三角形质心图), NavAgent + steering 系统 — **已完成**
- **网络 / 多人**: ReliableChannel (序列号+ACK+重传) + UnreliableChannel, ECS 复制 (Replicated+DeltaEncoder), 客户端预测 (PredictionState+InputBuffer) — **已完成**
- **高级后处理**: DOF (CoC+圆盘模糊), Motion Blur (速度buffer+方向模糊), Color Grading (曝光/对比度/饱和度+3D LUT) — **已完成**
- **开发工具**: 帧性能分析器 (CPU timing+percentile), Debug 渲染器 (线段/包围盒/球体), 游戏内调试控制台 (命令注册/help/history) — **已完成**

#### 补充完成项 — **已完成**

- **自动存档**: AutoSaveConfig + 槽位轮转计时器 — **已完成**
- **存档迁移框架**: SaveMigration trait + MigrationRunner 链式版本升级 — **已完成**
- **AssetCache**: 内容 hash → in-memory LRU 缓存 (可配置 max_size) — **已完成**
- **依赖追踪**: DependencyGraph + 级联卸载 (递归孤儿收集) — **已完成**
- **关节约束**: FixedJoint/RevoluteJoint/PrismaticJoint/SphericalJoint + sync_joints_to_rapier_system — **已完成**
- **Craft 引擎迁移**: 持久化→SaveManager+WorldStorage, 物理→Velocity+AabbCollider, HUD→UiRenderer — **已完成**

### Phase I: v0.3 — 架构重构 + 新 Crate 拆分 — **已完成**

#### Dead Code Removal & Deduplication — **已完成**

- **Dead Code 清理**: 删除 12 处废弃代码 (timed_system, chain/parallel, NetworkEvents, parent_child_sync, PluginGroup, MouseDelta, DebugOverlay 死标志, DebugMode 未实现变体, shadow 未使用类型等) — **已完成**
- **代码去重**: CachedBuffer 共享 GPU 缓冲区工具, MatrixUniform 共享类型, DebugRenderer + LineRenderer 合并为 OverlayLineRenderer — **已完成**

#### App Runner (anvilkit-app) — **已完成**

- **AnvilKitApp**: GameCallbacks trait 替代 ApplicationHandler 样板, GameConfig + GameContext, 自动 DeltaTime/输入转发/帧生命周期 — **已完成**
- **WindowSize**: ECS Resource 自动随 resize 更新 — **已完成**
- **游戏迁移**: Craft + Billiards 迁移到 AnvilKitApp::run() 模式 (各删除 ~300 行样板) — **已完成**

#### UI Core (anvilkit-ui) — **已完成**

- **UI 数据模型提取**: UiStyle/UiText/UiNode/Val/FlexDirection/Align 从 render 移到独立 crate — **已完成**
- **布局引擎**: UiLayoutEngine 递归树布局 (taffy Flexbox), UiTree 父子关系, UiPlugin — **已完成**
- **事件与焦点**: UiEventKind/UiEvent/UiEvents, hit test, Tab 焦点切换, UiInteraction — **已完成**
- **控件库**: Checkbox/Slider/TextInput/ScrollView/Dropdown, UiTheme 默认主题 — **已完成**

#### Gameplay Systems (anvilkit-gameplay) — **已完成**

- **属性系统**: Stat<T> 泛型属性 (base + modifier stack + computed), Additive/Multiplicative/Override 修改器 — **已完成**
- **生命值**: Health 组件 (current/max/regen), DamageEvent/HealEvent/DeathEvent — **已完成**
- **物品系统**: Inventory trait, SlotInventory (固定槽位), StackInventory (可堆叠), ItemDef/ItemStack — **已完成**
- **技能冷却**: Cooldown 组件 + CooldownPlugin — **已完成**
- **状态效果**: StatusEffect + StackPolicy (Replace/Extend/Stack) — **已完成**
- **对象池**: EntityPool<T> (acquire/release, 预分配+动态增长) — **已完成**

#### Data Tables & i18n (anvilkit-data) — **已完成**

- **DataTable<K,V>**: 类型化 KV 存储, RON/JSON 加载, DataTablePlugin — **已完成**
- **Locale**: i18n 翻译系统, translate(key) fallback, RON 翻译文件 — **已完成**

#### Crate Restructuring — **已完成**

- **Physics 目录化**: physics.rs → physics/ (mod.rs + components.rs + aabb.rs + rapier.rs + events.rs) — **已完成**
- **Render 文件拆分**: events.rs (1414行) → 6 文件, draw.rs (568行) → 5 文件 — **已完成**
- **Persistence 改进**: 错误类型统一为 Persistence 变体, Resource derive, prelude 导出 — **已完成**
- **CameraPlugin**: Orbit 模式, 加入 DefaultPlugins — **已完成**

#### Camera System Upgrade — **已完成**

- **架构重组**: 平铺 → 三层目录 (orbit/ effects/ constraints/)，14 文件 2478 行
- **CameraMode 简化**: 枚举不再内嵌数据，OrbitState 独立组件
- **8 系统管线**: rig → input → rail → mode → spring_arm → look_at → effects → transition
- **Trauma 抖动**: Perlin noise 替代正弦波，trauma^power 衰减曲线 (Eiserloh GDC 2016)
- **SpringArm**: Ray-AABB 碰撞检测，第三人称相机穿墙防护
- **CameraRig**: 实体跟随 + offset + 阻尼，帧率无关平滑 `1-e^(-speed*dt)`
- **CameraTransition**: 5 种缓动曲线 (Linear/SmoothStep/EaseInOutCubic 等)
- **CameraRail**: Catmull-Rom 轨道相机 + 循环支持
- **LookAtTarget**: 软约束 + 屏幕空间死区
- **InputCurve**: 死区 + 幂次响应曲线 (linear/quadratic/cubic)
- **67 测试 + 2 doc-tests**, 全 workspace 零回归
