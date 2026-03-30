## MODIFIED Requirements

### Requirement: Common UI Widgets
The system SHALL provide two tiers of UI widgets:

**Tier 1 — egui (menus, dialogs, interactive screens):**
The system SHALL integrate egui as the primary interactive UI framework. Games access `egui::Context` via `GameCallbacks::ui()` and use egui's built-in widgets: `Button`, `Label`, `Slider`, `Checkbox`, `TextEdit`, `ComboBox`, `Window`, `Menu`, `Grid`, `ScrollArea`, `Image`, `ImageButton`.

**Tier 2 — Custom renderers (HUD overlays):**
The system SHALL retain `UiRenderer` (rectangles with rounded corners/borders) and `TextRenderer` (bitmap text) for pixel-perfect HUD elements that do not require text layout or user interaction. HUD components (`HealthBar`, `Hotbar`, `Crosshair`) use this tier.

#### Scenario: egui menu
- **WHEN** a game implements `GameCallbacks::ui()` with `egui::CentralPanel::default().show(ctx, |ui| { ui.button("Play"); })`
- **THEN** a centered panel with a clickable "Play" button is rendered on top of the 3D scene

#### Scenario: egui slider
- **WHEN** a game adds `ui.add(egui::Slider::new(&mut volume, 0.0..=1.0).text("Volume"))`
- **THEN** a draggable slider with label and value display is rendered and the variable is updated on drag

#### Scenario: Custom HUD health bar
- **WHEN** a game uses `HealthBar::build_nodes(fraction)` and renders via `UiRenderer`
- **THEN** a segmented health bar with fill/empty colors is rendered at the specified screen position

#### Scenario: Button with custom image
- **WHEN** a game registers a texture via `EguiTextures::register()` and uses `egui::ImageButton`
- **THEN** the button renders with the custom texture as its background

### Requirement: UI Rendering
The system SHALL render UI in this order each frame:
1. 3D scene (existing render pipeline)
2. Custom HUD overlays (UiRenderer + TextRenderer, drawn in `GameCallbacks::render()`)
3. egui UI (drawn automatically after `GameCallbacks::ui()`)

egui rendering SHALL use `egui_wgpu::Renderer` which renders to the swapchain with alpha blending on top of existing content.

egui SHALL share the wgpu Device, Queue, and swapchain TextureView with the existing render pipeline.

#### Scenario: Render order
- **WHEN** both custom HUD and egui menus are active
- **THEN** egui menus appear on top of the custom HUD, which appears on top of the 3D scene

#### Scenario: egui-only frame
- **WHEN** the game is on a main menu screen with no 3D scene
- **THEN** egui renders directly on the cleared swapchain without errors

## ADDED Requirements

### Requirement: egui Integration Layer
The system SHALL provide `EguiIntegration` in `anvilkit-app` that manages the full egui lifecycle:
- Creates `egui::Context`, `egui_winit::State`, and `egui_wgpu::Renderer`
- Forwards winit `WindowEvent` to egui for input handling
- Manages egui frame begin/end around the `GameCallbacks::ui()` call
- Renders egui paint output to the swapchain TextureView

Games SHALL NOT need to manage egui state directly — `AnvilKitApp` handles everything.

#### Scenario: Zero-config egui
- **WHEN** a game implements `GameCallbacks::ui()` without any egui setup code
- **THEN** the egui context is available and widgets render correctly

#### Scenario: Input sharing
- **WHEN** the mouse hovers over an egui widget
- **THEN** `egui::Context::wants_pointer_input()` returns true, and the engine does NOT forward the click to `InputState`

### Requirement: UI Texture Bridge
The system SHALL provide `EguiTextures` as an ECS `Resource` that maps string names to `egui::TextureId`.

Games SHALL register wgpu `TextureView`s as egui textures via `EguiTextures::register(name, device, view)` during initialization.

Registered textures SHALL be usable in any egui widget that accepts `TextureId` (Image, ImageButton, custom painting).

#### Scenario: Register and use texture
- **WHEN** a game calls `egui_textures.register("heart", device, &heart_view)` in `init()`
- **THEN** `egui_textures.get("heart")` returns `Some(TextureId)` usable in `ui.image()`

### Requirement: GameCallbacks UI Method
The `GameCallbacks` trait SHALL include a `ui()` method with default no-op implementation:
```
fn ui(&mut self, ctx: &mut GameContext, egui_ctx: &egui::Context) {}
```

The `ui()` method SHALL be called each frame after `render()` and before frame presentation.

When `ui()` is not implemented, egui SHALL produce no draw calls (zero overhead).

#### Scenario: Optional UI
- **WHEN** a game does not override `ui()`
- **THEN** no egui rendering occurs and no performance overhead is added

#### Scenario: UI lifecycle
- **WHEN** `ui()` is called
- **THEN** the `egui::Context` is in an active frame (between `begin_frame` and `end_frame`)
