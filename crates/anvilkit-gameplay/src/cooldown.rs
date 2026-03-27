//! Cooldown timers for abilities and actions.
//!
//! Provides a [`Cooldown`] component that tracks time-based cooldowns,
//! and a [`cooldown_tick_system`] that decrements all cooldowns each frame.

use bevy_ecs::prelude::*;

/// A time resource holding the delta-time for the current frame.
///
/// This is a simple local definition so that `anvilkit-gameplay` does not
/// depend on `anvilkit-ecs` at the type level.  Users can insert their own
/// `DeltaTime` resource or use the one provided by the ECS crate.
#[derive(Resource, Debug, Clone, Copy)]
pub struct DeltaTime(pub f32);

impl Default for DeltaTime {
    fn default() -> Self {
        Self(1.0 / 60.0)
    }
}

/// A cooldown timer that can be attached to any entity.
///
/// `remaining` counts down toward zero; when it reaches zero the cooldown is
/// considered *ready*.
#[derive(Component, Debug, Clone)]
pub struct Cooldown {
    /// Seconds remaining until the cooldown is ready.
    pub remaining: f32,
    /// The full duration of the cooldown (used when re-triggering).
    pub duration: f32,
}

impl Cooldown {
    /// Creates a new cooldown that is immediately ready (`remaining = 0`).
    pub fn new(duration: f32) -> Self {
        Self {
            remaining: 0.0,
            duration,
        }
    }

    /// Returns `true` when the cooldown has elapsed and is ready to fire.
    pub fn is_ready(&self) -> bool {
        self.remaining <= 0.0
    }

    /// Triggers the cooldown, resetting `remaining` to `duration`.
    pub fn trigger(&mut self) {
        self.remaining = self.duration;
    }

    /// Advances the cooldown by `dt` seconds.  `remaining` is clamped to ≥ 0.
    pub fn tick(&mut self, dt: f32) {
        self.remaining = (self.remaining - dt).max(0.0);
    }

    /// Returns the fraction of the cooldown that has *not* yet elapsed.
    ///
    /// - `0.0` — ready (cooldown finished)
    /// - `1.0` — just triggered (full duration remaining)
    ///
    /// Returns `0.0` if `duration` is zero or negative to avoid division issues.
    pub fn fraction(&self) -> f32 {
        if self.duration <= 0.0 {
            return 0.0;
        }
        (self.remaining / self.duration).clamp(0.0, 1.0)
    }
}

/// System that ticks every [`Cooldown`] component by the current frame's
/// [`DeltaTime`].
pub fn cooldown_tick_system(dt: Res<DeltaTime>, mut query: Query<&mut Cooldown>) {
    for mut cd in &mut query {
        cd.tick(dt.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_cooldown_is_ready() {
        let cd = Cooldown::new(2.0);
        assert!(cd.is_ready());
        assert_eq!(cd.duration, 2.0);
        assert_eq!(cd.remaining, 0.0);
    }

    #[test]
    fn trigger_sets_remaining() {
        let mut cd = Cooldown::new(3.0);
        cd.trigger();
        assert!(!cd.is_ready());
        assert_eq!(cd.remaining, 3.0);
    }

    #[test]
    fn tick_decrements_and_clamps() {
        let mut cd = Cooldown::new(1.0);
        cd.trigger();
        cd.tick(0.4);
        assert!((cd.remaining - 0.6).abs() < f32::EPSILON);

        // Tick past zero — should clamp to 0.
        cd.tick(1.0);
        assert_eq!(cd.remaining, 0.0);
        assert!(cd.is_ready());
    }

    #[test]
    fn fraction_reports_progress() {
        let mut cd = Cooldown::new(4.0);
        assert_eq!(cd.fraction(), 0.0, "ready → 0.0");

        cd.trigger();
        assert_eq!(cd.fraction(), 1.0, "just triggered → 1.0");

        cd.tick(2.0);
        assert!((cd.fraction() - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn fraction_with_zero_duration() {
        let cd = Cooldown::new(0.0);
        assert_eq!(cd.fraction(), 0.0);
    }

    #[test]
    fn cooldown_tick_system_integration() {
        let mut world = World::new();
        world.insert_resource(DeltaTime(0.5));

        let entity = world.spawn(Cooldown::new(2.0)).id();
        world.entity_mut(entity).get_mut::<Cooldown>().unwrap().trigger();

        // Run the system once.
        let mut schedule = Schedule::default();
        schedule.add_systems(cooldown_tick_system);
        schedule.run(&mut world);

        let cd = world.entity(entity).get::<Cooldown>().unwrap();
        assert!((cd.remaining - 1.5).abs() < f32::EPSILON);
        assert!(!cd.is_ready());
    }
}
