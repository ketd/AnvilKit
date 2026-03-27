## Context
AnvilKit v0.2 shipped all planned features (Tier 1–4 + quality refactor), but rapid development left significant architectural debt. The engine works but is hard to extend: game developers must write 300+ lines of boilerplate, UI is unusable without manual wiring, and gameplay systems don't exist. v0.3 focuses on structure, not features.

## Goals / Non-Goals

### Goals
- Games can start with <20 lines of setup code (App runner handles lifecycle)
- UI system is usable: spawn UiNode entities, get layout + rendering + events automatically
- Common gameplay patterns (HP, inventory, cooldowns) are available as optional engine modules
- Each crate has a single clear responsibility
- Zero dead code in public API
- Settings actually control engine behavior

### Non-Goals
- No new rendering features (shaders, post-processing, etc.)
- No new physics features beyond cleanup
- No networking transport implementation (remains framework-only)
- No WASM/mobile platform support yet
- No visual editor / inspector tool

## Decisions

### Decision: New `anvilkit-app` crate for game loop
**Why:** Both games implement 300+ lines of identical ApplicationHandler boilerplate. The pattern is: create EventLoop → implement resumed/window_event/device_event/about_to_wait → forward input → tick ECS → manage DeltaTime → handle resize → render. This is engine infrastructure, not game code.

**Design:** `AnvilKitApp` wraps `RenderApp` + `App` + winit event loop. Games provide a `GameCallbacks` trait with `fn init(&mut self, world: &mut World)`, `fn post_update(&mut self, world: &mut World)`, and `fn render(&mut self, ...)`. The engine handles everything else.

**Alternative considered:** Just provide helper functions instead of a new crate. Rejected because the lifecycle ordering (input → ECS update → post-update → render → end_frame) is critical and error-prone.

### Decision: Extract UI to `anvilkit-ui`, keep GPU rendering in render crate
**Why:** ui.rs mixes 5 conceptual layers. The data model (UiNode, UiStyle, UiText) and layout engine (taffy) have zero GPU dependency. Hit testing and events are interaction logic. Only the vertex buffer + pipeline creation is rendering.

**Design:** `anvilkit-ui` owns: UiTree (parent/children), UiStyle, UiNode, UiText, UiLayoutEngine, UiEvents, Widget factories, focus management, keyboard navigation. `anvilkit-render` keeps: UiRenderer (GPU pipeline + vertex buffer), UiVertex. The render crate depends on `anvilkit-ui` for types.

### Decision: `anvilkit-gameplay` as optional feature-gated modules
**Why:** Not every game needs inventory or cooldowns. Feature flags let games opt in to specific systems.

**Design:** `features = ["stats", "inventory", "cooldown", "status-effect", "entity-pool", "data-table"]`. Each feature adds a module and a Plugin. All types derive Component/Resource for ECS integration.

### Decision: Move DeltaTime to app.rs, keep re-export in physics
**Why:** DeltaTime is used by 13+ modules across 5 crates. It is a core engine concept, not a physics concept. Moving it to `app.rs` (where the frame loop lives) is semantically correct.

**Migration:** `pub use crate::app::DeltaTime;` in physics.rs maintains backward compat.

### Decision: Split physics.rs into module directory
**Why:** 892 lines mixing abstract components, AABB collision, rapier integration, and joints is unmaintainable.

**Design:**
```
physics/
├── mod.rs         — re-exports, DeltaTime (re-export from app)
├── components.rs  — RigidBody, Collider, Velocity, ColliderShape, etc.
├── aabb.rs        — AabbCollider, collision_detection_system, PhysicsPlugin
├── rapier.rs      — RapierContext, sync systems, joints, RapierPhysicsPlugin
└── events.rs      — CollisionEvent (replaces deprecated CollisionEvents)
```

## Risks / Trade-offs

- **Risk:** Breaking import paths during crate extraction.
  **Mitigation:** Re-export types at original paths during transition. Deprecate old paths.

- **Risk:** Large scope — 100+ tasks across 10+ crates.
  **Mitigation:** Phased execution. Phase 0–1 (cleanup + app runner) can ship independently.

- **Trade-off:** Adding crates increases compile time for first build.
  **Mitigation:** New crates are small and incremental compilation handles subsequent builds well.

## Migration Plan
1. Phase 0: Dead code removal, deduplication (no breaking changes)
2. Phase 1: App runner crate + migrate games (breaking: games must adopt new pattern)
3. Phase 2: UI crate extraction (breaking: UI type import paths change)
4. Phase 3: Gameplay crate creation (additive, no breaking changes)
5. Phase 4: Crate splits (physics, persistence) with re-exports (soft-breaking)
6. Phase 5: Fix disconnections (Settings, ActionMap, Audio, Assets)
7. Phase 6: Data tables, i18n, animation graph (additive)

## Open Questions
- Should `anvilkit-physics` be a separate crate or remain a module in `anvilkit-ecs`? Current decision: module directory first, extract to crate if it grows further.
- Should `anvilkit-debug` be a separate crate? Current decision: consolidate within render, extract only if debug tools grow significantly.
