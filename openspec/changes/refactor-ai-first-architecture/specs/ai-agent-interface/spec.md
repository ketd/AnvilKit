## ADDED Requirements

### Requirement: MCP Server Integration
The engine SHALL provide an `anvilkit-mcp` crate that exposes a Model Context Protocol (MCP) server. The server SHALL be feature-gated (`mcp` feature) and activated via `McpPlugin`. When active, AI agents SHALL be able to interact with the running game through structured JSON-RPC tool calls over stdio.

#### Scenario: Agent lists available resources
- **WHEN** an agent calls the `list_resources` MCP tool
- **THEN** it receives a JSON array of all ECS resources with their `Describe` schemas

#### Scenario: Agent spawns an entity
- **WHEN** an agent calls `spawn_entity` with `{"components": {"Transform": {"position": [0,1,0]}, "Health": {"max": 100}}}`
- **THEN** a new entity is created with the specified components
- **AND** the tool returns the entity ID

#### Scenario: Agent queries world state
- **WHEN** an agent calls `query_world` with `{"components": ["Transform", "Health"], "filter": {"Health.current": {"lt": 50}}}`
- **THEN** it receives a JSON array of matching entities with their component data

#### Scenario: Agent inspects a specific entity
- **WHEN** an agent calls `inspect_component` with `{"entity": 42, "component": "Health"}`
- **THEN** it receives the full serialized component data as JSON

### Requirement: Component Type Registry for Agents
The engine SHALL maintain a runtime registry of all component and resource types that implement `Describe`. This registry SHALL be queryable via MCP tools and programmatic API. Types are registered automatically when plugins are added.

#### Scenario: Plugin registers describable types
- **WHEN** a plugin adds a `#[derive(Component, Describe)]` type to the world
- **THEN** the type appears in `list_components` MCP tool output

#### Scenario: Game-specific types are discoverable
- **WHEN** a game registers custom components with `Describe` implementations
- **THEN** agents can discover and inspect them alongside engine types

### Requirement: Describe Trait
The `Describe` trait SHALL be the foundation for all AI-agent introspection. It SHALL provide:
- Type name and description
- Per-field metadata (name, type, default value, valid range, description)
- A usage example as a code string
- JSON-serializable output via `serde`

#### Scenario: Minimal Describe implementation
- **WHEN** a struct derives `Describe` with no attributes
- **THEN** `schema()` returns field names and Rust type names with empty descriptions

#### Scenario: Fully annotated Describe implementation
- **WHEN** a struct derives `Describe` with `#[describe(...)]` on each field
- **THEN** `schema()` includes all annotations (range, hint, default, description)
