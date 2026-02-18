## 1. Vertex Buffer Module
- [x] 1.1 添加 `bytemuck` 依赖到 Cargo.toml
- [x] 1.2 创建 `renderer/buffer.rs` — `Vertex` trait、`ColorVertex` 类型
- [x] 1.3 实现 `create_vertex_buffer()` 和 `create_index_buffer()` 函数
- [x] 1.4 在 `renderer/mod.rs` 导出 buffer 模块

## 2. Pipeline Vertex Layout
- [x] 2.1 `RenderPipelineBuilder` 添加 `with_vertex_layouts()` 方法
- [x] 2.2 `BasicRenderPipeline::new()` 接受顶点布局参数 + `into_pipeline()` 消费方法
- [x] 2.3 `VertexState.buffers` 使用传入的 `vertex_layouts`

## 3. Draw Commands
- [x] 3.1 `RenderApp` 添加 `pipeline`、`vertex_buffer`、`vertex_count` 字段
- [x] 3.2 `set_pipeline()` / `render_device()` / `surface_format()` 公共方法
- [x] 3.3 `render()` 中绑定管线、设置顶点缓冲区、调用 `draw()`

## 4. WGSL Shaders
- [x] 4.1 编写顶点着色器（接收 position + color，输出 clip_position + color）
- [x] 4.2 编写片元着色器（输出插值颜色）
- [x] 4.3 着色器内联为 Rust 常量字符串

## 5. Example Program
- [x] 5.1 创建 `examples/hello_triangle.rs` — 使用 TriangleApp 包装器
- [x] 5.2 `cargo build --example hello_triangle` 编译通过

## 6. Quality
- [x] 6.1 `cargo check` 零错误零警告
- [x] 6.2 `cargo test --workspace` 全部通过 (495 tests)
- [x] 6.3 新增 API 添加文档注释和单元测试 (buffer 模块 4 个测试)

## Final Status
- **编译**: 0 errors, 0 warnings
- **测试**: 495 tests 全部通过
- **示例**: `hello_triangle` 编译成功
