# Change: Add open-source game rewrite project to showcase AnvilKit

## Why
AnvilKit 引擎的六项短板已补齐（程序化网格、DeltaTime、射线投射、线段渲染、管线解耦、文字渲染），台球游戏也已完成。需要选择一个 star 数高的开源游戏，用 AnvilKit 从头重写，以验证引擎能力并作为项目 showcase。

## Candidate Games

### 1. fogleman/Craft ⭐ 10,952
- **仓库**: https://github.com/fogleman/Craft
- **语言**: C (OpenGL)
- **类型**: Minecraft 克隆
- **代码量**: src/ 约 20 个文件，核心 main.c ~3000 行，总计 ~160KB
- **功能**: 方块放置/破坏、Perlin 噪声地形生成、第一人称摄像机 (WASD + 鼠标)、多种方块类型 (草/石/木/砖/沙等)、昼夜循环、基本多人联网
- **AnvilKit 适配度**: ⭐⭐⭐
  - ✅ `generate_box()` 天然适合方块渲染
  - ✅ ECS 管理方块实体和 chunk
  - ✅ PBR 材质区分不同方块类型
  - ✅ 键盘+鼠标输入系统
  - ✅ TextRenderer 显示坐标/HUD
  - ⚠️ 需要实现 chunk 合批系统 (将同一 chunk 内的方块面合并为单个网格)
  - ⚠️ 需要实现第一人称摄像机控制
  - ⚠️ 单独渲染每个方块性能不足，必须做 greedy meshing
- **工作量**: 大（约 2000-3000 行 Rust）
- **展示价值**: 极高 — Minecraft 克隆是游戏引擎的经典 benchmark

### 2. fogleman/Minecraft ⭐ 5,403
- **仓库**: https://github.com/fogleman/Minecraft
- **语言**: Python (Pyglet/OpenGL)
- **类型**: 简化版 Minecraft
- **代码量**: 单文件 main.py ~30KB (~800行)
- **功能**: 方块放置/破坏、简单地形生成、第一人称移动、6 种方块类型、基本物理 (重力/跳跃)
- **AnvilKit 适配度**: ⭐⭐⭐⭐
  - ✅ 所有 Craft 的优点
  - ✅ 代码量小，逻辑清晰，容易逐行对应移植
  - ✅ 无网络代码，纯单机
  - ⚠️ 同样需要 chunk 合批
  - ⚠️ Python 渲染逻辑需要翻译为 wgpu 对等实现
- **工作量**: 中（约 1500-2000 行 Rust）
- **展示价值**: 高 — 5.4k 星，知名项目

### 3. EvanBacon/Expo-Crossy-Road ⭐ 1,123
- **仓库**: https://github.com/EvanBacon/Expo-Crossy-Road
- **语言**: TypeScript (Three.js / React Native)
- **类型**: 3D Crossy Road (过马路)
- **代码量**: src/ 约 15+ 模块文件
- **功能**: 方块角色在程序化生成的道路/河流/铁路上跳跃前进、躲避车辆/火车/原木、摄像机跟随、粒子效果、计分
- **AnvilKit 适配度**: ⭐⭐⭐⭐⭐
  - ✅ 全部用 `generate_box()` + `generate_sphere()` 即可渲染所有元素
  - ✅ 碰撞检测（AABB 足够）
  - ✅ 摄像机跟随（简单偏移）
  - ✅ 程序化地形/关卡生成
  - ✅ ParticleRenderer 做碰撞特效
  - ✅ TextRenderer 做计分 HUD
  - ✅ LineRenderer 可做调试辅助
  - ✅ 无需复杂的 chunk 系统或 greedy meshing
- **工作量**: 中（约 1500-2000 行 Rust）
- **展示价值**: 中 — 1.1k 星，但玩法直观有趣

### 4. supertuxkart/stk-code ⭐ 5,104
- **仓库**: https://github.com/supertuxkart/stk-code
- **语言**: C++
- **类型**: 3D 卡丁车赛车
- **代码量**: ~385MB 仓库，数十万行 C++
- **功能**: 完整 3D 赛车游戏，多赛道、道具、AI、多人联网、物理引擎
- **AnvilKit 适配度**: ⭐
  - ❌ 代码量过于庞大，不适合重写
  - ❌ 需要完整物理引擎 (车辆动力学)
  - ❌ 需要复杂的 3D 模型加载 (非程序化)
  - ❌ AI 系统、道具系统、网络系统远超 AnvilKit 当前能力
- **工作量**: 不可行
- **展示价值**: N/A — 无法在合理时间内完成

### 5. Neverball/neverball ⭐ 396
- **仓库**: https://github.com/Neverball/neverball
- **语言**: C (OpenGL)
- **类型**: 3D 滚球过关 (类似 Super Monkey Ball)
- **代码量**: 中等 C 代码库
- **功能**: 通过倾斜平台控制球体滚动、收集金币、到达终点、物理模拟 (重力+摩擦+碰撞)
- **AnvilKit 适配度**: ⭐⭐⭐⭐
  - ✅ `generate_sphere()` 渲染球体
  - ✅ `generate_plane()` / `generate_box()` 构建关卡平台
  - ✅ 物理系统 (重力、摩擦、碰撞) 与台球游戏类似
  - ✅ 键盘/鼠标控制平台倾斜
  - ⚠️ 关卡设计需要数据格式
  - ⚠️ star 数偏低
- **工作量**: 中（约 1500-2000 行 Rust）
- **展示价值**: 中低 — 396 星

## Evaluation Matrix

| 维度 | Craft | Minecraft(Py) | Crossy Road | STK | Neverball |
|------|-------|---------------|-------------|-----|-----------|
| ⭐ Star 数 | 10,952 | 5,403 | 1,123 | 5,104 | 396 |
| 代码复杂度 | 高 | 中 | 中 | 极高 | 中 |
| AnvilKit 适配 | ⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐ | ⭐⭐⭐⭐ |
| 工作量 | 大 | 中 | 中 | 不可行 | 中 |
| 展示价值 | 极高 | 高 | 中 | N/A | 中低 |
| 需要新引擎能力 | chunk系统 | chunk系统 | 无 | 太多 | 关卡格式 |

## Recommendation

**首选: Crossy Road** — AnvilKit 现有能力完全覆盖，无需开发新引擎功能，工作量可控，玩法有趣。

**次选: fogleman/Minecraft** — 星数高，代码量小，但需要先实现 chunk 合批系统。

**挑战选: fogleman/Craft** — 最高星，最具展示价值，但工作量最大。

## What Changes
- 克隆选中的游戏仓库到 `.dev/` 目录作为参考
- 在 `games/` 下新建独立 workspace crate
- 用 AnvilKit 从头实现游戏逻辑和渲染
- 保持模块化代码结构（与 billiards 一致）

## Impact
- Affected specs: render-system, ecs-system, asset-system
- Affected code: `games/` 目录新增 crate
- 无 breaking changes
