## 1. Game Selection (pending approval)
- [ ] 1.1 Confirm game choice with project owner
- [ ] 1.2 Clone source repo to `.dev/` for reference

## 2. Project Setup
- [ ] 2.1 Create `games/<game-name>/` crate with Cargo.toml
- [ ] 2.2 Add to workspace members
- [ ] 2.3 Define module structure (components, resources, systems, render)

## 3. Core Implementation
- [ ] 3.1 Implement game-specific components and resources
- [ ] 3.2 Implement rendering setup (meshes, materials, pipeline)
- [ ] 3.3 Implement game logic systems
- [ ] 3.4 Implement input handling
- [ ] 3.5 Implement physics/collision

## 4. Polish
- [ ] 4.1 HUD / text UI
- [ ] 4.2 Sound effects (if AnvilKit audio is available)
- [ ] 4.3 Game state management (menu, play, game over)

## 5. Verification
- [ ] 5.1 Unit tests for core game logic
- [ ] 5.2 cargo check --workspace passes
- [ ] 5.3 cargo run -p <game-name> produces playable game
