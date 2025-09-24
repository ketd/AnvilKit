# AnvilKit - Rust 模块化游戏基础设施

## 1. 项目愿景与设计哲学

### 1.1 项目愿景
AnvilKit 是一个基于 Rust 构建的现代化游戏开发基础设施框架，旨在为开发者提供一套优雅、高性能且可自由组合的核心工具集。它支持无缝构建 2D 和 3D 游戏，同时保持对整个技术栈的完全透明和深度控制。

### 1.2 核心设计哲学

#### 统一但非均一 (Unified, not Uniform)
- 提供统一的顶层 API（如 `Transform`, `AssetServer`）处理 2D 和 3D 场景
- 底层为 2D 和 3D 提供各自高度优化的渲染和物理管线
- 确保开发者体验的一致性，同时保持性能优化的灵活性

#### 模块化组合与按需编译
- 通过 Cargo feature flags 实现深度模块化（`2d`, `3d`, `physics-2d`, `physics-3d`）
- 支持构建轻量级 2D 应用或功能完备的 3D 应用
- 避免为未使用功能付出编译或运行时成本

#### 拥抱生态系统
- 基于成熟的社区库：`bevy_ecs`, `wgpu`, `rapier`
- 核心价值在于优雅的集成和稳定的抽象层
- 提供符合人体工程学的统一接口

#### 开发者体验优先
- 追求极致的 API 简洁性和清晰的错误信息
- 快速的编译速度和丰富的示例代码
- 无论构建像素风格平台游戏还是低多边形 3D 游戏，都提供一流体验

### 1.3 项目边界 (Anti-Goals)
- **不创造一体化编辑器**：专注于代码优先 (Code-First) 的框架
- **不重复发明 ECS**：`bevy_ecs` 是数据驱动架构的坚实基础
- **不追求照片级真实感**：初期专注于风格化渲染（PBR Low-Poly）

---

## 2. 核心架构与技术栈

### 2.1 架构设计原则

#### 数据驱动的 ECS 架构
- 基于 `bevy_ecs` 构建高性能的实体组件系统
- 支持系统并行执行和自动调度优化
- 提供变更检测和事件系统

#### 现代渲染管线
- 基于 `wgpu` 的跨平台图形抽象
- 中间件渲染模式支持可组合的渲染组件
- 渲染图系统实现灵活的渲染通道组合

#### 统一物理接口
- 条件编译支持 `rapier2d` 和 `rapier3d`
- 维度无关的物理组件设计
- 高性能的 SIMD 优化物理计算

### 2.2 核心模块设计

#### 基础设施模块
- **`anvilkit-core`**: 核心类型、数学库、时间系统
- **`anvilkit-ecs`**: Bevy ECS 封装和扩展
- **`anvilkit-assets`**: 异步资源加载和管理系统

#### 渲染系统模块
- **`anvilkit-render`**: 统一渲染引擎
  - `render2d/`: 2D 精灵批处理渲染器
  - `render3d/`: 3D PBR 渲染管线
  - `middleware/`: 中间件渲染模式
  - `graph/`: 渲染图系统

#### 物理系统模块
- **`anvilkit-physics`**: 可切换的物理引擎
  - `physics2d/`: 2D 物理系统 (rapier2d)
  - `physics3d/`: 3D 物理系统 (rapier3d)
  - `unified/`: 统一物理接口

#### 扩展模块
- **`anvilkit-audio`**: Kira 音频引擎集成
- **`anvilkit-input`**: 跨平台输入系统
- **`anvilkit-devtools`**: 开发者工具套件

### 2.3 技术栈选择

| 模块 | 核心依赖 | 选型理由 |
|------|----------|----------|
| **ECS** | `bevy_ecs` | 社区标杆，性能卓越，人体工程学设计一流 |
| **渲染** | `wgpu` | 现代、安全、跨平台的图形 API 抽象层 |
| **窗口** | `winit` | Rust 生态系统的事实标准 |
| **物理** | `rapier2d/3d` | 功能强大、性能出色的纯 Rust 物理引擎 |
| **3D 模型** | `gltf` | 用于解析 glTF 格式的强大社区库 |
| **音频** | `kira` | 表现力强，专为游戏设计 |
| **数学** | `glam` | 简单、快速，为游戏和图形设计 |

---

## 3. API 设计与开发体验

### 3.1 统一 API 设计

AnvilKit 提供统一的 API 接口，支持在同一应用中混合使用 2D 和 3D 元素：

```rust
use anvilkit::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup_scene)
        .add_systems(Update, (movement_system, physics_system))
        .run();
}

fn setup_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // 3D 透视相机
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 2.0, 5.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // 3D 物体
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(0.8, 0.7, 0.6),
            metallic: 0.0,
            roughness: 0.5,
            ..default()
        }),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    });

    // 光源
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::WHITE,
            illuminance: 10000.0,
        },
        transform: Transform::from_rotation(
            Quat::from_euler(EulerRot::XYZ, -0.5, -0.5, 0.0)
        ),
        ..default()
    });
}
```

### 3.2 特性驱动的模块化

```toml
[features]
default = ["2d", "audio", "input"]
full = ["2d", "3d", "physics-2d", "physics-3d", "audio", "devtools"]

# 渲染特性
2d = ["anvilkit-render/2d", "anvilkit-render/sprite-batching"]
3d = ["anvilkit-render/3d", "anvilkit-render/pbr"]
advanced-3d = ["3d", "shadows", "post-processing", "hdr"]

# 物理特性
physics-2d = ["anvilkit-physics/rapier2d", "anvilkit-physics/unified"]
physics-3d = ["anvilkit-physics/rapier3d", "anvilkit-physics/unified"]

# 开发工具
devtools = ["anvilkit-devtools", "anvilkit-ecs/debug", "hot-reload"]
hot-reload = ["anvilkit-assets/hot-reload", "anvilkit-render/shader-reload"]
```

### 3.3 性能优化设计

#### ECS 性能优化
- 数据局部性优化的组件存储
- 自动并行化的系统执行
- 变更检测减少不必要的计算

#### 渲染性能优化
- 2D 精灵批处理减少绘制调用
- 3D 实例化渲染支持
- 中间件模式的可组合渲染

#### 物理性能优化
- SIMD 优化的碰撞检测
- 空间分割的高效查询
- 休眠系统减少计算开销

---

## 4. 开发里程碑与验收标准

### 里程碑 1: "核心地基" - ECS 与窗口系统
**目标**: 建立基于 Bevy ECS 的数据驱动架构基础
**验收标准**:
- ✅ ECS 系统正常运行，支持组件注册和查询
- ✅ 窗口创建和事件处理正常工作
- ✅ 基础插件系统可以正常加载和运行
- ✅ 性能基准：>1M entities/frame

### 里程碑 2: "你好，三角形！" - 3D 渲染验证
**目标**: 验证基于 wgpu 的 3D 渲染管线
**验收标准**:
- ✅ 成功渲染 3D 几何体（三角形/立方体）
- ✅ 3D 透视相机系统正常工作
- ✅ 基础着色器编译和执行
- ✅ 性能基准：60FPS @ 1080p

### 里程碑 3: "旋转的猴头" - 3D 资源与 PBR
**目标**: 完成 3D 资源加载和基础 PBR 渲染
**验收标准**:
- ✅ 成功加载和渲染 glTF 模型
- ✅ PBR 材质系统正常工作
- ✅ 基础光照模型（环境光 + 平行光）
- ✅ 资源异步加载系统

### 里程碑 4: "屏幕上的精灵" - 2D 渲染系统
**目标**: 在 3D 基础上构建高效的 2D 渲染能力
**验收标准**:
- ✅ 2D 精灵批处理渲染器
- ✅ 2D/3D 混合渲染场景
- ✅ 2D 相机和正交投影
- ✅ Z 轴排序和图层系统

### 里程碑 5: "滚动的球体" - 物理引擎集成
**目标**: 集成 2D 和 3D 物理引擎
**验收标准**:
- ✅ 2D 物理系统（rapier2d）集成
- ✅ 3D 物理系统（rapier3d）集成
- ✅ 统一的物理组件接口
- ✅ 性能基准：1000+ 物理对象 @ 60FPS

### 里程碑 6: "开发者工具" - 调试与性能分析
**目标**: 提供完整的开发者工具套件
**验收标准**:
- ✅ ECS 实体检查器和组件查看器
- ✅ 实时性能监控和基准测试
- ✅ 渲染调试器和着色器热重载
- ✅ 物理调试可视化

---

## 5. 风险评估与缓解策略

### 5.1 技术风险

#### 3D 渲染复杂性
**风险**: 3D 图形学涉及复杂的数学和渲染概念
**缓解策略**:
- 严格遵循里程碑，渐进式开发
- 深入学习 wgpu 官方示例和 Bevy 源码
- 初期不追求高级渲染特性

#### 跨平台兼容性
**风险**: 不同平台的图形 API 和行为差异
**缓解策略**:
- 使用 wgpu 的跨平台抽象
- 建立全面的平台测试流程
- 平台特定的优化代码路径

#### 性能瓶颈
**风险**: 游戏引擎对性能要求极高
**缓解策略**:
- 基于数据的设计模式
- 持续的性能基准测试
- SIMD 和并行优化

### 5.2 项目风险

#### 功能蔓延
**风险**: 总想添加更多功能，偏离核心目标
**缓解策略**:
- 严格遵守里程碑计划
- 完成当前里程碑前不开发下一个功能
- 定期审查项目范围和优先级

#### 时间估算
**风险**: 个人项目对工作量预估过于乐观
**缓解策略**:
- 保持学习和成长的心态
- 将过程本身视为重要回报
- 灵活调整时间表和期望

---

## 6. 成功指标与质量标准

### 6.1 技术指标
- **ECS 性能**: >1M entities @ 60FPS
- **渲染性能**: 60FPS @ 1080p (基础场景)
- **物理性能**: 1000+ 刚体 @ 60FPS
- **内存效率**: <100MB 基础占用
- **编译时间**: <30s 增量编译

### 6.2 开发体验指标
- **API 简洁性**: 最小化样板代码
- **错误信息**: 清晰的编译和运行时错误
- **文档覆盖**: 90%+ API 文档覆盖
- **示例完整性**: 每个功能都有对应示例

### 6.3 生态系统指标
- **社区参与**: GitHub Stars, Issues, PRs
- **第三方集成**: 插件和扩展数量
- **商业采用**: 实际项目使用案例
- **教育价值**: 教程和学习资源

AnvilKit 致力于成为 Rust 游戏开发生态系统中的重要基础设施，为开发者提供现代化、高性能且易于使用的游戏开发工具集。
