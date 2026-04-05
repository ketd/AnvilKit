## Phase 1: Simplification — Remove premature abstractions, replace custom App

### 1.1 Pre-flight
- [x] 1.1.1 Tag current commit as `v0.3-pre-refactor` for rollback safety
- [x] 1.1.2 Ensure `cargo test --workspace` passes (baseline)
- [x] 1.1.3 Ensure `cargo build --workspace` has 0 warnings (baseline)

### 1.2 Upgrade bevy 0.14 → 0.15 + Replace anvilkit-ecs::App with bevy_app::App
- [x] 1.2.1 Upgrade `bevy_ecs = "0.14"` → `bevy_ecs = "0.15"` in workspace Cargo.toml
- [x] 1.2.2 Add `bevy_app = "0.15"` to workspace dependencies
- [x] 1.2.3 Fix bevy 0.14 → 0.15 breaking changes across all crates (`app.world` field→method, `Events::get_reader`→`get_cursor`, Entity Result API)
- [x] 1.2.4 Added `AppExt` extension trait with `exit_game()` (wraps bevy's AppExit event)
- [x] 1.2.5 Replace all `use anvilkit_ecs::app::App` with re-export of `bevy_app::App`
- [x] 1.2.6 Keep `AnvilKitSchedule` labels, hook into `MainScheduleOrder`
- [x] 1.2.7 Keep `DeltaTime` as thin newtype alongside bevy_time::Time
- [x] 1.2.8 Replace custom `AppExit` resource with `bevy_app::AppExit` event
- [x] 1.2.9 Update `AnvilKitApp<G>` to hold `bevy_app::App`
- [x] 1.2.10 Gut `anvilkit-ecs/src/app.rs` (665 → ~80 lines)
- [x] 1.2.11 Run `cargo test --workspace` — 0 failures
- [x] 1.2.12 Run `cargo build --workspace` — 0 warnings

### 1.3 Resolve duplicate transform.rs
- [x] 1.3.1 Diff confirmed: NOT duplicates. Clean layering: core=math, ecs=hierarchy wrapper via `pub use`
- [x] 1.3.2 No action needed — existing layering is correct
- [x] 1.3.3 No imports need updating

### 1.4 Demote anvilkit-gameplay (remove from umbrella)
- [x] 1.4.1-1.4.7 Kept crate in-place, removed from umbrella re-exports. Craft depends directly via its own Cargo.toml. No file moves needed.

### 1.5 Demote anvilkit-data (remove from umbrella)
- [x] 1.5.1-1.5.6 Same approach: kept crate, removed from umbrella. Craft depends directly.

### 1.6 Demote anvilkit-camera (remove from umbrella + DefaultPlugins)
- [x] 1.6.1-1.6.2 Kept crate, removed from umbrella and `DefaultPlugins`
- [x] 1.6.3 Billiards and craft add `CameraPlugin` explicitly via `app.add_plugins(anvilkit_camera::plugin::CameraPlugin)`
- [x] 1.6.4-1.6.6 Games depend on `anvilkit-camera` directly in their Cargo.toml

### 1.7 Demote anvilkit-ui (remove from umbrella)
- [x] 1.7.1 Audit: NO game code uses anvilkit-ui directly (both games use egui)
- [x] 1.7.2 Removed from umbrella. Kept as internal dep of anvilkit-render.
- [x] 1.7.3-1.7.5 No further action needed at this stage

### 1.8 Suspend anvilkit-cli
- [x] 1.8.1 Removed `"tools/anvilkit-cli"` from workspace members
- [x] 1.8.2 Directory preserved in tree

### 1.9 Simplify GameContext construction
- [x] 1.9.1 Added `game_ctx!()` macro to eliminate 7 repeated `GameContext { ... }` constructions

### 1.10 Update umbrella crate
- [x] 1.10.1 Removed deleted crate re-exports from `crates/anvilkit/src/lib.rs`
- [x] 1.10.2 Removed `CameraPlugin` from `DefaultPlugins`
- [x] 1.10.3 Updated prelude to exclude demoted crates

### 1.11 Fix examples and games
- [x] 1.11.1-1.11.3 All games and examples compile; 41 files migrated for bevy 0.15 breaking changes
- [ ] 1.11.4 Rewrite `games/billiards/` from scratch — deferred to dedicated session
- [x] 1.11.5 Final `cargo test --workspace` — 1071 tests passed, 0 failures
- [x] 1.11.6 Final `cargo build --workspace` — 0 warnings
- [ ] 1.11.7 Manual smoke test: run craft (user verification)
- [ ] 1.11.8 Manual smoke test: run billiards (user verification)

## Phase 2: AI Infrastructure — Build the moat

### 2.1 anvilkit-describe crate
- [x] 2.1.1 Created `crates/anvilkit-describe/` with Cargo.toml
- [x] 2.1.2 Defined `Describe` trait: `fn schema() -> ComponentSchema`
- [x] 2.1.3 Defined `ComponentSchema`, `FieldSchema` structs (serde-serializable)
- [x] 2.1.4 Implemented `#[derive(Describe)]` proc macro (in `anvilkit-describe-derive`)
- [x] 2.1.5 Added `#[describe(range = "...", hint = "...", default = "...")]` field attributes
- [x] 2.1.6 Describe added to all pub engine Component/Resource types across core/ecs/render/app/input/audio
- [ ] 2.1.7 Add compile-time enforcement — deferred to future iteration
- [ ] 2.1.8 Add Describe to craft game types — deferred
- [x] 2.1.9 Added to umbrella crate and workspace
- [x] 2.1.10 Tests: 7 unit tests verify derive generates correct schemas
- [x] 2.1.11 Tests: JSON serialization verified

### 2.2 Structured error system
- [x] 2.2.1 Added `code()`, `hint()`, `to_agent_string()` methods to existing `AnvilKitError`
- [x] 2.2.2 `to_agent_string()` outputs parseable `[CODE] message | hint: <hint>` format
- [ ] 2.2.3 Migrate anvilkit-render errors to use specific error codes (beyond generic RENDER_ERROR)
- [ ] 2.2.4 Migrate anvilkit-assets errors to use specific error codes
- [x] 2.2.5 Added hints for all 13 error categories (Render, Physics, Asset, Audio, etc.)
- [x] 2.2.6 Tests: 4 tests verify code(), hint(), to_agent_string(), parseability

### 2.3 anvilkit-mcp crate (Phase 1 tools)
- [x] 2.3.1 Created `crates/anvilkit-mcp/` with Cargo.toml
- [x] 2.3.2 Implemented minimal stdio JSON-RPC 2.0 protocol types (`protocol.rs`)
- [x] 2.3.3 Implemented `ToolRegistry` + 5 built-in tools (`list_entities`, `entity_count`, `engine_info`, `spawn_empty_entity`, `despawn_entity`)
- [x] 2.3.4 Implemented `get_component_ids` tool (list component types on entity)
- [x] 2.3.5 Implemented `get_world_summary` tool (entity/resource/archetype counts)
- [x] 2.3.6 Implemented `list_schedules` tool (enumerate registered schedules)
- [ ] 2.3.7 Implement `spawn_entity` with typed component JSON spec — requires bevy_reflect integration
- [x] 2.3.8 Added `McpPlugin` (bevy Plugin) + `McpServer` (stdio JSON-RPC loop on background thread)
- [x] 2.3.9 Feature-gate: `mcp` feature in anvilkit umbrella crate
- [x] 2.3.10 Integration tests: 16 tests (spawn/despawn round-trip, error handling)

### 2.4 Feature-gate advanced render effects
- [x] 2.4.1 Defined `advanced-render` feature in anvilkit-render Cargo.toml
- [x] 2.4.2 Gated SSAO module behind feature
- [x] 2.4.3 Gated DoF module behind feature
- [x] 2.4.4 Gated Motion Blur module behind feature
- [x] 2.4.5 Gated Color Grading module behind feature
- [x] 2.4.6 IBL kept ungated — required by PBR pipeline (intentional decision, not a TODO)
- [x] 2.4.7 Verified default build compiles without advanced-render
- [x] 2.4.8 Verified `--features advanced-render` compiles with all effects

## Phase 3: Validation — Prove the AI-first thesis

### 3.1 Flappy Bird experiment
- [ ] 3.1.1 Open new Claude Code session with ONLY `cargo doc --open` output
- [ ] 3.1.2 Prompt: "Using AnvilKit, build a playable Flappy Bird clone"
- [ ] 3.1.3 Record entire prompt log (all agent messages + tool calls)
- [ ] 3.1.4 Identify top 10 friction points where agent got stuck
- [ ] 3.1.5 File issues for each friction point

### 3.2 Iterate based on experiment
- [ ] 3.2.1 Fix top 5 friction points (API naming, missing Describe impls, error hints)
- [ ] 3.2.2 Re-run Flappy Bird experiment, compare friction count
- [ ] 3.2.3 Target: agent completes game in <30 minutes with <5 friction points

### 3.3 Positioning
- [x] 3.3.1 Wrote `POSITIONING.md` — defines AI-first mission
- [x] 3.3.2 Updated `openspec/project.md` — AI-first purpose, bevy 0.15, new workspace structure
- [x] 3.3.3 Updated root README.md with AI-first tagline and positioning links
- [x] 3.3.4 Wrote `ROADMAP.md` with Phase 1/2/3 milestones
