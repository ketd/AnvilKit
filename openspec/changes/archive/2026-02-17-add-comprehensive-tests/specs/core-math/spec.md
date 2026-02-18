## ADDED Requirements

### Requirement: Math Edge Case Testing
数学模块 SHALL 对边界条件和退化输入进行测试验证。

#### Scenario: NaN 和 Infinity 输入处理
- **WHEN** Transform 或 Interpolation 函数接收 NaN/Infinity 作为输入
- **THEN** 函数不会 panic，行为可预测（返回 NaN 或默认值）

#### Scenario: 零向量和退化矩阵
- **WHEN** 对零长度向量执行归一化或对不可逆矩阵求逆
- **THEN** 行为符合文档约定（返回 None 或安全默认值）

#### Scenario: 浮点精度边界
- **WHEN** Transform 经过多层链式组合后与逆变换相乘
- **THEN** 结果与单位矩阵的误差在 `f32::EPSILON * 100` 范围内

### Requirement: Time Module Robustness Testing
时间模块 SHALL 对极端时间值和状态切换进行测试验证。

#### Scenario: 极大 delta_time
- **WHEN** Time::update 接收极大的 delta_time 值（如 1000 秒）
- **THEN** 内部状态保持一致，不会溢出

#### Scenario: Timer 快速暂停恢复
- **WHEN** Timer 在同一帧内多次暂停和恢复
- **THEN** 计时状态保持一致
