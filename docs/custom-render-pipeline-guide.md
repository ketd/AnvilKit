# AnvilKit 自定义渲染管线开发指南

基于 Craft 体素游戏开发过程中的经验总结。适用于绕过内置 PBR 管线、直接控制 wgpu 渲染循环的场景。

## 1. 项目脚手架

使用 `anvilkit-cli` 工具快速创建项目骨架：

```bash
cargo run -p anvilkit-cli -- new my_game --template first-person
```

可用模板：`3d-basic`、`topdown`、`first-person`、`empty`。

脚手架会自动：
- 创建 `games/my_game/` 目录结构（src/main.rs、components、resources、systems、render）
- 添加到 workspace members
- 运行 `cargo check` 验证编译

在脚手架基础上改造比从零开始更高效 — 保留 ApplicationHandler 模式、输入处理、tonemap 后处理等样板代码。

## 2. 左手坐标系与面绕序

AnvilKit 使用 **左手坐标系**（`look_at_lh` + `perspective_lh`）：

- +X 右，+Y 上，+Z 前（屏幕内）
- wgpu 默认 `FrontFace::Ccw`，但 LH 投影会翻转绕序

### 正确的面绕序

对于自定义几何体，在 LH 系统中 front-face 的绕序与 RH 相反。验证方法：

```
给定四边形 p0, p1, p2, p3：
- 计算 (p1-p0) × (p2-p0)
- 如果叉积方向与面的 **内法线** 一致 → 在 LH 系统中是 front-face
- 如果叉积方向与面的 **外法线** 一致 → 需要反转索引
```

实际做法：先按外法线方向排列顶点，然后用 **CW 索引**：

```rust
// 顶点按外法线 CCW 排列，索引用 CW（适配 LH）
indices.extend_from_slice(&[base, base+2, base+1, base, base+3, base+2]);
```

如果面不可见（被 back-face culling 吃掉），首先检查绕序。

## 3. 贴图图集（Texture Atlas）

### 加载与 Color Key 处理

许多像素艺术贴图使用洋红色 `(255, 0, 255)` 作为透明 color key，而非 PNG alpha 通道。加载时必须手动转换：

```rust
let mut atlas_img = image::open(path).expect("Failed").to_rgba8();
for pixel in atlas_img.pixels_mut() {
    if pixel[0] == 255 && pixel[1] == 0 && pixel[2] == 255 {
        *pixel = image::Rgba([0, 0, 0, 0]); // 转为真透明
    }
}
```

### NEAREST 采样 + Tile Inset

体素/像素风格必须用 `FilterMode::Nearest`，并在 UV 边缘加微小 inset 防止 tile 间采样溢出：

```rust
const TILE_INSET: f32 = 1.0 / 2048.0;
let u0 = du + TILE_INSET;
let u1 = du + TILE_UV - TILE_INSET;
```

### Tile 索引计算

Craft 风格的 flat tile index 转 UV：

```rust
pub fn tile_uv(tile: u8) -> (f32, f32) {
    let col = (tile % 16) as f32;
    let row = (tile / 16) as f32;
    (col / 16.0, row / 16.0)
}
```

### 验证贴图内容

**不要** 假设贴图的 tile 布局 — 用脚本实际检查像素内容：

```python
from PIL import Image
img = Image.open("texture.png")
for tile in range(256):
    col, row = tile % 16, tile // 16
    px = img.getpixel((col*16+8, row*16+8))
    if px != (255, 0, 255, 255):
        print(f"tile {tile} ({col},{row}) = {px}")
```

## 4. 自定义 Pipeline（绕过 PBR）

当不需要 PBR/IBL/Shadow 时，直接创建简单管线：

```rust
// 两个 bind group 就够了
// Group 0: scene uniform (view_proj, camera_pos, light_dir, fog)
// Group 1: texture atlas + sampler

let voxel_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
    layout: Some(&layout),           // 只有 2 个 bind group layout
    vertex: VertexState {
        buffers: &[BlockVertex::layout()],  // 自定义顶点格式
        ..
    },
    multisample: MultisampleState { count: 1, .. },  // 无 MSAA
    ..
});
```

对比 PBR 管线需要 3 个 bind group（scene + material + IBL/shadow）、MSAA resolve 等，自定义管线更轻量。

### 单 Pass 渲染多 Chunk

所有 chunk 共享同一 pipeline 和 bind group，单次 render pass 内通过切换 vertex/index buffer 绘制：

```rust
rp.set_pipeline(&gpu.voxel_pipeline);
rp.set_bind_group(0, &gpu.scene_bg, &[]);
rp.set_bind_group(1, &gpu.atlas_bg, &[]);

for chunk in &visible_chunks {
    rp.set_vertex_buffer(0, chunk.vb.slice(..));
    rp.set_index_buffer(chunk.ib.slice(..), IndexFormat::Uint32);
    rp.draw_indexed(0..chunk.index_count, 0, 0..1);
}
```

## 5. Chunk 管理与视锥体剔除

### Bounding Sphere 半径

Chunk 是 32x256x32 的长方体，bounding sphere 中心放在 `(cx+16, 128, cz+16)`，半径必须覆盖整个 AABB 的半对角线：

```
radius = sqrt(16² + 128² + 16²) ≈ 130
```

用 48 这样的小半径会导致向上/向下看时 chunk 被错误剔除。

### 动态加载/卸载

在 `about_to_wait` 中检测玩家所在 chunk 坐标变化，按需加载新 chunk、卸载远 chunk：

```rust
fn update_chunks(&mut self) {
    let cx = (cam_pos.x / CHUNK_SIZE as f32).floor() as i32;
    let cz = (cam_pos.z / CHUNK_SIZE as f32).floor() as i32;
    if (cx, cz) == self.last_chunk_pos { return; }

    // 1. 生成新 chunk 数据（不需要 GPU）
    // 2. Mesh + 上传 GPU
    // 3. 移除超出范围的 chunk
}
```

### 借用检查器与 RenderDevice

`self.render_app.render_device()` 返回 `&RenderDevice`，会借用 `self.render_app`。如果同时需要访问 `self.app.world` 和 `self.chunk_meshes`，必须用自由函数拆分借用：

```rust
// 错误 — device 借用 self，后续 self.xxx 会冲突
let device = self.render_app.render_device();
self.generate_chunks();  // borrows &mut self → conflict!
self.upload_chunks(device);

// 正确 — 分阶段，先生成（不需要 device），再获取 device 上传
{
    let mut world = self.app.world.resource_mut::<VoxelWorld>();
    generate_chunks(&mut world, &self.world_gen, cx, cz, radius);
}
{
    let device = self.render_app.render_device().unwrap();
    let world = self.app.world.resource::<VoxelWorld>();
    upload_chunks(&world, &mut self.chunk_meshes, device);
}
```

## 6. FPS 相机输入

### 使用 DeviceEvent::MouseMotion

`CursorMoved` 是窗口坐标，到达窗口边缘后就不再变化。FPS 相机应使用 `DeviceEvent::MouseMotion` 获取原始鼠标增量：

```rust
fn device_event(&mut self, .., ev: DeviceEvent) {
    if let DeviceEvent::MouseMotion { delta } = ev {
        mouse_delta.dx += delta.0 as f32;
        mouse_delta.dy += delta.1 as f32;
    }
}
```

配合光标锁定：

```rust
window.set_cursor_grab(CursorGrabMode::Confined)
    .or_else(|_| window.set_cursor_grab(CursorGrabMode::Locked));
window.set_cursor_visible(false);
```

### 移动方向基于 Yaw

移动方向只用 yaw 旋转（忽略 pitch），避免向上看时走向天空：

```rust
let forward = Quat::from_rotation_y(yaw) * Vec3::Z;
let right = Quat::from_rotation_y(yaw) * Vec3::X;
```

## 7. HDR → Tonemap 后处理

复用引擎提供的 `tonemap.wgsl`（ACES Filmic + gamma），只需：

1. 场景渲染到 `Rgba16Float` HDR render target（无 MSAA 时直接渲染，不需要 MSAA resolve）
2. Tonemap pass 用全屏三角形采样 HDR → 输出到 swapchain

```rust
// 无 MSAA 时，深度纹理也不需要 MSAA 版本
let (_, depth_view) = create_depth_texture(device, w, h, "Depth");  // 不是 create_depth_texture_msaa
let (_, hdr_view) = create_hdr_render_target(device, w, h, "HDR");  // 不是 create_hdr_msaa_texture
```

## 8. 常见陷阱清单

| 症状 | 原因 | 修复 |
|------|------|------|
| 面不可见（被剔除） | 顶点绕序在 LH 系统中反了 | 反转三角形索引顺序 |
| 贴图全是洋红色 | Tile 索引与实际贴图布局不匹配 | 用脚本验证实际像素内容 |
| 植物/树叶有洋红色背景 | Color key 未转换为 alpha | 加载时 (255,0,255) → (0,0,0,0) |
| 向下看 chunk 消失 | 视锥体剔除球半径太小 | 半径需覆盖完整 chunk AABB 对角线 |
| Borrow checker 报错 | `render_device()` 借用了 self | 用自由函数拆分不同字段的借用 |
| 鼠标到窗口边缘卡住 | 用了 CursorMoved 而非 MouseMotion | 改用 DeviceEvent::MouseMotion + 光标锁定 |
| Resize 后画面损坏 | HDR RT / depth 未重建 | 在 WindowEvent::Resized 中重建并更新 tonemap bind group |
