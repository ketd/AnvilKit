# M3 Plan: "你好，三角形！"

## 目标
在窗口中渲染一个彩色三角形，验证完整的渲染管线从顶点数据到屏幕像素的端到端能力。

## 现状分析

已有能力（M2 完成）：
- 窗口创建和事件循环 (`RenderApp` + `WindowConfig`)
- GPU 设备初始化 (`RenderDevice`)
- 交换链管理 (`RenderSurface`)
- 渲染管线构建器 (`RenderPipelineBuilder` + `BasicRenderPipeline`)
- WGSL 着色器编译
- 帧获取和清屏呈现

缺失的关键能力（4 个阻塞项）：
1. **无顶点缓冲区 API** — 无法将三角形数据上传到 GPU
2. **无顶点属性配置** — `VertexState.buffers` 硬编码为空数组
3. **无绘制命令 API** — render_pass 创建后立即丢弃，没有 draw 调用
4. **无示例程序** — 没有 examples/ 目录

## 实施方案

### Step 1: 新增顶点缓冲区模块 `renderer/buffer.rs`

新建 `crates/anvilkit-render/src/renderer/buffer.rs`，提供：

```rust
// 顶点数据 trait
pub trait Vertex: bytemuck::Pod + bytemuck::Zeroable {
    fn layout() -> wgpu::VertexBufferLayout<'static>;
}

// 内置顶点类型：位置 + 颜色
#[repr(C)]
pub struct ColorVertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

// 缓冲区创建
pub fn create_vertex_buffer<V: Vertex>(device: &RenderDevice, vertices: &[V]) -> wgpu::Buffer
pub fn create_index_buffer(device: &RenderDevice, indices: &[u16]) -> wgpu::Buffer
```

新增依赖：`bytemuck = { version = "1", features = ["derive"] }`（用于安全的顶点数据转换）

### Step 2: 扩展管线构建器支持顶点布局

修改 `renderer/pipeline.rs`：
- `RenderPipelineBuilder` 增加 `with_vertex_layout(layout: VertexBufferLayout)` 方法
- `BasicRenderPipeline::new()` 接受可选的 vertex buffer layouts 参数
- 移除硬编码的 `buffers: &[]`

### Step 3: 扩展渲染循环支持绘制命令

修改 `window/events.rs` 中的 `RenderApp`：
- 增加 `pipeline: Option<BasicRenderPipeline>` 字段
- 增加 `vertex_buffer: Option<wgpu::Buffer>` 字段
- 在 `init_render()` 中创建着色器、管线和顶点缓冲区
- 在 `render()` 中的 render_pass 上设置管线、绑定顶点缓冲区、调用 `draw()`

### Step 4: 编写 WGSL 着色器

新建 `crates/anvilkit-render/shaders/triangle.wgsl`：
- 顶点着色器：接收 position + color，输出 clip_position + color
- 片元着色器：输出插值后的顶色

也可内联为 Rust 字符串常量（更简单，无需资源加载）。

### Step 5: 创建示例程序

新建 `examples/hello_triangle.rs`：
- 创建 `EventLoop` + `RenderApp`
- 在 `resumed` 中初始化管线和顶点数据
- 运行事件循环渲染彩色三角形

### Step 6: OpenSpec 变更

创建 `openspec/changes/add-hello-triangle/`：
- `proposal.md` — M3 变更提案
- `tasks.md` — 实施清单
- `specs/render-system/spec.md` — ADDED: Vertex Buffer, Draw Commands; MODIFIED: Render Pipeline Builder

## 文件变更清单

| 操作 | 文件 | 说明 |
|------|------|------|
| 新建 | `crates/anvilkit-render/src/renderer/buffer.rs` | 顶点/索引缓冲区 API |
| 修改 | `crates/anvilkit-render/src/renderer/mod.rs` | 导出 buffer 模块 |
| 修改 | `crates/anvilkit-render/src/renderer/pipeline.rs` | 支持顶点布局 |
| 修改 | `crates/anvilkit-render/src/window/events.rs` | 渲染循环集成 draw 命令 |
| 修改 | `crates/anvilkit-render/src/lib.rs` | prelude 导出新类型 |
| 修改 | `crates/anvilkit-render/Cargo.toml` | 添加 bytemuck 依赖 |
| 新建 | `examples/hello_triangle.rs` | 示例程序 |
| 新建 | `openspec/changes/add-hello-triangle/` | 变更提案 |

## 设计决策

1. **着色器内联 vs 文件** — 选择内联为 Rust 常量字符串。M3 阶段不需要资源加载系统，内联更简单且编译时检查。
2. **Vertex trait vs 硬编码** — 使用 `bytemuck` 的 trait bound 提供类型安全的顶点定义，为后续扩展（纹理坐标、法线等）留出空间。
3. **RenderApp 内渲染 vs 外部控制** — M3 阶段将管线和缓冲区直接存储在 `RenderApp` 中。后续 M4+ 会重构为 ECS 驱动的渲染图。
4. **不修改 RenderContext** — `RenderContext` 当前的抽象层级不适合直接暴露 draw 命令，M3 在 `RenderApp.render()` 中直接使用 wgpu API，保持简单。

## 验收标准

- [ ] `cargo check` 零错误零警告
- [ ] `cargo test --workspace` 全部通过
- [ ] `cargo run --example hello_triangle` 打开窗口并显示一个彩色三角形
- [ ] 窗口可调整大小，三角形正确重绘
- [ ] 窗口关闭时程序正常退出
