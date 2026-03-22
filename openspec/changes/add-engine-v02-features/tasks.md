## Tier 1: 视觉质量飞跃

### 1.1 Bloom 后处理
- [ ] 1.1.1 编写 bloom 提取 shader（亮度阈值 + 降采样）
- [ ] 1.1.2 编写 Gaussian blur shader（水平 + 垂直分离）
- [ ] 1.1.3 实现 4 级 mip chain 降采样 + 升采样
- [ ] 1.1.4 在 HDR pipeline 中集成 bloom pass（tonemap 之前）
- [ ] 1.1.5 添加 `BloomSettings` 资源（threshold, intensity, enabled）
- [ ] 1.1.6 为 Craft 的 HDR pipeline 集成 bloom

### 1.2 SSAO
- [ ] 1.2.1 编写 SSAO shader（hemisphere sampling + noise texture）
- [ ] 1.2.2 实现半分辨率 SSAO 渲染 + 双边模糊上采样
- [ ] 1.2.3 添加 `SsaoSettings` 资源（quality, radius, bias）
- [ ] 1.2.4 在 PBR scene pass 中注入 AO 因子
- [ ] 1.2.5 支持从深度 buffer 重建法线（无 G-buffer 后备方案）

### 1.3 Cascade Shadow Maps
- [ ] 1.3.1 实现视锥体分割算法（对数/线性混合）
- [ ] 1.3.2 创建多级 shadow map texture array
- [ ] 1.3.3 修改 shadow pass 为每级渲染
- [ ] 1.3.4 修改 PBR fragment shader 选择最优 cascade
- [ ] 1.3.5 添加 cascade 间过渡混合

## Tier 2: 架构基础设施

### 2.1 Transform 层级运行时
- [ ] 2.1.1 实现 `transform_propagation_system`（拓扑排序传播）
- [ ] 2.1.2 注册到 `AnvilKitSchedule::PostUpdate`
- [ ] 2.1.3 处理动态 reparent（Parent 变更检测）
- [ ] 2.1.4 在 render_extract_system 中使用 GlobalTransform

### 2.2 场景序列化
- [ ] 2.2.1 为核心组件 derive serde（Transform, RigidBody, Collider, etc.）
- [ ] 2.2.2 实现 `SceneSerializer::save/load`（RON 格式）
- [ ] 2.2.3 添加 `Serializable` marker component
- [ ] 2.2.4 迁移 Craft 的 persistence.rs 到通用方案

### 2.3 异步资源加载
- [ ] 2.3.1 添加后台线程池到 AssetServer
- [ ] 2.3.2 实现 `load_async()` 返回 Handle + LoadState
- [ ] 2.3.3 完成回调机制（LoadState 变更事件）
- [ ] 2.3.4 实现资源依赖追踪和级联卸载

### 2.4 资源热重载
- [ ] 2.4.1 集成 `notify` crate 文件监视
- [ ] 2.4.2 实现 shader 热重载（检测 .wgsl 变更 → 重建 pipeline）
- [ ] 2.4.3 实现纹理热重载（检测 .png/.jpg 变更 → 重新上传 GPU）
- [ ] 2.4.4 通过 feature flag 控制（debug 默认开启，release 默认关闭）

## Tier 3: 游戏性核心

### 3.1 物理引擎运行时
- [ ] 3.1.1 添加 `rapier3d` / `rapier2d` 依赖（feature-gated）
- [ ] 3.1.2 实现 `PhysicsPlugin` + `PhysicsWorld` 资源
- [ ] 3.1.3 实现 `physics_step_system`（ECS Component ↔ Rapier 同步）
- [ ] 3.1.4 实现碰撞事件收集（CollisionEvents 资源）
- [ ] 3.1.5 实现 `PhysicsWorld::raycast()`
- [ ] 3.1.6 实现关节约束（Fixed, Revolute, Prismatic, Spherical）
- [ ] 3.1.7 迁移 Craft 的自定义 AABB 物理到引擎物理

### 3.2 UI 框架
- [ ] 3.2.1 实现 mini-flexbox 布局引擎（row/column, grow/shrink, padding/gap）
- [ ] 3.2.2 实现 UI 事件系统（hit test, click, hover, bubble）
- [ ] 3.2.3 实现 UI 渲染 pass（背景色、边框、文本、图片）
- [ ] 3.2.4 实现基础控件（Button, Label, Panel, ScrollView）
- [ ] 3.2.5 迁移 Craft HUD 到 UI 框架

### 3.3 骨骼动画管线
- [ ] 3.3.1 编写 skinning vertex shader（4 joints per vertex）
- [ ] 3.3.2 实现运行时 AnimationPlayer（播放、循环、速度控制）
- [ ] 3.3.3 实现动画混合（线性插值 blend）
- [ ] 3.3.4 从 glTF 提取 skin/animation 数据到 GPU buffer

## Tier 4: 高级功能

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
