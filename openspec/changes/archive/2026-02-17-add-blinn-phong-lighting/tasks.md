## 1. Uniform 数据结构
- [x] 1.1 SceneUniform: model + view_proj + normal_matrix + camera_pos + light_dir + light_color
- [x] 1.2 所有 vec3/vec4 使用 [f32; 4] 保证 16 字节对齐

## 2. Blinn-Phong WGSL 着色器
- [x] 2.1 顶点着色器：world_position = model * position, world_normal = normal_matrix * normal
- [x] 2.2 片元着色器：ambient + Lambert diffuse + Blinn-Phong specular (shininess=32)
- [x] 2.3 纹理采样 * (ambient + diffuse) + specular * 0.3

## 3. 示例程序
- [x] 3.1 创建 `examples/hello_lit.rs` — 相机轻微摇摆，光源固定方向
- [x] 3.2 注册 example 到 Cargo.toml

## 4. Quality
- [x] 4.1 `cargo check --workspace` 零错误零警告
- [x] 4.2 `cargo test --workspace` 517 tests 全部通过
- [x] 4.3 所有现有示例向后兼容
