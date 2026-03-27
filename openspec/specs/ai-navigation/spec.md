# ai-navigation Specification

## Purpose
TBD - created by archiving change add-engine-v02-features. Update Purpose after archive.
## Requirements
### Requirement: Navigation Mesh Generation
The system SHALL provide NavMesh generation from 3D geometry (triangle soup → walkable surface → convex polygon decomposition).

`NavMeshBuilder::from_colliders(world)` SHALL produce a `NavMesh` resource representing the walkable area.

#### Scenario: NavMesh from static geometry
- **WHEN** `NavMeshBuilder` is given a set of static colliders
- **THEN** a NavMesh is generated representing the walkable floor surface with obstacles carved out

#### Scenario: NavMesh regeneration
- **WHEN** the static geometry changes (e.g., a door opens)
- **THEN** `NavMesh::rebuild()` updates the affected region without regenerating the entire mesh

### Requirement: Pathfinding
The system SHALL provide A* pathfinding on the NavMesh, returning a sequence of waypoints from start to goal.

#### Scenario: Path found
- **WHEN** `nav_mesh.find_path(start, goal)` is called with reachable positions
- **THEN** a `Vec<Vec3>` of waypoints is returned forming a valid path along the NavMesh surface

#### Scenario: Unreachable goal
- **WHEN** the goal position is not on or near any NavMesh polygon
- **THEN** `None` is returned

### Requirement: Agent Steering
The system SHALL provide a `NavAgent` component with `speed`, `radius`, and `target` fields.

A `nav_agent_system` SHALL move agents along their computed paths, avoiding other agents (local avoidance).

#### Scenario: Agent follows path
- **WHEN** a `NavAgent` entity has a target set
- **THEN** it moves along the NavMesh path at the specified speed each frame

#### Scenario: Agent avoidance
- **WHEN** two `NavAgent` entities approach each other head-on
- **THEN** they steer around each other without overlapping

