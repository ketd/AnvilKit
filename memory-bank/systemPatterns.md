# σ₂: System Patterns
*v1.0 | Created: 2025-09-24 | Updated: 2025-09-24*
*Π: 🚧INITIALIZING | Ω: 💡INNOVATE*

## 🏛️ Architecture Overview
**AnvilKit 采用分层模块化架构**，基于 ECS 模式构建，支持按需编译和功能组合：

```
┌─────────────────────────────────────────────────────────┐
│                    Application Layer                    │
├─────────────────────────────────────────────────────────┤
│  Plugin System  │  Developer Tools  │  Debug Console   │
├─────────────────────────────────────────────────────────┤
│     2D Rendering     │     3D Rendering     │    UI     │
├─────────────────────────────────────────────────────────┤
│   Physics 2D   │   Physics 3D   │   Audio   │  Input   │
├─────────────────────────────────────────────────────────┤
│        Assets        │      Windowing      │   Events   │
├─────────────────────────────────────────────────────────┤
│                    ECS Core (bevy_ecs)                  │
├─────────────────────────────────────────────────────────┤
│                   Platform Abstraction                  │
└─────────────────────────────────────────────────────────┘
```

## 🧩 Core Components
- **🎯 ECS Core** - 基于 bevy_ecs 的实体组件系统
- **🖼️ Rendering Engine** - 统一的 2D/3D 渲染管线 (wgpu)
- **⚡ Physics Engine** - 可切换的 2D/3D 物理系统 (rapier)
- **📦 Asset System** - 统一的资源加载和管理系统
- **🎮 Input System** - 跨平台输入处理和事件系统
- **🔊 Audio System** - 游戏音频播放和管理 (kira)
- **🪟 Windowing** - 窗口管理和平台抽象 (winit)
- **🔧 Plugin System** - 可扩展的插件架构

## 🔄 Data Flow
[数据流图待绘制]

## 🎨 Design Patterns
- [设计模式1]
- [设计模式2]
- [设计模式3]

## 🔗 Integration Points
[集成点待定义]

## 📐 Design Decisions
*记录重要的架构决策*

## 🔧 Configuration
[配置管理策略待定]

## 📝 Notes
- 架构设计将在需求分析完成后进行
- 考虑可扩展性和维护性
