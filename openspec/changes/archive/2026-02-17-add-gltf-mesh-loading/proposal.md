# Change: Add glTF Mesh Loading (M4a)

## Why
M3.5 验证了 3D 渲染管线（Uniform Buffer、深度测试、索引绘制），但几何数据仍然是硬编码的。M4 目标是加载外部 3D 模型并渲染，M4a 作为第一步，建立 glTF 资源加载管线，渲染法线着色的 Suzanne 猴头模型。

## What Changes
- 新建 `anvilkit-assets` crate：glTF 文件解析、CPU 侧网格数据结构 (`MeshData`)
- 新增 `MeshVertex` 顶点类型（position + normal + texcoord）
- 新增 `create_index_buffer_u32()` 支持 u32 索引
- `RenderApp` 支持 u32 索引格式 (`set_pipeline_3d_u32`)
- 新增 `examples/hello_monkey.rs` — 加载 suzanne.glb，法线可视化渲染

## Impact
- Affected specs: `render-system` (新增 MeshVertex、u32 索引支持)，新建 `asset-system` spec
- Affected code: 新建 `crates/anvilkit-assets/`，修改 `crates/anvilkit-render/` (buffer.rs, events.rs, mod.rs, Cargo.toml)
- New files: `examples/hello_monkey.rs`, `assets/suzanne.glb`
