## Context

AnvilKit v0.1 使用 wgpu 0.19 + bevy_ecs 0.14 构建了完整的 PBR 渲染管线和 ECS 架构。
多项"已完成"里程碑（M9a 场景图、M10b 物理、M11a 动画、M11b UI）实际上只定义了数据结构和组件类型，缺少运行时后端。v0.2 需要补齐这些运行时实现，并添加全新的后处理管线。

## Goals / Non-Goals

### Goals
- 补齐所有"有组件无运行时"的系统（物理、UI、动画、场景图传播）
- 添加 Bloom/SSAO/CSM 三大视觉质量提升
- 提供通用场景序列化，替代各游戏自行实现
- 保持 v0.1 的模块化设计——所有新功能通过 feature flag 可选

### Non-Goals
- 不做可视化编辑器（短期用代码驱动）
- 不做 WASM/移动端支持（v0.3 考虑）
- 不做完整 AAA 级渲染（如 GI、体积雾、GPU-driven rendering）
- 不更换核心依赖版本（bevy_ecs 0.14, wgpu 0.19 维持不变）

## Decisions

### Bloom 实现策略
- **决定**: 双线性 13-tap downsample + 9-tap Gaussian blur，4 级 mip chain
- **理由**: 与现有 HDR pipeline 自然集成，质量/性能平衡好
- **替代方案**: Kawase blur（更快但质量略低），FFT bloom（过度复杂）

### 物理引擎选择
- **决定**: `rapier3d` + `rapier2d` 通过 Cargo feature flags 可选
- **理由**: Rust 生态最成熟的物理引擎，API 稳定，性能优秀
- **替代方案**: 自研物理（工作量太大），hecs_physics（不够成熟）

### UI 布局引擎
- **决定**: 自研 mini-flexbox（仅实现核心子集），不依赖 taffy/morphorm
- **理由**: 保持零外部依赖的设计哲学，游戏 UI 不需要完整 CSS 规范
- **替代方案**: taffy（功能全但引入大依赖），immediate mode（不适合复杂 UI）

### 场景序列化
- **决定**: 基于 serde + RON 格式，ECS World snapshot
- **理由**: Rust 生态标准序列化，RON 比 JSON 更人类可读，比 YAML 更类型安全
- **替代方案**: bincode（不可读），JSON（冗长），自定义格式（维护负担）

### 持久化存储后端
- **决定**: WorldStorage 使用 SQLite（`rusqlite`）作为默认 KV 后端
- **理由**: 单文件嵌入式数据库，原子写入，crash safety，跨平台，Rust 生态成熟
- **替代方案**: sled（纯 Rust 但维护停滞），rocksdb（太重），自研 append-only log（需要大量工作）
- **存档格式**: RON 用于场景快照 + SQLite 用于大规模数据（chunk 等），两者互补

### 网络架构
- **决定**: 预留接口但 v0.2 不实现，仅定义 NetworkPlugin trait 占位
- **理由**: 网络是最复杂的子系统，需要独立的深入设计

## Risks / Trade-offs

- **rapier 版本耦合**: rapier 更新频繁，需要 pin 版本并做 API 封装层
  - 缓解: 在 anvilkit-ecs 中定义物理 trait，rapier 只是一个后端实现
- **SSAO 性能**: 全分辨率 SSAO 在低端 GPU 可能太慢
  - 缓解: 默认半分辨率 + 双边模糊上采样
- **UI 布局正确性**: 自研 flexbox 可能有边缘情况 bug
  - 缓解: 用 taffy 的测试用例做回归测试

## Open Questions

- 是否需要 ECS 场景实例化（prefab/archetype cloning）？
- Cascade Shadow Maps 分几级？2 vs 4？
- 骨骼动画是否需要 GPU skinning（compute shader）还是 CPU 即可？
