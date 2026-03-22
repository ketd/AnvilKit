## ADDED Requirements

### Requirement: UI Layout Engine
The system SHALL provide a Flexbox-subset layout engine that computes screen positions and sizes for `UiNode` entities.

The layout engine SHALL support: `flex_direction` (row/column), `justify_content`, `align_items`, `gap`, `padding`, `margin`, `width`/`height` (fixed, percent, auto), `flex_grow`, `flex_shrink`.

#### Scenario: Horizontal layout
- **WHEN** a parent `UiNode` has `flex_direction: Row` with three children
- **THEN** children are arranged left-to-right with the specified gap between them

#### Scenario: Percentage sizing
- **WHEN** a child has `width: Val::Percent(50.0)`
- **THEN** its computed width is 50% of the parent's content area

#### Scenario: Nested layouts
- **WHEN** UiNode entities form a tree (root → panel → row → buttons)
- **THEN** each level's layout is computed correctly relative to its parent

### Requirement: UI Event System
The system SHALL provide UI interaction events: `Click`, `Hover`, `FocusIn`, `FocusOut`, `TextInput`.

Events SHALL propagate from leaf to root (bubble phase), with the ability to stop propagation.

The system SHALL perform hit testing using the computed layout rectangles.

#### Scenario: Button click
- **WHEN** the mouse clicks within a `UiNode`'s computed rectangle
- **THEN** a `Click` event is dispatched to that node's event handler

#### Scenario: Event bubbling
- **WHEN** a child node does not handle a `Click` event
- **THEN** the event bubbles up to the parent node

### Requirement: UI Rendering
The system SHALL render UI nodes as a separate pass on top of the 3D scene (after tone mapping).

The system SHALL support: solid color backgrounds, border rendering, text rendering (using existing `TextRenderer`), and image/texture fills.

UI rendering SHALL use an orthographic projection matching the window dimensions.

#### Scenario: UI overlay
- **WHEN** the frame is rendered
- **THEN** UI nodes appear on top of the 3D scene without depth testing

#### Scenario: Text in UI
- **WHEN** a `UiNode` has a `UiText` component
- **THEN** the text is rendered within the node's computed rectangle with specified font size and color

### Requirement: Common UI Widgets
The system SHALL provide built-in widget constructors: `Button`, `Label`, `Panel`, `ScrollView`, `TextInput`, `Slider`, `Checkbox`.

#### Scenario: Button with text
- **WHEN** `Button::new("Start Game")` is added to the UI tree
- **THEN** a clickable node with centered text, background color, and hover state is rendered
