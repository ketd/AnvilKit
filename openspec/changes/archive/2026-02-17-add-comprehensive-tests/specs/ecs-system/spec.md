## ADDED Requirements

### Requirement: Resource Management Testing
ECS 资源系统 SHALL 对资源增删改查全流程进行测试验证。

#### Scenario: 资源生命周期
- **WHEN** 通过 `World::insert_resource` 添加资源后，使用 `World::get_resource` 获取
- **THEN** 返回的资源值与插入时一致

#### Scenario: 资源不存在
- **WHEN** 查询未注册的资源类型
- **THEN** 返回 `None` 而非 panic

### Requirement: System Execution Order Testing
系统调度器 SHALL 保证系统按声明的依赖关系顺序执行。

#### Scenario: 系统顺序验证
- **WHEN** 系统 A 声明在系统 B 之前执行
- **THEN** 系统 A 的副作用在系统 B 执行时可见

### Requirement: Plugin Lifecycle Testing
插件系统 SHALL 对注册生命周期和错误处理进行测试验证。

#### Scenario: 重复插件注册
- **WHEN** 同一个 Plugin 被注册两次
- **THEN** 第二次注册被忽略或产生明确警告

#### Scenario: 插件构建顺序
- **WHEN** Plugin A 依赖 Plugin B 的资源
- **THEN** 按正确顺序注册后，资源在 Plugin A 的 build 中可用

### Requirement: Transform Hierarchy Deep Testing
Transform 层级系统 SHALL 对深层嵌套和动态变更进行测试验证。

#### Scenario: 深层嵌套同步
- **WHEN** Transform 层级深度超过 5 层
- **THEN** GlobalTransform 同步结果与手动计算一致

#### Scenario: 父节点删除
- **WHEN** 层级中的父节点被删除
- **THEN** 子节点的 Parent 组件被正确清理
