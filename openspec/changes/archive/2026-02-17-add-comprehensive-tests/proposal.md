# Change: 全面补充测试覆盖

## Why
AnvilKit 项目当前测试覆盖不均衡：core 模块有 76 个测试（较好），ecs 模块 37 个（中等），render 模块仅 22 个（薄弱）。缺少集成测试、性能基准测试，且部分关键 API 路径（错误处理、边界条件）未覆盖。项目约定要求"每个模块需要完整的单元测试覆盖"和"每个公共 API 需要文档测试"。

## What Changes
- 为 anvilkit-core 补充缺失的单元测试和边界条件测试
- 为 anvilkit-ecs 补充资源管理、系统执行顺序、插件生命周期等高级功能测试
- 为 anvilkit-render 大幅补充渲染设备、管线、表面管理等测试
- 新增 `tests/` 目录下的集成测试，验证跨 crate 协作
- 为关键公共 API 补充文档测试 (doc tests)
- 修复 transform.rs 中 `Vec3` 未声明的编译错误

## Impact
- Affected specs: `core-math`, `ecs-system`, `render-system`（均为增强测试覆盖，不改变功能行为）
- Affected code:
  - `crates/anvilkit-core/src/` — 补充单元测试
  - `crates/anvilkit-ecs/src/` — 补充单元测试，修复编译错误
  - `crates/anvilkit-render/src/` — 大幅补充单元测试
  - `crates/anvilkit-core/tests/` — 新增集成测试
  - `crates/anvilkit-ecs/tests/` — 新增集成测试
  - `crates/anvilkit-render/tests/` — 新增集成测试
- 新增 capability: `testing-infrastructure` — 测试基础设施和约定
