use bevy_ecs::prelude::*;

/// Marker for the cue ball (ball 0).
#[derive(Component)]
pub struct CueBall;

/// A numbered ball (1-15).
#[derive(Component)]
pub struct NumberedBall {
    pub number: u8,
    pub potted: bool,
}

/// Marker for the table surface entity.
#[derive(Component)]
pub struct TableSurface;
