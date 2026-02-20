use std::collections::HashMap;
use bevy_ecs::prelude::*;

use crate::block::BlockType;
use crate::chunk::{ChunkData, CHUNK_SIZE, CHUNK_HEIGHT};

/// Stores all loaded chunk data keyed by (cx, cz).
#[derive(Resource, Default)]
pub struct VoxelWorld {
    pub chunks: HashMap<(i32, i32), ChunkData>,
}

impl VoxelWorld {
    /// Get block at absolute world coordinates. Returns Air for unloaded/OOB.
    pub fn get_block(&self, x: i32, y: i32, z: i32) -> BlockType {
        if y < 0 || y >= CHUNK_HEIGHT as i32 {
            return BlockType::Air;
        }
        let cx = x.div_euclid(CHUNK_SIZE as i32);
        let cz = z.div_euclid(CHUNK_SIZE as i32);
        let lx = x.rem_euclid(CHUNK_SIZE as i32);
        let lz = z.rem_euclid(CHUNK_SIZE as i32);
        match self.chunks.get(&(cx, cz)) {
            Some(chunk) => chunk.get_safe(lx, y, lz),
            None => BlockType::Air,
        }
    }

    /// Set block at absolute world coordinates. Returns false if chunk not loaded.
    pub fn set_block(&mut self, x: i32, y: i32, z: i32, block: BlockType) -> bool {
        if y < 0 || y >= CHUNK_HEIGHT as i32 {
            return false;
        }
        let cx = x.div_euclid(CHUNK_SIZE as i32);
        let cz = z.div_euclid(CHUNK_SIZE as i32);
        let lx = x.rem_euclid(CHUNK_SIZE as i32) as usize;
        let lz = z.rem_euclid(CHUNK_SIZE as i32) as usize;
        match self.chunks.get_mut(&(cx, cz)) {
            Some(chunk) => {
                chunk.set(lx, y as usize, lz, block);
                true
            }
            None => false,
        }
    }
}

/// Player state for FPS camera.
#[derive(Debug, Resource)]
pub struct PlayerState {
    pub yaw: f32,
    pub pitch: f32,
    pub flying: bool,
    pub move_speed: f32,
    pub mouse_sensitivity: f32,
    pub velocity: glam::Vec3,
    pub on_ground: bool,
    pub jump_requested: bool,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.0,
            flying: true,
            move_speed: 10.0,
            mouse_sensitivity: 0.003,
            velocity: glam::Vec3::ZERO,
            on_ground: false,
            jump_requested: false,
        }
    }
}

/// Accumulated mouse delta per frame (set in window_event, consumed in system).
#[derive(Debug, Default, Resource)]
pub struct MouseDelta {
    pub dx: f32,
    pub dy: f32,
}

/// Currently selected block type for placement.
#[derive(Debug, Resource)]
pub struct SelectedBlock {
    pub block_type: BlockType,
}

impl Default for SelectedBlock {
    fn default() -> Self {
        Self {
            block_type: BlockType::Grass,
        }
    }
}

/// Day/night cycle state.
#[derive(Debug, Resource)]
pub struct DayNightCycle {
    /// Normalized time of day: 0.0 = sunrise, 0.25 = noon, 0.5 = sunset, 0.75 = midnight
    pub time: f32,
    /// Full cycle duration in seconds.
    pub cycle_duration: f32,
}

impl Default for DayNightCycle {
    fn default() -> Self {
        Self {
            time: 0.15, // start near morning
            cycle_duration: 600.0, // 10 minutes
        }
    }
}

impl DayNightCycle {
    /// Advance time by dt seconds.
    pub fn advance(&mut self, dt: f32) {
        self.time = (self.time + dt / self.cycle_duration) % 1.0;
    }

    /// Sun direction: rotates around X axis over the day.
    /// At time=0.25 (noon) sun is directly overhead.
    pub fn light_dir(&self) -> glam::Vec3 {
        let angle = (self.time - 0.25) * std::f32::consts::TAU; // noon = 0 angle
        let y = angle.cos();
        let z = angle.sin();
        glam::Vec3::new(0.2, y, z).normalize()
    }

    /// Ambient light level.
    pub fn ambient(&self) -> f32 {
        let sun_height = self.light_dir().y;
        if sun_height > 0.0 {
            0.12 + 0.28 * sun_height.min(1.0) // day: 0.12 to 0.40
        } else {
            0.08 // night
        }
    }

    /// Fog color: blue day, orange sunset, dark blue night.
    pub fn fog_color(&self) -> [f32; 4] {
        let sun_h = self.light_dir().y;
        if sun_h > 0.15 {
            // Day
            [0.53, 0.71, 0.92, 1.0]
        } else if sun_h > -0.05 {
            // Sunrise/sunset transition
            let t = (sun_h + 0.05) / 0.20; // 0 at -0.05, 1 at 0.15
            let day = [0.53, 0.71, 0.92];
            let sunset = [0.85, 0.45, 0.25];
            [
                day[0] * t + sunset[0] * (1.0 - t),
                day[1] * t + sunset[1] * (1.0 - t),
                day[2] * t + sunset[2] * (1.0 - t),
                1.0,
            ]
        } else {
            // Night
            let t = ((-sun_h - 0.05) / 0.3).min(1.0);
            let sunset = [0.85, 0.45, 0.25];
            let night = [0.05, 0.05, 0.12];
            [
                sunset[0] * (1.0 - t) + night[0] * t,
                sunset[1] * (1.0 - t) + night[1] * t,
                sunset[2] * (1.0 - t) + night[2] * t,
                1.0,
            ]
        }
    }

    /// Sky top color (zenith).
    pub fn sky_top(&self) -> [f32; 3] {
        let sun_h = self.light_dir().y;
        if sun_h > 0.15 {
            [0.25, 0.47, 0.85]
        } else if sun_h > -0.05 {
            let t = (sun_h + 0.05) / 0.20;
            lerp3([0.15, 0.15, 0.35], [0.25, 0.47, 0.85], t)
        } else {
            let t = ((-sun_h - 0.05) / 0.3).min(1.0);
            lerp3([0.15, 0.15, 0.35], [0.02, 0.02, 0.08], t)
        }
    }

    /// Sky horizon color.
    pub fn sky_horizon(&self) -> [f32; 3] {
        let sun_h = self.light_dir().y;
        if sun_h > 0.15 {
            [0.55, 0.73, 0.94]
        } else if sun_h > -0.05 {
            let t = (sun_h + 0.05) / 0.20;
            lerp3([0.90, 0.50, 0.25], [0.55, 0.73, 0.94], t)
        } else {
            let t = ((-sun_h - 0.05) / 0.3).min(1.0);
            lerp3([0.90, 0.50, 0.25], [0.05, 0.05, 0.12], t)
        }
    }

    /// Sky bottom color (ground/fog).
    pub fn sky_bottom(&self) -> [f32; 3] {
        let fc = self.fog_color();
        [fc[0] * 0.7, fc[1] * 0.7, fc[2] * 0.7]
    }
}

fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] * (1.0 - t) + b[0] * t,
        a[1] * (1.0 - t) + b[1] * t,
        a[2] * (1.0 - t) + b[2] * t,
    ]
}
