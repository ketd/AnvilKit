## ADDED Requirements

### Requirement: Network Transport Layer
The system SHALL provide a `NetworkPlugin` with pluggable transport (UDP via `laminar` or WebSocket via `tungstenite`).

The system SHALL support client-server architecture with authoritative server.

#### Scenario: Client connects to server
- **WHEN** `NetworkClient::connect(address)` is called
- **THEN** a reliable connection is established and `ConnectionState` transitions to `Connected`

#### Scenario: Connection failure
- **WHEN** the server is unreachable
- **THEN** `ConnectionState` transitions to `Failed` with a timeout error after 5 seconds

### Requirement: State Synchronization
The system SHALL provide automatic ECS component replication from server to clients for entities marked with `Replicated`.

The system SHALL use delta compression — only changed component fields are transmitted.

#### Scenario: Entity replication
- **WHEN** the server spawns an entity with `Replicated` + `Transform` + `Health`
- **THEN** all connected clients automatically spawn a corresponding entity with the same component data

#### Scenario: Component update
- **WHEN** a replicated entity's `Transform` changes on the server
- **THEN** the delta is sent to clients and applied within one network tick

### Requirement: Client-Side Prediction
The system SHALL provide input prediction for the local player to hide latency.

The system SHALL support rollback and replay when the server's authoritative state diverges from the client's prediction.

#### Scenario: Smooth local movement
- **WHEN** the player presses a movement key
- **THEN** the local entity moves immediately (predicted) without waiting for server confirmation

#### Scenario: Prediction correction
- **WHEN** the server's authoritative position differs from the client's predicted position by more than a threshold
- **THEN** the client smoothly corrects toward the authoritative state over several frames
