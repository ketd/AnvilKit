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

### 依赖配置

AnvilKit 使用 Cargo workspace，各模块独立引用：

```toml
[dependencies]
anvilkit-core = { path = "crates/anvilkit-core", features = ["bevy_ecs"] }
anvilkit-ecs = { path = "crates/anvilkit-ecs" }
anvilkit-render = { path = "crates/anvilkit-render" }
anvilkit-input = { path = "crates/anvilkit-input" }
```

### 基础示例

```rust
use anvilkit_render::prelude::*;
use anvilkit_ecs::prelude::*;
use anvilkit_ecs::schedule::AnvilKitSchedule;

fn main() {
    let mut app = App::new();
    app.add_plugins(RenderPlugin::default());
    app.add_systems(AnvilKitSchedule::Update, my_system);

    // 通过 RenderApp 驱动 winit 事件循环
    RenderApp::run(app);
}

fn my_system() {
    // 游戏逻辑
}
```

完整 PBR 渲染示例参见 `examples/showcase.rs`。

## 📋 开发路线图

### 当前状态：Phase G 完成 (M12c)

**已完成的里程碑：**
- **M0-M1**: 核心地基 — ECS + 数学/时间系统
- **M2-M3**: 渲染系统 — wgpu 管线 + 3D 渲染验证
- **M4a-M4d**: 资源系统 — glTF 加载 + 纹理 + Blinn-Phong + PBR
- **M5-M6**: ECS 多物体架构 + PBR 统一 + 法线贴图 + HDR + IBL
- **M7**: 多光源 + 阴影 + 完整材质 + MSAA
- **M8**: Frustum Culling + GPU Instancing + 多 Submesh
- **M9**: 场景图 + 资产管线 + 2D 渲染栈
- **M10**: 输入系统 + 物理集成 + 音频集成
- **M11**: 骨骼动画 + UI 系统 + 粒子系统
- **M12**: 调试工具 + 性能分析 + 文档

**示例游戏：**
- `games/craft` — Minecraft 风格体素沙盒（地形生成、方块交互、昼夜循环）
- `games/billiards` — 台球模拟（2D 物理、碰撞、开球规则）

### 性能目标

- **ECS 性能**: >1M entities @ 60FPS
- **渲染性能**: 60FPS @ 1080p (PBR 场景)
- **编译时间**: <30s 增量编译

## 📚 文档和示例

- 📖 **[在线文档](docs/)** — 完整的中英双语文档站点（基于 Fumadocs）
- 🎮 **[Showcase 示例](examples/showcase.rs)** - PBR 全功能演示（DamagedHelmet）
- 🏗️ **[AnvilKit CLI](tools/anvilkit-cli/)** - 项目脚手架工具

### 本地运行文档

```bash
cd docs
pnpm install
pnpm dev
# 访问 http://localhost:3000
```

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
