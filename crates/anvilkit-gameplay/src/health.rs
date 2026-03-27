//! # Health System
//!
//! Health component with damage, healing, regeneration, and death detection.
//!
//! ## Events
//!
//! - [`DamageEvent`] — request damage on a target entity
//! - [`HealEvent`] — request healing on a target entity
//! - [`DeathEvent`] — emitted when an entity's health reaches zero
//!
//! ## Systems
//!
//! - [`health_system`] — reads `DamageEvent` / `HealEvent`, applies them to
//!   [`Health`] components, and emits [`DeathEvent`] when health drops to zero.
//! - [`health_regen_system`] — applies `regen_rate * dt` each tick.
//!
//! ## Example
//!
//! ```rust
//! use anvilkit_gameplay::health::Health;
//!
//! let mut hp = Health::new(100.0);
//! hp.damage(40.0);
//! assert_eq!(hp.current, 60.0);
//! hp.heal(20.0);
//! assert_eq!(hp.current, 80.0);
//! assert!(hp.is_alive());
//! ```

use bevy_ecs::prelude::*;

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

/// Health component tracking current / max hit-points and passive regen.
#[derive(Debug, Clone, Component)]
pub struct Health {
    /// Current hit-points (clamped to `0.0..=max`).
    pub current: f32,
    /// Maximum hit-points.
    pub max: f32,
    /// Hit-points regenerated per second (applied by [`health_regen_system`]).
    pub regen_rate: f32,
}

impl Health {
    /// Create a new `Health` at full HP with zero regen.
    pub fn new(max: f32) -> Self {
        Self {
            current: max,
            max,
            regen_rate: 0.0,
        }
    }

    /// Builder helper to set a regeneration rate.
    pub fn with_regen(mut self, rate: f32) -> Self {
        self.regen_rate = rate;
        self
    }

    /// `true` while current HP is above zero.
    pub fn is_alive(&self) -> bool {
        self.current > 0.0
    }

    /// `true` when current HP is zero or below.
    pub fn is_dead(&self) -> bool {
        self.current <= 0.0
    }

    /// Returns current health as a fraction of max, clamped to `0.0..=1.0`.
    pub fn fraction(&self) -> f32 {
        if self.max > 0.0 {
            (self.current / self.max).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// Reduce current HP by `amount`, clamping at zero.
    pub fn damage(&mut self, amount: f32) {
        self.current = (self.current - amount).max(0.0);
    }

    /// Increase current HP by `amount`, clamping at max.
    pub fn heal(&mut self, amount: f32) {
        self.current = (self.current + amount).min(self.max);
    }
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

/// Request to deal damage to a target entity.
#[derive(Debug, Clone, Event)]
pub struct DamageEvent {
    /// Entity that should receive the damage.
    pub target: Entity,
    /// Raw damage amount (before any reduction).
    pub amount: f32,
    /// Optional entity responsible for the damage.
    pub source: Option<Entity>,
}

/// Request to heal a target entity.
#[derive(Debug, Clone, Event)]
pub struct HealEvent {
    /// Entity that should receive the healing.
    pub target: Entity,
    /// Amount of HP to restore.
    pub amount: f32,
}

/// Emitted when an entity's health reaches zero.
#[derive(Debug, Clone, Event)]
pub struct DeathEvent {
    /// The entity that just died.
    pub entity: Entity,
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Reads [`DamageEvent`] and [`HealEvent`], applies them to [`Health`]
/// components, and emits [`DeathEvent`] when health drops to zero.
pub fn health_system(
    mut health_query: Query<&mut Health>,
    mut damage_events: EventReader<DamageEvent>,
    mut heal_events: EventReader<HealEvent>,
    mut death_events: EventWriter<DeathEvent>,
) {
    for ev in damage_events.read() {
        if let Ok(mut hp) = health_query.get_mut(ev.target) {
            let was_alive = hp.is_alive();
            hp.damage(ev.amount);
            if was_alive && hp.is_dead() {
                death_events.send(DeathEvent { entity: ev.target });
            }
        }
    }

    for ev in heal_events.read() {
        if let Ok(mut hp) = health_query.get_mut(ev.target) {
            hp.heal(ev.amount);
        }
    }
}

/// Applies passive regeneration (`regen_rate * delta`) to every living entity.
///
/// The `delta` parameter is a plain `f32` representing seconds elapsed since
/// the last tick, making this function easy to test without a full engine time
/// resource.
pub fn health_regen_system(delta: f32, mut query: Query<&mut Health>) {
    for mut hp in query.iter_mut() {
        if hp.is_alive() && hp.regen_rate > 0.0 {
            let amount = hp.regen_rate * delta;
            hp.heal(amount);
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Health component unit tests ----------------------------------------

    #[test]
    fn new_health_starts_full() {
        let h = Health::new(100.0);
        assert_eq!(h.current, 100.0);
        assert_eq!(h.max, 100.0);
        assert_eq!(h.regen_rate, 0.0);
        assert!(h.is_alive());
        assert!(!h.is_dead());
    }

    #[test]
    fn damage_reduces_current() {
        let mut h = Health::new(100.0);
        h.damage(30.0);
        assert_eq!(h.current, 70.0);
        assert!(h.is_alive());
    }

    #[test]
    fn damage_clamps_to_zero() {
        let mut h = Health::new(50.0);
        h.damage(999.0);
        assert_eq!(h.current, 0.0);
        assert!(h.is_dead());
    }

    #[test]
    fn heal_restores_current() {
        let mut h = Health::new(100.0);
        h.damage(60.0);
        h.heal(30.0);
        assert_eq!(h.current, 70.0);
    }

    #[test]
    fn heal_clamps_to_max() {
        let mut h = Health::new(100.0);
        h.heal(50.0);
        assert_eq!(h.current, 100.0);
    }

    #[test]
    fn fraction_returns_ratio() {
        let mut h = Health::new(200.0);
        h.damage(100.0);
        assert!((h.fraction() - 0.5).abs() < f32::EPSILON);

        h.damage(200.0);
        assert_eq!(h.fraction(), 0.0);
    }

    #[test]
    fn fraction_zero_max_returns_zero() {
        let h = Health::new(0.0);
        assert_eq!(h.fraction(), 0.0);
    }

    #[test]
    fn regen_heals_over_time() {
        let mut h = Health::new(100.0).with_regen(10.0);
        h.damage(50.0);
        assert_eq!(h.current, 50.0);

        // Simulate 1 second of regen
        if h.is_alive() && h.regen_rate > 0.0 {
            h.heal(h.regen_rate * 1.0);
        }
        assert_eq!(h.current, 60.0);
    }

    #[test]
    fn regen_does_not_exceed_max() {
        let mut h = Health::new(100.0).with_regen(200.0);
        h.damage(10.0);
        if h.is_alive() && h.regen_rate > 0.0 {
            h.heal(h.regen_rate * 1.0);
        }
        assert_eq!(h.current, 100.0);
    }

    // -- ECS system integration tests ---------------------------------------

    #[test]
    fn health_system_applies_damage_and_emits_death() {
        let mut world = World::new();
        world.init_resource::<Events<DamageEvent>>();
        world.init_resource::<Events<HealEvent>>();
        world.init_resource::<Events<DeathEvent>>();

        let entity = world.spawn(Health::new(50.0)).id();

        // Send a lethal damage event
        world.resource_mut::<Events<DamageEvent>>().send(DamageEvent {
            target: entity,
            amount: 50.0,
            source: None,
        });

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(health_system);
        schedule.run(&mut world);

        // Health should be zero
        let hp = world.get::<Health>(entity).unwrap();
        assert_eq!(hp.current, 0.0);

        // DeathEvent should have been emitted
        let death_events = world.resource::<Events<DeathEvent>>();
        let mut reader = death_events.get_reader();
        let deaths: Vec<_> = reader.read(death_events).collect();
        assert_eq!(deaths.len(), 1);
        assert_eq!(deaths[0].entity, entity);
    }

    #[test]
    fn health_system_applies_healing() {
        let mut world = World::new();
        world.init_resource::<Events<DamageEvent>>();
        world.init_resource::<Events<HealEvent>>();
        world.init_resource::<Events<DeathEvent>>();

        let entity = world.spawn(Health::new(100.0)).id();

        // Pre-damage the entity directly
        world.get_mut::<Health>(entity).unwrap().damage(60.0);

        // Send heal event
        world.resource_mut::<Events<HealEvent>>().send(HealEvent {
            target: entity,
            amount: 25.0,
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(health_system);
        schedule.run(&mut world);

        let hp = world.get::<Health>(entity).unwrap();
        assert_eq!(hp.current, 65.0);
    }
}
