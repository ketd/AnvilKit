//! Health system with damage/heal/death events.

use bevy_ecs::prelude::*;

/// Health component.
#[derive(Debug, Clone, Component)]
pub struct Health {
    pub current: f32,
    pub max: f32,
    pub regen_rate: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self { current: max, max, regen_rate: 0.0 }
    }

    pub fn with_regen(mut self, rate: f32) -> Self { self.regen_rate = rate; self }

    pub fn is_alive(&self) -> bool { self.current > 0.0 }
    pub fn is_dead(&self) -> bool { self.current <= 0.0 }
    pub fn fraction(&self) -> f32 {
        if self.max > 0.0 { (self.current / self.max).clamp(0.0, 1.0) } else { 0.0 }
    }

    pub fn damage(&mut self, amount: f32) {
        self.current = (self.current - amount).max(0.0);
    }

    pub fn heal(&mut self, amount: f32) {
        self.current = (self.current + amount).min(self.max);
    }

    pub fn regen_tick(&mut self, dt: f32) {
        if self.is_alive() && self.regen_rate > 0.0 {
            self.heal(self.regen_rate * dt);
        }
    }
}

/// Damage event.
#[derive(Debug, Clone, Event)]
pub struct DamageEvent {
    pub target: Entity,
    pub amount: f32,
    pub source: Option<Entity>,
}

/// Heal event.
#[derive(Debug, Clone, Event)]
pub struct HealEvent {
    pub target: Entity,
    pub amount: f32,
}

/// Death event — emitted when health reaches zero.
#[derive(Debug, Clone, Event)]
pub struct DeathEvent {
    pub entity: Entity,
}

/// System that processes damage and heal events.
pub fn health_event_system(
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_new() {
        let h = Health::new(100.0);
        assert_eq!(h.current, 100.0);
        assert_eq!(h.max, 100.0);
        assert!(h.is_alive());
    }

    #[test]
    fn test_damage() {
        let mut h = Health::new(100.0);
        h.damage(30.0);
        assert_eq!(h.current, 70.0);
        assert!(h.is_alive());
    }

    #[test]
    fn test_damage_clamp() {
        let mut h = Health::new(50.0);
        h.damage(999.0);
        assert_eq!(h.current, 0.0);
        assert!(h.is_dead());
    }

    #[test]
    fn test_heal() {
        let mut h = Health::new(100.0);
        h.damage(60.0);
        h.heal(30.0);
        assert_eq!(h.current, 70.0);
    }

    #[test]
    fn test_heal_clamp() {
        let mut h = Health::new(100.0);
        h.heal(50.0);
        assert_eq!(h.current, 100.0);
    }

    #[test]
    fn test_regen() {
        let mut h = Health::new(100.0).with_regen(10.0);
        h.damage(50.0);
        h.regen_tick(1.0);
        assert_eq!(h.current, 60.0);
    }

    #[test]
    fn test_fraction() {
        let mut h = Health::new(200.0);
        h.damage(100.0);
        assert!((h.fraction() - 0.5).abs() < 0.01);
    }
}
