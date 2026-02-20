use bevy_ecs::prelude::*;

/// First-person camera controller marker.
#[derive(Debug, Clone, Component)]
pub struct FpsCamera;

/// Marks an entity as a renderable chunk.
#[derive(Debug, Clone, Component)]
pub struct ChunkEntity {
    pub cx: i32,
    pub cz: i32,
}
