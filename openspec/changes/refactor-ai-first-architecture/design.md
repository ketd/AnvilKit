## Context

AnvilKit is a solo-developed Rust game engine (62k lines, 12 crates) that aspires to be the first AI-agent-native game engine. The current codebase was built as a traditional modular engine with no AI-specific infrastructure. This design document covers the technical decisions for the AI-first pivot.

**Stakeholders**: Project author (ketd), future AI agent users, future human developers.

**Constraints**:
- Solo developer — every crate added must earn its maintenance cost
- bevy_ecs 0.14 dependency is kept (replacing ECS is out of scope)
- wgpu 0.19 / winit 0.30 are kept (upgrades are a separate change)
- Two existing games (craft, billiards) must keep working after refactor

## Goals / Non-Goals

### Goals
- Reduce engine crate count from 12 to ~8 by removing premature abstractions
- Replace custom App/Schedule layer with bevy_app to eliminate redundant code
- Introduce `Describe` trait as the foundation for AI-agent introspection
- Introduce MCP server crate for agent-engine interaction
- Make all engine errors structured and agent-parseable
- Establish "cold-start agent builds a game in 30 minutes" as the north-star metric

### Non-Goals
- Replacing bevy_ecs with a custom ECS
- Upgrading wgpu/winit/bevy_ecs versions (separate change)
- Building an editor GUI
- Supporting mobile/web platforms
- Implementing full game-specific systems in engine crates

## Decisions

### D1: Upgrade to bevy 0.15 + Replace anvilkit-ecs::App with bevy_app::App

**Decision**: Upgrade entire bevy dependency from 0.14 → 0.15. Depend on `bevy_app` crate directly. Delete `anvilkit_ecs::app.rs` (665 lines).

**Rationale**: Current `anvilkit_ecs::App` reimplements 95% of `bevy_app::App` (add_plugins, add_systems, insert_resource, add_event, FixedUpdate accumulator, AppExit). The only unique features are `register_serializable` (can be an extension trait) and `AnvilKitSchedule::Main` (can be a custom label on bevy schedules). `App::run()` is dead code — games all use `AnvilKitApp::run()`.

Upgrading to bevy 0.15 unlocks:
- **Required Components** — cleaner entity spawning, better for agent-generated code
- **Improved scheduling** — more flexible system ordering
- **bevy_reflect improvements** — useful for Describe trait integration
- **Entity::from_raw stability** — better for MCP entity ID references

**Trade-off**: Pulls in `bevy_app` + `bevy_time` + `bevy_tasks` + `bevy_utils`. Acceptable because these replace equivalent hand-rolled code. bevy 0.14 → 0.15 migration has breaking changes in ECS query syntax and system params.

**Migration**: `bevy_ecs 0.14` → `bevy_ecs 0.15`. `anvilkit_ecs::app::App` → `bevy_app::App`. `AnvilKitSchedule::Update` → `bevy_app::Update`. `anvilkit_ecs::app::DeltaTime` → `bevy_time::Time`. Games update imports; `AnvilKitApp<G>` holds `bevy_app::App` instead.

### D2: Demote gameplay/data/camera/ui crates to game-level code

**Decision**: Move `anvilkit-gameplay`, `anvilkit-data`, `anvilkit-camera`, `anvilkit-ui` contents into `games/craft/src/` modules. Remove from workspace.

**Rationale**: These crates have exactly 1 consumer each (craft). Premature abstractions:
- `anvilkit-gameplay`: Health/Inventory/StatusEffect are game-specific (every game has different needs)
- `anvilkit-data`: DataTable is `HashMap<K,V>` + RON; Locale is game-specific i18n
- `anvilkit-camera`: Camera controller modes are game-specific (FPS vs orbit vs top-down)
- `anvilkit-ui`: Taffy-based layout engine unused by either game; both games use egui

**When to re-extract**: When a second game genuinely needs the same abstraction. The code isn't deleted — it moves to craft/ and can be re-extracted later with a real API boundary.

### D3: Feature-gate advanced render effects

**Decision**: Move SSAO, DoF, Motion Blur, Color Grading, IBL behind `advanced-render` Cargo feature. Default build includes: PBR, sprites, text, debug lines, shadow, bloom.

**Rationale**: Agent API surface area directly impacts agent usability. 6 extra post-processing systems with ~20 configurable parameters each means the agent must understand them all. Most voxel/2D games don't need them. Feature-gating keeps them available for users who want them while reducing default complexity.

### D4: Describe trait design

**Decision**: Introduce a `Describe` trait with a derive macro.

```rust
pub trait Describe {
    fn schema() -> ComponentSchema;
}

pub struct ComponentSchema {
    pub name: &'static str,
    pub description: &'static str,
    pub fields: Vec<FieldSchema>,
    pub example: &'static str,
}

pub struct FieldSchema {
    pub name: &'static str,
    pub type_name: &'static str,
    pub description: &'static str,
    pub default: Option<String>,
    pub range: Option<(String, String)>,
}
```

**Rationale**: AI agents need to discover and understand engine types without reading source code. The `Describe` trait provides machine-readable introspection that can be:
- Queried via MCP tools
- Included in `cargo doc` output
- Used to generate tool schemas automatically

**Alternative considered**: Using Rust's built-in reflection (not available) or `bevy_reflect` (too heavy, not agent-oriented). A purpose-built trait with derive macro is simplest.

### D5: MCP server architecture

**Decision**: New `anvilkit-mcp` crate providing an MCP server that runs alongside the game.

**Core tools**:
- `list_resources` — enumerate all ECS resources with their Describe schemas
- `list_components` — enumerate registered component types
- `spawn_entity` — create entity with specified components (JSON input)
- `query_world` — run ECS queries with filter expressions
- `inspect_component` — get component data for a specific entity
- `modify_component` — update component fields on a live entity
- `capture_frame` — screenshot current frame as base64 PNG
- `replay_frame(n)` — re-render frame N with current state (requires deterministic replay)

**Rationale**: MCP is the emerging standard for AI agent tool interfaces. An MCP-native game engine lets any MCP-compatible agent (Claude, GPT, etc.) interact with the engine natively without custom integration.

**Phase 1 scope**: `list_resources`, `list_components`, `spawn_entity`, `query_world`, `inspect_component`. Phase 2 adds `capture_frame` and `replay_frame`.

### D6: Structured error system

**Decision**: All engine error types gain structured fields.

```rust
pub struct EngineError {
    pub code: &'static str,       // e.g. "RENDER_PIPELINE_CREATE_FAILED"
    pub message: String,           // human-readable
    pub hint: String,              // agent-readable suggestion
    pub suggested_fix: Option<String>,  // concrete code patch if applicable
    pub context: HashMap<String, String>, // key-value context
}
```

**Rationale**: Current errors use `thiserror` with human-readable messages. Agents need structured data: error code for matching, hint for next action, suggested fix for auto-repair.

## Risks / Trade-offs

| Risk | Impact | Mitigation |
|------|--------|------------|
| bevy_app version coupling | Medium — bevy_app updates may break AnvilKitApp | Pin version, upgrade proactively |
| Demoted crates lost if craft/ refactored | Low — git history preserves all code | Tag pre-refactor commit |
| MCP server adds runtime overhead | Low — only active when agent connected | Feature-gated, lazy initialization |
| Describe derive macro maintenance | Medium — proc macros are complex | Keep derive simple, hand-impl for complex types |

## Migration Plan

### Phase 1: Simplification (2-3 weeks)
1. Tag current state as `v0.3-pre-refactor`
2. Replace `anvilkit-ecs::App` → `bevy_app::App`
3. Move gameplay/data/camera/ui code into craft/
4. Remove 5 crates from workspace
5. Fix all games and examples
6. Verify: `cargo test`, `cargo build --all`

### Phase 2: AI Infrastructure (3-4 weeks)
1. Implement `anvilkit-describe` crate + derive macro
2. Add `#[derive(Describe)]` to all engine types
3. Implement `anvilkit-mcp` crate (Phase 1 tools)
4. Add structured error fields to engine errors
5. Feature-gate advanced render effects

### Phase 3: Validation (1-2 weeks)
1. Run "Flappy Bird test": new Claude session, only pub API docs, build game from scratch
2. Record prompt log, identify top 10 friction points
3. Iterate on Describe schemas and MCP tools based on results
4. Write `POSITIONING.md`

## Resolved Decisions (formerly Open Questions)

1. **bevy version**: **Upgrade to bevy 0.15 full suite.** Both `bevy_ecs` and `bevy_app` upgrade from 0.14 → 0.15. This unlocks Required Components, improved scheduling, and other 0.15 features that benefit AI-first ergonomics. Adds migration work but is a one-time cost.

2. **MCP transport**: **stdio.** JSON-RPC over stdin/stdout, same as Claude Code's MCP. Agent launches engine as subprocess. Simplest implementation, no auth/concurrency complexity. HTTP/SSE can be added later as a separate feature if remote debugging is needed.

3. **Describe trait scope**: **Mandatory for ALL pub Component/Resource types.** Both engine and game types. Enforced via `#[deny(missing_describe)]` custom lint or compile-time check. This is the strictest option — maximally agent-friendly. craft game serves as the reference implementation with 100% Describe coverage.

4. **billiards game**: **Full rewrite, no backward compatibility.** Don't preserve old code — rewrite billiards from scratch using the new API after Phase 1 is complete. No `games/shared/` crate. Each game is fully self-contained. The old billiards code is in git history if needed.
