## 1. Asset: 纹理和材质提取
- [x] 1.1 新增 `TextureData` 结构体 (width, height, rgba_data)
- [x] 1.2 新增 `MaterialData` 结构体 (base_color_texture, base_color_factor)
- [x] 1.3 新增 `load_gltf_scene()` 返回 SceneData (mesh + material + texture)
- [x] 1.4 从 glTF 嵌入 PNG 图像加载纹理，支持 RGB→RGBA 转换

## 2. Render: GPU 纹理和采样器
- [x] 2.1 `create_texture()` — 从 RGBA 数据创建 wgpu Texture + TextureView
- [x] 2.2 `create_sampler()` — 创建线性过滤采样器
- [x] 2.3 更新 renderer/mod.rs 导出

## 3. RenderApp: 多 Bind Group 支持
- [x] 3.1 新增 `material_bind_group` 字段，render() 中绑定 group 1
- [x] 3.2 新增 `set_material_bind_group()` 方法

## 4. 着色器 + 示例
- [x] 4.1 WGSL 着色器：@group(1) texture + sampler，简单漫反射光照
- [x] 4.2 生成带纹理的 textured_sphere.glb (棋盘格纹理 UV 球体)
- [x] 4.3 创建 `examples/hello_textured.rs`

## 5. Quality
- [x] 5.1 `cargo check --workspace` 零错误零警告
- [x] 5.2 `cargo test --workspace` 517 tests 全部通过
- [x] 5.3 hello_triangle / hello_cube / hello_monkey 向后兼容
