# AnvilKit 🔨

> 一个基于 Rust 的现代化模块化游戏基础设施框架

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)](https://github.com/ketd/AnvilKit)
[![Docs](https://img.shields.io/badge/docs-anvilkit.io-blue.svg)](https://anvilkit.io)

**[文档](https://anvilkit.io)** | **[快速开始](https://anvilkit.io/zh/docs/getting-started)** | **[示例游戏](https://anvilkit.io/zh/docs/games/billiards)** | **[GitHub Discussions](https://github.com/ketd/AnvilKit/discussions)**

---

## ✨ 核心特性

- 🏗️ **统一但非均一架构** - 统一的 API，针对性能优化的底层实现
- 🧩 **模块化组合** - 通过 Cargo features 实现按需编译
- 🚀 **现代化技术栈** - 基于 `bevy_ecs`、`wgpu`、`rapier` 等顶级库
- 🎮 **2D/3D 混合支持** - 在同一项目中无缝使用 2D 和 3D 功能
- 🛠️ **开发者体验优先** - 清晰的 API、丰富的示例、快速编译

## 🚀 快速开始

```toml
[dependencies]
anvilkit-core = { path = "crates/anvilkit-core", features = ["bevy_ecs"] }
anvilkit-ecs = { path = "crates/anvilkit-ecs" }
anvilkit-render = { path = "crates/anvilkit-render" }
```

```rust
use anvilkit_render::prelude::*;
use anvilkit_ecs::prelude::*;
use anvilkit_ecs::schedule::AnvilKitSchedule;

fn main() {
    let mut app = App::new();
    app.add_plugins(RenderPlugin::default());
    app.add_systems(AnvilKitSchedule::Update, my_system);
    RenderApp::run(app);
}

fn my_system() {
    // 游戏逻辑
}
```

更多内容请查看 [在线文档](https://anvilkit.io/zh/docs/getting-started)。

## 🏛️ 架构

```
anvilkit/
├── anvilkit-core/      # 核心类型、数学库、时间系统
├── anvilkit-ecs/       # Bevy ECS 封装和扩展
├── anvilkit-render/    # 统一渲染引擎 (2D/3D)
├── anvilkit-physics/   # 可切换物理引擎 (rapier2d/3d)
├── anvilkit-assets/    # 异步资源加载和管理
├── anvilkit-audio/     # Kira 音频引擎集成
├── anvilkit-input/     # 跨平台输入系统
├── anvilkit-camera/    # 相机系统
└── tools/anvilkit-cli/ # 项目脚手架 & 代码生成
```

| 模块 | 核心依赖 | 选型理由 |
|------|----------|----------|
| **ECS** | `bevy_ecs` | 社区标杆，性能卓越，人体工程学设计一流 |
| **渲染** | `wgpu` | 现代、安全、跨平台的图形 API 抽象层 |
| **物理** | `rapier2d/3d` | 功能强大、性能出色的纯 Rust 物理引擎 |
| **音频** | `kira` | 表现力强，专为游戏设计 |
| **数学** | `glam` | 简单、快速，为游戏和图形设计 |

## 🎮 示例游戏

- **[Craft](games/craft/)** — Minecraft 风格体素沙盒（地形生成、方块交互、昼夜循环）
- **[Billiards](games/billiards/)** — 台球模拟（2D 物理、碰撞、开球规则）

## 🤝 贡献

```bash
git clone https://github.com/ketd/AnvilKit.git
cd AnvilKit
cargo build
cargo test
```

详见 [在线文档](https://anvilkit.io) 或本地运行文档站点：

```bash
cd docs && pnpm install && pnpm dev
```

## 📄 许可证

双许可：[MIT](LICENSE-MIT) / [Apache 2.0](LICENSE-APACHE)，任选其一。

## 🙏 致谢

[Bevy](https://bevyengine.org/) · [wgpu](https://wgpu.rs/) · [Rapier](https://rapier.rs/) · [winit](https://github.com/rust-windowing/winit)

---

<div align="center">

**用 Rust 锻造游戏的未来 🔨**

[文档](https://anvilkit.io) · [示例](examples/) · [Issues](https://github.com/ketd/AnvilKit/issues) · [Discussions](https://github.com/ketd/AnvilKit/discussions)

</div>
