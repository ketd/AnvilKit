# Change: Engine v0.3 Architecture — Crate Restructuring, UI Core, Gameplay Systems, Dead Code Cleanup

## Why
Full audit of the AnvilKit engine reveals 5 categories of architectural debt accumulated during v0.1–v0.2 rapid development:
1. **God files and mixed concerns** — events.rs (1414 lines, 6 responsibilities), ui.rs (898 lines, 5 layers), draw.rs (568 lines, 10+ types), physics.rs (892 lines, 3 systems)
2. **Missing core systems** — No UI framework (just render primitives), no gameplay systems (Stats/Inventory/Cooldown), no app runner (300 lines of boilerplate per game), no data tables
3. **Disconnected systems** — Settings→engine, ActionMap→games, AudioPlugin→games, AssetServer/Cache/Dependencies, SerializableRegistry are all scaffolded but never wired
4. **Dead code** — 15+ deprecated/unused items, DebugMode variants with no shader support, ShadowAtlas never used, StateTransitionEvent never emitted
5. **Code duplication** — Cached VB pattern (5 copies), ortho uniform structs (5 copies), LineRenderer vs DebugRenderer, 300+ lines of identical game boilerplate

## What Changes

### New Crates
- **`anvilkit-app`** — Event loop, frame lifecycle, input forwarding, resize handling (eliminates game boilerplate)
- **`anvilkit-ui`** — UI node tree, flexbox layout, event/focus system, text integration, widget library (extracted from render)
- **`anvilkit-gameplay`** — Stats/Health, Inventory, Cooldown, StatusEffect, EntityPool, DataTable
- **`anvilkit-data`** — Data-driven configuration tables, i18n/localization

### Crate Restructuring
- Extract persistence module from `anvilkit-core` → dedicated persistence handling
- Extract UI data model + layout from `anvilkit-render` → `anvilkit-ui`
- Split `physics.rs` into module directory (components / aabb / rapier)
- Move `DeltaTime` from `physics.rs` to `app.rs`
- Move `Aabb` and `raycast` from render to `anvilkit-core::math`
- Consolidate `debug.rs` + `debug_renderer.rs`
- Merge `LineRenderer` into `DebugRenderer`

### Fix Disconnections
- Wire Settings → BloomSettings/SsaoSettings/AudioBus
- Wire ActionMap → both games (replace hardcoded KeyCode)
- Integrate AssetServer ↔ AssetCache ↔ DependencyGraph
- Wire SerializableRegistry into SceneSerializer save/load
- Add AudioPlugin to both games
- Add CameraPlugin + include in DefaultPlugins
- Emit StateTransitionEvent in state_transition_system

### Cleanup
- Remove 15+ deprecated/dead items
- Deduplicate cached VB pattern → shared utility
- Deduplicate ortho uniform structs → shared type
- Fix AudioEngine unsafe Send+Sync
- Add Persistence error category
- Remove DebugOverlay dead flags, DebugMode unimplemented variants
- Remove shadow.rs unused types (PointShadowConfig, SpotShadowConfig, ShadowAtlas)

## Impact
- Affected specs: app-runner (create), ui-core (create), gameplay-systems (create), data-tables (create), ecs-system, render-system, asset-system, audio-system, input-system, camera-system, persistence, core-math, engine-dx
- Affected code: All 8 engine crates, both games, examples, CLI tools
- **BREAKING**: DeltaTime import path changes, UI types move to new crate, persistence types move
- Migration: Re-exports in original locations will maintain backward compat during transition
