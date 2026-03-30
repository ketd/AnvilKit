## Context

AnvilKit is a Rust game engine using wgpu 0.19 + winit 0.30 + bevy_ecs 0.14. The custom UI system (UiRenderer + TextRenderer) handles basic rectangles and ASCII text but cannot produce functional game menus. egui is the de facto standard for immediate-mode GUI in Rust, with official wgpu and winit bindings.

### Goals
- Games can build complete UI (menus, settings, inventory) with ~50 lines of egui code
- Support custom textures/images in UI (block icons, button backgrounds, health hearts)
- TrueType font rendering with proper text layout (centering, wrapping, CJK)
- Zero boilerplate: egui integration handled entirely by AnvilKitApp
- Custom theming: games can set Minecraft-style or any other visual style via `egui::Visuals`
- Input sharing: egui consumes mouse/keyboard when UI is active, passes through when not

### Non-Goals
- Replacing the retained-mode UiNode/UiLayoutEngine system (it serves a different purpose: ECS-driven in-world UI)
- Building a visual UI editor
- Supporting multiple egui contexts per window

## Decisions

### Decision: egui integration lives in `anvilkit-app`, not `anvilkit-render`
**Why:** The integration requires access to winit Window (for input forwarding), the wgpu Device/Queue (for rendering), AND the ECS world (for state). Only `anvilkit-app` has access to all three. Putting it in `anvilkit-render` would require plumbing winit events through additional APIs.

**Design:**
```
anvilkit-app/src/egui_integration/
├── mod.rs        — EguiIntegration struct, public API
├── state.rs      — egui_winit::State wrapper, input forwarding
└── renderer.rs   — egui_wgpu::Renderer wrapper, paint to swapchain
```

### Decision: New `GameCallbacks::ui()` method (not replacing `render()`)
**Why:** egui UI must be drawn AFTER the 3D scene but BEFORE frame present. The `render()` callback draws the 3D scene. Adding a separate `ui()` callback makes the lifecycle explicit: `render()` → `ui()` → present.

**Design:**
```rust
pub trait GameCallbacks: 'static {
    fn init(&mut self, ctx: &mut GameContext) {}
    fn post_update(&mut self, ctx: &mut GameContext) {}
    fn render(&mut self, ctx: &mut GameContext) {}
    fn ui(&mut self, ctx: &mut GameContext, egui_ctx: &egui::Context) {}
    fn on_resize(&mut self, ctx: &mut GameContext, width: u32, height: u32) {}
    fn on_window_event(&mut self, ctx: &mut GameContext, event: &WindowEvent) -> bool { false }
}
```

Frame lifecycle: `tick() → post_update() → render() → ui() → egui_render → present`.

Games that don't implement `ui()` get a no-op — zero overhead (egui skips rendering when no widgets are drawn).

### Decision: Texture bridge via `EguiTextures` resource
**Why:** Games need to display custom images (block icons, button backgrounds) in egui. The bridge registers wgpu TextureViews as egui TextureIds.

**Design:**
```rust
#[derive(Resource)]
pub struct EguiTextures {
    texture_map: HashMap<String, egui::TextureId>,
}

impl EguiTextures {
    pub fn get(&self, name: &str) -> Option<egui::TextureId>;
}

// In GameCallbacks::init():
let terrain_view = load_terrain_atlas(device);
ctx.egui_textures().register("terrain", device, &terrain_view);

// In GameCallbacks::ui():
let tex_id = ctx.egui_textures().get("terrain").unwrap();
ui.image((tex_id, egui::vec2(32.0, 32.0)));
```

### Decision: egui input takes priority over game input
**Why:** When the user is interacting with an egui menu (hovering a button, dragging a slider), mouse clicks should NOT also trigger in-game actions (breaking blocks, shooting).

**Design:** `egui::Context::wants_pointer_input()` and `wants_keyboard_input()` are checked. When egui wants input, the engine skips forwarding events to `InputState`. This is handled automatically in `AnvilKitApp`'s event handler.

### Decision: Keep `anvilkit-ui` hud/ and render hud/ modules
**Why:** HUD elements (health bar segments, hotbar with block icons, crosshair) benefit from pixel-perfect positioning via the custom UiRenderer. egui is great for menus/dialogs but adds overhead for simple geometry like a crosshair or segmented health bar.

**Hybrid approach:**
- **Menus, dialogs, settings, inventory** → egui (text, interaction, theming)
- **In-game HUD** → custom UiRenderer + TextRenderer (pixel-perfect, no input needed)

## Risks / Trade-offs

- **Risk:** egui version dependency lock. egui updates frequently and may break wgpu compatibility.
  **Mitigation:** Pin to a specific egui version. The egui-wgpu crate handles wgpu compat.

- **Risk:** egui's default visual style looks like a debug tool, not a game.
  **Mitigation:** Full theme customization via `Visuals`. Load pixel fonts. Use `ImageButton` for styled buttons.

- **Trade-off:** Adding ~15 new crate dependencies.
  **Justification:** This replaces thousands of lines of broken custom UI code with a battle-tested solution.

## Migration Plan
1. Add egui dependencies to workspace Cargo.toml
2. Implement EguiIntegration in anvilkit-app
3. Extend GameCallbacks with ui() method
4. Rewrite Craft menus using egui
5. Remove broken menu/ modules from anvilkit-ui and anvilkit-render
6. Keep hud/ modules intact

## Open Questions
- Which egui version to target? 0.31 is latest stable with wgpu 0.19 support — need to verify compat.
- Should we ship a default "game" theme preset in the engine?
