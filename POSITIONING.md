# AnvilKit — The First AI-Agent-Native Game Engine

## What AnvilKit Is

**AnvilKit is a Rust game engine designed for AI agents as first-class users.**

Every existing game engine (Unity, Unreal, Godot, Bevy, Fyrox) assumes a human developer sits at a keyboard, clicks editor widgets, reads API docs, and debugs visually. AnvilKit makes the opposite bet: **AI agents are the primary user, humans are the secondary user.**

This is a different engine, not a better version of an existing one.

## What "AI-First" Means Here

Five concrete commitments define AnvilKit's AI-native design:

### 1. Self-Describing API Types
Every public `Component` and `Resource` implements the `Describe` trait. Agents query `BloomSettings::schema()` and get back machine-readable JSON with:
- Field names and Rust types
- Valid value ranges
- Default values
- Human-written hints explaining intent
- Usage examples

No reading source code. No guessing at parameters. No hallucinating APIs.

### 2. MCP-Native Tool Interface
AnvilKit ships an MCP (Model Context Protocol) server. Any MCP-compatible agent (Claude, GPT, local models) can connect via stdio JSON-RPC and call engine tools natively:

```
spawn_entity(components) → entity_id
query_world(filter) → [entities]
inspect_component(entity, type) → component_data
list_resources() → [schemas]
capture_frame() → base64_png
replay_frame(n) → diff
```

Not a chat wrapper. Not a prompt template. A first-class agent interface equivalent to how humans use `cargo doc`.

### 3. Structured Errors with Fix Hints
Every engine error is JSON-serializable and includes:
- `code`: stable identifier for pattern matching (`"ASSET_NOT_FOUND"`)
- `message`: human-readable description
- `hint`: what to try next (`"Check that the file path is relative to assets/"`)
- `suggested_fix`: optional code patch
- `context`: key-value metadata about the failure

Agents don't waste tokens on error archaeology.

### 4. Deterministic Replay
Agents fix bugs by editing code → running → observing difference. AnvilKit makes this loop explicit: `replay_frame(n)` re-renders frame N with current code, returning a visual diff. No manual game session required.

### 5. Minimal Default API Surface
Default build includes: PBR, sprites, text, debug lines, shadows, bloom. Advanced effects (SSAO, DoF, Motion Blur, Color Grading, IBL) live behind `advanced-render` feature. Less surface area = fewer decisions for the agent = faster time-to-working-game.

## What AnvilKit Is NOT

- **Not a Bevy fork.** AnvilKit uses `bevy_ecs` and `bevy_app` directly. No custom ECS reimplementation. The value is in the AI layer, not in reinventing solved problems.
- **Not a visual editor.** There is no Inspector, no Prefab system, no Blueprint graphs. Code is the interface.
- **Not feature-competitive with Bevy/Unity.** AnvilKit has fewer rendering features, no asset editor, no networking prefab. It has what AI agents need to build small-to-medium games efficiently.
- **Not for humans who love GUI tools.** If you want Unity, use Unity.

## Success Metric

**A cold-start AI agent (no prior AnvilKit context) can build a playable Flappy Bird clone in under 30 minutes using only AnvilKit's public API.**

This is the north star. Every feature decision, API change, and architectural choice is evaluated against this benchmark.

## Current Status

- **Phase 1** (simplification): Complete. Upgraded to bevy 0.15, replaced custom `App` with `bevy_app::App`, demoted 5 premature-abstraction crates, zero warnings.
- **Phase 2** (AI infrastructure): In progress. `anvilkit-describe` trait + derive macro complete. MCP server and structured errors pending.
- **Phase 3** (validation): Not started. Flappy Bird experiment will validate the thesis.

See `openspec/changes/refactor-ai-first-architecture/` for the full roadmap.

---

*AnvilKit is an experiment. If the Flappy Bird benchmark fails, the thesis is wrong and we pivot. If it succeeds, we're building a new category.*
