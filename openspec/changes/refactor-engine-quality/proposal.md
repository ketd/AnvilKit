# Change: 引擎质量大重构 — 修复关键缺陷、统一架构、提升性能

## Why

深度 code review 发现 AnvilKit 引擎存在多层问题：2 个内存安全隐患（unsafe Send/Sync、unsound 生命周期转换）、2 个渲染正确性 bug（shadow pass 逐 draw 清除、per-draw encoder submit）、多个游戏逻辑缺陷（帧率依赖物理、骨骼动画 TRS 顺序错误）、以及广泛的架构不一致（日志策略混乱、Cargo metadata 未统一、GPU 资源泄漏、代码大量重复）。这些问题阻碍了项目从原型走向可发布状态。

## What Changes

### P0 — 安全与正确性 (必须立即修复)
- **BREAKING** 移除 `AudioEngine` 的 `unsafe impl Send/Sync`，改用 `Mutex<AudioEngine>` 或约束到主线程
- **BREAKING** 重构 `RenderSurface` 生命周期，让 surface 持有 `Arc<Window>` clone，消除 unsound unsafe
- 修复渲染器 per-draw encoder submit → 单 encoder 批量 draw call
- 修复 shadow pass `LoadOp::Clear` 移到循环外（仅首次 clear）
- 修复 uniform buffer 在多次 submit 间无同步写入的 race condition

### P1 — 游戏逻辑修复
- Craft: `DeltaTime` 从实际帧时间更新，不再硬编码 1/60
- Craft: 异步 chunk 传入邻居数据，消除边界接缝
- Craft: 初始 chunk 生成走异步管线 + loading screen，不阻塞主线程
- Assets: 修复 `compute_bone_matrices` 的 TRS 组合顺序
- Assets: 实现 `CubicSpline` 插值（当前 fallback 到 linear）
- Billiards: `BallTracker.on_table[0]` 在 scratch 后恢复为 true
- Billiards: 修复 MSAA resolve 逻辑（所有 pass 都 resolve 到 HDR target）

### P2 — 架构统一
- 所有 crate 统一使用 `version.workspace = true` 和 workspace metadata
- 统一日志框架：全部使用 `log` crate + `env_logger`，移除所有 `println!`
- 提取 `pack_scene_lights` 到 `anvilkit-render` 共享模块，消除 5 处重复
- 删除 ECS 中的 no-op stub 方法（`timed_system`, `chain`, `parallel`）或实现其功能
- `App::add_plugins` 增加去重检查
- `App::update` 不再静默忽略 schedule 错误

### P3 — 渲染性能优化
- 子系统渲染器（sprite/particle/ui/line/text）引入 buffer pool，不再每帧 allocate
- `RenderAssets` 增加 `remove_mesh/material/pipeline` 资源卸载 API
- `compute_matrix()` 单次计算结果复用，不再 per-entity 调用两次
- Surface lost/outdated 错误触发 reconfigure，不再永久 broken
- BRDF LUT 预计算为二进制资产，不再 CPU 启动时生成 67M 次迭代
- PBR / Skinned-PBR 着色器统一 BRDF 函数，消除公式差异
- Shadow map texel size 通过 uniform 传入，不在 shader 硬编码

### P4 — 工具链与文档
- CLI: `--watch` flag 实现或移除
- CLI: codegen 增加标识符校验和 import 生成
- CLI: workspace 检测改用 TOML 解析
- 删除未使用依赖（`anyhow`, `walkdir` in CLI; `serde` feature on glam in Craft）
- Camera: 修复 hardcoded FOV=70 和 third-person look-at 坐标系
- 补充 `anvilkit-audio` 和 `anvilkit-camera` 单元测试
- README Quick Start 更新为实际 API
- PLAN.md 更新至当前项目状态

## Impact
- Affected specs: `render-system`, `ecs-system`, `asset-system`（已有）；`audio-system`, `camera-system`（新增）
- Affected code: 全部 8 个引擎 crate、2 个游戏、1 个 CLI 工具、共享 shader、文档
- Breaking changes: `AudioEngine` API 变化、`RenderSurface` 构造签名变化、ECS stub 方法移除
- Risk: 渲染器重构影响所有示例和游戏的渲染循环，需要逐一验证
