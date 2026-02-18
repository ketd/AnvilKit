## 1. GPU Resource Helpers
- [x] 1.1 `DEPTH_FORMAT` 常量 (Depth32Float)
- [x] 1.2 `create_uniform_buffer()` — UNIFORM | COPY_DST
- [x] 1.3 `create_depth_texture()` — Texture + TextureView
- [x] 1.4 更新 mod.rs 导出

## 2. Pipeline Builder
- [x] 2.1 `with_depth_format()` builder 方法
- [x] 2.2 `with_bind_group_layouts()` builder 方法
- [x] 2.3 `BasicRenderPipeline::new()` 支持 depth_format + bind_group_layouts
- [x] 2.4 DepthStencilState (CompareFunction::Less, depth_write_enabled)

## 3. RenderApp 3D Support
- [x] 3.1 新增 index_buffer, bind_group, depth_texture_view 字段
- [x] 3.2 `set_pipeline_3d()` 方法
- [x] 3.3 render() 支持深度附件 + BindGroup + draw_indexed
- [x] 3.4 handle_resize() 重建深度纹理

## 4. Hello Cube Example
- [x] 4.1 MVP Uniform 着色器 (WGSL)
- [x] 4.2 24 顶点 + 36 索引立方体数据 (6 面 6 色)
- [x] 4.3 CubeApp 包装器 — 每帧更新 MVP 矩阵
- [x] 4.4 透视投影 + look_at_lh 相机

## 5. Quality
- [x] 5.1 `cargo check` 零错误零警告
- [x] 5.2 `cargo test --workspace` 498 tests 全部通过
- [x] 5.3 hello_triangle 向后兼容
