## ADDED Requirements

### Requirement: Open-Source Game Rewrite
AnvilKit SHALL include at least one rewrite of a well-known open-source game as a showcase project under `games/`.

#### Scenario: Game compiles and runs
- **WHEN** user runs `cargo run -p <game-name>`
- **THEN** a playable game window opens with functional gameplay

#### Scenario: Game is self-contained crate
- **WHEN** the game project is examined
- **THEN** it EXISTS as an independent workspace crate under `games/<name>/`
- **AND** it depends only on AnvilKit crates and standard dependencies

### Requirement: Modular Code Structure
The game rewrite SHALL follow the same modular pattern as `games/billiards/`: separate modules for components, resources, physics, systems, and render.

#### Scenario: Code organization
- **WHEN** the game source is reviewed
- **THEN** game logic, rendering, input, and physics are in separate modules
- **AND** no single file exceeds 500 lines

### Requirement: Reference Source Available
The original open-source game source SHALL be cloned to `.dev/` for reference during development.

#### Scenario: Reference repo present
- **WHEN** developer needs to consult the original implementation
- **THEN** the source is available at `.dev/<original-repo-name>/`
