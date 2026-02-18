# Change: Add Render System (M2)

## Why
AnvilKit 需要基础渲染能力来支持后续的 3D/2D 图形管线。这是里程碑 M2 的核心内容，为后续的三角形渲染 (M3)、资源加载 (M4)、2D 精灵 (M5) 奠定基础。

## What Changes
- 新增 `anvilkit-render` crate，提供 wgpu 渲染基础设施
- 窗口管理层：基于 winit 0.30 的 `WindowConfig`、`RenderApp`、`ApplicationHandler` 事件处理
- 渲染器层：`RenderDevice` (GPU 适配器/设备)、`RenderSurface<'w>` (交换链)、`RenderContext<'w>` (统一渲染上下文)
- 渲染管线：`RenderPipelineBuilder` (fluent builder)、`BasicRenderPipeline` (WGSL 着色器)
- ECS 集成：`RenderPlugin`、渲染组件 (`RenderComponent`, `CameraComponent`, `MeshComponent`, `MaterialComponent`)、`RenderConfig` 资源

## Impact
- Affected specs: 无 (新增能力 `render-system`)
- Affected code: `crates/anvilkit-render/` (11 个源文件)
- Dependencies: `anvilkit-core`, `anvilkit-ecs`, `wgpu 0.19`, `winit 0.30`, `bevy_ecs 0.14`
