## 1. Fix Compilation Errors
- [x] 1.1 修复 winit API 兼容性 — 升级 winit 0.29 → 0.30，获得 `ApplicationHandler` / `ActiveEventLoop`
- [x] 1.2 修复 wgpu `Surface` 生命周期标注 — `RenderSurface<'w>`, `RenderContext<'w>` 添加生命周期参数
- [x] 1.3 修复 ECS 宏导入 — 添加 `bevy_ecs` 作为直接依赖解决 derive 宏路径
- [x] 1.4 清理未使用的导入警告 (13 个)
- [x] 1.5 修复 `AnvilKitError::Render(msg)` → `AnvilKitError::render(msg)` 工厂方法调用
- [x] 1.6 通过 `cargo check` 零错误零警告

## 2. Window Management
- [x] 2.1 验证 `WindowConfig` builder 可正常创建窗口配置
- [x] 2.2 重写 `RenderApp` 为独立持有 `RenderDevice` + `RenderSurface`，解决自引用问题
- [x] 2.3 窗口管理单元测试通过 (config builder, state, events)

## 3. Render Device & Surface
- [x] 3.1 `RenderDevice` 可获取 GPU 适配器和设备 (wgpu Instance 创建测试通过)
- [x] 3.2 `RenderSurface` 格式选择、呈现模式选择、alpha 模式选择测试通过
- [x] 3.3 `RenderContext` 统一渲染接口测试通过
- [x] 3.4 渲染器单元测试通过

## 4. Render Pipeline
- [x] 4.1 `RenderPipelineBuilder` fluent API 测试通过 (defaults, chaining, all options)
- [x] 4.2 `BasicRenderPipeline` WGSL 着色器加载 API 就绪

## 5. ECS Integration
- [x] 5.1 `RenderPlugin` 可注册到 App
- [x] 5.2 渲染组件 derive `Component` 成功，默认值测试通过
- [x] 5.3 `RenderConfig` 资源 derive `Resource` 成功，`RenderSystemSet` derive `SystemSet` 成功

## 6. Documentation & Quality
- [x] 6.1 所有公共 API 已有中文文档注释
- [x] 6.2 文档测试全部通过 (83 doc tests)
- [x] 6.3 `cargo check` 零警告

## Final Status
- **编译**: 0 errors, 0 warnings
- **测试**: 22 unit tests + 83 doc tests = 105 tests, 全部通过
