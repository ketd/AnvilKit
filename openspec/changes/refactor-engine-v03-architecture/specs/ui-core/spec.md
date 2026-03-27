## ADDED Requirements

### Requirement: UI Node Tree
The system SHALL provide a `UiTree` that manages parent-child relationships between UI nodes, supporting arbitrary nesting depth.

Each `UiNode` SHALL be an ECS entity with style, text, visibility, and interaction state.

The tree SHALL support dynamic add/remove of nodes at runtime.

#### Scenario: Nested panel layout
- **WHEN** a panel node has two child button nodes
- **THEN** the layout engine computes positions for both buttons relative to the panel

#### Scenario: Dynamic node insertion
- **WHEN** a new UiNode entity is spawned with a parent reference to an existing panel
- **THEN** the node appears in the layout on the next frame

### Requirement: Recursive Flexbox Layout
The system SHALL provide a layout engine that computes positions and sizes for the full UI tree using CSS Flexbox semantics (via taffy).

Layout SHALL be computed recursively for the entire tree, not just one level of children.

The layout engine SHALL run as an ECS system each frame, updating `computed_rect` on all visible UiNode entities.

#### Scenario: Three-level nesting
- **WHEN** a root panel contains a row, which contains three buttons
- **THEN** the row arranges buttons horizontally, and the panel positions the row according to its own flex direction

### Requirement: UI Event System
The system SHALL provide an ECS system that processes mouse hover, click, and keyboard focus events on UI nodes.

The system SHALL support focus management: Tab to cycle focus, Enter/Space to activate focused element.

The system SHALL provide `UiInteraction` component on each interactive node with states: None, Hovered, Pressed, Focused.

#### Scenario: Button click
- **WHEN** the mouse clicks on a button node
- **THEN** a `UiClickEvent` is emitted with the button's entity

#### Scenario: Keyboard focus navigation
- **WHEN** the user presses Tab
- **THEN** focus advances to the next focusable UI node in tree order

### Requirement: Text Integration
The system SHALL render text content of UiNode entities as part of the UI rendering pass.

Text SHALL support configurable font size, color, and alignment within the node's computed rect.

The layout engine SHALL measure text content to inform flexbox sizing (text nodes have intrinsic size).

#### Scenario: Auto-sized label
- **WHEN** a label node has text "Hello" with font_size 16 and width Val::Auto
- **THEN** the layout engine sizes the node to fit the text content

### Requirement: Widget Library
The system SHALL provide factory functions for common UI patterns: Button, Label, Panel, Row, Column, Checkbox, Slider, TextInput, ScrollView, Dropdown.

Each widget SHALL be a composition of UiNode entities with appropriate default styles and interaction behavior.

#### Scenario: Checkbox toggle
- **WHEN** a Checkbox widget is clicked
- **THEN** its checked state toggles and a `UiChangeEvent` is emitted

#### Scenario: Slider drag
- **WHEN** the user drags a Slider widget handle
- **THEN** the slider value updates continuously and `UiChangeEvent` is emitted with the new value

### Requirement: Theme System
The system SHALL provide a `UiTheme` resource defining default colors, fonts, spacing, and border styles for all widget types.

Widgets SHALL read from `UiTheme` when using default styling, allowing global appearance changes by modifying the theme resource.

#### Scenario: Dark theme switch
- **WHEN** `UiTheme` is replaced with a dark variant
- **THEN** all widgets using default styling update their colors on the next frame
