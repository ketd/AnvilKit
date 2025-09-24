# AnvilKit 🔨

> 一个基于 Rust 的现代化模块化游戏基础设施框架

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)](https://github.com/ketd/AnvilKit)

## 🎯 项目愿景

AnvilKit 致力于为 Rust 游戏开发者提供一套优雅、高性能且可自由组合的核心工具集。它支持无缝构建 2D 和 3D 游戏，同时保持对整个技术栈的完全透明和深度控制。

### ✨ 核心特性

- 🏗️ **统一但非均一架构** - 统一的 API，针对性能优化的底层实现
- 🧩 **模块化组合** - 通过 Cargo features 实现按需编译
- 🚀 **现代化技术栈** - 基于 `bevy_ecs`、`wgpu`、`rapier` 等顶级库
- 🎮 **2D/3D 混合支持** - 在同一项目中无缝使用 2D 和 3D 功能
- 🛠️ **开发者体验优先** - 清晰的 API、丰富的示例、快速编译

## 🏛️ 架构设计

### 核心模块

```
anvilkit/
├── anvilkit-core/      # 核心类型、数学库、时间系统
├── anvilkit-ecs/       # Bevy ECS 封装和扩展
├── anvilkit-render/    # 统一渲染引擎 (2D/3D)
├── anvilkit-physics/   # 可切换物理引擎 (rapier2d/3d)
├── anvilkit-assets/    # 异步资源加载和管理
├── anvilkit-audio/     # Kira 音频引擎集成
├── anvilkit-input/     # 跨平台输入系统
├── anvilkit-devtools/  # 开发者工具套件
└── anvilkit/           # 主 crate 和插件系统
```

### 技术栈

| 模块 | 核心依赖 | 选型理由 |
|------|----------|----------|
| **ECS** | `bevy_ecs` | 社区标杆，性能卓越，人体工程学设计一流 |
| **渲染** | `wgpu` | 现代、安全、跨平台的图形 API 抽象层 |
| **物理** | `rapier2d/3d` | 功能强大、性能出色的纯 Rust 物理引擎 |
| **音频** | `kira` | 表现力强，专为游戏设计 |
| **数学** | `glam` | 简单、快速，为游戏和图形设计 |

## 🚀 快速开始

### 安装依赖

```toml
[dependencies]
anvilkit = { version = "0.1", features = ["default"] }

# 可选特性
# anvilkit = { version = "0.1", features = ["2d", "3d", "physics-2d", "audio"] }
```

### 基础示例

```rust
use anvilkit::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup_scene)
        .add_systems(Update, movement_system)
        .run();
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 3D 相机
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 2.0, 5.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // 3D 立方体
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(0.8, 0.7, 0.6),
            ..default()
        }),
        ..default()
    });

    // 光源
    commands.spawn(DirectionalLightBundle::default());
}

fn movement_system(mut query: Query<&mut Transform, With<Handle<Mesh>>>) {
    for mut transform in &mut query {
        transform.rotate_y(0.01);
    }
}
```

## 🎮 特性配置

AnvilKit 支持通过 Cargo features 进行模块化编译：

```toml
[features]
default = ["2d", "audio", "input"]
full = ["2d", "3d", "physics-2d", "physics-3d", "audio", "devtools"]

# 渲染特性
2d = ["anvilkit-render/2d", "anvilkit-render/sprite-batching"]
3d = ["anvilkit-render/3d", "anvilkit-render/pbr"]
advanced-3d = ["3d", "shadows", "post-processing", "hdr"]

# 物理特性
physics-2d = ["anvilkit-physics/rapier2d"]
physics-3d = ["anvilkit-physics/rapier3d"]

# 开发工具
devtools = ["anvilkit-devtools", "hot-reload"]
```

## 📋 开发路线图

### 当前状态：🚧 开发中

- [x] **项目规划** - 完成技术研究和架构设计
- [x] **PRD 文档** - 完整的产品需求文档
- [ ] **M1: 核心地基** - ECS 系统 + 窗口管理
- [ ] **M2: 你好，三角形！** - 3D 渲染验证
- [ ] **M3: 旋转的猴头** - 3D 资源与 PBR
- [ ] **M4: 屏幕上的精灵** - 2D 渲染系统
- [ ] **M5: 滚动的球体** - 物理引擎集成
- [ ] **M6: 开发者工具** - 调试与性能分析

### 性能目标

- **ECS 性能**: >1M entities @ 60FPS
- **渲染性能**: 60FPS @ 1080p (基础场景)
- **物理性能**: 1000+ 刚体 @ 60FPS
- **编译时间**: <30s 增量编译

## 📚 文档和示例

- 📖 **[产品需求文档](prd.md)** - 完整的项目愿景和技术规范
- 🔬 **[技术研究报告](memory-bank/technical-research.md)** - 深度技术分析
- 📋 **[详细开发计划](memory-bank/detailed-plan.md)** - 具体的实施路线图
- 🚀 **[优化实施计划](memory-bank/optimized-implementation-plan.md)** - 基于研究的优化策略

## 🤝 贡献指南

AnvilKit 是一个开源项目，欢迎社区贡献！

### 开发环境设置

```bash
# 克隆仓库
git clone https://github.com/ketd/AnvilKit.git
cd AnvilKit

# 安装 Rust (如果尚未安装)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 构建项目
cargo build

# 运行测试
cargo test

# 运行示例
cargo run --example basic_3d
```

### 贡献流程

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'feat(core): 添加惊人的新特性'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request

## 📄 许可证

本项目采用双许可证：

- [MIT License](LICENSE-MIT)
- [Apache License 2.0](LICENSE-APACHE)

您可以选择其中任一许可证使用本项目。

## 🙏 致谢

AnvilKit 站在巨人的肩膀上，感谢以下优秀的开源项目：

- [Bevy](https://bevyengine.org/) - 现代化的 Rust 游戏引擎
- [wgpu](https://wgpu.rs/) - 安全、可移植的图形 API
- [Rapier](https://rapier.rs/) - 快速的 2D/3D 物理引擎
- [winit](https://github.com/rust-windowing/winit) - 跨平台窗口创建库

## 📞 联系方式

- **GitHub Issues**: [问题反馈](https://github.com/ketd/AnvilKit/issues)
- **Discussions**: [社区讨论](https://github.com/ketd/AnvilKit/discussions)

---

<div align="center">

**用 Rust 锻造游戏的未来 🔨**

[开始使用](prd.md) • [查看示例](examples/) • [贡献代码](CONTRIBUTING.md)

</div>
