# AnvilKit Roadmap

## Phase 1: Simplification (COMPLETE)

Removed premature abstractions, upgraded core dependencies, established AI-first architecture.

- [x] Upgraded bevy_ecs + bevy_app from 0.14 to 0.15
- [x] Replaced custom `App`/`Plugin`/`Schedule` with bevy_app native types
- [x] Demoted 5 crates from umbrella (camera, gameplay, data, ui, cli)
- [x] Feature-gated advanced render effects (SSAO, DoF, MotionBlur, ColorGrading)
- [x] 1075 tests, 0 failures, 0 warnings

## Phase 2: AI Infrastructure (IN PROGRESS)

Building the competitive moat — the things that make AnvilKit uniquely useful for AI agents.

- [x] `anvilkit-describe` — `Describe` trait + `#[derive(Describe)]` proc macro
- [x] `anvilkit-describe` applied to 10 engine types with full field annotations
- [x] `anvilkit-mcp` — MCP server scaffold with JSON-RPC protocol + ToolRegistry + 5 built-in tools
- [x] `McpPlugin` — bevy plugin that runs stdio MCP server alongside the game
- [x] Structured errors — `code()`, `hint()`, `to_agent_string()` on `AnvilKitError`
- [ ] Expand MCP tools: `list_components`, `spawn_entity` (with JSON component spec), `query_world`, `inspect_component`
- [ ] Apply `#[derive(Describe)]` to ALL remaining pub engine types
- [ ] Compile-time enforcement that all pub types implement `Describe`

## Phase 3: Validation (NOT STARTED)

Prove the thesis with a real experiment.

- [ ] **Flappy Bird benchmark**: Cold-start agent builds a playable Flappy Bird using only AnvilKit's public API
- [ ] Target: < 30 minutes, < 5 friction points
- [ ] Iterate on API/Describe/errors based on friction log
- [ ] Public announcement if benchmark passes

## Future

- Deterministic replay for agent debugging loops
- `capture_frame()` → base64 PNG for visual inspection
- HTTP/SSE transport option for remote agent connections
- wgpu 0.24 upgrade
- WebAssembly target for browser-based agent testing
