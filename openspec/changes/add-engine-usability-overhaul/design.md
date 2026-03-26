## Context

AnvilKit v0.1 + refactor-engine-quality 修复了安全/正确性/性能问题，add-engine-v02-features 补齐了组件和运行时。但引擎可用性仍然很低：用户直接面对裸 wgpu API，无高级抽象。本次变更聚焦于"让引擎可以被人用起来"——从技术演示走向开发者工具。

### 利益相关者
- 引擎用户（游戏开发者）：最大受益者，hello world 从 340 行 → ~30 行
- Craft/Billiards 游戏：需迁移到新 API，但代码量大幅减少
- 15 个 demo：从复制粘贴变为共享基础设施

### 约束
- bevy_ecs 0.14 版本锁定
- wgpu 0.19 / winit 0.30 版本锁定
- 不引入新的大型依赖（利用已有 Bevy Events, States 等能力）
- 向后兼容：低级 API 保留，高级抽象为新增层

## Goals / Non-Goals

### Goals
- 将最小可运行示例从 340 行降低到 30 行以内
- 所有已实现的后处理效果可通过配置启用（零代码改动）
- ECS 提供完整的游戏逻辑基础设施（事件、状态、固定步长）
- Audio 模块所有已声明字段均有实际功能
- Asset Pipeline 支持常见独立工作流（加载纹理、加载动画、热重载）
- Input 系统支持 gamepad（winit 0.30 已有 gamepad 事件支持）

### Non-Goals
- 不做可视化编辑器
- 不做 deferred rendering（保留 forward pipeline）
- 不做 GPU-driven rendering（compute culling, indirect draw）
- 不做完整 UI 框架（scrolling, text input, clipping 留后续）
- 不做网络传输层（保留现有抽象框架）
- 不升级核心依赖版本

## Decisions

### D1: 渲染抽象层架构

**决定**: 在现有低级 wgpu 封装之上新增 `SceneRenderer` 编排层 + `StandardMaterial` / `MeshHandle` 组件层。

**架构**:
```
用户代码
  ↓ spawn(MeshHandle + StandardMaterial + Transform)
SceneRenderer（编排层）
  ↓ 自动提取 DrawCommand, 管理 pass 序列
  ↓ shadow → scene → [SSAO] → [DOF] → [MotionBlur] → bloom → [ColorGrading] → tonemap
现有低级 API（RenderAssets, RenderPipeline, BindGroup）
  ↓
wgpu
```

**替代方案**:
- (a) Bevy-style `RenderGraph` — 过度复杂，当前只有一条渲染路径
- (b) 仅提供 helper 函数 — 不够，无法自动管理 resize/bind group/pipeline 生命周期
- (c) 完全隐藏低级 API — 过于限制，高级用户需要自定义 shader/pass

**权衡**: SceneRenderer 是"中间层"，既封装常见场景又保留低级逃生口。用户可以只用 `StandardMaterial` 做 PBR，也可以注册自定义 pipeline 做特殊效果。

### D2: 后处理管线集成方案

**决定**: `PostProcessSettings` ECS 资源控制各效果开关 + 参数。`SceneRenderer` 在 tonemap 前按固定顺序插入启用的效果。

```rust
pub struct PostProcessSettings {
    pub ssao: Option<SsaoSettings>,       // None = 禁用
    pub dof: Option<DofSettings>,
    pub motion_blur: Option<MotionBlurSettings>,
    pub color_grading: Option<ColorGradingSettings>,
    pub bloom: Option<BloomSettings>,      // 已有，迁入统一接口
}
```

**执行顺序**: SSAO → Scene composite → DOF → Motion Blur → Bloom → Color Grading → Tonemap

**替代方案**:
- (a) 用户自定义顺序（render graph） — 灵活但复杂度高
- (b) 全部启用/全部禁用 — 颗粒度太粗

### D3: ECS 事件系统迁移方案

**决定**: 将 `CollisionEvents` 和 `NetworkEvents` 从 `Resource(Vec<T>)` 迁移到 Bevy 的 `Events<T>` 系统。

**实现**:
1. `App::add_event::<CollisionEvent>()` 注册事件类型
2. 碰撞检测系统使用 `EventWriter<CollisionEvent>` 写入
3. 消费系统使用 `EventReader<CollisionEvent>` 读取
4. Bevy 自动管理双缓冲和生命周期（事件存活 2 帧）

**BREAKING**: 现有代码 `Res<CollisionEvents>` → `EventReader<CollisionEvent>`

**替代方案**:
- (a) 保留 Vec 但加双缓冲 — 重复造轮子
- (b) 自研 channel — 不必要，Bevy Events 已优化

### D4: 游戏状态机方案

**决定**: 在 `AnvilKitEcsPlugin` 中封装 Bevy `States` 系统，提供 `GameState` derive macro 和 `AppStateExt` trait。

```rust
#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState { #[default] Menu, Playing, Paused }

app.init_state::<GameState>()
   .add_systems(OnEnter(GameState::Playing), setup_game)
   .add_systems(OnExit(GameState::Playing), cleanup_game);
```

**替代方案**: 自研状态机 — 不必要，Bevy States 已成熟

### D5: FixedUpdate 方案

**决定**: 新增 `FixedUpdate` schedule，在 `App::update()` 中通过累加器以固定间隔（默认 60Hz）运行。物理系统从 `Update` 迁入 `FixedUpdate`。

```rust
// App::update() 伪代码
let fixed_dt = 1.0 / 60.0;
self.accumulated_time += frame_dt;
while self.accumulated_time >= fixed_dt {
    world.run_schedule(FixedUpdate);
    self.accumulated_time -= fixed_dt;
}
world.run_schedule(Update);
```

**替代方案**: 外部 fixed-step wrapper — 侵入性太强，不如内置

### D6: 标准材质自动管线方案

**决定**: `StandardMaterial` 组件在首次渲染时（lazy init）通过 `SceneRenderer` 自动创建对应的 `RenderPipeline` + `BindGroup`。材质参数变化触发 bind group 重建（非 pipeline 重建）。

**缓存策略**:
- Pipeline 按 (vertex_format, blend_mode, cull_mode) 缓存 — 通常 < 10 种变体
- BindGroup 按 (material_id, texture_set) 缓存
- Texture 按路径去重（已有 AssetServer 去重）

**替代方案**:
- (a) 每帧重建 — 性能灾难
- (b) 用户手动管理 — 当前方案，可用性差
- (c) Material ID hash — 复杂度适中，但 bind group 变化频率低，lazy + dirty flag 更简单

### D7: 多相机方案

**决定**: 每个 `CameraComponent` 可配置 `RenderTarget`（默认 = swapchain，或 = 自定义 TextureView）。`SceneRenderer` 按优先级排序所有 active camera，逐一渲染。

```rust
pub enum RenderTarget {
    Window,                     // 默认：渲染到 swapchain
    Texture(TextureHandle),     // 渲染到纹理（用于 minimap、后视镜等）
}
```

**约束**: 每个 camera 独立的 shadow/SSAO pass 会成倍增加 GPU 开销，初期限制最多 4 个 active camera。

### D8: 空间音频方案

**决定**: 基于 `AudioListener` 和 `AudioSource` 的 `Transform` 计算距离，通过 rodio `Sink::set_volume()` 实现距离衰减。

**衰减模型**: `volume = source_volume * max(0, 1 - distance / spatial_range)` (线性衰减)

**替代方案**:
- (a) rodio spatial — rodio 的 SpatialSink 已废弃
- (b) kira — 需要迁移整个 audio 后端
- (c) 自研 HRTF — 过度复杂

**权衡**: 仅实现距离衰减（无 HRTF 方位感），足够大多数游戏使用

### D9: Asset 热重载集成方案

**决定**: `AssetServer` 内部持有 `FileWatcher` 实例。每帧 `process_completed()` 时同时调用 `watcher.poll_changes()`，对变更文件触发 `reload(asset_id)`。

**实现**:
1. `AssetServer` 维护 `path → asset_id` 反向映射
2. `poll_changes()` 返回变更路径列表
3. 对每个变更路径，查找 asset_id，设状态为 Loading，重新 dispatch 到 worker
4. 加载完成后替换 `AssetStorage` 中的数据

## Risks / Trade-offs

| Risk | Impact | Mitigation |
|------|--------|------------|
| SceneRenderer 抽象层与低级 API 职责模糊 | 用户困惑该用哪层 | 文档明确分层：StandardMaterial 用于 90% 场景，低级 API 用于自定义 |
| 多相机 × 多 pass 性能爆炸 | GPU 过载 | 限制 4 camera，每个 camera 可配置跳过哪些 pass |
| Events 迁移破坏现有游戏代码 | 编译错误 | 提供 migration guide，变更为机械替换（Res → EventReader） |
| FixedUpdate 与 Update 数据同步 | 物理/渲染不一致 | 提供 interpolation helper（alpha = accumulated / fixed_dt） |
| 空间音频精度不足 | 定位不准 | 线性衰减足够 indie 游戏，后续可升级到 inverse-square |

## Migration Plan

### Phase 1: 渲染抽象层（核心变化）
1. 实现 SceneRenderer + StandardMaterial + MeshHandle
2. 实现 PostProcessSettings 统一后处理
3. 实现 AutoInputPlugin + AutoDeltaTimePlugin
4. 实现 DefaultPlugins

### Phase 2: ECS 基础设施
5. 事件系统迁移
6. 游戏状态机封装
7. FixedUpdate schedule
8. SystemSet 排序配置
9. Scene 序列化扩展

### Phase 3: 音频 + 资产 + 输入补完
10. Audio 空间/循环/音高/mixer
11. Asset 缓存 + 热重载集成 + 动画加载 + 纹理加载
12. Gamepad + 轴向输入

### Phase 4: 渲染修复
13. CSM FOV + 坐标系修复
14. Mipmap 生成
15. 可配置 MSAA/clear/cull
16. 正交相机 + 多相机

### Phase 5: 清理 + 迁移
17. 死依赖清理
18. Example 迁移到新抽象
19. 游戏迁移
20. 全量测试验证

### Rollback
- 每个 Phase 独立 commit，可逐 Phase 回滚
- Phase 1 变化最大，建议 feature branch

## Open Questions

1. StandardMaterial 是否需要支持 alpha blend / alpha test 模式切换？（影响 pipeline 变体数量）
2. SceneRenderer 的 pass 序列是否允许用户插入自定义 pass？（影响 API 设计复杂度）
3. FixedUpdate 默认频率是 60Hz 还是用户可配？（建议可配但默认 60）
4. gamepad 支持是否需要 rumble/haptic feedback？（winit 0.30 可能不支持）
5. 热重载 shader 是否需要运行时 WGSL 编译？（当前 shader 是 include_str 编译期内联）
