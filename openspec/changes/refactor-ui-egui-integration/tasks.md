## Phase 0: 依赖 + 版本验证

- [ ] 0.1 确认 egui 0.31 + egui-wgpu 0.31 + egui-winit 0.31 与 wgpu 0.19 + winit 0.30 兼容
- [ ] 0.2 添加 egui/egui-wgpu/egui-winit 到 workspace Cargo.toml
- [ ] 0.3 `cargo check --workspace` 验证依赖解析无冲突

## Phase 1: 引擎集成层 (anvilkit-app)

### 1.1 EguiIntegration 核心
- [ ] 1.1.1 创建 `crates/anvilkit-app/src/egui_integration/mod.rs` — EguiIntegration struct + 公共 API
- [ ] 1.1.2 创建 `crates/anvilkit-app/src/egui_integration/state.rs` — egui_winit::State 包装, 输入转发
- [ ] 1.1.3 创建 `crates/anvilkit-app/src/egui_integration/renderer.rs` — egui_wgpu::Renderer 包装, GPU 渲染
- [ ] 1.1.4 `EguiIntegration::new(device, surface_format, window)` — 初始化 egui context + renderer + state
- [ ] 1.1.5 `EguiIntegration::handle_event(window_event)` → bool — 转发事件给 egui, 返回是否被消费
- [ ] 1.1.6 `EguiIntegration::begin_frame(window)` — 开始 egui 帧
- [ ] 1.1.7 `EguiIntegration::end_frame_and_render(device, queue, encoder, target)` — 结束帧 + GPU 渲染
- [ ] 1.1.8 `EguiIntegration::context()` → &egui::Context — 获取 egui 上下文引用

### 1.2 GameCallbacks 扩展
- [ ] 1.2.1 `GameCallbacks` trait 添加 `fn ui(&mut self, ctx: &mut GameContext, egui_ctx: &egui::Context) {}`
- [ ] 1.2.2 AnvilKitApp 渲染流程: render() → begin_frame → ui() → end_frame_and_render → present
- [ ] 1.2.3 输入优先级: egui.wants_pointer_input() 时跳过 InputState 转发

### 1.3 纹理桥接
- [ ] 1.3.1 创建 `EguiTextures` Resource — name → TextureId 映射
- [ ] 1.3.2 `EguiTextures::register(name, device, texture_view)` — 注册 wgpu 纹理给 egui
- [ ] 1.3.3 `EguiTextures::get(name)` → Option<TextureId> — 查询已注册纹理
- [ ] 1.3.4 GameContext 添加 egui_textures 访问器

### 1.4 Facade 导出
- [ ] 1.4.1 `anvilkit-app` prelude 添加 egui 相关类型 re-export
- [ ] 1.4.2 facade crate prelude 添加 egui::Context re-export
- [ ] 1.4.3 `cargo check --workspace` 全量验证

## Phase 2: Craft 游戏 UI 重写

### 2.1 egui 主题
- [ ] 2.1.1 创建 `games/craft/src/ui/theme.rs` — Craft 暗色主题 (Visuals + Style + pixel font)
- [ ] 2.1.2 在 init() 中应用主题到 egui::Context

### 2.2 主菜单
- [ ] 2.2.1 重写 `games/craft/src/ui/main_menu.rs` — 用 egui CentralPanel + 居中按钮
- [ ] 2.2.2 "CRAFT" 大标题 + Play/Settings/Quit 按钮
- [ ] 2.2.3 按钮点击 → 状态转换

### 2.3 暂停菜单
- [ ] 2.3.1 重写 `games/craft/src/ui/pause_menu.rs` — 半透明 egui Window 覆盖
- [ ] 2.3.2 Resume/Settings/Save&Quit 按钮

### 2.4 设置界面
- [ ] 2.4.1 重写 `games/craft/src/ui/settings_menu.rs` — egui Slider 控件
- [ ] 2.4.2 Volume/Sensitivity/FOV/ViewDist 滑条 + 实时应用

### 2.5 背包界面
- [ ] 2.5.1 重写 `games/craft/src/ui/inventory.rs` — egui Grid + ImageButton
- [ ] 2.5.2 注册方块图标纹理 (从 terrain atlas 裁切)
- [ ] 2.5.3 3x3 网格 + 选中高亮 + 数量显示

### 2.6 main.rs 集成
- [ ] 2.6.1 GameCallbacks::ui() 实现 — CraftScreen 分发到各菜单
- [ ] 2.6.2 移除旧的 render_menu_screen / render_overlay_screen 方法
- [ ] 2.6.3 移除 CraftGame 中的 Menu 字段

## Phase 3: 清理

### 3.1 移除废弃代码
- [ ] 3.1.1 删除 `crates/anvilkit-ui/src/menu/` 目录 (5 files)
- [ ] 3.1.2 删除 `crates/anvilkit-render/src/renderer/menu/` 目录 (3 files)
- [ ] 3.1.3 更新 `anvilkit-ui/src/lib.rs` 和 `anvilkit-render/src/renderer/mod.rs` 移除 menu 模块声明
- [ ] 3.1.4 `cargo check --workspace` 零错误
- [ ] 3.1.5 `cargo test --workspace` 全量通过

## Phase 4: 验证

- [ ] 4.1 运行 Craft: MainMenu → Play → ESC → Paused → Settings → Back → Resume → E → Inventory → ESC → Paused → Save&Quit → MainMenu
- [ ] 4.2 验证 egui 主题: 暗色面板, 悬停高亮, 滑条可拖拽
- [ ] 4.3 验证输入隔离: 菜单打开时鼠标点击不触发方块破坏
- [ ] 4.4 验证 HUD: 血条/快捷栏/准心不受 egui 影响
- [ ] 4.5 `cargo check --workspace` 零错误零警告
- [ ] 4.6 `cargo test --workspace` 全量通过
