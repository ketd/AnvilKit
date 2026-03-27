# Capability: ecs-system

## Purpose
Entity Component System infrastructure for AnvilKit, built on Bevy ECS 0.14.

**Crate**: `anvilkit-ecs` | **Status**: Implemented and verified (37 unit tests + 67 doc tests, zero errors) | **Dependencies**: `anvilkit-core`, `bevy_ecs 0.14`, `glam`, `thiserror`
## Requirements
### Requirement: Application Framework

The system SHALL provide an `App` container that manages the ECS `World`, system scheduling, and the main application loop.

`App` SHALL support adding plugins (`add_plugins`), inserting systems (`add_systems`), inserting resources (`insert_resource`, `init_resource`), and running the main loop (`run`) or single updates (`update`).

`App::add_plugins` SHALL check `Plugin::is_unique()` and skip registration if a unique plugin of the same type is already registered, logging a warning.

`App::update` SHALL log schedule execution errors via `log::error!()` instead of silently discarding them.

The system SHALL provide `AppExit` as a resource to control graceful application shutdown.

#### Scenario: Basic application lifecycle
- **WHEN** `App::new()` is created, plugins and systems are added, and `run()` is called
- **THEN** the application executes startup systems once, then runs update systems in a loop until exit

#### Scenario: Single update step
- **WHEN** `app.update()` is called
- **THEN** exactly one frame's worth of systems are executed

#### Scenario: Application exit
- **WHEN** `app.exit()` is called
- **THEN** `should_exit()` returns `true` and the main loop terminates

#### Scenario: Duplicate plugin prevention
- **WHEN** `app.add_plugins(MyPlugin)` is called twice and `MyPlugin::is_unique()` returns `true`
- **THEN** the second call is skipped and a warning is logged

#### Scenario: Schedule error reporting
- **WHEN** a schedule execution returns an error during `app.update()`
- **THEN** the error is logged via `log::error!()` with the schedule label and error details

### Requirement: Plugin System

The system SHALL provide a `Plugin` trait with a required `build(&self, app: &mut App)` method for modular functionality extension.

The system SHALL provide `AnvilKitEcsPlugin` as the core ECS plugin and `PluginGroup<T>` for grouping multiple plugins.

Plugins SHALL be unique by default (`is_unique()` returns `true`), preventing duplicate registration.

#### Scenario: Plugin registration
- **WHEN** `app.add_plugins(MyPlugin)` is called
- **THEN** `MyPlugin::build()` is invoked with mutable access to the App

#### Scenario: Plugin group
- **WHEN** a `PluginGroup` containing multiple plugins is added to an App
- **THEN** all plugins in the group are registered in order

### Requirement: Schedule System

The system SHALL provide `AnvilKitSchedule` enum with phases: `Startup`, `Main`, `PreUpdate`, `FixedUpdate`, `Update`, `PostUpdate`, `Cleanup`.

The `FixedUpdate` schedule SHALL run at a configurable fixed timestep (default 1/60 seconds) using a time accumulator. When the accumulated time exceeds the fixed step, the schedule runs one or more times to catch up.

The system SHALL provide `AnvilKitSystemSet` enum for grouping systems by concern: `Input`, `Time`, `Physics`, `GameLogic`, `Transform`, `Render`, `Audio`, `UI`, `Network`, `Debug`.

The system SHALL configure inter-set execution order: `Input` → `Time` → `Physics` → `GameLogic` → `Transform` → `Render` → `Audio` → `UI` → `Network` → `Debug`.

`AnvilKitSchedule` SHALL implement the `ScheduleLabel` trait from `bevy_ecs`.

The system SHALL provide `ScheduleBuilder` for constructing schedules with system sets.

#### Scenario: System ordering by schedule phase
- **WHEN** systems are added to `PreUpdate`, `Update`, and `PostUpdate`
- **THEN** they execute in that order each frame

#### Scenario: System set grouping
- **WHEN** systems are assigned to `AnvilKitSystemSet::Physics`
- **THEN** they can be collectively ordered relative to other system sets

#### Scenario: Fixed update physics
- **WHEN** a physics system is added to `FixedUpdate` and the frame takes 32ms
- **THEN** the physics system runs twice (2 × 16.67ms) to maintain 60Hz simulation

#### Scenario: System set ordering
- **WHEN** an Input system and a Physics system are registered in their respective sets
- **THEN** the Input system always executes before the Physics system within the same schedule phase

### Requirement: Core Components

The system SHALL provide reusable components:
- `Name` — entity identification with string content
- `Tag` — generic labeling with pattern matching
- `Visibility` — visibility control with `Visible`, `Hidden`, `Inherited` variants
- `Layer` — rendering layer/z-order as `i32`

All components SHALL derive `Component` and support creation from `String`, `&str`, or appropriate primitive types.

#### Scenario: Entity naming
- **WHEN** `Name::new("Player")` is assigned to an entity
- **THEN** `name.as_str()` returns `"Player"`

#### Scenario: Visibility toggle
- **WHEN** `visibility.toggle()` is called on `Visibility::Visible`
- **THEN** the value becomes `Visibility::Hidden`

#### Scenario: Layer ordering
- **WHEN** entities have different `Layer` values
- **THEN** they can be sorted by `layer.value()` for render ordering

### Requirement: Bundle System

The system SHALL provide pre-built entity bundles:
- `EntityBundle` — basic entity with `Name` and `Tag`
- `SpatialBundle` — spatial entity with `Transform`, `GlobalTransform`, `Visibility`, and `Layer`
- `RenderBundle` — rendering entity extending `SpatialBundle` with a render tag

Each bundle SHALL support builder-pattern methods for customization (`with_position`, `with_rotation`, `with_scale`, etc.).

#### Scenario: Spatial entity creation
- **WHEN** `SpatialBundle::new().with_position(1.0, 2.0, 3.0).with_layer(5)` is created
- **THEN** the bundle contains a Transform at position (1,2,3) and Layer(5)

#### Scenario: Render bundle defaults
- **WHEN** `RenderBundle::new()` is created with defaults
- **THEN** it includes identity Transform, GlobalTransform, Visibility::Visible, and Layer(0)

### Requirement: Transform Hierarchy

The system SHALL provide `Parent` and `Children` components for parent-child entity relationships.

The system SHALL provide a `TransformPlugin` that adds transform propagation systems to the `PostUpdate` phase:
- `sync_simple_transforms()` — synchronize local to global for root entities
- `propagate_transforms()` — propagate parent transforms down to children

The system SHALL provide `TransformHierarchy` utility with methods: `set_parent()`, `remove_parent()`, `get_ancestors()`, `get_descendants()`.

#### Scenario: Root entity transform sync
- **WHEN** a root entity (no parent) has a `Transform`
- **THEN** its `GlobalTransform` matches the local `Transform` matrix

#### Scenario: Child transform propagation
- **WHEN** a child entity's parent has a non-identity Transform
- **THEN** the child's `GlobalTransform` equals parent's global matrix multiplied by child's local matrix

#### Scenario: Parent removal
- **WHEN** `TransformHierarchy::remove_parent()` is called
- **THEN** the entity is removed from the parent's `Children` list and its `Parent` component is removed

### Requirement: System Utilities

The system SHALL provide utility systems:
- `DebugSystems` — entity count, named entity listing, transform debug, performance monitoring (via `log` crate, not `println!`)
- `UtilitySystems` — time update, visibility filtering (respecting parent hierarchy for `Inherited` variant), layer sorting, generic cleanup

All debug and utility output SHALL use the `log` crate (`log::info!`, `log::debug!`, `log::warn!`) instead of `println!`.

`performance_monitor_system` SHALL guard against division by zero on the first frame (when `delta_seconds() == 0.0`) and use elapsed-time-based throttling (e.g., report every 1.0 second) instead of modulo-based counting.

`visibility_filter_system` SHALL resolve `Visibility::Inherited` by querying the parent entity's visibility state, not by unconditionally setting it to `Visible`.

#### Scenario: Performance monitoring
- **WHEN** `DebugSystems::performance_monitor_system()` runs
- **THEN** it reports FPS and frame time statistics via the `log` crate at `info` level, throttled to once per second

#### Scenario: Performance monitoring first frame
- **WHEN** the system runs on the first frame with `delta_seconds() == 0.0`
- **THEN** the system skips reporting without panicking or producing infinity/NaN values

#### Scenario: Conditional cleanup
- **WHEN** `UtilitySystems::cleanup_system::<T>()` runs
- **THEN** entities with component `T` are despawned

#### Scenario: Inherited visibility resolution
- **WHEN** an entity has `Visibility::Inherited` and its parent has `Visibility::Hidden`
- **THEN** the entity is treated as hidden during visibility filtering

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

### Requirement: Event System

The system SHALL use Bevy's `Events<T>` system for all engine-level events, providing automatic double-buffering, per-system cursor tracking via `EventReader<T>`, and write access via `EventWriter<T>`.

The system SHALL register the following event types:
- `CollisionEvent` — physics collision notifications
- `NetworkEvent` — network state change notifications

Game code SHALL be able to register custom event types via `app.add_event::<T>()`.

#### Scenario: Collision event lifecycle
- **WHEN** the collision detection system detects a collision between entity A and entity B
- **THEN** it writes a `CollisionEvent` via `EventWriter<CollisionEvent>`, and any system with `EventReader<CollisionEvent>` can read it for up to 2 frames

#### Scenario: Multiple readers
- **WHEN** two independent systems both have `EventReader<CollisionEvent>`
- **THEN** each system sees all events independently (each has its own cursor)

#### Scenario: Custom game events
- **WHEN** `app.add_event::<PlayerDied>()` is called and a system writes `PlayerDied` events
- **THEN** other systems can read them via `EventReader<PlayerDied>`

### Requirement: Game State Machine

The system SHALL provide integration with Bevy's `States` system for managing game state transitions (e.g., Menu, Playing, Paused, GameOver).

The system SHALL support `OnEnter(state)`, `OnExit(state)`, and `OnTransition { from, to }` schedule hooks for state-specific system registration.

State transitions SHALL be requested via `NextState<S>` resource and applied during the `StateTransition` schedule point.

#### Scenario: State transition
- **WHEN** `next_state.set(GameState::Playing)` is called during the Menu state
- **THEN** `OnExit(GameState::Menu)` systems run, then `OnEnter(GameState::Playing)` systems run

#### Scenario: State-conditional systems
- **WHEN** a system is added with `.run_if(in_state(GameState::Playing))`
- **THEN** it only executes when the current state is `Playing`

#### Scenario: Pause/Resume
- **WHEN** the game transitions from Playing → Paused → Playing
- **THEN** OnExit(Playing) runs on pause, OnEnter(Playing) runs on resume

### Requirement: Scene Serialization Extended

The system SHALL support serializing arbitrary ECS components (not only Transform) through a component registration system.

The system SHALL provide `app.register_serializable::<T>()` for registering component types that participate in scene save/load.

Serialization SHALL preserve entity hierarchy (Parent/Children relationships).

#### Scenario: Full scene round-trip
- **WHEN** a scene with entities having Transform, Name, Tag, Visibility, and custom components is saved and loaded
- **THEN** all registered components are restored with their original values

#### Scenario: Hierarchy preservation
- **WHEN** a scene with parent-child relationships is serialized and deserialized
- **THEN** the Parent and Children components are correctly restored

### Requirement: Hierarchy Recursive Despawn

The system SHALL provide `TransformHierarchy::despawn_recursive(commands, entity)` that despawns an entity and all its descendants in the hierarchy.

#### Scenario: Despawn subtree
- **WHEN** `despawn_recursive(commands, parent)` is called on an entity with 3 children (one of which has 2 grandchildren)
- **THEN** the parent, 3 children, and 2 grandchildren (6 entities total) are all despawned

#### Scenario: Despawn leaf
- **WHEN** `despawn_recursive(commands, leaf)` is called on an entity with no children
- **THEN** only the leaf entity is despawned

### Requirement: Unified Logging
All ECS crate modules SHALL use the `log` crate for all diagnostic output. No module SHALL use `println!`, `eprintln!`, or `dbg!` for production logging.

The following log levels SHALL be used:
- `log::error!` — schedule execution failures, invariant violations
- `log::warn!` — duplicate plugin registration, deprecated API usage
- `log::info!` — performance metrics, system lifecycle events
- `log::debug!` — entity counts, transform dumps, per-frame diagnostics

#### Scenario: Debug system output
- **WHEN** `entity_count_system` runs with log level set to `debug`
- **THEN** the entity count is emitted as a `log::debug!` message, not printed to stdout

#### Scenario: No stdout pollution
- **WHEN** the ECS crate is used in a production application
- **THEN** no output is written to stdout/stderr unless a `log` backend is configured

