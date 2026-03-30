## MODIFIED Requirements

### Requirement: Game Screens
The Craft game SHALL use a `CraftScreen` state machine (MainMenu, Playing, Paused, Inventory, Settings) managed by `ScreenPlugin`.

All game screens (main menu, pause menu, settings, inventory) SHALL be rendered using egui via the `GameCallbacks::ui()` method.

The in-game HUD (health bar, hotbar, crosshair, debug text) SHALL continue using custom renderers (`UiRenderer`, `TextRenderer`, `OverlayLineRenderer`) for pixel-perfect positioning.

#### Scenario: Main menu
- **WHEN** the game starts
- **THEN** an egui-rendered main menu with "Play", "Settings", "Quit" is displayed over a dark background

#### Scenario: Pause menu
- **WHEN** the player presses ESC during gameplay
- **THEN** the game state transitions to Paused, the 3D scene freezes, and an egui pause overlay with "Resume", "Settings", "Save & Quit" appears

#### Scenario: Settings screen
- **WHEN** the player enters Settings from the main menu or pause menu
- **THEN** egui sliders for Volume, Sensitivity, FOV, and View Distance are displayed, and changes apply in real-time

#### Scenario: Inventory screen
- **WHEN** the player presses E during gameplay
- **THEN** an egui grid showing 9 inventory slots with block icons and quantities is displayed, and the cursor is freed for interaction

### Requirement: Craft Visual Theme
The Craft game SHALL apply a custom dark game theme to egui via `egui::Visuals`, replacing the default developer-tool appearance.

The theme SHALL use dark panel backgrounds, subtle borders, and a pixel-art or blocky font style consistent with the voxel aesthetic.

#### Scenario: Themed buttons
- **WHEN** a menu button is rendered
- **THEN** it uses the game theme colors (dark background, light text, highlight on hover) rather than egui defaults
