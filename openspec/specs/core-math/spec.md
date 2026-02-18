# Capability: core-math

## Purpose
Core mathematics, time management, and error handling infrastructure for AnvilKit.

**Crate**: `anvilkit-core` | **Status**: Implemented and verified (zero errors, zero warnings) | **Dependencies**: `glam`, `thiserror`, `serde` (optional), `bevy_ecs` (optional)
## Requirements
### Requirement: 3D Transform System

The system SHALL provide a `Transform` component representing 3D position (translation), rotation (quaternion), and scale, built on top of `glam` types.

The system SHALL provide a `GlobalTransform` component wrapping a `Mat4` for world-space transformations.

Both types SHALL support creation from individual components (`from_xyz`, `from_translation`, `from_rotation`, `from_scale`), matrix conversion, point/vector transformation, inverse computation, and `looking_at` for camera-style orientation.

#### Scenario: Create transform from position
- **WHEN** `Transform::from_xyz(1.0, 2.0, 3.0)` is called
- **THEN** a Transform with translation `Vec3(1.0, 2.0, 3.0)`, identity rotation, and unit scale is returned

#### Scenario: Compute world matrix
- **WHEN** `compute_matrix()` is called on a Transform
- **THEN** a `Mat4` combining translation, rotation, and scale is returned

#### Scenario: Transform hierarchy multiplication
- **WHEN** `mul_transform()` is called with a parent and child Transform
- **THEN** the resulting Transform represents the combined parent-child transformation

### Requirement: 2D/3D Geometry Primitives

The system SHALL provide geometric primitives: `Rect` (2D axis-aligned rectangle), `Circle` (2D circle), and `Bounds3D` (3D axis-aligned bounding box).

Each primitive SHALL support intersection tests, containment checks, area/volume calculation, and union/expansion operations.

#### Scenario: Rectangle intersection test
- **WHEN** two overlapping `Rect` values are tested with `intersects()`
- **THEN** `true` is returned

#### Scenario: Circle-rectangle intersection
- **WHEN** a `Circle` and `Rect` are tested with `intersects_rect()` / `intersects_circle()`
- **THEN** correct intersection result is returned

#### Scenario: Bounding box expansion
- **WHEN** `expand_to_include()` is called with a point outside the bounds
- **THEN** the bounds expand to contain the new point

### Requirement: Interpolation and Easing

The system SHALL provide `Lerp`, `Slerp`, and `Interpolate` traits for generic interpolation.

The system SHALL provide easing functions: `smoothstep`, `smootherstep`, `remap`, and parametric easing (quad, cubic, quart, elastic, bounce) in in/out/in-out variants.

#### Scenario: Linear interpolation
- **WHEN** `lerp(a, b, 0.5)` is called on two values
- **THEN** the midpoint between `a` and `b` is returned

#### Scenario: Easing function clamping
- **WHEN** an easing function receives `t` outside `[0, 1]`
- **THEN** the result is clamped to valid output range

### Requirement: Math Constants and Utilities

The system SHALL provide utility functions: `degrees_to_radians`, `radians_to_degrees`, `approximately_equal`, `is_nearly_zero`, `safe_sqrt`, `distance_2d`, `distance_3d`.

The system SHALL provide pre-defined constants in `vec2`, `vec3`, and `colors` submodules.

#### Scenario: Safe square root
- **WHEN** `safe_sqrt(-1.0)` is called
- **THEN** `0.0` is returned instead of `NaN`

#### Scenario: Approximate equality
- **WHEN** `approximately_equal(1.0, 1.0 + 1e-8)` is called
- **THEN** `true` is returned

### Requirement: Time Management System

The system SHALL provide a `Time` resource tracking frame delta time, total elapsed time, frame count, and FPS (both average and instant).

The system SHALL provide a `Timer` supporting one-shot and repeating modes, with pause/resume, duration adjustment, and percentage-based progress queries.

The system SHALL provide `ScaledTime` for time scaling effects (slow-motion, fast-forward).

#### Scenario: Frame delta tracking
- **WHEN** `Time::update()` is called each frame
- **THEN** `delta_seconds()` returns the time since the last update

#### Scenario: Repeating timer
- **WHEN** a repeating `Timer` elapses its duration
- **THEN** `just_finished()` returns `true` and the timer resets automatically

#### Scenario: Timer pause and resume
- **WHEN** a timer is paused via `pause()`
- **THEN** `tick()` calls do not advance the timer until `resume()` is called

### Requirement: Error Handling Infrastructure

The system SHALL provide an `AnvilKitError` enum with variants for each subsystem: `Render`, `Physics`, `Asset`, `Audio`, `Input`, `Ecs`, `Window`, `Config`, `Network`, `Io`, `Serialization`, `Generic`.

Each variant SHALL include a descriptive message string. The error type SHALL implement `std::error::Error` and `Display`.

The system SHALL provide `ErrorCategory` for classification and a `Result<T>` type alias.

#### Scenario: Error creation with context
- **WHEN** `AnvilKitError::render("GPU device lost")` is called
- **THEN** an error with category `ErrorCategory::Render` and the given message is created

#### Scenario: Error category query
- **WHEN** `error.is_category(ErrorCategory::Asset)` is called
- **THEN** `true` is returned only if the error is an Asset variant

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

