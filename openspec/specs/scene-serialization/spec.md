# scene-serialization Specification

## Purpose
TBD - created by archiving change add-engine-v02-features. Update Purpose after archive.
## Requirements
### Requirement: Scene Serialization
The system SHALL provide serialization and deserialization of ECS scenes using `serde` with RON as the default format.

A "scene" SHALL be a snapshot of selected entities with their components, serializable to/from a file.

The system SHALL support selective serialization — only entities with a `Serializable` marker component are included.

#### Scenario: Save scene to file
- **WHEN** `SceneSerializer::save(world, path)` is called
- **THEN** all entities marked with `Serializable` are written to a `.ron` file with their component data

#### Scenario: Load scene from file
- **WHEN** `SceneSerializer::load(world, path)` is called
- **THEN** entities are spawned in the world with the deserialized components

#### Scenario: Round-trip fidelity
- **WHEN** a scene is saved and then loaded into an empty world
- **THEN** the resulting entities have identical component values to the originals

### Requirement: Transform Hierarchy Runtime
The system SHALL provide a `TransformPropagationSystem` that computes `GlobalTransform` from local `Transform` + `Parent`/`Children` hierarchy each frame.

The system SHALL update transforms in topological order (parents before children) to ensure correct results in a single pass.

#### Scenario: Parent-child transform inheritance
- **WHEN** a child entity has `Parent` pointing to a parent entity
- **THEN** the child's `GlobalTransform` equals `parent.GlobalTransform * child.Transform`

#### Scenario: Multi-level hierarchy
- **WHEN** entities form a chain: A → B → C
- **THEN** C's `GlobalTransform` equals `A.global * B.local * C.local`

#### Scenario: Dynamic reparenting
- **WHEN** a child's `Parent` component is changed at runtime
- **THEN** the next frame's `GlobalTransform` reflects the new parent

