## ADDED Requirements

### Requirement: Window Configuration Comprehensive Testing
窗口配置系统 SHALL 对所有 builder 方法和边界值进行测试验证。

#### Scenario: Builder 方法链完整性
- **WHEN** WindowConfig 的所有 builder 方法被依次调用
- **THEN** 最终配置反映所有设置值

#### Scenario: 无效窗口尺寸
- **WHEN** 窗口尺寸设为 0x0 或负值
- **THEN** 使用默认尺寸或返回错误

### Requirement: Render Device Initialization Testing
渲染设备初始化 SHALL 对创建流程和错误路径进行测试验证。

#### Scenario: 实例创建验证
- **WHEN** wgpu Instance 被创建
- **THEN** 支持的后端列表非空

#### Scenario: 格式选择逻辑
- **WHEN** 查询首选表面格式
- **THEN** 返回 sRGB 格式或平台默认格式

### Requirement: Render Pipeline Builder Testing
渲染管线构建器 SHALL 对 builder 模式和默认值进行测试验证。

#### Scenario: 默认管线配置
- **WHEN** 使用默认参数构建 RenderPipelineBuilder
- **THEN** 所有必填字段有合理默认值

#### Scenario: 自定义管线配置
- **WHEN** 通过 builder 设置自定义顶点格式和着色器
- **THEN** 构建的管线描述符反映自定义设置

### Requirement: Render Plugin ECS Integration Testing
RenderPlugin SHALL 对 ECS 集成流程进行测试验证。

#### Scenario: 插件注册系统
- **WHEN** RenderPlugin 被添加到 App
- **THEN** 必要的系统和资源被注册到对应的 Schedule

#### Scenario: 插件配置传递
- **WHEN** RenderPlugin 使用自定义 RenderConfig 创建
- **THEN** 配置值在插件 build 时被正确应用
