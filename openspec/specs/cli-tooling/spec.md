# cli-tooling Specification

## Purpose
TBD - created by archiving change refactor-engine-quality. Update Purpose after archive.
## Requirements
### Requirement: CLI Flag Integrity
Every CLI flag accepted by the tool SHALL have a functional implementation. Flags that are accepted but silently ignored are not permitted.

If a feature is not yet implemented, the flag SHALL either be removed from the CLI definition or produce a clear "not yet implemented" error message when used.

#### Scenario: --watch flag
- **WHEN** the user runs `anvil run --watch`
- **THEN** either file watching is active and the project re-runs on changes, or the CLI prints "Error: --watch is not yet implemented" and exits with a non-zero status code

#### Scenario: Undocumented flags
- **WHEN** the CLI `--help` output is reviewed
- **THEN** every listed flag has a corresponding functional implementation

### Requirement: Code Generator Input Validation
The `anvil generate` commands (component, resource, system) SHALL validate that the provided name is a valid Rust identifier before generating code.

A valid Rust identifier SHALL match `[a-zA-Z_][a-zA-Z0-9_]*` and SHALL NOT be a Rust keyword.

Generated code SHALL include all necessary `use` import statements to compile without additional manual edits.

#### Scenario: Valid identifier
- **WHEN** `anvil generate component PlayerHealth` is run
- **THEN** a Rust file is generated with `use bevy_ecs::prelude::*;` and `#[derive(Component)] pub struct PlayerHealth;`

#### Scenario: Invalid identifier
- **WHEN** `anvil generate component "123invalid"` is run
- **THEN** the CLI prints an error: "Error: '123invalid' is not a valid Rust identifier" and exits without generating files

#### Scenario: Rust keyword
- **WHEN** `anvil generate component "struct"` is run
- **THEN** the CLI prints an error: "Error: 'struct' is a reserved Rust keyword" and exits without generating files

### Requirement: Robust Workspace Detection
The CLI SHALL detect Cargo workspaces by parsing the `Cargo.toml` file with a proper TOML parser, not by string matching (e.g., `contains("[workspace]")`).

Workspace member counting SHALL parse the `[workspace].members` array from the TOML AST, not count lines matching heuristic patterns.

#### Scenario: Standard workspace
- **WHEN** `anvil doctor` is run in a directory with a valid `Cargo.toml` containing `[workspace]`
- **THEN** the workspace is correctly detected and the member count matches the actual `members` array length

#### Scenario: Commented workspace section
- **WHEN** a `Cargo.toml` contains `# [workspace]` in a comment but no actual workspace section
- **THEN** the CLI correctly reports "not a workspace" instead of false-positive detection

#### Scenario: Complex TOML formatting
- **WHEN** the workspace `members` array uses multi-line format with comments and trailing commas
- **THEN** the member count is accurate

### Requirement: No Unused Dependencies
The CLI crate SHALL NOT declare dependencies in `Cargo.toml` that are not imported or used in any source file.

#### Scenario: Dependency audit
- **WHEN** `cargo udeps` (or equivalent) is run on the CLI crate
- **THEN** no unused dependencies are reported

