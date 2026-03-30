# Change: UI Framework — egui Integration, Replace Custom Menu System

## Why

The custom UI rendering pipeline (UiRenderer + TextRenderer + Menu builder) has fundamental architectural flaws:
1. **Bitmap-only text** — 8x16 CP437 font, no TrueType, no CJK, broken centering (text truncation visible in-game)
2. **No real widget system** — Manual rect computation, manual hit testing, manual hover state tracking. Every "widget" is hand-coded geometry
3. **Linear/sRGB color confusion** — Colors appear washed out because linear values are written to sRGB swapchain without gamma correction
4. **No texture/image support in UI** — Cannot render item icons, custom button backgrounds, or UI art assets
5. **No text layout** — No word wrap, no text alignment, no multi-line support

The effort to fix these issues equals building a full GUI toolkit from scratch. Instead, integrate **egui** — a mature (15K+ star, 5+ years) immediate-mode GUI that already solves all of these problems, with official wgpu + winit bindings matching AnvilKit's exact stack.

## What Changes

### Replace: Custom Menu System → egui
- **Remove** `anvilkit-ui/menu/` module (5 files) — broken immediate-mode menu builder
- **Remove** `anvilkit-render/menu/` module (3 files) — broken menu renderer
- **Remove** `games/craft/src/ui/` menu files that depend on the broken system
- **Keep** `anvilkit-ui` layout engine, widgets, controls, events, focus — these are GPU-independent data models still useful for retained-mode ECS UI
- **Keep** `anvilkit-render` UiRenderer — still useful for pixel-perfect HUD elements (health bars, hotbar)
- **Keep** `anvilkit-ui/hud/` and `anvilkit-render/hud/` — HUD components are fine

### Add: egui Integration Layer
- **`anvilkit-app`** — Add `EguiIntegration` struct wrapping `egui::Context` + `egui_wgpu::Renderer` + `egui_winit::State`
- **`anvilkit-app`** — Extend `GameCallbacks` with `fn ui(&mut self, ctx: &mut GameContext, egui_ctx: &egui::Context)`
- **`anvilkit-app`** — Forward winit events to egui, manage frame begin/end, render egui output to swapchain
- **`anvilkit-app`** — Expose `register_texture(TextureView)` for games to use custom images in egui
- **`anvilkit`** facade — Re-export egui types in prelude

### Upgrade: Craft Game UI
- Main menu, pause menu, settings, inventory — all rewritten with egui
- Custom Minecraft-style theme via `egui::Visuals` + pixel font
- Block icons from terrain texture atlas registered as egui textures
- HUD (health bar, hotbar, crosshair) stays on custom renderers (pixel-perfect positioning)

## Impact
- Affected specs: `ui-framework` (major), `engine-dx` (GameCallbacks extension), `game-craft` (UI rewrite)
- Affected code: `anvilkit-app` (egui integration), `games/craft` (UI rewrite), `anvilkit-ui` + `anvilkit-render` (remove broken menu modules)
- **New dependencies**: `egui 0.31`, `egui-wgpu 0.31`, `egui-winit 0.31`
- **NOT breaking**: The GameCallbacks `ui()` method has a default no-op. Existing games are unaffected.
