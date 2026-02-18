## Context
AnvilKit 渲染系统是框架的图形基础设施层，基于 wgpu 0.19 和 winit 0.29 构建，通过 ECS 插件模式集成。

本模块需要解决：
- 跨平台窗口创建和事件处理
- GPU 设备发现和管理
- 渲染表面 (swap chain) 配置
- 渲染管线抽象

## Goals / Non-Goals
- Goals:
  - 提供可用的 wgpu 渲染基础设施
  - 通过 ECS Plugin 无缝集成
  - 支持 WGSL 着色器
  - 为后续 3D/2D 渲染管线提供基础
- Non-Goals:
  - 本阶段不实现完整的 PBR 渲染
  - 不实现 2D 精灵批处理
  - 不实现场景图或资源管理

## Decisions
- **窗口管理使用 winit 0.29**: 跨平台支持最好，但需要注意 0.29 版本 API 的 `ApplicationHandler` trait 变更
- **渲染使用 wgpu 0.19**: 原生 Rust，支持 Vulkan/Metal/D3D12/WebGL
- **中间件渲染模式**: 可组合的渲染组件设计，参考 Bevy 渲染架构
- **Builder 模式**: `RenderPipelineBuilder` 使用 fluent builder 简化管线创建

## Risks / Trade-offs
- **winit 0.29 API 变动风险** → 当前编译失败的主因，需确认正确的 API 路径或考虑适配
- **wgpu Surface 生命周期** → wgpu 的 Surface 引用窗口，需要仔细管理生命周期
- **异步初始化** → GPU 设备创建是异步的，使用 `pollster` 同步阻塞

## Open Questions
- winit 0.29 是否已经稳定了 `ApplicationHandler` trait？是否需要降级到 0.28 的事件循环模式？
- wgpu 0.19 的 `Surface` 生命周期管理最佳实践是什么？
