## Tier 1: 视觉质量飞跃 ✅

### 1.1 Bloom 后处理 ✅
- [x] 1.1.1 编写 bloom 提取 shader（亮度阈值 + 13-tap 降采样）
- [x] 1.1.2 编写 9-tap tent filter 升采样 shader（additive blend）
- [x] 1.1.3 实现 5 级 mip chain 降采样 + 升采样
- [x] 1.1.4 在 HDR pipeline 中集成 bloom pass（tonemap 之前）
- [x] 1.1.5 添加 `BloomSettings` 资源（threshold, knee, intensity, enabled）
- [x] 1.1.6 为 Craft 的 HDR pipeline 集成 bloom

### 1.2 SSAO ✅
- [x] 1.2.1 编写 SSAO shader（hemisphere sampling + noise texture）
- [x] 1.2.2 实现半分辨率 SSAO 渲染 + box blur 上采样
- [x] 1.2.3 添加 `SsaoSettings` 资源（quality Low/Medium/High, radius, bias, intensity）
- [x] 1.2.4 在 tonemap 合成中注入 AO 因子
- [x] 1.2.5 支持从深度 buffer 重建法线（cross-product screen-space derivatives）

### 1.3 Cascade Shadow Maps ✅
- [x] 1.3.1 实现视锥体分割算法（CSM_SPLIT_RATIOS 百分比分割）
- [x] 1.3.2 创建多级 shadow map texture array (D2Array, 3 层)
- [x] 1.3.3 修改 shadow pass 为每级 cascade 独立渲染
- [x] 1.3.4 修改 PBR fragment shader 按 view-Z 选择最优 cascade
- [x] 1.3.5 PbrSceneUniform 升级: cascade_view_projs[3] + cascade_splits

## Tier 2: 架构基础设施 ✅

### 2.1 Transform 层级运行时 ✅
- [x] 2.1.1 ~~实现 `transform_propagation_system`~~ 已有（sync_simple_transforms + propagate_transforms）
- [x] 2.1.2 ~~注册到 `AnvilKitSchedule::PostUpdate`~~ 已有（TransformPlugin）
- [x] 2.1.3 ~~处理动态 reparent~~ 已有（propagate_recursive 处理 Children 变更）
- [x] 2.1.4 在 render_extract_system 中使用 GlobalTransform（修复 bug: 原来用的是本地 Transform）

### 2.2 场景序列化 ✅
- [x] 2.2.1 为 Parent/Children 组件 derive serde（conditional on "serde" feature）
- [x] 2.2.2 实现 `SceneSerializer::save/load`（RON 格式）
- [x] 2.2.3 添加 `Serializable` marker component
- [ ] 2.2.4 迁移 Craft 的 persistence.rs 到通用方案（留待后续整合）

### 2.3 持久化系统 ✅
- [x] 2.3.1 实现 `SaveManager`（存档槽位、元数据、list/save/load/delete）
- [ ] 2.3.2 实现自动存档（定时器 + `_autosave` 槽位）— 接口已有，定时器逻辑由游戏实现
- [ ] 2.3.3 实现存档版本迁移框架（`SaveMigration` trait）— 留待实际需求驱动
- [x] 2.3.4 实现 `Settings` 资源（RON 格式、类型化分区、default fallback）
- [x] 2.3.5 实现 `WorldStorage` KV 后端（文件系统，原子写入 write-tmp-rename）
- [x] 2.3.6 实现 batch 写入（batch_put，每个 key 原子性）
- [ ] 2.3.7 实现 `AssetCache`（内容 hash → 编译产物，LRU 逐出）— 留待热重载集成
- [ ] 2.3.8 迁移 Craft 的 persistence.rs 到引擎 SaveManager + WorldStorage — 留待后续整合

### 2.4 异步资源加载 ✅
- [x] 2.4.1 添加后台线程到 AssetServer（std::thread::spawn per load）
- [x] 2.4.2 实现 `load_async()` 返回 Handle + LoadState
- [x] 2.4.3 完成通道机制（mpsc channel + process_completed + drain_completed）
- [ ] 2.4.4 实现资源依赖追踪和级联卸载 — 留待实际需求驱动

### 2.5 资源热重载 ✅
- [x] 2.5.1 集成 `notify` crate 文件监视（FileWatcher）
- [x] 2.5.2 is_shader / is_texture 文件类型判断
- [x] 2.5.3 poll_changes() 轮询变更 + 去重
- [x] 2.5.4 通过 feature flag 控制（"hot-reload"，无 feature 时提供 no-op stub）

## Tier 3: 游戏性核心 ✅

### 3.1 物理引擎运行时 ✅
- [x] 3.1.1 ~~添加 `rapier3d` 依赖~~ 已有（feature "rapier"）
- [x] 3.1.2 ~~实现 `RapierPhysicsPlugin` + `RapierContext` 资源~~ 已有
- [x] 3.1.3 ~~实现 sync_to_rapier → step → sync_from_rapier 管线~~ 已有
- [x] 3.1.4 实现碰撞事件收集（extract_collision_events_system，从 NarrowPhase 提取）
- [x] 3.1.5 实现 `RapierContext::raycast()`（遍历 collider_set, ray-shape 检测）
- [ ] 3.1.6 实现关节约束（Fixed, Revolute, Prismatic, Spherical）— 留待实际需求驱动
- [ ] 3.1.7 迁移 Craft 的自定义 AABB 物理到引擎物理 — 留待后续整合

### 3.2 UI 框架 ✅
- [x] 3.2.1 ~~实现 flexbox 布局引擎~~ 已有（UiLayoutEngine + taffy）
- [x] 3.2.2 实现 UI 事件系统（ui_hit_test + process_ui_interactions，HoverEnter/Leave/Click）
- [x] 3.2.3 ~~实现 UI 渲染 pass~~ 已有（UiRenderer + ui.wgsl）
- [x] 3.2.4 实现基础控件（Widget::button/label/panel/row/column）
- [ ] 3.2.5 迁移 Craft HUD 到 UI 框架 — 留待后续整合

### 3.3 骨骼动画管线 ✅
- [x] 3.3.1 ~~编写 skinning vertex shader~~ 已有（skinned_pbr.wgsl, MAX_JOINTS=128）
- [x] 3.3.2 ~~实现运行时 AnimationPlayer~~ 已有（advance + looping + speed）
- [x] 3.3.3 Skeleton/AnimationPlayer 升级为 ECS Component（conditional bevy_ecs derive）
- [x] 3.3.4 新增 BoneMatrices component（compute_bone_matrices → GPU upload）

## Tier 4: 高级功能（待实现）

### 4.1 AI / 寻路
- [ ] 4.1.1 实现 NavMesh 生成（Recast 算法或集成 `recast-rs`）
- [ ] 4.1.2 实现 A* 路径规划
- [ ] 4.1.3 实现 NavAgent 组件 + steering 系统

### 4.2 网络 / 多人
- [ ] 4.2.1 定义 NetworkPlugin trait 和接口
- [ ] 4.2.2 实现 UDP transport（laminar）
- [ ] 4.2.3 实现 ECS 组件复制（delta compression）
- [ ] 4.2.4 实现客户端预测和回滚

### 4.3 高级后处理
- [ ] 4.3.1 DOF（景深）— 基于 CoC 的散焦模糊
- [ ] 4.3.2 Motion Blur — 基于速度 buffer 的方向模糊
- [ ] 4.3.3 Color Grading — LUT 3D 纹理调色

### 4.4 开发工具
- [ ] 4.4.1 帧性能分析器（GPU timestamp queries + CPU timing）
- [ ] 4.4.2 Debug 渲染模式扩展（碰撞体、NavMesh、光源体积）
- [ ] 4.4.3 游戏内调试控制台（命令注册 + 文本输入）
