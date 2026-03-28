<div align="center">

<img src="public/logo.svg" alt="AnvilKit" width="480" />

<br/><br/>

A modular game engine built with Rust — forging games from composable crates.

[![crates.io](https://img.shields.io/crates/v/anvilkit.svg?style=flat-square&color=00E5FF)](https://crates.io/crates/anvilkit)
[![docs.rs](https://img.shields.io/docsrs/anvilkit?style=flat-square&color=9D00FF)](https://docs.rs/anvilkit)
[![license](https://img.shields.io/crates/l/anvilkit?style=flat-square&color=FF0055)](LICENSE-MIT)
[![CI](https://img.shields.io/github/actions/workflow/status/ketd/AnvilKit/ci.yml?style=flat-square&label=CI)](https://github.com/ketd/AnvilKit/actions)

[Docs](https://anvilkit.io) · [Quick Start](https://anvilkit.io/en/docs/getting-started) · [Games](https://anvilkit.io/en/docs/games/craft) · [crates.io](https://crates.io/crates/anvilkit)

English | **[中文](README_ZH.md)**

</div>

---

## What is AnvilKit?

AnvilKit is a **modular game infrastructure** — not a monolithic engine, but a set of focused crates you compose to build exactly the game you need.

```toml
# Use the facade crate for everything:
[dependencies]
anvilkit = "0.1"

# Or pick individual crates:
[dependencies]
anvilkit-core = "0.1"
anvilkit-ecs = "0.1"
anvilkit-render = "0.1"
```

```rust
use anvilkit::prelude::*;

struct MyGame;

impl GameCallbacks for MyGame {
    fn init(&mut self, ctx: &mut GameContext) {
        ctx.app.add_systems(AnvilKitSchedule::Update, hello);
    }
}

fn main() {
    AnvilKitApp::run(
        GameConfig::new("My Game").with_size(1280, 720),
        MyGame,
    );
}

fn hello() {
    println!("Hello from AnvilKit!");
}
```

## Crate Map

```
                          ┌──────────┐
                          │ anvilkit │  ← facade, re-exports everything
                          └────┬─────┘
    ┌──────┬──────┬────┬───┼───┬───────┬────────┬─────┬────┬──────────┐
    ▼      ▼      ▼    ▼   ▼   ▼       ▼        ▼     ▼    ▼          ▼
 ┌──────┐┌───┐┌──────┐┌──────┐┌─────┐┌─────┐┌──────┐┌───┐┌──┐┌────────┐┌────┐
 │ core ││ecs││render││assets││input││audio││camera││app││ui││gameplay││data│
 └──────┘└─┬─┘└──┬───┘└──────┘└─────┘└─────┘└──────┘└─┬─┘└──┘└────────┘└────┘
           │     │                                     │
      bevy_ecs  wgpu + winit                      winit + ecs
```

| Crate | What it does | Key deps |
|-------|-------------|----------|
| **anvilkit-core** | Math (glam), transforms, time, errors, persistence | `glam` |
| **anvilkit-ecs** | ECS world, schedules, plugins, physics | `bevy_ecs` |
| **anvilkit-render** | GPU pipelines, sprites, particles, text | `wgpu`, `winit` |
| **anvilkit-assets** | glTF loader, asset server, procedural meshes | `gltf` |
| **anvilkit-input** | Keyboard/mouse/gamepad state, action mapping | `winit` |
| **anvilkit-audio** | Spatial audio, playback, mixing | `rodio` |
| **anvilkit-camera** | Camera system: 5 modes, trauma shake, spring arm, rail, transitions | `bevy_ecs`, `glam` |
| **anvilkit-app** | App runner, GameCallbacks, window lifecycle | `winit` |
| **anvilkit-ui** | Flexbox layout, events, widgets, themes | `taffy` |
| **anvilkit-gameplay** | Stats, health, inventory, cooldowns, effects | `bevy_ecs` |
| **anvilkit-data** | Data tables (RON/JSON), i18n locale | `ron` |

## Games

<table>
<tr>
<td width="50%">

### Craft

Minecraft-style voxel sandbox with terrain generation, block building, water, day/night cycle, greedy meshing, health system with fall damage and drowning, slot-based inventory, data-driven blocks, and player state persistence.

```bash
cargo run -p craft
```

</td>
<td width="50%">

### Billiards

PBR pool simulation with AABB physics, ball-to-ball collision, break shots, rule enforcement, and orbit camera controls.

```bash
cargo run -p billiards
```

</td>
</tr>
</table>

## CLI

The `anvil` CLI scaffolds new projects from templates:

```bash
cargo install anvilkit-cli
anvil new my-game --template first_person
cd my-game && cargo run
```

Templates: `3d_basic`, `first_person`, `topdown`

## Building from Source

```bash
git clone https://github.com/ketd/AnvilKit.git
cd AnvilKit
cargo build --workspace
cargo test --workspace
```

Run the docs site locally:

```bash
cd docs && pnpm install && pnpm dev
```

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache 2.0](LICENSE-APACHE), at your option.

## Acknowledgments

Built on the shoulders of [Bevy ECS](https://bevyengine.org/) · [wgpu](https://wgpu.rs/) · [winit](https://github.com/rust-windowing/winit) · [glam](https://github.com/bitshifter/glam-rs) · [rodio](https://github.com/RustAudio/rodio)

<div align="center">

---

**Forging games with Rust 🔨**

</div>
