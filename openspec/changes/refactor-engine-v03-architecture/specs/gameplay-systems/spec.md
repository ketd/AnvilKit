## ADDED Requirements

### Requirement: Stats and Health System
The system SHALL provide `Stat<T>` as a generic numeric stat component with base value, modifier stack, and computed current value.

The system SHALL provide `Health` component with `current`, `max`, and `regeneration_rate` fields, built on `Stat<f32>`.

The system SHALL provide `DamageEvent` and `HealEvent` as ECS events, processed by a `health_system` that applies damage/healing and emits `DeathEvent` when health reaches zero.

The modifier stack SHALL support additive, multiplicative, and override modifiers with priority ordering.

#### Scenario: Damage application
- **WHEN** a `DamageEvent { target, amount: 25.0 }` is sent
- **THEN** the target's `Health.current` decreases by 25.0 (after armor/modifiers)

#### Scenario: Death detection
- **WHEN** damage reduces `Health.current` to zero or below
- **THEN** a `DeathEvent { entity }` is emitted

#### Scenario: Stat modifier stacking
- **WHEN** a +10 additive modifier and a 1.5x multiplicative modifier are applied to a stat with base 100
- **THEN** the computed value is (100 + 10) * 1.5 = 165

### Requirement: Inventory System
The system SHALL provide an `Inventory` trait with `add_item`, `remove_item`, `contains`, `slots`, and `capacity` methods.

The system SHALL provide `SlotInventory` (fixed-size grid, items occupy slots) and `StackInventory` (stackable items with max stack size) as default implementations.

The system SHALL provide `ItemStack` (item ID + count) and `ItemDef` (item definition: name, max_stack, weight, tags).

#### Scenario: Add item to slot inventory
- **WHEN** `inventory.add_item(ItemStack::new(sword_id, 1))` is called on a non-full inventory
- **THEN** the item is placed in the first available slot and `true` is returned

#### Scenario: Full inventory rejection
- **WHEN** `inventory.add_item(item)` is called on a full inventory
- **THEN** `false` is returned and the inventory is unchanged

#### Scenario: Stackable items
- **WHEN** 32 wood items are added to a StackInventory with max_stack_size 64, then 40 more are added
- **THEN** the first slot has 64 wood and the second slot has 8 wood

### Requirement: Cooldown System
The system SHALL provide `Cooldown` as an ECS component with `duration`, `remaining`, and `ready` fields.

The system SHALL provide `CooldownPlugin` with a `cooldown_tick_system` that decrements `remaining` by `DeltaTime` each frame and sets `ready = true` when `remaining <= 0`.

The system SHALL provide `Cooldown::trigger()` to reset `remaining` to `duration`.

#### Scenario: Cooldown countdown
- **WHEN** a `Cooldown { duration: 5.0, remaining: 5.0 }` exists and 2 seconds pass
- **THEN** `remaining` is 3.0 and `ready` is false

#### Scenario: Cooldown ready
- **WHEN** `remaining` reaches 0.0
- **THEN** `ready` is true and the ability can be used

### Requirement: Status Effect System
The system SHALL provide `StatusEffect` component with `effect_type`, `duration`, `remaining`, `stack_count`, and `max_stacks` fields.

The system SHALL provide `StatusEffectPlugin` with a tick system that decrements durations and removes expired effects.

The system SHALL support stack policies: `Replace` (new effect replaces old), `Extend` (duration is refreshed), `Stack` (stack_count increments up to max_stacks).

#### Scenario: Poison tick
- **WHEN** a poison status effect with duration 10s and damage_per_second 5 is applied
- **THEN** the entity takes 5 damage per second for 10 seconds, then the effect is removed

#### Scenario: Buff stacking
- **WHEN** a strength buff with max_stacks 3 is applied twice
- **THEN** stack_count is 2 and the stat modifier is applied twice

### Requirement: Entity Pool
The system SHALL provide `EntityPool<T>` for recycling frequently spawned/despawned entities (bullets, particles, pickups).

The pool SHALL pre-allocate entities with a configurable initial capacity and grow dynamically.

`EntityPool::acquire()` SHALL return a recycled entity (re-enabling its components) or spawn a new one. `EntityPool::release()` SHALL disable the entity without despawning.

#### Scenario: Bullet recycling
- **WHEN** 100 bullets are fired and 50 hit targets
- **THEN** the 50 hit bullets are released to the pool, and subsequent `acquire()` calls reuse them instead of spawning new entities
