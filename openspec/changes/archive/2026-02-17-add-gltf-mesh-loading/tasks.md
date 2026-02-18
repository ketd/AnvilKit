## 1. anvilkit-assets Crate
- [x] 1.1 创建 crate 骨架 (Cargo.toml, lib.rs)
- [x] 1.2 实现 `MeshData` 结构体 (mesh.rs)
- [x] 1.3 实现 `load_gltf_mesh()` 函数 (gltf_loader.rs)
- [x] 1.4 workspace Cargo.toml 添加成员

## 2. MeshVertex + u32 索引
- [x] 2.1 `MeshVertex` 结构体 (position + normal + texcoord, 32 bytes)
- [x] 2.2 `create_index_buffer_u32()` 函数
- [x] 2.3 更新 renderer/mod.rs 导出

## 3. RenderApp u32 索引支持
- [x] 3.1 `index_format` 字段
- [x] 3.2 `set_pipeline_3d_u32()` 方法
- [x] 3.3 `render()` 使用 `self.index_format`

## 4. 示例程序
- [x] 4.1 准备 `assets/suzanne.glb` 模型文件 (icosphere, 642 vertices, 1280 faces)
- [x] 4.2 法线可视化 WGSL 着色器
- [x] 4.3 `examples/hello_monkey.rs`
- [x] 4.4 注册 example 到 Cargo.toml

## 5. Quality
- [x] 5.1 `cargo check --workspace` 零错误零警告
- [x] 5.2 `cargo test --workspace` 509 tests 全部通过
- [x] 5.3 hello_triangle / hello_cube 向后兼容
