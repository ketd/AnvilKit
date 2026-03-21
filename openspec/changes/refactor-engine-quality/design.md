## Context

AnvilKit 已从原型阶段（M0-M12c 全部完成）进入需要质量收敛的阶段。深度 code review 揭示了涵盖内存安全、渲染正确性、性能、架构一致性四个层面的系统性问题。本重构需要在不破坏现有游戏功能的前提下，逐层修复这些问题。

### 利益相关者
- 引擎用户（游戏开发者）：API 变更需平滑迁移
- Craft/Billiards 游戏：重构后功能不能回退
- CI/CD：重构后所有测试必须通过

### 约束
- bevy_ecs 0.14 版本锁定
- wgpu 0.18/0.19 版本（workspace 0.18, billiards 0.19 需统一）
- 不引入新的大型依赖

## Goals / Non-Goals

### Goals
- 消除所有 unsafe 代码的 soundness 问题
- 渲染器从 per-draw submit 重构为 batched submit，性能提升 5-10x
- 所有 crate 的 Cargo metadata、日志、错误处理统一
- 游戏逻辑不再帧率依赖
- 达到 zero-warning 编译标准

### Non-Goals
- 不重写渲染架构（保留当前 multi-pass 结构）
- 不升级 bevy_ecs 版本
- 不新增功能特性
- 不做 API 美化重构（保留现有公共接口，仅修必要的 breaking changes）
- 不追求 100% 测试覆盖率（聚焦关键路径）

## Decisions

### D1: AudioEngine 线程安全方案
**决定**: 用 `Mutex<Inner>` 包裹非 Send 内部状态，AudioEngine 对外保持 `Resource` 语义。
**替代方案**:
- (a) 限制 audio system 只在 main thread schedule 运行 — bevy_ecs 0.14 不提供原生 main-thread-only schedule 约束
- (b) 用 `Arc<Mutex<>>` — 过度包裹，AudioEngine 已经是 Resource（单例）
**权衡**: Mutex 引入锁争用，但 audio system 每帧只调用一次，争用概率极低

### D2: RenderSurface 生命周期方案
**决定**: `RenderSurface` 持有 `Arc<Window>` clone 而非借用 window 引用。wgpu 0.18+ 的 `Surface` 已支持 `'static` 生命周期配合 `Arc<Window>`。
**替代方案**:
- (a) `ouroboros` 自引用结构 — 引入复杂依赖，维护成本高
- (b) 保留 unsafe 但加 safety invariant 文档 — 不解决根本问题
**权衡**: Arc clone 增加一次引用计数操作，可忽略不计

### D3: 渲染器批量提交方案
**决定**: 每个 render pass 创建一个 encoder，所有 draw call 在同一个 render pass 内执行，pass 结束后统一 submit。
**实现路径**:
1. `render_ecs()` 按 pass 分组 draw commands（shadow / scene / transparent）
2. 每个 pass 内：单一 `begin_render_pass()` + 循环 `set_pipeline / set_bind_group / draw_indexed`
3. 不同管线的 draw command 需排序（按 pipeline → material → mesh 排序已有）
4. Uniform buffer 改为 dynamic uniform buffer with offsets，每个 draw command 对应一个 offset
**替代方案**:
- (a) 一个 encoder 多个 pass — 更简单但仍每个 draw 一个 pass
- (b) Indirect drawing — 需要 compute shader 支持，复杂度过高
**权衡**: Dynamic uniform buffer 需要预分配 max draw commands 大小的 buffer，但内存可控

### D4: DeltaTime 更新策略
**决定**: 在 `ApplicationHandler::about_to_wait()` 或每帧开始时用 `Instant::elapsed()` 更新 `DeltaTime` resource。
**约束**: delta 需 clamp 到 `[0.001, 0.1]` 范围，防止暂停/调试时产生巨大 dt 导致物理爆炸

### D5: 异步 Chunk 邻居数据方案
**决定**: Worker thread 生成 chunk 后，main thread 在将 chunk 插入 world 时检查四邻并 re-mesh（已有 dirty 机制）。不传邻居数据给 worker。
**理由**: 邻居数据可能在 worker 处理期间变化（玩家编辑），main thread re-mesh 使用最新数据更准确。
**实现**: 新 chunk 插入时自动标记自身和四邻为 dirty，下一帧 mesh update 系统统一 re-mesh。

### D6: Buffer Pool 方案
**决定**: 引入 `BufferPool` 结构：维护 `Vec<(wgpu::Buffer, usize)>` 按大小排序，`acquire(min_size)` 返回足够大的闲置 buffer 或创建新 buffer，`release()` 回收。每帧结束后 pool 回收所有本帧使用的 buffer。
**约束**: pool 设置上限（如 64 个 buffer），超过后丢弃最小的 buffer，防止无限增长。

### D7: wgpu 版本统一
**决定**: workspace 统一到 wgpu 0.19（billiards 已在用），同步更新 render crate。
**理由**: wgpu 0.19 的 `Surface` API 已原生支持 `Arc<Window>`，简化 D2 实现。

## Risks / Trade-offs

| Risk | Impact | Mitigation |
|------|--------|------------|
| 渲染器批量提交重构可能引入排序 bug | 渲染顺序错误 | 每改一个 pass 立即运行 showcase 视觉验证 |
| Dynamic uniform buffer 预分配大小不足 | draw call 超限 crash | 设默认 1024 draw commands，超限时 fallback 到多次 submit |
| wgpu 0.18→0.19 升级可能有 API 变化 | 编译错误 | 查阅 wgpu changelog，预留 1 天迁移 |
| AudioEngine Mutex 影响音频延迟 | 音频卡顿 | Mutex 只在 system 入口锁，不在 hot path |
| 移除 ECS stub 方法破坏用户代码 | 编译错误 | 方法标记 `#[deprecated]` 保留一个版本后删除 |

## Migration Plan

### Phase 0: 准备 (无功能变化)
1. 统一 Cargo.toml workspace metadata
2. 统一日志框架
3. 删除未使用依赖
4. 补充 audio/camera 测试

### Phase 1: 安全修复 (最小 API 变化)
5. 修复 AudioEngine Send/Sync
6. 修复 RenderSurface 生命周期
7. 修复 shadow pass clear
8. wgpu 版本统一到 0.19

### Phase 2: 渲染器性能重构 (核心变化)
9. 实现 batched render pass
10. 引入 dynamic uniform buffer
11. 引入 buffer pool
12. Surface error 自动 reconfigure
13. BRDF LUT 预计算

### Phase 3: 游戏逻辑修复
14. DeltaTime 实时更新
15. Chunk 异步加载 + 邻居 re-mesh
16. 骨骼动画 TRS 修复
17. Billiards bug 修复

### Phase 4: 收尾
18. 统一 PBR 着色器
19. CLI 修复
20. 文档更新
21. 全量测试验证

### Rollback
- 每个 Phase 独立提交，可逐 Phase 回滚
- Phase 2 风险最大，建议在 feature branch 上完成后 squash merge

## Open Questions

1. wgpu 0.19 的 `Surface::new()` 是否在所有目标平台（macOS/Windows/Linux）上支持 `Arc<Window>`？需要验证。
2. `anvilkit-camera` crate 是新增未跟踪代码（git status 显示 `??`），是否需要纳入此次重构还是暂时搁置？
3. ECS stub 方法（`timed_system`, `chain`, `parallel`）是直接删除还是实现其功能？取决于是否有外部用户依赖这些 API。
4. 是否需要在此次重构中统一 wgpu 版本（workspace 0.18 vs billiards 0.19）？如果统一到 0.19，render crate 可能需要较多 API 适配。
