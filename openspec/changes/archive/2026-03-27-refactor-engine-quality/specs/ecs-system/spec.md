## MODIFIED Requirements

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

## ADDED Requirements

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
