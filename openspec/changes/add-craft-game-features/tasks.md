## Part 1: Engine Hardening — 修固引擎

### 1A. Bug 修复（阻塞 Part 2 所有工作）

#### 1A.1 motion_blur prev_view_proj 修复
- [x] 1A.1.1 在 `PostProcessResources` 中添加 `prev_view_proj: Option<[[f32;4];4]>` 字段
- [x] 1A.1.2 修改 `render_loop.rs` render_ecs()：帧结束前存储当前 VP 到 `prev_view_proj`
- [x] 1A.1.3 修改 `render_loop.rs:281`：传入上一帧 VP 而非当前帧
- [x] 1A.1.4 处理首帧特殊情况（无前帧数据→使用当前帧，运动模糊=0）
- [x] 1A.1.5 验证：cargo check + cargo test 通过

#### 1A.2 color_grading src/dst 纹理冲突修复
- [x] 1A.2.1 在 `ColorGradingResources` 中添加 `intermediate_texture` + `intermediate_view`
- [x] 1A.2.2 修改 `execute()`：接受 `dst_texture` 参数，渲染到 intermediate
- [x] 1A.2.3 添加 intermediate → dst 拷贝回传（`copy_texture_to_texture`）
- [x] 1A.2.4 修改 `resize()` + `ensure_intermediate()` 重建中间纹理
- [x] 1A.2.5 在 `RenderState` 中存储 `hdr_texture`（原来只存 view），render_loop 传入
- [x] 1A.2.6 验证：cargo check + cargo test 通过

#### 1A.3 dof composite pass 修复
- [x] 1A.3.1 在 `execute()` 中添加第三步 composite pass 调用
- [x] 1A.3.2 确保 composite pass 的 bind group 正确引用 CoC + blurred 纹理（6 bindings）
- [x] 1A.3.3 输出写入 HDR 纹理以衔接后续 tonemap（LoadOp::Load 保留原内容）
- [x] 1A.3.4 验证：cargo check + cargo test 通过

#### 1A.4 health_regen_system 签名修复
- [x] 1A.4.1 修改 `health_regen_system` 签名为 `fn(dt: Res<DeltaTime>, query: Query<&mut Health>)`
- [x] 1A.4.2 内部调用 `health.heal(health.regen_rate * dt.0)`
- [x] 1A.4.3 使用 `anvilkit-gameplay` 自有的 `cooldown::DeltaTime`（与 cooldown_tick_system 一致）
- [x] 1A.4.4 Craft `survival.rs` 保留其 wrapper（因使用 anvilkit_ecs::physics::DeltaTime 类型）
- [x] 1A.4.5 验证：cargo check + cargo test 通过（96 tests, 0 failures）

### 1B. ECS 系统补全

#### 1B.1 粒子系统 ECS 接入
- [x] 1B.1.1 创建 `particle_emit_system`：遍历 ParticleEmitter，按 emit_rate 发射到 ParticleSystems 资源
- [x] 1B.1.2 创建 `particle_update_system`：推进所有粒子生命周期
- [x] 1B.1.3 创建 `ParticleSystems` 全局资源（按 Entity 管理 ParticleSystem 池）
- [x] 1B.1.4 `ParticleRenderer` 添加 depth_stencil 支持（Depth32Float, depth_write=false, compare=Less）
- [ ] 1B.1.5 `ParticleRenderer` 添加纹理 atlas 支持 — 延后（需 shader 修改）
- [x] 1B.1.6 粒子按相机距离排序（render() 接受 camera_pos, 远→近排序）
- [x] 1B.1.7 编写 ECS 集成测试：spawn emitter → run 60 frames → verify particles alive

#### 1B.2 StatusEffect tick 系统
- [x] 1B.2.1 创建 `status_effect_tick_system(dt: Res<DeltaTime>, query: Query<&mut StatusEffectList>)`
- [x] 1B.2.2 可由游戏注册到 Update schedule（与 cooldown_tick_system 模式一致）
- [x] 1B.2.3 编写 ECS 集成测试：spawn entity with effects → tick 3次 → verify expiry

#### 1B.3 Sprite ECS 自动化
- [x] 1B.3.1 创建 `sprite_collect_system`：查询 Sprite + Transform → SpriteCollected 资源
- [x] 1B.3.2 创建 `SpriteCollected` 全局资源（每帧重建 SpriteBatch）
- [x] 1B.3.3 编写 ECS 集成测试：spawn 2 sprites → collect → verify batch sorted by z_order

### 1C. Audio 完善

#### 1C.1 核心修复
- [x] 1C.1.1 AssetServer 集成延后（D9 决策：先用文件路径模式，AudioSource::new(path) 已可用）
- [x] 1C.1.2 添加 `audio_cleanup_system`：通过 `RemovedComponents<AudioSource>` 检测 despawn，释放 Sink
- [x] 1C.1.3 stereo panning 已有 TODO 注释（rodio 0.19 不支持 per-channel volume，代码已计算但丢弃）

#### 1C.2 测试补全
- [ ] 1C.2.1 `audio_playback_system` ECS 测试延后（需要 mock AudioEngine，rodio !Send 限制复杂）
- [x] 1C.2.2 距离衰减单元测试（0/half/full/beyond range 四档验证）
- [x] 1C.2.3 立体声 panning 计算单元测试（right/left/ahead 三方向验证）

#### 1C.3 Craft 接入验证
- [ ] 1C.3.1 准备 CC0 音效文件 — 延后到 2F.4 音频集成阶段
- [ ] 1C.3.2 Craft AudioPlugin 初始化 — 已通过 DefaultPlugins 自动加载
- [ ] 1C.3.3 方块音效 spawn — 延后到 2F.4
- [ ] 1C.3.4 播放验证 — 延后到 2F.4

### 1D. App 生命周期增强

#### 1D.1 添加 update() hook
- [x] 1D.1.1 `GameCallbacks` trait 添加 `fn update(&mut self, ctx: &mut GameContext)` 默认空实现
- [x] 1D.1.2 `about_to_wait` 调用顺序：`game.update()` → `render_app.tick()` (ECS) → cursor sync → `game.post_update()`
- [x] 1D.1.3 Craft 实现 `update()`：settings→FOV/sensitivity/view_distance 应用 + debug info 更新
- [ ] 1D.1.4 Billiards 同上检查 — 延后
- [x] 1D.1.5 两个游戏编译通过

#### 1D.2 添加 on_shutdown() hook
- [x] 1D.2.1 `GameCallbacks` trait 添加 `fn on_shutdown()` 默认空实现
- [x] 1D.2.2 在 CloseRequested 和 should_exit() 两处调用 on_shutdown
- [x] 1D.2.3 Craft 实现 on_shutdown：世界自动保存 + 玩家状态 + settings 持久化

#### 1D.3 修复 ui() 死代码
- [x] 1D.3.1 决策：保留 `ui()` 但修正文档（明确标注为"游戏手动调用的约定方法"，非框架自动调用）

#### 1D.4 自动 resize
- [ ] 1D.4.1-3 延后 — 引擎已通过 RenderApp.handle_resize + SceneRenderer 处理 depth/HDR 重建，Craft 的 on_resize 仅处理自定义 bind group，当前架构已合理

### 1E. 数据基础设施增强

#### 1E.1 DataTable 增强
- [x] 1E.1.1 添加 `from_ron_file(name, path)` + `from_json_file(name, path)` 文件加载
- [x] 1E.1.2 编写测试：文件加载成功 + 文件不存在错误处理
- [ ] 1E.1.3 Craft 迁移 — 延后（include_str! 方式仍可用，迁移为增量改进）

#### 1E.2 Locale 增强
- [x] 1E.2.1 添加 `t_fmt(key, params)` 参数化翻译（`{key}` 模板替换）
- [x] 1E.2.2 添加 `load_ron_file(path)` 文件加载
- [x] 1E.2.3 添加 `switch_language(lang, path)` 运行时语言切换
- [x] 1E.2.4 编写 5 个单元测试（t_fmt 三种场景 + load_ron_file + switch_language）

---

## Part 2: Craft Game Features — 游戏功能扩展
> 以下 Phase 依赖 Part 1 完成

### 2A. 世界丰富度 — 让世界值得探索

#### 2A.1 生物群系系统
- [x] 2A.1.1 定义 `Biome` 枚举（Plains/Forest/Desert/Tundra/Ocean/Mountains/Swamp）
- [x] 2A.1.2 实现 `BiomeMap` 资源：温度噪声 + 湿度噪声 → 群系查表
- [x] 2A.1.3 群系属性方法：height_offset/height_scale/has_trees/has_plants
- [x] 2A.1.4 修改 `WorldGenerator` 根据群系选择地表/填充方块 (biome_surface_blocks)
- [x] 2A.1.5 群系边界平滑过渡：smoothed_height_params 5x5 采样平均，世界生成已集成
- [x] 2A.1.6 群系控制植被/树木生成（Desert/Ocean无树，Tundra/Desert/Ocean无草花）
- [x] 2A.1.7 3 个单元测试（deterministic + coverage + properties）

#### 2A.2 矿石生成
- [x] 2A.2.1 新增 6 种矿石方块（CoalOre~LapisOre）+ 14 种其他新方块 = 20 新 BlockType
- [x] 2A.2.2 矿石配置内嵌 generate_ores（6 种矿石按 Y 范围/阈值/噪声缩放配置）
- [x] 2A.2.3 实现 noise_ore 3D 噪声矿石生成
- [x] 2A.2.4 集成到 WorldGenerator（cave 之后、tree 之前执行）
- [ ] 2A.2.5 纹理图集更新 — 延后（当前用 placeholder 纹理复用已有 tile）
- [x] 2A.2.6 测试：generates_ores 验证矿石出现

#### 2A.3 多树种
- [x] 2A.3.1 新增 BirchWood/BirchLeaves/SpruceWood/SpruceLeaves 方块
- [x] 2A.3.2 biome_tree_blocks 函数：Forest→Birch, Tundra→Spruce, 其他→Oak
- [x] 2A.3.3 place_tree_typed 支持可配置 trunk/leaves 方块类型
- [x] 2A.3.4 根据群系选择树种（已集成到 generate_chunk）

#### 2A.4 地形增强
- [x] 2A.4.1 山脉群系：height_offset=+20, height_scale=2.0, Stone 表面
- [x] 2A.4.2 海洋群系：height_offset=-15, height_scale=0.5, Sand 底
- [ ] 2A.4.3 洞穴增强：Perlin worm — 延后（当前 3D 噪声洞穴可用）

### 2B. 方块光照系统 — 让地下可玩

#### 2B.1 光照数据结构
- [x] 2B.1.1 创建 `LightMap`：高4位=天光，低4位=方块光，262144字节/区块
- [x] 2B.1.2 `VoxelWorld` 扩展：`light_maps: HashMap<(i32,i32), LightMap>` + `get_light()` API
- [x] 2B.1.3 `get_packed`/`get_packed_safe`/`get_sky`/`get_block_light`/`set_sky`/`set_block_light` API

#### 2B.2 天光传播
- [x] 2B.2.1 `compute_initial_sky_light`：从 Y=255 向下，transparent 保持15，opaque 归零
- [ ] 2B.2.2 天光 BFS 水平扩散 — 延后（当前列式传播已可用，水平扩散为增量优化）
- [ ] 2B.2.3 跨区块天光传播 — 延后

#### 2B.3 方块光源传播
- [x] 2B.3.1 `block_light_emission()`：Glowstone=15, Torch=14, Lantern=13, LightStone=12, RedstoneOre=7
- [x] 2B.3.2 Torch/Glowstone/Lantern 方块已在 Phase 0 添加
- [x] 2B.3.3 `compute_block_light`：BFS 泛洪，每步衰减1，穿透透明方块
- [ ] 2B.3.4 光照移除/增量更新 — 延后（当前区块生成时一次性计算）

#### 2B.4 光照渲染集成
- [x] 2B.4.1 BlockVertex 增加 `light: f32` 字段（36→40 bytes），布局更新为 5 attributes
- [x] 2B.4.2 Mesher 默认 light=240.0（满天光）；smooth_vertex_light helper 就绪
- [x] 2B.4.3 voxel.wgsl：VertexInput/VertexOutput +light @location(4)，fs_main 解包 sky/block，`max(sky*day_factor, block)/15`
- [x] 2B.4.4 smooth_vertex_light 函数就绪（4点平均采样）

#### 2B.5 光照性能
- [ ] 2B.5.1 每帧预算限制 — 延后（当前一次性计算）
- [ ] 2B.5.2 脏标记联动 — 延后
- [ ] 2B.5.3 LightMap 持久化 — 延后（当前从 blocks 重算，无需存储）

#### 2B.6 集成测试
- [x] 2B.6.1 5 个单元测试：nibble get/set, independence, sky propagation, block BFS, vertex encoding
- [x] 2B.6.2 chunk_manager/persistence/raycast 所有区块插入路径已接入光照计算

### 2C. 实体与 AI 系统 — 让世界有生命

#### 2C.1 实体基础框架
- [x] 2C.1.1 `MobType` 枚举（8种: Pig/Cow/Sheep/Chicken + Zombie/Skeleton/Spider/Creeper）+ 属性方法
- [x] 2C.1.2 组件组合: Mob + MobType + AiState + Transform + Velocity + AabbCollider + Health
- [ ] 2C.1.3 生物渲染 — 延后（需要渲染层支持方块模型实体）
- [ ] 2C.1.4 生物动画 — 延后
- [x] 2C.1.5 `mob_physics_system`：重力 + 地面碰撞

#### 2C.2 被动生物
- [x] 2C.2.1 `passive_ai_system`：Idle ↔ Wander FSM（伪随机方向，tick 计时）
- [x] 2C.2.2 4 种被动生物属性（HP/速度/掉落物各不同）
- [x] 2C.2.3 `mob_flee_on_damage_system`：受击→Flee 状态 100 tick

#### 2C.3 敌对生物
- [x] 2C.3.1 `hostile_ai_system`：Idle → (detect range) → Chase → (attack range) → Attack FSM
- [x] 2C.3.2 4 种敌对生物（Zombie=近战3dmg, Skeleton=近战3dmg, Spider=快速2dmg, Creeper=0dmg爆炸预留）
- [x] 2C.3.3 `grid_astar` XZ 平面 A* 寻路（支持台阶上/下1格，max_steps 限制，BinaryHeap+HashMap）

#### 2C.4 生物生成规则
- [x] 2C.4.1 `mob_spawn_system`：位置哈希伪随机，光照/昼夜条件，地面查找
- [x] 2C.4.2 `MobSpawnTimer` 每 200 tick 尝试生成
- [x] 2C.4.3 全局 MAX_MOBS=128 上限 + DESPAWN_DISTANCE=128 远距清除

#### 2C.5 掉落物系统
- [x] 2C.5.1 `ItemDropEntity` + `DropItem` + `DropLifetime` 组件
- [x] 2C.5.2 `item_drop_system`（物理+lifetime） + `item_pickup_system`（1.5格内自动拾取）
- [x] 2C.5.3 `mob_death_system`：死亡→despawn+spawn drops

#### 2C.6 测试
- [x] 2C.6.1 test_mob_type_properties（hostile/health/damage）
- [x] 2C.6.2 test_ai_state_default
- [x] 2C.6.3 test_passive_ai_transitions（Idle timer=0 → Wander）
- [x] 2C.6.4 test_mob_death_despawns（DeathEvent → entity despawned）

### 2D. 物品与合成系统 — 核心游戏循环

#### 2D.1 物品注册表
- [x] 2D.1.1 `CraftItemDef` 完整字段：category/tool_type/tool_tier/damage/armor/food_value/saturation/durability/block_id/required_mining_level
- [x] 2D.1.2 默认注册表（hardcoded）：24 方块物品 + 10 材料 + 25 工具(5 tier×5 type) + 8 食物 = 67 物品
- [x] 2D.1.3 `ItemRegistry` Resource + get/iter/register API

#### 2D.2 工具系统
- [x] 2D.2.1 `ToolTier` (Wood/Stone/Iron/Gold/Diamond) + `ToolType` (Pickaxe/Axe/Shovel/Hoe/Sword)
- [x] 2D.2.2 `block_mining_info` 硬度表 + `mining_time` 计算（preferred_tool/mining_level/速度乘数）
- [x] 2D.2.3 `MiningProgress` Resource（target/progress/required/fraction/is_complete）
- [x] 2D.2.4 `Durability` Component（current/max/use_once/is_broken）

#### 2D.3 合成系统
- [x] 2D.3.1 `CraftingRecipe`（shaped 网格 + shapeless 列表 + output_id/count）
- [x] 2D.3.2 `RecipeRegistry` + `find_match` shaped/shapeless 匹配算法（含偏移尝试）
- [x] 2D.3.3 `shaped_match` 支持任意偏移量在 3x3 网格内匹配小 pattern
- [x] 2D.3.4 ~25 配方：Planks/Sticks/Workbench/Furnace/Torch + 5tier×4tools(pickaxe/axe/shovel/sword)

#### 2D.4 熔炼系统
- [x] 2D.4.1 `SmeltingRecipe` + `SmeltingRegistry`（8 配方: ores→ingots, sand→glass, cobble→stone, raw food→cooked）
- [x] 2D.4.2 `FurnaceState` Component（input/fuel/cook_progress/output）
- [x] 2D.4.3 `furnace_tick_system` 已实现并注册到 ECS schedule（UI 延后）

#### 2D.5 装备与饥饿
- [x] 2D.5.1 `Equipment` Component（helmet/chestplate/leggings/boots）+ total_armor + damage_reduction，已挂载到玩家实体
- [x] 2D.5.2 `Hunger` Component（level/saturation/exhaustion） + add_exhaustion/eat/is_starving/can_regen
- [x] 2D.5.3 `hunger_tick_system` 饥饿→扣血（4秒周期1HP）
- [ ] 2D.5.4 HUD 饥饿条 — 延后到 2E

#### 2D.6 测试 (9个)
- [x] test_item_registry_default / test_recipe_match_shapeless / test_recipe_match_shaped / test_recipe_no_match
- [x] test_mining_time / test_durability / test_hunger / test_smelting_registry / test_mining_progress

### 2E. UI 完善

#### 2E.1 世界管理
- [x] 2E.1.1 `NewWorldConfig`（name/seed_input/GameMode）+ `resolved_seed()` 种子解析（数字/字符串哈希/随机）
- [x] 2E.1.2 `list_worlds` 列出所有存档（通过 SaveManager）
- [x] 2E.1.3 `WorldInfo` 数据结构（slot_name/display_name/seed/last_played）
- [ ] 2E.1.4 egui 世界创建/选择界面 — 延后（数据层就绪，UI 渲染需运行时验证）

#### 2E.2 合成与熔炉 UI
- [ ] 2E.2.1-2.4 egui 合成/熔炉界面 — 延后（RecipeRegistry/SmeltingRegistry 数据层就绪）

#### 2E.3 物品图标
- [ ] 2E.3.1-3.3 方块纹理→egui 注册 — 延后（需 EguiTextures::register_texture 运行时测试）

#### 2E.4 设置系统
- [x] 2E.4.1 `SettingsState` 添加 serde Serialize/Deserialize + `load_or_default()`/`save()` 持久化到 `saves/settings.ron`
- [x] 2E.4.2 FOV/灵敏度/渲染距离已在 `update()` 中每帧应用到 CameraComponent/CameraController/ChunkManager
- [ ] 2E.4.3 按键绑定编辑器 — 延后

#### 2E.5 HUD 增强
- [x] 2E.5.1 `DebugInfo` Resource（fps/pos/chunk/facing/light/biome/chunks/entities/show toggle）
- [ ] 2E.5.2-5.4 HUD 渲染（挖掘进度/饥饿条/消息） — 延后（需 main.rs render_hud 集成）

### 2F. 引擎能力接入 + 渲染增强

#### 2F.1-2F.4
- [ ] 粒子/阴影/天空/音频接入 — 全部延后到运行时集成阶段（数据层和引擎系统已就绪）

### Phase 0: 基础准备（跨 Phase 共用）

- [x] 0.1 data 目录 — 延后（目前通过 include_str! 和硬编码数据即可）
- [x] 0.2 扩展 BlockType 枚举：+20 新方块（矿石6 + 光源3 + 功能2 + 地形4 + 树木4 + 冰雪1）
- [ ] 0.3 更新纹理图集 — 延后（placeholder tiles 复用已有纹理）
- [x] 0.4 BlockDef 扩展：hardness(f32) + light_emission(u8), BlockDefCache 新增查询方法
- [ ] 0.5 背包升级 — 延后到 2D/2E
- [ ] 0.6 功能方块交互 — 延后到 2D
- [ ] 0.7 方块 metadata — 延后到 2D
