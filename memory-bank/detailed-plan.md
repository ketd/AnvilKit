# 📝 AnvilKit 详细开发计划
*v1.1 | Created: 2025-09-24 | Updated: 2025-09-24*
*Π: 🚧INITIALIZING | Ω: 📝PLAN*

## 🎯 项目概览

**AnvilKit** 是一个基于 Rust 的模块化游戏基础设施框架，采用分阶段开发策略，预计开发周期 12-18 个月。

### 🔬 **基于技术研究的优化**
本计划已基于深度技术研究进行优化，整合了 Bevy ECS、wgpu 和 Rapier 的最佳实践和架构模式。

### 📊 开发时间表
```
阶段1: 项目基础设施    [月份 1-2]   ████████░░░░░░░░░░░░
阶段2: 核心引擎开发    [月份 3-8]   ░░░░░░░░████████████░░
阶段3: 开发者工具      [月份 9-12]  ░░░░░░░░░░░░░░░░████░░
阶段4: 跨平台支持      [月份 13-15] ░░░░░░░░░░░░░░░░░░██░░
阶段5: 生态建设        [月份 16-18] ░░░░░░░░░░░░░░░░░░░░██
```

---

## 🏗️ 阶段1: 项目基础设施 (2个月)

### 🎯 目标
建立稳固的项目基础，包括代码结构、开发环境和CI/CD流程。

### 📋 详细任务

#### 1.1 项目结构设计 (1周) 🔬 **基于研究优化**
```
anvilkit/
├── Cargo.toml              # Workspace 配置
├── crates/
│   ├── anvilkit-core/      # 核心类型、数学、时间系统
│   ├── anvilkit-ecs/       # Bevy ECS 封装和扩展
│   ├── anvilkit-render/    # wgpu 渲染引擎 (2D/3D)
│   │   ├── src/
│   │   │   ├── render2d/   # 2D 精灵批处理渲染器
│   │   │   ├── render3d/   # 3D PBR 渲染管线
│   │   │   ├── middleware/ # 中间件渲染模式
│   │   │   └── graph/      # 渲染图系统
│   ├── anvilkit-physics/   # Rapier 物理引擎集成
│   │   ├── src/
│   │   │   ├── physics2d/  # 2D 物理系统
│   │   │   ├── physics3d/  # 3D 物理系统
│   │   │   └── unified/    # 统一物理接口
│   ├── anvilkit-assets/    # 资源系统 (glTF, 纹理等)
│   ├── anvilkit-audio/     # Kira 音频引擎集成
│   ├── anvilkit-input/     # 跨平台输入系统
│   ├── anvilkit-devtools/  # 开发者工具套件
│   └── anvilkit/           # 主 crate 和插件系统
├── examples/               # 分层示例 (基础→高级)
│   ├── basic/              # 基础示例
│   ├── intermediate/       # 中级示例
│   └── advanced/           # 高级示例
├── docs/                   # 文档和教程
├── tools/                  # 开发和构建工具
└── benches/                # 性能基准测试
```

**Cargo.toml 特性配置** 🔬 **基于研究优化**:
```toml
[features]
default = ["2d", "audio", "input"]
full = ["2d", "3d", "physics-2d", "physics-3d", "audio", "devtools"]

# 渲染特性 - 基于 wgpu 中间件模式
2d = ["anvilkit-render/2d", "anvilkit-render/sprite-batching"]
3d = ["anvilkit-render/3d", "anvilkit-render/pbr"]
advanced-3d = ["3d", "shadows", "post-processing", "hdr", "ray-tracing"]

# 物理特性 - 基于 Rapier 双引擎架构
physics-2d = ["anvilkit-physics/rapier2d", "anvilkit-physics/unified"]
physics-3d = ["anvilkit-physics/rapier3d", "anvilkit-physics/unified"]

# 开发工具特性 - 基于 ECS 调试模式
devtools = ["anvilkit-devtools", "anvilkit-ecs/debug", "hot-reload"]
hot-reload = ["anvilkit-assets/hot-reload", "anvilkit-render/shader-reload"]

# 平台特性
web = ["wgpu/webgl", "anvilkit-audio/web"]
mobile = ["anvilkit-render/mobile-optimized", "anvilkit-physics/mobile"]
```

#### 1.2 基础模块创建 (2周)
- **anvilkit-core**: 基础类型、数学、时间系统
- **anvilkit-ecs**: bevy_ecs 封装和扩展
- **anvilkit-windowing**: winit 集成和窗口管理
- **anvilkit-input**: 输入事件处理
- **anvilkit-assets**: 资源加载框架

#### 1.3 开发环境配置 (1周)
- **Rust 工具链**: 配置 rustfmt, clippy, rust-analyzer
- **VSCode 配置**: 调试配置、任务配置、扩展推荐
- **开发脚本**: 构建、测试、文档生成脚本

#### 1.4 CI/CD 流程 (1周)
```yaml
# .github/workflows/ci.yml
name: CI
on: [push, pull_request]
jobs:
  test:
    runs-on: [ubuntu-latest, windows-latest, macos-latest]
    steps:
      - uses: actions/checkout@v3
      - name: Install Rust
        uses: actions-rs/toolchain@v1
      - name: Run tests
        run: cargo test --all-features
      - name: Check formatting
        run: cargo fmt --all -- --check
      - name: Run clippy
        run: cargo clippy --all-features -- -D warnings
```

#### 1.5 文档框架 (1周)
- **README.md**: 项目介绍、快速开始
- **CONTRIBUTING.md**: 贡献指南
- **docs/**: API 文档和教程
- **examples/**: 基础示例代码

### ✅ 验收标准
- [ ] 项目可以成功编译
- [ ] CI/CD 流程正常运行
- [ ] 基础示例可以运行（空白窗口）
- [ ] 文档结构完整

---

## ⚙️ 阶段2: 核心引擎开发 (6个月) 🔬 **基于研究优化**

### 🎯 目标
基于技术研究成果，实现高性能的游戏引擎核心功能，采用数据驱动的ECS架构和现代渲染管线。

### 📋 里程碑式开发

#### M1: "核心地基" (1个月) - **ECS + 窗口系统**
**目标**: 建立基于 Bevy ECS 的数据驱动架构
```rust
// 基于研究的 ECS 架构模式
use anvilkit::prelude::*;

#[derive(Component)]
struct Position { x: f32, y: f32, z: f32 }

#[derive(Component)]
struct Velocity { x: f32, y: f32, z: f32 }

fn main() {
    App::new()
        .add_plugins(CorePlugins)
        .add_systems(Startup, setup_system)
        .add_systems(Update, movement_system)
        .run();
}

fn setup_system(mut commands: Commands) {
    // 创建实体
    commands.spawn((
        Position { x: 0.0, y: 0.0, z: 0.0 },
        Velocity { x: 1.0, y: 0.0, z: 0.0 },
    ));
}

// 高性能的数据驱动系统
fn movement_system(mut query: Query<(&mut Position, &Velocity)>) {
    for (mut position, velocity) in &mut query {
        position.x += velocity.x;
        position.y += velocity.y;
        position.z += velocity.z;
    }
}
```

**技术验证点**:
- ✅ ECS 系统正常运行
- ✅ 组件查询性能达标 (>1M entities/frame)
- ✅ 系统并行执行验证

#### M2: "你好，三角形！" (1.5个月) - **wgpu 渲染管线**
**目标**: 基于 wgpu 中间件模式的 3D 渲染验证
```rust
// 基于研究的中间件渲染架构
use anvilkit::prelude::*;

fn setup_3d_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 3D 透视相机
    commands.spawn(Camera3dBundle {
        camera: Camera {
            projection: Projection::Perspective(PerspectiveProjection {
                fov: 45.0_f32.to_radians(),
                aspect_ratio: 16.0 / 9.0,
                near: 0.1,
                far: 100.0,
            }),
            ..default()
        },
        transform: Transform::from_xyz(0.0, 0.0, 5.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // 基础三角形网格
    commands.spawn(PbrBundle {
        mesh: meshes.add(create_triangle_mesh()),
        material: materials.add(StandardMaterial {
            base_color: Color::RED,
            metallic: 0.0,
            roughness: 0.5,
            ..default()
        }),
        ..default()
    });

    // 环境光
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.3,
    });
}

fn create_triangle_mesh() -> Mesh {
    // 手动创建三角形顶点数据
    let vertices = vec![
        [0.0, 1.0, 0.0],   // 顶点
        [-1.0, -1.0, 0.0], // 左下
        [1.0, -1.0, 0.0],  // 右下
    ];

    let normals = vec![[0.0, 0.0, 1.0]; 3];
    let uvs = vec![[0.5, 1.0], [0.0, 0.0], [1.0, 0.0]];
    let indices = vec![0, 1, 2];

    Mesh::new(PrimitiveTopology::TriangleList)
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_indices(Some(Indices::U32(indices)))
}
```

**技术验证点**:
- ✅ wgpu 渲染管线正常工作
- ✅ 3D 透视投影正确
- ✅ 基础着色器编译和执行
- ✅ 60FPS @ 1080p 性能达标

#### M3: "旋转的猴头" (1.5个月)
**目标**: 资源系统 + PBR 渲染
```rust
fn setup_monkey_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 加载 glTF 模型
    let monkey_handle = asset_server.load("models/monkey.gltf#Scene0");
    
    commands.spawn(SceneBundle {
        scene: monkey_handle,
        transform: Transform::from_rotation(Quat::from_rotation_y(0.1)),
        ..default()
    });
    
    // 光源
    commands.spawn(DirectionalLightBundle::default());
}
```

#### M4: "屏幕上的精灵" (1个月)
**目标**: 2D 渲染系统
```rust
fn setup_2d_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // 2D 相机
    commands.spawn(Camera2dBundle::default());
    
    // 精灵
    commands.spawn(SpriteBundle {
        texture: asset_server.load("sprites/player.png"),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..default()
    });
}
```

#### M5: "滚动的球体" (1个月)
**目标**: 物理引擎集成
```rust
fn setup_physics_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 地面
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 10.0 })),
            material: materials.add(Color::GREEN.into()),
            ..default()
        },
        RigidBody::Fixed,
        Collider::cuboid(5.0, 0.1, 5.0),
    ));
    
    // 球体
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::UVSphere::default())),
            material: materials.add(Color::BLUE.into()),
            transform: Transform::from_xyz(0.0, 5.0, 0.0),
            ..default()
        },
        RigidBody::Dynamic,
        Collider::ball(0.5),
    ));
}
```

### 🔧 技术实现细节

#### 渲染系统架构
```rust
pub struct RenderPlugin {
    pub backend: RenderBackend,
    pub features: RenderFeatures,
}

pub enum RenderBackend {
    Wgpu(WgpuConfig),
}

pub struct RenderFeatures {
    pub pbr: bool,
    pub shadows: bool,
    pub post_processing: bool,
    pub hdr: bool,
}
```

#### 物理系统架构
```rust
pub struct PhysicsPlugin<D: PhysicsDimension> {
    pub gravity: D::Vector,
    pub timestep: f32,
    pub substeps: usize,
}

pub trait PhysicsDimension {
    type Vector;
    type Rotation;
    type RigidBody;
    type Collider;
}
```

### ✅ 验收标准
- [ ] 可以渲染 3D 场景（立方体、光照）
- [ ] 可以加载和显示 glTF 模型
- [ ] 可以渲染 2D 精灵
- [ ] 物理模拟正常工作
- [ ] 音频播放功能正常
- [ ] 性能达标（60FPS @ 1080p）

---

## 🛠️ 阶段3: 开发者工具 (4个月)

### 🎯 目标
构建世界级的开发者工具，提升开发体验。

### 📋 核心工具

#### 3.1 调试控制台 (1.5个月)
```rust
use anvilkit::devtools::*;

fn setup_debug_console(mut commands: Commands) {
    commands.spawn(DebugConsoleBundle {
        console: DebugConsole::new()
            .with_entity_inspector()
            .with_performance_monitor()
            .with_console_commands(),
        ..default()
    });
}
```

**功能特性**:
- 🔍 实体检查器 - 实时查看和修改组件
- 📊 性能监控 - FPS、内存、渲染统计
- 🎛️ 参数调节 - 实时调整游戏参数
- 📝 命令行接口 - 执行调试命令

#### 3.2 热重载系统 (1.5个月)
```rust
App::new()
    .add_plugins(DefaultPlugins)
    .add_plugin(HotReloadPlugin {
        watch_assets: true,
        watch_shaders: true,
        watch_configs: true,
    })
    .run();
```

**支持的热重载**:
- 📦 资源文件 (纹理、模型、音频)
- 🎨 着色器文件 (WGSL)
- ⚙️ 配置文件 (JSON/TOML)
- 📜 脚本文件 (Lua/WASM)

#### 3.3 性能分析器 (1个月)
- CPU 性能分析
- GPU 性能分析  
- 内存使用分析
- 渲染统计分析

### ✅ 验收标准
- [ ] 调试控制台功能完整
- [ ] 热重载系统稳定工作
- [ ] 性能分析器提供有用信息
- [ ] 开发效率显著提升

---

## 🌐 后续阶段概览

### 阶段4: 跨平台支持 (3个月)
- 移动端优化 (Android/iOS)
- Web 平台支持 (WASM)
- 主机平台适配

### 阶段5: 生态建设 (3个月)
- 插件系统架构
- 社区建设和文档
- 示例项目和教程

---

## 📊 风险评估与缓解

### 🚨 高风险项目
1. **3D 渲染复杂性** - 分步实现，先简单后复杂
2. **性能优化挑战** - 持续性能测试和优化
3. **跨平台兼容性** - 早期测试多平台

### 🛡️ 缓解策略
- 严格遵循里程碑开发
- 持续集成和测试
- 社区反馈和迭代改进

---

## 🎯 成功指标

### 技术指标
- ✅ 60FPS @ 1080p 性能
- ✅ < 100MB 内存占用
- ✅ < 3秒 启动时间
- ✅ 90%+ 测试覆盖率

### 社区指标  
- 🌟 1000+ GitHub Stars
- 👥 50+ 活跃贡献者
- 📚 完整的文档和教程
- 🎮 10+ 示例项目
