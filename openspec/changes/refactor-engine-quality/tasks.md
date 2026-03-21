## Phase 0: 准备工作 (无功能变化)

- [x] 0.1 统一所有 crate 的 Cargo.toml 使用 `version.workspace = true` 和 workspace metadata（ecs, render, input, audio, assets, camera）
- [x] 0.2 统一 bevy_ecs 依赖引用方式 — 所有 crate 使用 `bevy_ecs = { workspace = true }` 而非硬编码版本
- [x] 0.3 替换所有 `println!` 为 `log` crate 调用（anvilkit-ecs 的 debug systems、layer_sorting_system）+ 修复 performance_monitor_system 首帧除零和 %1 恒真 bug
- [x] 0.4 删除 CLI 工具未使用依赖 `anyhow` 和 `walkdir`（tools/anvilkit-cli/Cargo.toml）
- [x] 0.5 删除 Craft 游戏 glam 的未使用 `serde` feature（games/craft/Cargo.toml）
- [x] 0.6 anvilkit-audio 已在 Phase 1 补充 4 个单元测试（engine_creation, send_sync, operations, sink_result）
- [x] 0.7 anvilkit-camera 补充 5 个单元测试（default、rotation、forward、toggle_perspective、pitch_limits）
- [x] 0.8 所有 7 个引擎 crate 添加 `#![warn(missing_docs)]` lint

## Phase 1: 安全与正确性修复

- [x] 1.1 重构 AudioEngine — unsafe Send/Sync 移到 newtype Inner 上，添加安全文档，get_or_create_sink 返回 Result，添加 4 个单元测试
- [x] 1.2 `get_or_create_sink` 返回 `Result` 替代 `.expect()`（engine.rs）
- [x] 1.3 `audio_playback_system` 使用 ResMut + Result 错误处理替代 Res + expect（systems.rs）
- [x] 1.4 重构 `RenderSurface` — surface 持有 `Arc<Window>` clone，移除 unsound unsafe 块（events.rs + surface.rs）
- [x] 1.5 `RenderSurface` 移除生命周期参数，`RenderApp.render_surface` 改为 `Option<RenderSurface>`
- [x] 1.6 修复 shadow pass — `LoadOp::Clear(1.0)` 仅在首次 draw command 时使用，后续使用 `LoadOp::Load`
- [x] 1.7 新增 `get_current_frame_with_recovery()` + `reconfigure()`，自动恢复 Lost/Outdated
- [x] 1.8 统一 workspace wgpu 到 0.19, winit 到 0.30（Cargo.toml）
- [x] 1.9 修复 `PbrSceneUniform` 注释 "768 字节" → 实际 848 字节（state.rs:39）
- [x] 1.10 新增 `RenderSurface::new_with_vsync()`，events.rs 传入 `config.vsync`

## Phase 2: 渲染器性能重构

- [x] 2.1 重构 `render_ecs()` — shadow pass: 单 encoder + 单 render pass + 循环 draw calls + Clear 仅一次
- [x] 2.2 重构 `render_ecs()` — scene pass: 单 encoder + 单 render pass + 循环 draw calls + MSAA resolve 正确
- [ ] 2.3 引入 `DynamicUniformBuffer` — 预分配 1024 draw commands 容量，每个 draw 使用 offset 索引 uniform 数据
- [ ] 2.4 消除 per-draw uniform buffer write — 所有 draw command 的 uniform 数据一次性写入 dynamic buffer
- [x] 2.5 `compute_matrix()` 结果缓存 — per-entity 只调用一次，存入局部 `model` 变量
- [x] 2.6 引入 `BufferPool` struct — acquire/release API，上限 64 buffers，2 个单元测试
- [x] 2.7 SpriteRenderer: cached_vb 字段 + write_buffer 复用（不再 create_buffer_init 每帧）
- [x] 2.8 ParticleRenderer: cached_instance_buf + write_buffer 复用
- [x] 2.9 UiRenderer: cached_vb + write_buffer 复用
- [x] 2.10 LineRenderer: cached_vb + write_buffer 复用
- [x] 2.11 TextRenderer: cached_vb + write_buffer 复用
- [x] 2.12 `RenderAssets` 增加 `remove_mesh/material/pipeline` + `mesh/material/pipeline_count()` 方法
- [ ] 2.13 BRDF LUT 预计算为 256x256 binary 文件，启动时直接加载（events.rs:409）
- [x] 2.14 TextRenderer / LineRenderer 复用 bind group layout（不再 create 两次相同的 layout）
- [x] 2.15 normal matrix: uniform-scale 检测 → transpose()，non-uniform → inverse().transpose()

## Phase 3: 游戏逻辑修复

- [x] 3.1 Craft: `DeltaTime` 从 `Instant::elapsed()` 实时更新，clamp 到 [0.001, 0.1]
- [x] 3.2 Craft: 新 chunk 插入 world 时标记自身和四邻为 dirty，触发 re-mesh
- [ ] 3.3 Craft: 初始 chunk 生成走异步管线，添加 loading screen（main.rs:318-320）
- [x] 3.4 Craft: 提取 `mark_dirty_with_neighbors()` 共享函数，消除重复
- [x] 3.5 Craft: `WorldSeed` resource 替代 3 处硬编码 seed
- [x] 3.6 Craft: texture 加载使用 `Result` + 紫黑棋盘 fallback 替代 `.expect()`
- [x] 3.7 Craft: greedy mesh mask 在循环外分配一次，per-slice `fill()` 复用
- [x] 3.8 Craft: cross-chunk neighbor lookup 改为独立检查 X/Z（不再 if/else if），diagonal corner 返回 Air
- [x] 3.9 Craft: Sky+Voxel+Water+Tonemap 合并为单 encoder 单 submit（9 submits → 1+HUD）
- [x] 3.10 Assets: `compute_bone_matrices` 改为分别累积 T/R/S 后 `Mat4::from_scale_rotation_translation(s,r,t)` 组合
- [x] 3.11 Assets: CubicSpline 插值使用 Hermite basis (h00/h01)，不再 fallback 到 linear
- [x] 3.12 Assets: `MeshData::validate()` 检查 positions/normals/texcoords/tangents 长度一致
- [x] 3.13 Billiards: scratch 后恢复 `tracker.on_table[0] = true`（game_logic.rs + ResMut）
- [x] 3.14 Billiards: 所有 scene pass 始终 resolve MSAA 到 HDR target（不再仅 last pass）
- [x] 3.15 Billiards: 删除未使用的 `Cushion`/`Pocket` 组件 + `let _ = entity;`

## Phase 4: ECS 与架构改进

- [x] 4.1 `SystemUtils::timed_system` 标记 `#[deprecated]`
- [x] 4.2 `SystemCombinator::chain` 和 `parallel` 标记 `#[deprecated]`
- [x] 4.3 `App::add_plugins` 增加 type_name 去重检查 + log::warn
- [x] 4.4 `App::update` schedule 错误通过 `log::error!` 记录
- [x] 4.5 修复 `performance_monitor_system` 首帧除零和 `% 1` 恒真 bug（Phase 0 已完成）
- [x] 4.6 `visibility_filter_system` 重写：查询 Parent 实体可见性解析 Inherited，避免 bevy query 冲突
- [x] 4.7 `pack_lights()` / `compute_light_space_matrix()` 公开并重导出，5 处重复替换为共享调用
- [x] 4.8 Camera: `base_fov` 字段添加到 CameraController，FOV offset 基于 base_fov 而非硬编码 70
- [x] 4.9 Camera: third-person look-at 改为 right=forward.cross(up), up=right.cross(forward) + 近垂直 fallback

## Phase 5: 着色器与工具链

- [x] 5.1 统一 skinned_pbr.wgsl BRDF 签名与 pbr.wgsl 一致 + fresnel_schlick clamp
- [x] 5.2 Shadow map texel size 通过 emissive_factor.w uniform 传入，pbr.wgsl 不再硬编码
- [x] 5.3 craft_tonemap.wgsl 水下 UV distortion 添加 clamp(0,1)
- [x] 5.4 tonemap gamma 修复：FilterUniform.apply_gamma + sRGB 检测，条件跳过 pow(1/2.2)
- [x] 5.5 删除 ui.wgsl 中未使用的 `inner_dist` 变量
- [x] 5.6 CLI: `--watch` flag 从 CLI 定义中移除（未实现功能不暴露）
- [x] 5.7 CLI: codegen 增加 `validate_identifier()` + 自动添加 `use bevy_ecs::prelude::*;`
- [x] 5.8 CLI: workspace 检测 + doctor member counting 改用 `toml::Value` 解析
- [x] 5.9 修复 demo.rs run command 注释 showcase → demo

## Phase 6: 文档与收尾

- [x] 6.1 README.md Quick Start 更新为实际 API（RenderPlugin + RenderApp::run）
- [x] 6.2 README.md 路线图更新至 Phase G 完成状态，移除虚构 features table
- [x] 6.3 PLAN.md 更新至实际状态（M0-M12c 全部标为已完成 + 示例游戏列表）
- [x] 6.4 `ScaledTime::delta()` 和 `delta_seconds()` 文档说明负 scale 行为差异
- [x] 6.5 `remap()` 增加 division-by-zero 守卫（from_min==from_max 返回目标中点）
- [x] 6.6 全量 `cargo check --workspace` 零错误验证通过
- [x] 6.7 全量 `cargo test --workspace` 662 测试通过验证
- [ ] 6.8 运行 showcase 和两个游戏视觉回归测试（需手动运行，CI 无 GPU）
