## ADDED Requirements

### Requirement: World Creation Screen
The game SHALL provide a World Creation screen accessible from the main menu with fields for: world name (text input), world seed (text input, random if empty), and game mode selection (Survival/Creative).

#### Scenario: Random seed generation
- **WHEN** the player leaves the seed field empty and clicks Create
- **THEN** a random seed is generated and used for world generation

#### Scenario: Custom seed
- **WHEN** the player enters "my_seed_123" in the seed field
- **THEN** the world is generated deterministically from that seed string

### Requirement: World Selection Screen
The game SHALL provide a World Selection screen listing all saved worlds, displaying world name, last-played timestamp, and world size. The screen SHALL support loading and deleting worlds.

#### Scenario: World list display
- **WHEN** the player navigates to the World Selection screen with 3 saved worlds
- **THEN** all 3 worlds are listed sorted by last-played time (newest first)

#### Scenario: World deletion
- **WHEN** the player clicks Delete on a world and confirms
- **THEN** the save slot and all associated data are permanently removed

### Requirement: Crafting UI
The game SHALL provide a 3x3 crafting grid interface when interacting with a Workbench, and a 2x2 grid in the player's inventory screen. The output slot SHALL display the matching recipe result in real-time.

#### Scenario: Recipe preview
- **WHEN** the player arranges items in the crafting grid matching a known recipe
- **THEN** the output slot immediately shows the crafting result with correct quantity

#### Scenario: Craft execution
- **WHEN** the player clicks the output slot
- **THEN** the input items are consumed and the crafted item is added to the cursor/inventory

### Requirement: Furnace UI
The game SHALL provide a furnace interface with input slot, fuel slot, output slot, and two progress indicators (flame for fuel remaining, arrow for smelting progress).

#### Scenario: Smelting progress
- **WHEN** the furnace has valid input and burning fuel
- **THEN** the arrow progress indicator advances, and upon completion the output item appears

### Requirement: Item Drag and Drop
Inventory and crafting UIs SHALL support mouse-based item manipulation: left-click picks up/places entire stacks, right-click picks up/places single items.

#### Scenario: Stack splitting
- **WHEN** the player right-clicks a stack of 64 Cobblestone
- **THEN** 32 items are picked up as cursor stack, 32 remain in the slot

### Requirement: Item Texture Icons
The hotbar and inventory UI SHALL display block/item texture thumbnails instead of text-only labels, using the texture atlas registered with egui.

#### Scenario: Hotbar visual
- **WHEN** the player has a Grass block in hotbar slot 1
- **THEN** the hotbar slot displays the Grass block's top-face texture as a small icon with stack count
