use bevy_ecs::prelude::*;
use anvilkit_ecs::physics::DeltaTime;
use crate::resources::DayNightCycle;

/// Advances the day/night cycle each frame.
pub fn day_night_system(dt: Res<DeltaTime>, mut cycle: ResMut<DayNightCycle>) {
    cycle.advance(dt.0);
}
