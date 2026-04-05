## MODIFIED Requirements

### Requirement: Health Regeneration System
The `health_regen_system` SHALL accept `Res<DeltaTime>` as a Bevy ECS system parameter instead of a plain `f32`, enabling direct registration as a Bevy system without game-side wrappers.

#### Scenario: Direct system registration
- **WHEN** a game adds `health_regen_system` to its ECS schedule
- **THEN** it runs correctly each frame, reading delta time from the `DeltaTime` resource and applying `regen_rate * dt` healing to all `Health` components

## ADDED Requirements

### Requirement: Status Effect Tick System
The engine SHALL provide a `status_effect_tick_system` that automatically ticks all `StatusEffectList` components each frame, decrementing remaining time and removing expired effects.

#### Scenario: Effect expiration
- **WHEN** a `StatusEffect` with 5-second duration has been active for 5 seconds
- **THEN** the `status_effect_tick_system` removes it from the entity's `StatusEffectList`

### Requirement: Item Definition Extension
The `ItemDef` struct SHALL be extended with gameplay-relevant fields: `tool_type`, `tool_tier`, `damage`, `armor_value`, `food_value`, `max_stack_size`, and `durability`.

`ItemDef` SHALL derive `Component` and `Resource` as needed for ECS integration.

#### Scenario: ItemDef with tool properties
- **WHEN** an `ItemDef` is created for a Diamond Pickaxe
- **THEN** it has `tool_type: Pickaxe`, `tool_tier: Diamond`, `durability: 1561`, and `damage: 5.0`
