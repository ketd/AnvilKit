## Phase 0: Dead Code Removal & Deduplication (无 breaking changes)

### 0.1 Dead code removal
- [x] 0.1.1 删除 `SystemUtils::timed_system`（已废弃的空操作，system.rs:114）
- [x] 0.1.2 删除 `SystemCombinator::chain` 和 `parallel`（已废弃的空操作，system.rs:367/393）
- [x] 0.1.3 删除 `NetworkEvents` 废弃资源 + `network_events_cleanup_system`（network.rs:113/436）
- [x] 0.1.4 删除 `parent_child_sync_system`（定义但从未注册到任何调度，transform.rs:254）
- [x] 0.1.5 删除 `PluginGroup<T>`（仅测试代码使用，plugin.rs:255）
- [x] 0.1.6 删除 `MAX_DELTA_SECONDS`（定义但从未引用，auto_plugins.rs:89）
- [x] 0.1.7 删除 `MouseDelta` 废弃结构体（camera/systems.rs:14-37）
- [x] 0.1.8 删除 `DebugOverlay` 死标志：show_wireframe/show_bounds/show_lights/show_skeleton（debug.rs）
- [x] 0.1.9 删除未实现的 `DebugMode` 变体（Normals/Metallic/Roughness/AO/UVs/Depth — 无 shader 支持）
- [x] 0.1.10 删除 shadow.rs 未使用类型（PointShadowConfig/SpotShadowConfig/ShadowAtlas）
- [x] 0.1.11 修复 `state_transition_system` 发出 `StateTransitionEvent`（state.rs:130-140）
- [x] 0.1.12 删除 AnvilKitEcsPlugin 中重复的 schedule 创建（plugin.rs:189-191，App::new() 已处理）

### 0.2 Code deduplication
- [x] 0.2.1 提取 `CachedBuffer` 共享工具（替代 5 处 cached_vb 重复: ui/text/sprite/line/particle）
- [x] 0.2.2 提取 `ProjectionUniform` 共享类型（替代 5 处 64 字节 ortho/scene uniform 重复）
- [x] 0.2.3 合并 `debug.rs` + `debug_renderer.rs` 为单一 debug 模块
- [x] 0.2.4 合并 `LineRenderer` 功能到 `DebugRenderer`，删除 LineRenderer
- [x] 0.2.5 更新 Craft + Billiards 使用 DebugRenderer 替代 LineRenderer
- [x] 0.2.6 `cargo test --workspace` 全量验证

## Phase 1: App Runner (消除游戏样板)

- [x] 1.1 创建 `crates/anvilkit-app/` crate，添加到 workspace
- [x] 1.2 实现 `AnvilKitApp` — ApplicationHandler + 事件循环 + 输入转发 + 帧生命周期
- [x] 1.3 实现 `GameConfig` — 窗口配置 + 插件列表 + 回调注册
- [x] 1.4 将 `DeltaTime` 定义移到 `anvilkit-ecs/src/app.rs`，physics.rs 添加 re-export
- [x] 1.5 添加 `WindowSize` ECS 资源，自动随 resize 更新
- [x] 1.6 迁移 Craft 到 `AnvilKitApp::run()` 模式（删除 ~300 行 ApplicationHandler 样板）
- [x] 1.7 迁移 Billiards 到 `AnvilKitApp::run()` 模式
- [x] 1.8 Billiards 使用 `RenderApp::forward_input()` 替代手动输入转发
- [x] 1.9 添加 `anvilkit-app` 到 facade crate 依赖和 re-export
- [x] 1.10 `cargo test --workspace` + 两个游戏运行验证

## Phase 2: UI Core (独立 UI 框架)

- [x] 2.1 创建 `crates/anvilkit-ui/` crate，添加到 workspace
- [x] 2.2 从 render/ui.rs 提取 UI 数据模型到 anvilkit-ui：UiStyle, UiText, UiNode, Val, FlexDirection, Align
- [x] 2.3 从 render/ui.rs 提取 UiLayoutEngine 到 anvilkit-ui（带 taffy 依赖）
- [x] 2.4 从 render/ui.rs 提取事件系统到 anvilkit-ui：UiEventKind, UiEvent, UiEvents, ui_hit_test, process_ui_interactions
- [x] 2.5 从 render/ui.rs 提取 Widget 工厂到 anvilkit-ui
- [x] 2.6 实现 UiTree — 父子节点关系管理（利用 bevy_ecs Parent/Children）
- [x] 2.7 实现递归树布局（UiLayoutEngine 支持多层嵌套而非单层 children）
- [x] 2.8 实现 UiPlugin — 注册 layout_system + event_system 到 ECS 调度
- [x] 2.9 实现焦点管理 — Tab 切换焦点 + UiInteraction 组件 (None/Hovered/Pressed/Focused)
- [x] 2.10 实现文字集成 — TextRenderer 作为 UiRenderer 的 text pass（文字在矩形内渲染）
- [x] 2.11 实现 UiTheme 资源 — 默认颜色/字体/间距/边框
- [x] 2.12 新增控件：Checkbox（点击切换 + UiChangeEvent）
- [x] 2.13 新增控件：Slider（拖拽 handle + 值更新）
- [x] 2.14 新增控件：TextInput（光标 + 键盘输入 + 选区）
- [x] 2.15 新增控件：ScrollView（可滚动容器 + 滚轮/拖拽）
- [x] 2.16 新增控件：Dropdown（下拉选择列表）
- [x] 2.17 render/ui.rs 瘦身 — 只保留 UiRenderer + UiVertex（GPU 部分），依赖 anvilkit-ui 的类型
- [x] 2.18 anvilkit-render 的 Cargo.toml 移除 taffy 直接依赖（通过 anvilkit-ui 间接获取）
- [x] 2.19 添加 anvilkit-ui 到 facade crate 依赖和 re-export
- [x] 2.20 `cargo test --workspace` 全量验证

## Phase 3: Gameplay Systems (游戏性核心)

- [x] 3.1 创建 `crates/anvilkit-gameplay/` crate，添加到 workspace，features = ["stats", "inventory", "cooldown", "status-effect", "entity-pool"]
- [x] 3.2 实现 `Stat<T>` — 泛型属性组件（base_value + modifier_stack + computed_value）
- [x] 3.3 实现 modifier 系统 — Additive/Multiplicative/Override 修改器 + 优先级排序
- [x] 3.4 实现 `Health` 组件 — current/max/regen_rate，基于 Stat<f32>
- [x] 3.5 实现 `DamageEvent`/`HealEvent`/`DeathEvent` + `health_system`
- [x] 3.6 实现 `Inventory` trait + `SlotInventory`（固定槽位网格）
- [x] 3.7 实现 `StackInventory`（可堆叠物品 + max_stack_size）
- [x] 3.8 实现 `ItemStack`/`ItemDef` 数据类型
- [x] 3.9 实现 `Cooldown` 组件 + `CooldownPlugin`（cooldown_tick_system）
- [x] 3.10 实现 `StatusEffect` 组件 + `StatusEffectPlugin`（duration tick + stack policy: Replace/Extend/Stack）
- [x] 3.11 实现 `EntityPool<T>`（acquire/release，预分配 + 动态增长）
- [x] 3.12 每个模块编写单元测试（≥5 个/模块）
- [x] 3.13 添加 anvilkit-gameplay 到 facade crate 依赖
- [x] 3.14 `cargo test --workspace` 全量验证

## Phase 4: Crate Restructuring (结构清理)

### 4.1 Physics 模块目录化
- [x] 4.1.1 physics.rs → physics/ 目录：mod.rs + components.rs + aabb.rs + rapier.rs + events.rs
- [x] 4.1.2 删除废弃的 `CollisionEvents` 资源，Rapier 系统改用 `EventWriter<CollisionEvent>`
- [x] 4.1.3 更新所有 import path（craft/billiards/examples）

### 4.2 Render 文件整理
- [x] 4.2.1 拆分 events.rs (1414 行) → events/ 目录：mod.rs + lighting.rs + render_app.rs + gpu_init.rs + render_loop.rs + input.rs
- [x] 4.2.2 移动 `Aabb` + `Frustum` 到 `anvilkit-core::math`（cfg_attr bevy_ecs Component）
- [x] 4.2.3 移动 `raycast.rs` 函数到 `anvilkit-core::math::raycast`（render 保留 re-export）
- [x] 4.2.4 从 `RenderPlugin::build()` 移除 `InputState` 和 `DeltaTime` 初始化
- [x] 4.2.5 draw.rs 拆分 → draw/ 目录：mod.rs + culling.rs + lighting.rs + commands.rs + gpu.rs

### 4.3 Persistence 独立化
- [x] 4.3.1 添加 `AnvilKitError::Persistence` 变体 + `persistence()`/`persistence_with_path()` 构造函数
- [x] 4.3.2 persistence 模块所有函数改用 `Persistence` 错误变体替代 `generic()`
- [x] 4.3.3 persistence 类型添加 `#[cfg_attr(feature = "bevy_ecs", derive(Resource))]`
- [x] 4.3.4 实现 `PersistencePlugin`（anvilkit-ecs, feature-gated, 注册 AutoSaveConfig/State + tick system）
- [x] 4.3.5 persistence 类型添加到 core crate prelude（cfg-gated）

### 4.4 验证
- [x] 4.4.1 `cargo test --workspace` 全量验证
- [ ] 4.4.2 两个游戏运行验证

## Phase 5: Fix Disconnections (修复断联系统)

### 5.1 Settings → Engine
- [x] 5.1.1 实现 `SettingsApplyPlugin`（audio sync: Settings.audio → AudioBus）
- [x] 5.1.2 Settings.audio → AudioBus 音量控制（master/music/sfx）
- [x] 5.1.3 Settings.input.mouse_sensitivity → CameraController sync（camera crate persistence feature）

### 5.2 ActionMap → Games
- [x] 5.2.1 InputPlugin 已由 AutoInputPlugin 覆盖（初始化 InputState + end_frame）
- [x] 5.2.2 Craft ActionMap 绑定（WASD/Space/Shift/Ctrl + mouse + slots 1-9）
- [x] 5.2.3 Billiards ActionMap 绑定（shoot/aim mouse buttons）
- [x] 5.2.4 ActionMap::apply_overrides + InputBinding::from_key_name + KeyCode::from_name

### 5.3 Audio → Games
- [x] 5.3.1 AudioEngine Send/Sync fix — NonSend 资源 + 移除 unsafe impl
- [x] 5.3.2 spatial_audio_system 立体声平移（listener right vector panning）
- [x] 5.3.3 AudioSource 添加 asset_id: Option<u64> 字段 + from_asset_id() 构造函数

### 5.4 Assets Integration
- [x] 5.4.1 DependencyGraph u64 → AssetId 类型安全迁移
- [x] 5.4.2 AssetServer::insert_parsed + get_parsed（ParsedAsset 桥接）
- [x] 5.4.3 AssetCache 集成到 AssetServer（content hash 缓存 + load_async 缓存命中跳过）
- [x] 5.4.4 DependencyGraph 集成到 AssetServer（add_dependency 接口 + 访问器）
- [x] 5.4.5 级联卸载 wiring（unload() 调用 remove_and_cascade + 清理 cache/parsed/states）

### 5.5 Scene Serialization
- [x] 5.5.1 SerializableRegistry 增强 — serialize/deserialize fn pointers
- [x] 5.5.2 SceneSerializer::save 查询 registry 填充 custom_data
- [x] 5.5.3 SceneSerializer::load 查询 registry 恢复自定义组件
- [x] 5.5.4 自定义组件 round-trip 测试（Health/Mana + 向后兼容）

### 5.6 Camera
- [x] 5.6.1 实现 `CameraPlugin`（注册 camera_controller_system）
- [x] 5.6.2 实现 `CameraMode::Orbit`（鼠标拖拽旋转 + 滚轮缩放 + 距离限制）
- [x] 5.6.3 CameraPlugin 添加到 DefaultPlugins
- [x] 5.6.4 Billiards 使用 Orbit 相机模式

### 5.7 DefaultPlugins & Facade
- [x] 5.7.1 DefaultPlugins 添加 CameraPlugin
- [x] 5.7.2 Facade prelude 添加 AudioSource/AudioListener/PlaybackState re-export
- [x] 5.7.3 Facade persistence feature passthrough（anvilkit-core/persistence + anvilkit-ecs/persistence）
- [x] 5.7.4 Games 改用 DefaultPlugins
- [x] 5.7.5 Games 减少直接依赖（移除手动 InputState/DeltaTime/camera_controller_system）

### 5.8 验证
- [x] 5.8.1 `cargo test --workspace` 全量验证
- [ ] 5.8.2 两个游戏完整功能测试

## Phase 6: Data Tables & i18n (数据驱动)

- [x] 6.1 创建 `crates/anvilkit-data/` crate，添加到 workspace
- [x] 6.2 实现 `DataTable<K, V>` — RON/JSON 加载 + hot-reload 支持
- [x] 6.3 实现 `DataTablePlugin` — 注册表为 ECS Resource
- [x] 6.4 实现 `Locale` 资源 + `translate(key)` + `.ron` 翻译文件加载
- [x] 6.5 实现翻译文件 fallback（缺失 key 返回 key 本身）
- [x] 6.6 添加 anvilkit-data 到 facade crate
- [x] 6.7 单元测试 + `cargo test --workspace`

## Phase 7: 收尾

- [x] 7.1 `cargo check --workspace` 零错误零警告
- [x] 7.2 `cargo test --workspace` 全量测试通过
- [ ] 7.3 两个游戏运行 + 视觉验证 — requires GPU runtime
- [x] 7.4 更新 project.md 路线图（添加 Phase I: v0.3 + 更新 Workspace Structure）
- [x] 7.5 更新 README.md Quick Start（使用 AnvilKitApp::run + GameCallbacks 模式）+ Crate Map 添加 4 新 crate

## Phase 8: Camera System Upgrade (全面升级)

### 8.1 Foundation (Phase 1)
- [x] 8.1.1 `noise.rs` — 1D/2D 梯度噪声 (Perlin)，camera shake 用
- [x] 8.1.2 `orbit.rs` — OrbitState 组件 (替代枚举内嵌数据)
- [x] 8.1.3 `input_curve.rs` — InputCurve 死区 + 幂次响应曲线
- [x] 8.1.4 `effects.rs` 重写 — Trauma 系统 + Perlin noise (替代正弦)

### 8.2 Core Refactor (Phase 2)
- [x] 8.2.1 CameraMode 简化 — 枚举无数据，PartialEq/Eq derive
- [x] 8.2.2 systems.rs 拆分 — camera_input_system + camera_mode_system + camera_effects_apply_system
- [x] 8.2.3 OrbitState 集成 — ThirdPerson/Orbit 统一代码路径 (消除 ~60 行重复)
- [x] 8.2.4 平滑公式修复 — `1 - e^(-speed * dt)` 替代 `(1-s).powf(dt*60)`
- [x] 8.2.5 plugin.rs + lib.rs 更新 — 注册拆分后系统 + 导出新模块

### 8.3 New Features (Phase 3)
- [x] 8.3.1 `rig.rs` — CameraRig (实体跟随 + offset + 阻尼)
- [x] 8.3.2 `spring_arm.rs` — SpringArm 碰撞避免 (Ray-AABB 扫射)
- [x] 8.3.3 `transition.rs` — CameraTransition (5 种缓动曲线)
- [x] 8.3.4 `look_at.rs` — LookAtTarget 软约束 (屏幕空间死区)

### 8.4 Advanced + Migration (Phase 4)
- [x] 8.4.1 `rail.rs` — CameraRail 轨道相机 (Linear + Catmull-Rom)
- [x] 8.4.2 Craft 迁移 — OrbitState spawn, camera_fx.rs 更新
- [x] 8.4.3 Billiards 迁移 — CameraMode::Orbit + OrbitState

### 8.5 Engineering (Phase 5)
- [x] 8.5.1 目录结构工程化 — 平铺 → orbit/ effects/ constraints/ 三层
- [x] 8.5.2 文档更新 — README.md + project.md + tasks.md
- [x] 8.5.3 67 测试 + 2 doc-tests, `cargo test --workspace` 全量通过
