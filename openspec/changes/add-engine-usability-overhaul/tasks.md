## Phase 1: 渲染抽象层（核心可用性提升）

### 1.1 StandardMaterial 组件
- [ ] 1.1.1 定义 `StandardMaterial` 组件（base_color, metallic, roughness, normal_scale, emissive_factor, textures, blend_mode, cull_mode, bus/category fields）
- [ ] 1.1.2 实现 pipeline 缓存 HashMap<PipelineKey, PipelineHandle>，key = (vertex_format, blend_mode, cull_mode)
- [ ] 1.1.3 实现 bind group 缓存 HashMap<MaterialId, BindGroup>，lazy 创建 + dirty flag 重建
- [ ] 1.1.4 实现 1x1 fallback textures（white, default_normal, black, white_ao）用于无纹理材质
- [ ] 1.1.5 添加 StandardMaterial 单元测试（默认值、pipeline key 生成、dirty 检测）

### 1.2 MeshHandle 组件 + 自动提取
- [ ] 1.2.1 定义 `MeshHandle` 组件（引用 RenderAssets 中的 mesh）
- [ ] 1.2.2 修改 `render_extract_system` — 查询 (MeshHandle, StandardMaterial, GlobalTransform) 三元组自动生成 DrawCommand
- [ ] 1.2.3 补充 MeshHandle 跳过无材质实体的测试

### 1.3 SceneRenderer 编排层
- [ ] 1.3.1 提取 `render_ecs()` 中的 pass 编排逻辑到 `SceneRenderer` struct
- [ ] 1.3.2 实现自动 resize — 监听 winit Resized 事件，重建所有 size-dependent 资源
- [ ] 1.3.3 实现 uniform batch write — 所有 draw commands 的 uniform 数据一次性写入 buffer
- [ ] 1.3.4 实现后处理链调度 — 根据 PostProcessSettings 动态插入/跳过 pass

### 1.4 PostProcessSettings 资源
- [ ] 1.4.1 定义 `PostProcessSettings` 资源（ssao, dof, motion_blur, bloom, color_grading 各为 Option）
- [ ] 1.4.2 接入 SSAO pass 到 SceneRenderer（scene pass 后、tonemap 前）
- [ ] 1.4.3 接入 DOF pass
- [ ] 1.4.4 接入 Motion Blur pass
- [ ] 1.4.5 接入 Color Grading pass（bloom 之后、tonemap 之前）
- [ ] 1.4.6 修改 tonemap shader 接受可选 AO texture input
- [ ] 1.4.7 添加 PostProcessSettings 开关测试（全禁用、全启用、部分启用）

### 1.5 DefaultPlugins + Auto 插件
- [ ] 1.5.1 实现 `AutoInputPlugin` — winit keyboard/mouse 事件自动转发到 InputState + end_frame
- [ ] 1.5.2 实现 `AutoDeltaTimePlugin` — Instant::elapsed → DeltaTime (clamped) + Time.update()
- [ ] 1.5.3 实现 `DefaultPlugins` PluginGroup（ECS + Render + Input + DeltaTime + Audio）
- [ ] 1.5.4 添加 `DefaultPlugins::new().with_window(config)` builder 方法
- [ ] 1.5.5 添加 AutoInputPlugin + AutoDeltaTimePlugin 单元测试

### 1.6 便捷方法
- [ ] 1.6.1 实现 `MeshData::to_pbr_vertices() -> Vec<PbrVertex>`
- [ ] 1.6.2 实现 `generate_sphere/box/plane` 返回 (MeshData, MeshHandle) 的便捷工厂方法
- [ ] 1.6.3 添加便捷方法单元测试

## Phase 2: ECS 基础设施

### 2.1 事件系统迁移
- [ ] 2.1.1 `PhysicsPlugin::build()` 中添加 `app.add_event::<CollisionEvent>()`
- [ ] 2.1.2 `collision_detection_system` 改用 `EventWriter<CollisionEvent>` 替代 `ResMut<CollisionEvents>`
- [ ] 2.1.3 `NetworkPlugin::build()` 中添加 `app.add_event::<NetworkEvent>()`
- [ ] 2.1.4 删除 `CollisionEvents` 和 `NetworkEvents` Resource 类型及其手动清除系统
- [ ] 2.1.5 更新 Billiards 碰撞事件消费代码
- [ ] 2.1.6 添加事件系统单元测试（写入、多读者、过期）

### 2.2 游戏状态机
- [ ] 2.2.1 在 prelude 中 re-export Bevy `States`, `NextState`, `OnEnter`, `OnExit`, `in_state`
- [ ] 2.2.2 在 `App` 上添加 `init_state::<S>()` 便捷方法
- [ ] 2.2.3 添加 GameState 使用示例和文档测试
- [ ] 2.2.4 添加状态转换单元测试（OnEnter/OnExit 执行验证）

### 2.3 FixedUpdate 调度
- [ ] 2.3.1 新增 `AnvilKitSchedule::FixedUpdate` 变体
- [ ] 2.3.2 在 `App` 中添加时间累加器（accumulated_time + fixed_timestep 配置）
- [ ] 2.3.3 修改 `App::update()` — 在 PreUpdate 之后、Update 之前运行 FixedUpdate 累加循环
- [ ] 2.3.4 将 `PhysicsPlugin` 的系统从 Update 迁移到 FixedUpdate
- [ ] 2.3.5 将 `RapierPhysicsPlugin` 的系统从 Update 迁移到 FixedUpdate
- [ ] 2.3.6 添加 FixedUpdate 累加器单元测试（正常帧、长帧追赶、零帧跳过）

### 2.4 SystemSet 排序
- [ ] 2.4.1 在 `AnvilKitEcsPlugin::build()` 中调用 `configure_sets` 设置 10 个 set 的相对顺序
- [ ] 2.4.2 添加系统排序集成测试（验证 Input 在 Physics 之前执行）

### 2.5 Scene 序列化扩展
- [ ] 2.5.1 新增 `SerializableRegistry` 资源 — 存储 (TypeId → serialize/deserialize fn) 映射
- [ ] 2.5.2 新增 `app.register_serializable::<T>()` 方法
- [ ] 2.5.3 修改 `SceneSerializer::save` — 遍历 registry 序列化所有已注册组件
- [ ] 2.5.4 修改 `SceneSerializer::load` — 反序列化并插入所有已注册组件
- [ ] 2.5.5 序列化/反序列化保留 Parent/Children 层级关系
- [ ] 2.5.6 添加 round-trip 测试（Transform + Name + 自定义组件 + 层级）

### 2.6 层级递归销毁
- [ ] 2.6.1 实现 `TransformHierarchy::despawn_recursive(commands, entity)` — BFS/DFS 遍历 Children 递归 despawn
- [ ] 2.6.2 添加 despawn_recursive 测试（深层嵌套、叶子节点、单实体）

## Phase 3: Audio 补完

### 3.1 播放功能修复
- [ ] 3.1.1 修改 `audio_playback_system` — 读取 `AudioSource.looping` 并使用 `rodio::source::Repeat` 或 `Sink::append` 循环
- [ ] 3.1.2 修改 `audio_playback_system` — 使用 `Sink::set_speed(source.pitch)` 应用音高
- [ ] 3.1.3 添加 looping/pitch 单元测试

### 3.2 空间音频
- [ ] 3.2.1 新增 `spatial_audio_system` — 查询 AudioListener + AudioSource 的 Transform，计算距离衰减
- [ ] 3.2.2 衰减公式：`volume = src_vol * max(0, 1 - dist / spatial_range) * bus_vol * master_vol`
- [ ] 3.2.3 非 spatial 源跳过距离计算
- [ ] 3.2.4 添加空间音频距离衰减单元测试

### 3.3 音频 Bus
- [ ] 3.3.1 新增 `AudioBus` 资源（master, music, sfx, voice 四个 f32 音量）
- [ ] 3.3.2 新增 `AudioBusCategory` 枚举 + `AudioSource.bus` 字段（默认 SFX）
- [ ] 3.3.3 修改 volume 计算链：source × category × master × spatial
- [ ] 3.3.4 添加 AudioBus 单元测试

### 3.4 AssetServer 集成
- [ ] 3.4.1 修改 `audio_playback_system` — 通过 AssetServer 加载音频文件替代直接 File::open
- [ ] 3.4.2 添加 AudioAsset 类型到 AssetStorage
- [ ] 3.4.3 添加集成测试

## Phase 4: Asset Pipeline v2

### 4.1 内存缓存
- [ ] 4.1.1 在 `AssetServer` 中添加 `loaded_cache: HashMap<AssetId, LoadState>` 缓存层
- [ ] 4.1.2 `load_async` 命中缓存时直接返回现有 handle，不 dispatch I/O
- [ ] 4.1.3 新增 `AssetServer::reload(id)` 强制重新加载
- [ ] 4.1.4 添加缓存命中/失效单元测试

### 4.2 热重载集成
- [ ] 4.2.1 `AssetServer` 内部持有 `Option<FileWatcher>`（hot-reload feature 启用时创建）
- [ ] 4.2.2 维护 `path_to_id: HashMap<PathBuf, AssetId>` 反向映射
- [ ] 4.2.3 `process_completed()` 中同时调用 `watcher.poll_changes()` 并触发 reload
- [ ] 4.2.4 添加热重载集成测试

### 4.3 glTF 动画提取
- [ ] 4.3.1 实现 `load_gltf_animations(path) -> Result<Vec<(Skeleton, Vec<AnimationClip>)>>`
- [ ] 4.3.2 提取 skin 数据：joint nodes + inverse bind matrices
- [ ] 4.3.3 提取 animation channels：translation/rotation/scale 按 joint 索引
- [ ] 4.3.4 支持 Step/Linear/CubicSpline 插值模式
- [ ] 4.3.5 添加 glTF 动画加载单元测试

### 4.4 独立纹理加载
- [ ] 4.4.1 实现 `load_texture(path) -> Result<TextureData>` 使用 `image` crate
- [ ] 4.4.2 支持 PNG/JPEG → RGBA8 转换
- [ ] 4.4.3 添加纹理加载单元测试

### 4.5 自动卸载
- [ ] 4.5.1 `AssetHandle<T>` 添加 weak reference 检测（Arc::strong_count）
- [ ] 4.5.2 实现 `process_unloads()` — 遍历 storage，移除 strong_count == 1 的条目（仅 storage 自身持有）
- [ ] 4.5.3 在 `process_completed()` 末尾调用 `process_unloads()`
- [ ] 4.5.4 添加自动卸载单元测试

### 4.6 后台解析
- [ ] 4.6.1 修改 `load_async` worker — 在 worker thread 中执行 glTF/PNG 解析，返回 parsed data
- [ ] 4.6.2 定义 `ParsedAsset` enum 包装不同资产类型的解析结果
- [ ] 4.6.3 main thread `process_completed` 只做 storage 插入，不做解析

## Phase 5: Input 系统 v2

### 5.1 Gamepad 支持
- [ ] 5.1.1 新增 `GamepadState` 资源（connected gamepads, button states, axis values）
- [ ] 5.1.2 新增 `GamepadButton` 枚举（South/East/West/North/DPad/Shoulders/Triggers/Sticks/Start/Select）
- [ ] 5.1.3 新增 `GamepadAxis` 枚举（LeftStickX/Y, RightStickX/Y, LeftTrigger, RightTrigger）
- [ ] 5.1.4 在 `AutoInputPlugin` 中映射 winit gamepad 事件到 GamepadState
- [ ] 5.1.5 添加 GamepadState 单元测试

### 5.2 轴向输入
- [ ] 5.2.1 新增 `InputAxis` 类型（continuous value [-1, 1] 或 [0, 1]）
- [ ] 5.2.2 `ActionMap` 新增 `bind_axis(action, binding)` 方法
- [ ] 5.2.3 `ActionMap` 新增 `axis_value(action) -> f32` 查询方法
- [ ] 5.2.4 键盘模拟轴：负键 + 正键 → [-1, 0, 1]
- [ ] 5.2.5 添加轴向输入单元测试

### 5.3 ActionMap 性能优化
- [ ] 5.3.1 新增 `ActionId(u32)` 类型
- [ ] 5.3.2 `ActionMap::register_action(name) -> ActionId` 分配索引
- [ ] 5.3.3 内部存储从 `HashMap<String, _>` 改为 `Vec<ActionEntry>` + `HashMap<String, ActionId>` 索引
- [ ] 5.3.4 添加 zero-allocation lookup 基准测试

## Phase 6: 渲染修复

### 6.1 CSM 修复
- [ ] 6.1.1 修改 `events.rs` — cascade 矩阵使用 `camera.fov` 替代 `FRAC_PI_4`
- [ ] 6.1.2 修改 CSM 投影从 `perspective_rh` 到 `perspective_lh` 统一坐标系
- [ ] 6.1.3 添加 CSM FOV 一致性测试

### 6.2 Mipmap 生成
- [ ] 6.2.1 修改 `create_texture()` — 计算 mip_level_count = floor(log2(max(w,h))) + 1
- [ ] 6.2.2 实现 blit chain mipmap generation（逐级从前一级 downsample）
- [ ] 6.2.3 修改 sampler — `mipmap_filter: Linear`
- [ ] 6.2.4 添加 mipmap level count 单元测试

### 6.3 可配置渲染参数
- [ ] 6.3.1 `RenderConfig` 新增 `msaa_samples: u32`（默认 4），`clear_color: [f32; 4]`，`default_cull_mode: CullMode`
- [ ] 6.3.2 `SceneRenderer` 读取 RenderConfig 创建 pipeline 时使用配置值替代硬编码
- [ ] 6.3.3 glTF loader 读取 `doubleSided` 属性设置 cull_mode
- [ ] 6.3.4 添加配置覆盖测试

### 6.4 正交相机
- [ ] 6.4.1 `CameraComponent` 新增 `Projection` 枚举（Perspective { fov, near, far } | Orthographic { left, right, bottom, top, near, far }）
- [ ] 6.4.2 camera_system 根据 Projection 类型计算 view-projection 矩阵
- [ ] 6.4.3 添加正交投影单元测试

### 6.5 多相机
- [ ] 6.5.1 `CameraComponent` 新增 `render_target: RenderTarget` 和 `priority: i32`
- [ ] 6.5.2 `SceneRenderer` 按 priority 排序 active cameras，逐一渲染
- [ ] 6.5.3 `RenderTarget::Texture(handle)` 支持渲染到纹理
- [ ] 6.5.4 添加多相机渲染排序测试

### 6.6 点光/聚光阴影
- [ ] 6.6.1 点光源：创建 cubemap depth texture，6 面各一次 depth-only pass
- [ ] 6.6.2 聚光灯：创建 2D depth texture，perspective projection 匹配 cone angle
- [ ] 6.6.3 修改 PBR shader — 采样点光 cubemap shadow / 聚光 2D shadow
- [ ] 6.6.4 `MAX_SHADOW_LIGHTS` 常量限制有阴影的光源数量（建议 4）

## Phase 7: 清理与迁移

### 7.1 死依赖清理
- [ ] 7.1.1 从 workspace Cargo.toml 移除 `rapier2d`
- [ ] 7.1.2 从 workspace Cargo.toml 移除 `kira`
- [ ] 7.1.3 将 `egui`/`egui-wgpu`/`egui-winit` 移到 `[dev-dependencies]` 或 `dev` feature
- [ ] 7.1.4 运行 `cargo check --workspace` 确认零错误

### 7.2 Umbrella crate 统一
- [ ] 7.2.1 扩展 `anvilkit/src/lib.rs` prelude — re-export 所有常用类型
- [ ] 7.2.2 迁移所有 example 的 import 到 `use anvilkit::prelude::*`
- [ ] 7.2.3 迁移 billiards/craft 的 import 到 umbrella crate

### 7.3 Example 重构
- [ ] 7.3.1 实现 `DemoApp` 共享脚手架（init_scene, render_frame, resize, capture 共享逻辑）
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
- [ ] 8.1.1 `cargo check --workspace` 零错误
- [ ] 8.1.2 `cargo test --workspace` 全部通过
- [ ] 8.1.3 `cargo clippy --workspace` 零警告
- [ ] 8.1.4 运行 showcase 视觉回归验证
- [ ] 8.1.5 运行 billiards + craft 功能回归验证

### 8.2 网络模块标注
- [ ] 8.2.1 `network.rs` 模块级文档标注为 "Framework only — no transport layer. Provides channel/replication/prediction abstractions for future socket integration."
