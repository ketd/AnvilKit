## Phase 1: 渲染抽象层（核心可用性提升）

### 1.1 StandardMaterial 组件
- [x] 1.1.1 定义 `StandardMaterial` 组件（base_color, metallic, roughness, normal_scale, emissive_factor）
- [ ] 1.1.2 实现 pipeline 缓存 HashMap<PipelineKey, PipelineHandle>，key = (vertex_format, blend_mode, cull_mode)
- [ ] 1.1.3 实现 bind group 缓存 HashMap<MaterialId, BindGroup>，lazy 创建 + dirty flag 重建
- [x] 1.1.4 实现 1x1 fallback textures（white, default_normal, black, white_ao）用于无纹理材质
- [x] 1.1.5 添加 StandardMaterial 单元测试（默认值、builder 模式）

### 1.2 MeshHandle 组件 + 自动提取
- [x] 1.2.1 定义 `MeshHandle` 组件（已存在于 RenderAssets）
- [x] 1.2.2 修改 `render_extract_system` — 查询 (MeshHandle, StandardMaterial, GlobalTransform) 三元组自动生成 DrawCommand
- [x] 1.2.3 补充 MeshHandle 跳过无材质实体的测试

### 1.3 SceneRenderer 编排层
- [x] 1.3.1 提取 resize 逻辑到 `SceneRenderer::handle_resize()` static method
- [x] 1.3.2 实现自动 resize — 委托给 SceneRenderer 重建所有 size-dependent 资源
- [ ] 1.3.3 实现 uniform batch write — 所有 draw commands 的 uniform 数据一次性写入 buffer
- [x] 1.3.4 实现后处理链调度 — 根据 PostProcessSettings 动态插入/跳过 pass

### 1.4 PostProcessSettings 资源
- [x] 1.4.1 定义 `PostProcessSettings` 资源（ssao, dof, motion_blur, bloom, color_grading 各为 Option）
- [x] 1.4.2 接入 SSAO pass 到 render_ecs（scene pass 后、tonemap 前）
- [x] 1.4.3 接入 DOF pass
- [x] 1.4.4 接入 Motion Blur pass
- [x] 1.4.5 接入 Color Grading pass（bloom 之后、tonemap 之前）
- [ ] 1.4.6 修改 tonemap shader 接受可选 AO texture input
- [x] 1.4.7 添加 PostProcessSettings 开关测试（全禁用、全启用、部分启用）

### 1.5 DefaultPlugins + Auto 插件
- [x] 1.5.1 实现 `AutoInputPlugin` — winit keyboard/mouse 事件自动转发到 InputState + end_frame
- [x] 1.5.2 实现 `AutoDeltaTimePlugin` — Instant::elapsed → DeltaTime (clamped) + Time.update()
- [x] 1.5.3 实现 `DefaultPlugins` PluginGroup（ECS + Render + Audio）
- [x] 1.5.4 添加 `DefaultPlugins::new().with_window(config)` builder 方法
- [x] 1.5.5 添加 DefaultPlugins 单元测试

### 1.6 便捷方法
- [x] 1.6.1 实现 `MeshData::to_pbr_vertices() -> Vec<InterleavedPbrVertex>`
- [x] 1.6.2 实现 `generate_sphere/box/plane` 返回 (MeshData, MeshHandle) 的便捷工厂方法
- [x] 1.6.3 添加便捷方法单元测试

## Phase 2: ECS 基础设施

### 2.1 事件系统迁移
- [x] 2.1.1 `PhysicsPlugin::build()` 中添加 `app.add_event::<CollisionEvent>()`
- [x] 2.1.2 `collision_detection_system` 改用 `EventWriter<CollisionEvent>` 替代 `ResMut<CollisionEvents>`
- [x] 2.1.3 `NetworkPlugin::build()` 中添加 `app.add_event::<NetworkEvent>()`
- [x] 2.1.4 `CollisionEvents` 和 `NetworkEvents` 标记 `#[deprecated]`，手动清除系统移除
- [x] 2.1.5 更新 Billiards 碰撞事件消费代码
- [x] 2.1.6 添加事件系统单元测试（collision detection 使用 EventReader 验证）

### 2.2 游戏状态机
- [x] 2.2.1 自研 `GameState<S>` / `NextGameState<S>` / `in_state()` （bevy_state 不兼容 0.14）
- [x] 2.2.2 实现 `state_transition_system<S>` — PreUpdate 自动处理状态转换
- [x] 2.2.3 添加 GameState 文档和使用示例
- [x] 2.2.4 添加状态转换单元测试（transition 执行 + NextState 自动清除）

### 2.3 FixedUpdate 调度
- [x] 2.3.1 新增 `AnvilKitSchedule::FixedUpdate` 变体
- [x] 2.3.2 在 `App` 中添加时间累加器（accumulated_time + fixed_timestep 配置）
- [x] 2.3.3 修改 `App::update()` — 在 PreUpdate 之后、Update 之前运行 FixedUpdate 累加循环
- [x] 2.3.4 将 `PhysicsPlugin` 的系统从 Update 迁移到 FixedUpdate
- [x] 2.3.5 将 `RapierPhysicsPlugin` 的系统从 Update 迁移到 FixedUpdate
- [x] 2.3.6 添加 FixedUpdate 累加器单元测试

### 2.4 SystemSet 排序
- [x] 2.4.1 在 `AnvilKitEcsPlugin::setup_schedules()` 中配置 10 个 set 链式排序
- [x] 2.4.2 添加系统排序集成测试

### 2.5 Scene 序列化扩展
- [x] 2.5.1 SerializedEntity 扩展 name/tag/parent_index/custom_data 字段
- [x] 2.5.2 新增 `app.register_serializable::<T>()` 方法（类型擦除 registry）
- [x] 2.5.3 修改 `SceneSerializer::save` — 序列化 Transform + Name + Tag + 层级
- [x] 2.5.4 修改 `SceneSerializer::load` — 恢复所有组件 + Parent/Children 层级
- [x] 2.5.5 序列化/反序列化保留 Parent/Children 层级关系
- [x] 2.5.6 添加 round-trip 测试 (serde feature)

### 2.6 层级递归销毁
- [x] 2.6.1 实现 `TransformHierarchy::despawn_recursive(commands, entity)`
- [x] 2.6.2 添加 despawn_recursive 测试

## Phase 3: Audio 补完

### 3.1 播放功能修复
- [x] 3.1.1 修改 `audio_playback_system` — looping 通过 `rodio::Source::repeat_infinite()` 实现
- [x] 3.1.2 修改 `audio_playback_system` — pitch 通过 `Sink::set_speed()` 应用
- [x] 3.1.3 添加 looping/pitch 单元测试

### 3.2 空间音频
- [x] 3.2.1 新增 `spatial_audio_system` — 距离衰减计算
- [x] 3.2.2 衰减公式：`AudioBus::effective_volume()` 计算 source × category × master
- [x] 3.2.3 非 spatial 源跳过距离计算
- [x] 3.2.4 空间音频系统实现（含距离衰减逻辑）

### 3.3 音频 Bus
- [x] 3.3.1 新增 `AudioBus` 资源（master, music, sfx, voice 四个 f32 音量）
- [x] 3.3.2 新增 `AudioBusCategory` 枚举 + `AudioSource.bus` 字段（默认 SFX）
- [x] 3.3.3 实现 `effective_volume()` 计算链：category × master
- [x] 3.3.4 添加 AudioBus 单元测试

### 3.4 AssetServer 集成
- [x] 3.4.1 修改 `audio_playback_system` — 通过 AssetServer 加载音频文件
- [x] 3.4.2 添加 AudioAsset 类型到 AssetStorage
- [x] 3.4.3 添加集成测试

## Phase 4: Asset Pipeline v2

### 4.1 内存缓存
- [x] 4.1.1 在 `AssetServer` 中添加 `loaded_cache: HashMap<AssetId, Arc<Vec<u8>>>` 缓存层
- [x] 4.1.2 `process_completed` 自动缓存加载结果
- [x] 4.1.3 新增 `AssetServer::reload(id)` 强制重新加载 + 缓存失效
- [x] 4.1.4 添加缓存命中/失效单元测试

### 4.2 热重载集成
- [x] 4.2.1 `AssetServer` 内部持有 `Option<FileWatcher>`
- [x] 4.2.2 维护 `id_to_path: HashMap<AssetId, PathBuf>` 反向映射
- [x] 4.2.3 `process_completed()` 中调用 `watcher.poll_changes()` 并触发 reload
- [x] 4.2.4 添加热重载集成测试

### 4.3 glTF 动画提取
- [x] 4.3.1 实现 `load_gltf_animations(path) -> Result<Vec<(Skeleton, Vec<AnimationClip>)>>`
- [x] 4.3.2 提取 skin 数据：joint nodes + inverse bind matrices
- [x] 4.3.3 提取 animation channels：translation/rotation/scale 按 joint 索引
- [x] 4.3.4 支持 Step/Linear/CubicSpline 插值模式
- [x] 4.3.5 添加 glTF 动画加载单元测试

### 4.4 独立纹理加载
- [x] 4.4.1 实现 `load_texture(path) -> Result<TextureData>` 使用 `image` crate
- [x] 4.4.2 支持 PNG/JPEG → RGBA8 转换 + `load_texture_from_memory()`
- [x] 4.4.3 添加纹理加载单元测试（file not found + memory PNG + invalid data）

### 4.5 自动卸载
- [x] 4.5.1 `AssetHandle<T>` 添加 weak reference 检测
- [x] 4.5.2 实现 `process_unloads()`
- [x] 4.5.3 在 `process_completed()` 末尾调用 `process_unloads()`
- [x] 4.5.4 添加自动卸载单元测试

### 4.6 后台解析
- [x] 4.6.1 修改 `load_async` worker — 在 worker thread 中执行解析
- [x] 4.6.2 定义 `ParsedAsset` enum
- [x] 4.6.3 main thread `process_completed` 只做 storage 插入

## Phase 5: Input 系统 v2

### 5.1 Gamepad 支持
- [x] 5.1.1 新增 `GamepadState` 资源（connect/disconnect/button/axis）
- [x] 5.1.2 新增 `GamepadButton` 枚举（16 种按钮）
- [x] 5.1.3 新增 `GamepadAxis` 枚举（6 种轴）
- [x] 5.1.4 在 `AutoInputPlugin` 中映射 winit gamepad 事件到 GamepadState
- [x] 5.1.5 添加 GamepadState 单元测试（connect/button/axis/end_frame）

### 5.2 轴向输入
- [x] 5.2.1 新增 `AxisBinding` 类型
- [x] 5.2.2 `ActionMap` 新增 `bind_axis(action, binding)` 方法
- [x] 5.2.3 `ActionMap` 新增 `axis_value(action) -> f32` 查询方法
- [x] 5.2.4 键盘模拟轴
- [x] 5.2.5 添加轴向输入单元测试

### 5.3 ActionMap 性能优化
- [x] 5.3.1 新增 `ActionId(u32)` 类型
- [x] 5.3.2 `ActionMap::register_action(name) -> ActionId` 分配索引
- [x] 5.3.3 新增 `is_action_active_by_id()` + `action_state_by_id()` 零堆分配查询
- [x] 5.3.4 添加 zero-allocation lookup 基准测试

## Phase 6: 渲染修复

### 6.1 CSM 修复
- [x] 6.1.1 修改 `events.rs` — cascade 矩阵使用 `active_camera.fov_radians` 替代 `FRAC_PI_4`
- [x] 6.1.2 修改 CSM 投影从 `perspective_rh` 到 `perspective_lh` 统一坐标系
- [x] 6.1.3 添加 CSM FOV 一致性测试

### 6.2 Mipmap 生成
- [x] 6.2.1 新增 `compute_mip_levels()` + `create_texture()` 自动计算 mip chain（≥4x4）
- [ ] 6.2.2 实现 blit chain mipmap generation（GPU 逐级 downsample）
- [x] 6.2.3 修改 sampler — `mipmap_filter: Linear`
- [x] 6.2.4 添加 mipmap level count 单元测试

### 6.3 可配置渲染参数
- [x] 6.3.1 `RenderConfig` 新增 `msaa_samples`、`clear_color`、`default_cull_mode`
- [ ] 6.3.2 `SceneRenderer` 读取 RenderConfig 创建 pipeline 时使用配置值
- [x] 6.3.3 glTF loader 读取 `doubleSided` 属性设置 cull_mode
- [x] 6.3.4 添加 RenderConfig 默认值测试

### 6.4 正交相机
- [x] 6.4.1 `CameraComponent` 新增 `Projection` 枚举（Perspective | Orthographic）
- [x] 6.4.2 camera_system 根据 Projection 类型计算 view-projection 矩阵
- [x] 6.4.3 添加正交投影单元测试

### 6.5 多相机
- [x] 6.5.1 `CameraComponent` 新增 `priority: i32` 字段
- [ ] 6.5.2 `SceneRenderer` 按 priority 排序 active cameras
- [ ] 6.5.3 `RenderTarget::Texture(handle)` 支持渲染到纹理
- [x] 6.5.4 添加多相机渲染排序测试

### 6.6 点光/聚光阴影
- [ ] 6.6.1 点光源 cubemap shadow
- [ ] 6.6.2 聚光灯 2D shadow
- [ ] 6.6.3 PBR shader 采样
- [ ] 6.6.4 `MAX_SHADOW_LIGHTS` 常量

## Phase 7: 清理与迁移

### 7.1 死依赖清理
- [x] 7.1.1 从 workspace Cargo.toml 移除 `rapier2d`
- [x] 7.1.2 从 workspace Cargo.toml 移除 `kira`
- [x] 7.1.3 从 workspace Cargo.toml 移除 `egui`/`egui-wgpu`/`egui-winit`
- [x] 7.1.4 运行 `cargo check --workspace` 确认零错误

### 7.2 Umbrella crate 统一
- [x] 7.2.1 扩展 `anvilkit/src/lib.rs` — 添加 DefaultPlugins 模块和 prelude 导出
- [ ] 7.2.2 迁移所有 example 的 import 到 `use anvilkit::prelude::*`
- [ ] 7.2.3 迁移 billiards/craft 的 import 到 umbrella crate

### 7.3 Example 重构
- [ ] 7.3.1 实现 `DemoApp` 共享脚手架
- [ ] 7.3.2 迁移 `hello_ecs.rs` 到新 API（目标 < 30 行）
- [ ] 7.3.3 迁移 `hello_pbr_ecs.rs` 到新 API
- [ ] 7.3.4 迁移 `demo.rs` 到新 API
- [ ] 7.3.5 迁移 `showcase.rs` 到新 API
- [ ] 7.3.6 迁移 `game.rs` 到新 API
- [ ] 7.3.7 迁移 15 个 demo_*.rs 到 DemoApp 脚手架
- [ ] 7.3.8 删除迁移后的冗余代码

### 7.4 游戏迁移
- [ ] 7.4.1 Billiards: 迁移到 DefaultPlugins + StandardMaterial + Events
- [ ] 7.4.2 Craft: 迁移到 DefaultPlugins + AutoDeltaTime + FixedUpdate
- [ ] 7.4.3 验证两个游戏功能不回退

## Phase 8: 验证与文档

### 8.1 全量验证
- [x] 8.1.1 `cargo check --workspace` 零错误
- [x] 8.1.2 `cargo test --workspace` 全部通过（23 测试套件）
- [x] 8.1.3 `cargo clippy` 无新增警告
- [ ] 8.1.4 运行 showcase 视觉回归验证
- [ ] 8.1.5 运行 billiards + craft 功能回归验证

### 8.2 网络模块标注
- [x] 8.2.1 `network.rs` 模块级文档标注为 "框架抽象层，不含传输实现"
