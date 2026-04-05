## Context
对 AnvilKit 引擎 6 个核心 crate 和 Craft 游戏的全面审计发现：引擎有多处 bug、缺少 ECS 系统接入、Audio 未使用、App 生命周期不完整。同时 Craft 游戏缺少核心游戏性。本设计采用"先固后扩"策略。

**关键审计发现**:
- `anvilkit-render`: motion_blur/color_grading/dof 三个后处理模块有 bug
- `anvilkit-render`: 粒子系统 GPU 完整但无 ECS system、无深度测试、无纹理
- `anvilkit-gameplay`: 6 个模块中仅 Health 和 SlotInventory 被使用
- `anvilkit-audio`: rodio 集成可用但零游戏使用，AssetServer 集成缺失
- `anvilkit-app`: 无 `update()` hook，游戏被迫在 `render()` 中跑逻辑
- `anvilkit-ecs`: Navigation A* 完整但未被游戏使用
- `anvilkit-data`: DataTable 仅解析字符串，无文件 I/O

**关键约束**:
- 所有系统基于 bevy_ecs 0.14 Component/System/Resource 模式
- 体素代码保留在 Craft 游戏侧，不做引擎提取
- 增量式开发——每个 Phase 完成后引擎和游戏都应可编译运行
- 引擎修改须向后兼容（新 trait 方法提供默认实现）

## Goals / Non-Goals

**Goals**:
- 修复引擎已知 bug，确保所有已实现模块可正常使用
- 补齐 ECS 系统接入，让引擎能力真正可被游戏消费
- 完善 Audio 到可用状态，在 Craft 中验证
- 增强 App 生命周期，消除游戏侧 workaround
- 建立完整的 Craft 单人生存模式游戏循环

**Non-Goals**:
- 不从 Craft 提取通用体素 crate（保持简单）
- 不做多人联网（网络层存在但不在本提案范围）
- 不做红石/电路系统
- 不做 Mod 支持

## Decisions

### D1: motion_blur prev_view_proj 修复
- **Decision**: 在 `PostProcessResources` 中添加 `prev_view_proj: Mat4` 字段，每帧 render_loop 结束前存储当前帧 VP 矩阵供下帧使用
- **Why**: 当前 `render_loop.rs:281` 将当前帧 VP 传入 prev_view_proj，导致速度 buffer 全零
- **Risk**: 首帧无前帧数据→用当前帧矩阵（运动模糊=0，可接受）

### D2: color_grading src/dst 修复
- **Decision**: 添加 `color_grading_intermediate` Rgba16Float 纹理，读 HDR → 写 intermediate → 拷贝回 HDR
- **Alternatives**: Ping-pong 双 HDR 纹理（更高效但改动大）
- **Why**: `render_loop.rs:299` 将 `hdr_texture_view` 同时作为 src 和 dst

### D3: 粒子系统 ECS 接入
- **Decision**: 添加 `particle_emit_system` 和 `particle_update_system` 到引擎 `Update` schedule；`ParticleEmitter` Component 自动发射，`ParticleRenderer` 自动收集渲染
- **Why**: 当前粒子系统是手动 API，游戏无法通过 ECS 使用

### D4: GameCallbacks 生命周期扩展
- **Decision**: 新增 `update(&mut self, ctx)` 在 ECS schedule 之前调用；新增 `on_shutdown(&mut self, ctx)` 在退出前调用。均提供空默认实现，向后兼容
- **Why**: 当前游戏被迫在 `render()` 中跑游戏逻辑（Craft main.rs ~1400 行中约 700 行是非渲染逻辑）
- **Alternatives**: 完全 ECS 化（移除 GameCallbacks，纯 system）——改动太大

### D5: 生物群系 — 双噪声查表法
- **Decision**: 温度噪声 + 湿度噪声 2D 查表确定群系类型
- **Implementation**: `BiomeMap` 资源，`Biome` 枚举 Plains/Forest/Desert/Tundra/Ocean/Mountains

### D6: 方块光照 — BFS 泛洪 + 分离天光/方块光
- **Decision**: 每方块 1 字节额外存储（高4位=天光，低4位=方块光），BFS 队列传播
- **每帧预算**: 最多 1024 个光照更新

### D7: 实体/AI — ECS + FSM
- **Decision**: 生物作为 ECS 实体，AI 使用简单 FSM（Idle/Wander/Chase/Attack/Flee）
- **Why**: 与引擎 ECS 架构一致，FSM 对基础 AI 足够

### D8: 物品/合成 — 扩展 anvilkit-gameplay
- **Decision**: 扩展 `ItemDef`（当前是死类型），新增 `CraftingRecipe` DataTable
- **Why**: 复用引擎现有物品系统

### D9: Audio 集成策略
- **Decision**: 先修复 `audio_playback_system` 的 AssetServer 集成，然后在 Craft 中用路径直接加载（因 AssetServer 修复可能复杂，先走简单路径）
- **Fallback**: 如果 AssetServer 集成代价大，仅用 `AudioSource::new(path)` 文件路径模式

## Risks / Trade-offs

| Risk | Impact | Mitigation |
|------|--------|------------|
| GameCallbacks 改动影响两个游戏 | 中 | 新方法提供空默认实现，零破坏 |
| 后处理 bug 修复可能引入渲染回归 | 高 | 每个修复后运行 Craft + Billiards 验证 |
| 光照传播性能 | 中 | 每帧预算限制 + 异步传播 |
| 引擎改动与游戏功能并行开发冲突 | 中 | 严格按 Phase 顺序，Part 1 完成后再开 Part 2 |

## Open Questions
- Audio 资源从哪里获取？（建议：CC0 免费音效包，首轮用最少量音效验证管线）
- 是否修复 DOF composite pass？（建议：修，但保持 `enabled: false` 默认值）
- Navigation A* 是否用于生物 AI？（建议：不用——体素世界的网格 A* 更合适，NavMesh 适合连续空间）
