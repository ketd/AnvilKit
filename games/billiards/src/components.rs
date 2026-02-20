use bevy_ecs::prelude::*;
use glam::Vec3;

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

/// A cushion collision plane.
#[derive(Component)]
pub struct Cushion {
    pub normal: Vec3,
    pub distance: f32,
}

/// A pocket on the table.
#[derive(Component)]
pub struct Pocket {
    pub position: Vec3,
    pub radius: f32,
}
