# Change: Refactor AnvilKit to AI-First Game Engine Architecture

## Why

AnvilKit's current architecture mirrors a traditional Rust game engine (modular crates for gameplay, camera, data, UI, etc.) but has no differentiation from Bevy/Fyrox/Macroquad. The project's stated goal is to be an **AI-agent-first game engine** — one where AI agents are first-class users alongside human developers. The current codebase has zero AI-specific infrastructure: no self-describing APIs, no agent tool schemas, no structured errors with fix hints, and no MCP integration. Meanwhile, 5 of 12 crates are premature abstractions that dilute focus (gameplay, data, camera, ui, cli).

This refactor realigns the codebase with the AI-first mission by: (1) removing premature crate abstractions, (2) replacing the custom `anvilkit-ecs::App` with `bevy_app::App` to eliminate ~665 lines of redundant framework code, (3) feature-gating advanced render effects, and (4) introducing 2 new AI-specific crates (`anvilkit-mcp`, `anvilkit-describe`) that form the actual competitive moat.

## What Changes

### Crate Deletions / Demotions
- **BREAKING** — Remove `anvilkit-gameplay` crate; move Health/Inventory/StatusEffect/Cooldown/EntityPool into `games/craft/src/gameplay/`
- **BREAKING** — Remove `anvilkit-data` crate; move DataTable/Locale into `games/craft/src/data/`
- **BREAKING** — Remove `anvilkit-camera` crate; move camera systems into `games/craft/src/camera/` (keep as game-specific, not engine-level)
- **BREAKING** — Remove `anvilkit-ui` crate; merge relevant types into `anvilkit-app` (egui is the UI strategy, taffy layout engine is unused by any game)
- **BREAKING** — Suspend `anvilkit-cli` (template scaffolding is premature without external users)

### Core Simplification
- **BREAKING** — Replace `anvilkit-ecs::App` with `bevy_app::App`; delete custom App/Plugin/Schedule/DeltaTime/AppExit reimplementations (~665 lines)
- **BREAKING** — Replace `anvilkit_ecs::schedule::AnvilKitSchedule` with Bevy's native schedule labels (`PreUpdate`, `Update`, `PostUpdate`, `FixedUpdate`, etc.)
- Resolve duplicate `transform.rs` — keep one canonical location (anvilkit-core or anvilkit-ecs, not both)
- Simplify `GameContext` construction in `AnvilKitApp` (6 redundant constructions in `window_event()`)

### Render Feature-Gating
- Move SSAO, DoF, Motion Blur, Color Grading, IBL behind `advanced-render` Cargo feature flag
- Keep in tree but don't compile by default — reduces API surface for agents
- Default build provides: basic PBR, sprites, text, debug lines, shadow, bloom

### New AI-First Crates
- **`anvilkit-describe`** — `Describe` trait + derive macro; every Component/Resource self-reports schema (name, fields, types, defaults, constraints, usage example) as structured data
- **`anvilkit-mcp`** — MCP (Model Context Protocol) server exposing engine tools: `spawn_entity`, `query_world`, `inspect_component`, `list_resources`, `capture_frame`, `replay_frame`

### Structured Error System
- All engine errors gain `hint: String` and `suggested_fix: Option<String>` fields
- Errors are JSON-serializable for agent consumption

### Project Positioning
- Update `openspec/project.md` to reflect AI-first mission
- Add `POSITIONING.md` to repo root defining what "AI-first" means for AnvilKit

## Impact
- Affected specs: `ecs-system`, `engine-dx`, `render-system`, `render-post-processing`, `render-advanced`, `camera-system`, `ui-framework`, `cli-tooling`, `game-craft`, `game-billiards`
- New spec: `ai-agent-interface`
- Affected code: all crates, both games, all examples
- Breaking changes: games must update imports; external users (if any) must migrate
- Net code delta: estimated -3000 to -4000 lines (deletions exceed additions in Phase 1)
