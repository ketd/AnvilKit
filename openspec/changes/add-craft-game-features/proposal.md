# Change: AnvilKit engine hardening + Craft game feature expansion

## Why
对 AnvilKit 引擎和 Craft 游戏进行全面审计后发现两层问题：

1. **引擎层**：多个已实现模块存在 bug（motion_blur/color_grading/dof）、关键系统缺少 ECS 接入（粒子/StatusEffect）、Audio 未被任何游戏使用、App 生命周期 hook 不完整、DataTable/Locale 功能薄弱。
2. **游戏层**：Craft 缺少核心游戏性（合成系统、生物/AI、方块光照、矿石、物品系统、挖掘进度），且引擎已实现的粒子系统、阴影等能力未接入。

本提案采用 **"先固后扩"** 策略：先修复引擎 bug 和补齐通用能力，再为 Craft 添加游戏功能。

## What Changes

### Part 1: Engine Hardening — 修固引擎

#### 1A. Bug 修复
- **motion_blur**: `prev_view_proj` 始终等于当前帧导致运动模糊为零，需存储上一帧矩阵
- **color_grading**: render_loop 中 src == dst 纹理导致 GPU 未定义行为，需加中间纹理
- **dof**: `execute()` 只执行 2/3 pass，composite pass 未调用
- **health_regen_system**: 签名 `fn(f32)` 无法直接作为 Bevy system，应改为 `Res<DeltaTime>`

#### 1B. ECS 系统补全
- **粒子系统**：添加 `ParticleEmitSystem` + `ParticleUpdateSystem` ECS system，添加深度测试，添加纹理支持
- **StatusEffect**：添加 `status_effect_tick_system`（与 cooldown_tick_system 对称）
- **Sprite**：添加 ECS 自动收集+批处理 system

#### 1C. Audio 完善
- 修复 AssetServer 集成（当前 `audio_playback_system` 忽略 `asset_id`）
- 添加实体 despawn 时 Sink 清理
- 添加 system 级测试
- 在 Craft 中实际接入 Audio（方块音效、环境音）

#### 1D. App 生命周期增强
- 添加 `update()` lifecycle hook（当前游戏被迫在 `render()` 中跑逻辑）
- 添加 `on_shutdown()` hook（安全退出/自动保存）
- 修复 `ui()` callback 死代码（框架声明但从未调用）
- 添加自动 resize 处理（深度/HDR/bind group 重建不应由游戏手动管理）

#### 1E. 数据基础设施增强
- **DataTable**: 添加 `from_file(path)` 文件加载、添加 schema 校验
- **Locale**: 添加参数化字符串 `t("hello", &[("name", "Alice")])`、添加运行时语言切换

### Part 2: Craft Game Features — 游戏功能扩展

#### 2A. 世界丰富度
- 生物群系系统（温度/湿度噪声→6+群系选择→地形/植被/方块变化）
- 矿石生成（煤/铁/金/钻石/红石，按Y轴高度分布）
- 多树种（橡木/桦木/云杉，按群系分布）
- 地形增强（山脉/峡谷/河流/海洋）
- 洞穴系统增强（Perlin worm 隧道式洞穴）

#### 2B. 方块光照系统
- 天光传播（从天空向下，遇不透明方块阻断）
- 方块光源传播（火把/萤石等，BFS 泛洪 15 级衰减）
- 光照与渲染集成（顶点光照）
- 光照增量更新（方块放置/破坏时局部更新）

#### 2C. 实体与 AI 系统
- 实体基础框架（生成/despawn/tick/碰撞）
- 被动生物（猪/牛/羊/鸡 — 简单漫游AI）
- 敌对生物（僵尸/骷髅/蜘蛛/苦力怕 — 追踪/攻击AI）
- 生物生成规则（光照等级/群系/时间）
- 掉落物系统

#### 2D. 物品与合成系统
- 物品注册表（工具/武器/盔甲/食物/材料）
- 工具系统（挖掘速度/耐久/等级）
- 合成系统（配方注册/匹配/工作台UI）
- 熔炼系统（熔炉+燃料+产物）
- 装备系统 + 饥饿系统

#### 2E. UI 完善
- 世界创建/选择界面
- 合成/熔炉 UI
- 设置持久化 + 实际生效
- F3 调试覆盖层
- 挖掘进度条 + 物品图标

#### 2F. 引擎能力接入
- 接入粒子系统（方块碎片/火焰/水花/天气）
- 接入阴影映射（方向光 CSM）
- 星空/月亮渲染
- 音频集成（方块/生物/环境音效）

## Impact
- Affected engine crates: `anvilkit-render`(bug fix), `anvilkit-audio`(完善), `anvilkit-app`(lifecycle), `anvilkit-gameplay`(health_regen fix + StatusEffect tick), `anvilkit-data`(DataTable/Locale增强)
- Affected game code: `games/craft/src/` 大量新模块
- **BREAKING**: `health_regen_system` 签名变更（minor，当前无游戏直接调用）
- **BREAKING**: `GameCallbacks` trait 新增 `update()` + `on_shutdown()` 默认方法（向后兼容）
