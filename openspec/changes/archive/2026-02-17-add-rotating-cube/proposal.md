# Change: Add Rotating Cube (M3.5)

## Why
M3 验证了 2D 三角形渲染，但缺少 3D 渲染核心能力：Uniform Buffer、深度缓冲、索引渲染、MVP 矩阵变换。M3.5 补齐这些能力，为 M4 (glTF + PBR) 奠定基础。

## What Changes
- 新增 GPU 资源辅助函数：`create_uniform_buffer()`、`create_depth_texture()`、`DEPTH_FORMAT` 常量
- 扩展 `RenderPipelineBuilder` 支持深度格式 (`with_depth_format`) 和 BindGroupLayout (`with_bind_group_layouts`)
- 扩展 `RenderApp` 支持索引缓冲区、BindGroup、深度纹理附件、索引绘制
- 新增 `examples/hello_cube.rs` — 旋转彩色立方体示例（MVP 变换、深度测试、24 顶点 + 36 索引）

## Impact
- Affected specs: `render-system` (新增 Uniform Buffer、Depth Testing、Indexed Drawing 能力；修改 Pipeline Builder)
- Affected code: `crates/anvilkit-render/` (buffer.rs, pipeline.rs, events.rs, mod.rs, Cargo.toml)
- New files: `examples/hello_cube.rs`
