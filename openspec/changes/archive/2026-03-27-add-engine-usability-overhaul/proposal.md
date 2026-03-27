# Change: AnvilKit 可用性大修 — 渲染抽象层 + ECS 基础设施 + 系统补完

## Why

深度 code review 揭示 AnvilKit 的核心可用性瓶颈：最简单的 hello world 需要 340 行裸 wgpu 代码（Bevy 仅需 ~15 行），4/5 后处理效果已实现但为死代码（SSAO/DOF/MotionBlur/ColorGrading 从未接入渲染循环），ECS 缺少事件系统/状态机/固定步长调度，Audio 模块仅 269 行且 spatial/looping/pitch 字段均未生效，Asset Pipeline 无缓存/无热重载集成/无 glTF 动画提取。15 个 demo 共 ~7000 行代码中 90% 为复制粘贴。这些问题使引擎停留在"技术演示"阶段，无法作为游戏开发工具投入使用。

## What Changes

### P0 — 渲染抽象层（最大可用性提升）

- 新增 `StandardMaterial` 组件 — PBR 材质参数包装，自动创建 pipeline + bind group
- 新增 `MeshHandle` 组件 — 标记实体参与自动渲染
- 新增 `SceneRenderer` — 共享渲染编排（自动 resize、多 pass 管理、后处理串联）
- 新增 `DefaultPlugins` — 一站式初始化（Window + Render + Input + Audio + Time + DeltaTime）
- 新增 `AutoInputPlugin` — 自动将 winit 事件转发至 `InputState`
- 新增 `AutoDeltaTimePlugin` — 自动从帧间隔更新 `DeltaTime` 和 `Time` 资源
- 重构所有 15 个 demo 使用新抽象（每个从 ~500 行 → ~50 行）
- 新增 `MeshData::to_pbr_vertices()` 便捷方法

### P1 — 渲染器修复与增强

- **后处理管线集成** — SSAO/DOF/MotionBlur/ColorGrading 串入 `render_ecs()` 渲染循环，通过 `PostProcessSettings` 资源逐一开关
- **Mipmap 自动生成** — `create_texture()` 自动生成 mip chain（compute shader 或 blit chain）
- **CSM FOV 修复** — cascade 矩阵使用 `CameraComponent.fov` 而非硬编码 `FRAC_PI_4`
- **CSM 坐标系统一** — shadow pass 统一使用 LH 坐标系（与主相机一致）
- **可配置渲染参数** — MSAA sample count、clear color、backface cull mode 通过 `RenderConfig` 配置
- **正交 3D 相机** — `CameraComponent` 新增 `Projection::Orthographic` 变体
- **多相机支持** — 多个 active camera 各自渲染到不同 render target
- **点光/聚光阴影** — 点光源 cubemap shadow、聚光灯 2D shadow map

### P2 — ECS 基础设施

- **事件系统** — `CollisionEvents` / `NetworkEvents` 改用 Bevy `Events<T>` + `EventReader<T>` / `EventWriter<T>`，替代手写 `Vec<T>` + 手动清除
- **游戏状态机** — 封装 Bevy `States` / `NextState` / `OnEnter` / `OnExit`，支持 Menu→Play→Pause 等状态转换
- **FixedUpdate 调度** — 新增 `FixedUpdate` schedule + 固定步长累加器，物理系统迁入
- **SystemSet 排序** — 在 `AnvilKitEcsPlugin::build()` 中配置 10 个 SystemSet 的相对执行顺序
- **Scene 序列化扩展** — 支持通过 `Reflect` trait 注册任意组件序列化（不仅限 Transform）
- **层级递归销毁** — `TransformHierarchy::despawn_recursive()` 销毁实体及所有后代

### P3 — Audio 补完

- **空间音频** — 基于 `AudioListener` 位置和 `AudioSource.spatial_range` 计算距离衰减（线性/反比例/指数）
- **循环播放** — `audio_playback_system` 读取并应用 `AudioSource.looping` 字段
- **音高调节** — 通过 rodio `Sink::set_speed()` 应用 `AudioSource.pitch`
- **音频 Bus** — `AudioBus` 资源（Master + Music/SFX/Voice 分类），每个 bus 独立音量
- **AssetServer 集成** — 音频文件通过 `AssetServer::load_async()` 加载，不再直接 `File::open()`

### P4 — Asset Pipeline v2

- **内存缓存** — `AssetServer` 对已加载资产做 `HashMap<AssetId, Arc<T>>` 缓存，避免重复加载
- **热重载集成** — `FileWatcher` 变更事件自动触发 `AssetServer` 重新加载对应资产
- **glTF 动画提取** — 新增 `load_gltf_animations()` 函数，从 glTF 提取 Skeleton + AnimationClip
- **独立纹理加载** — `load_texture(path)` 直接加载 PNG/JPEG 文件到 `TextureData`（不依赖 glTF）
- **Handle drop 卸载** — `AssetHandle<T>` 引用计数归零时自动从 `AssetStorage` 移除
- **后台解析** — `load_async` 在 worker thread 中完成 glTF/PNG 解析，不只是 I/O

### P5 — Input 系统 v2

- **Gamepad 支持** — `GamepadState` 资源 + `GamepadButton` / `GamepadAxis` 枚举 + winit gamepad 事件映射
- **轴向输入** — `InputAxis` 类型表示连续值输入（摇杆、扳机），ActionMap 支持轴绑定
- **ActionMap 优化** — 键从 `String` 改为 interned `ActionId`（u32 索引），消除堆分配

### P6 — 清理与统一

- 移除死依赖：`rapier2d`（无 2D 物理使用）、`kira`（audio 使用 rodio）
- 标记 `egui` 系列为 `[dev-dependencies]`（仅调试用）
- 统一所有 example 使用 `anvilkit` umbrella crate 而非单独 crate 导入
- 网络模块文档标注为 "framework only — no transport layer"

## Impact

- Affected specs: `render-system`, `ecs-system`, `asset-system`
- New specs: `render-abstraction`, `audio-system`, `input-system`, `engine-dx`
- Affected code: 全部 8 个引擎 crate、2 个游戏、20 个 example、16 个 shader
- **BREAKING**: `RenderApp` 事件处理内部重构（用户侧被 `DefaultPlugins` 封装），`CollisionEvents` / `NetworkEvents` 从 Resource 改为 Event，`ActionMap` 键类型从 `String` 改为 `ActionId`
- Risk: 渲染抽象层是核心架构变化，需要所有 example 逐一迁移验证
