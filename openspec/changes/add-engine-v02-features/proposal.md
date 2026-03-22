# Change: AnvilKit v0.2 — 补齐现代游戏引擎缺失功能

## Why

AnvilKit v0.1 已完成基础渲染管线（PBR + HDR + Shadow + MSAA）、ECS 架构、输入/音频/相机系统。但与 Bevy/Godot/Unity 等现代引擎相比，仍缺少多项关键功能，使得 Craft 等 demo 必须手写大量基础设施代码。本提案系统性地梳理所有缺失项，按优先级分层规划 v0.2 路线图。

## What Changes

### Tier 1 — 视觉质量飞跃（直接影响所有游戏的画面表现）

- **Bloom 后处理** — 基于现有 HDR pipeline，添加亮度提取 → 逐级降采样 → 高斯模糊 → 合成
- **SSAO（屏幕空间环境遮蔽）** — 利用深度+法线 buffer，hemisphere sampling，模糊去噪
- **Cascade Shadow Maps** — 方向光多级阴影，替代当前单级 2048x2048 shadow map

### Tier 2 — 架构基础设施（解锁后续所有功能的前置条件）

- **Transform 层级运行时** — 当前 M9a 有 Parent/Children/GlobalTransform 类型但缺少运行时传播系统
- **场景序列化** — 通用的 ECS 场景 save/load（serde 驱动），替代 Craft 的自定义方案
- **异步资源加载** — AssetServer 加入后台线程加载 + 加载状态回调
- **资源热重载** — 文件监视 + 自动刷新纹理/shader

### Tier 3 — 游戏性核心（制作"真正的游戏"所需）

- **物理引擎运行时** — 当前 M10b 有 RigidBody/Collider 组件但无 rapier 运行时集成
- **UI 框架** — 当前 M11b 有 UiNode/UiStyle 类型但无布局引擎和事件系统。需要：Flexbox 布局计算、事件冒泡、文本输入、渲染到纹理
- **骨骼动画管线** — 当前 M11a 有 Skeleton/AnimationClip 数据结构但无 GPU skinning shader 和运行时播放器

### Tier 4 — 高级功能（锦上添花）

- **AI / 寻路** — NavMesh 生成 + A* 路径规划 + agent steering
- **网络 / 多人** — 状态同步框架 + 输入预测 + 回滚
- **高级后处理** — DOF（景深）、Motion Blur（运动模糊）、Color Grading（LUT 调色）
- **开发工具** — 帧性能分析器（GPU/CPU 时间拆分）、Debug 渲染模式（碰撞体、NavMesh、法线）

## Impact

- Affected specs: `render-system`, `ecs-system`, `asset-system`
- New specs: `render-post-processing`, `render-advanced`, `scene-serialization`, `physics-runtime`, `ui-framework`, `asset-pipeline`, `ai-navigation`, `networking`, `dev-tools`
- Affected code: 所有引擎 crate + 所有 demo game
- **BREAKING**: 无。所有新功能为增量添加，现有 API 不变。
