# Change: Add Hello Triangle (M3)

## Why
M2 建立了渲染基础设施（窗口、GPU 设备、交换链、管线构建器），但缺少顶点缓冲区和绘制命令能力，无法实际渲染任何几何体。M3 补齐这些关键缺口，实现端到端的三角形渲染验证。

## What Changes
- 新增顶点缓冲区模块 (`renderer/buffer.rs`)：`ColorVertex` 类型、`Vertex` trait、缓冲区创建函数
- 扩展 `RenderPipelineBuilder` 支持自定义顶点布局
- 扩展 `RenderApp.render()` 支持管线绑定和 draw 调用
- 新增内联 WGSL 着色器（vertex + fragment）
- 新增 `examples/hello_triangle.rs` 示例程序
- 新增 `bytemuck` 依赖

## Impact
- Affected specs: `render-system` (新增 Vertex Buffer、Draw Commands 能力；修改 Render Pipeline Builder)
- Affected code: `crates/anvilkit-render/` (新增 buffer.rs，修改 pipeline.rs、events.rs、mod.rs、lib.rs、Cargo.toml)
- New files: `examples/hello_triangle.rs`
