//! Block lighting system — sky light + block light, BFS propagation.
//!
//! Each block stores one byte: high 4 bits = sky light (0-15), low 4 bits = block light (0-15).
//! Sky light propagates from Y=255 downward; block light propagates from light-emitting blocks via BFS.

use std::collections::VecDeque;
use crate::block::BlockType;
use crate::chunk::{ChunkData, CHUNK_SIZE, CHUNK_HEIGHT};

/// Per-chunk light data, parallel to ChunkData.
pub struct LightMap {
    /// High nibble = sky light, low nibble = block light.
    data: Box<[u8; CHUNK_SIZE * CHUNK_HEIGHT * CHUNK_SIZE]>,
}

impl LightMap {
    pub fn new() -> Self {
        Self {
            data: Box::new([0u8; CHUNK_SIZE * CHUNK_HEIGHT * CHUNK_SIZE]),
        }
    }

    #[inline]
    fn index(x: usize, y: usize, z: usize) -> usize {
        x + z * CHUNK_SIZE + y * CHUNK_SIZE * CHUNK_SIZE
    }

    #[inline]
    pub fn get_sky(&self, x: usize, y: usize, z: usize) -> u8 {
        self.data[Self::index(x, y, z)] >> 4
    }

    #[inline]
    pub fn get_block_light(&self, x: usize, y: usize, z: usize) -> u8 {
        self.data[Self::index(x, y, z)] & 0x0F
    }

    #[inline]
    pub fn set_sky(&mut self, x: usize, y: usize, z: usize, val: u8) {
        let i = Self::index(x, y, z);
        self.data[i] = (val.min(15) << 4) | (self.data[i] & 0x0F);
    }

    #[inline]
    pub fn set_block_light(&mut self, x: usize, y: usize, z: usize, val: u8) {
        let i = Self::index(x, y, z);
        self.data[i] = (self.data[i] & 0xF0) | val.min(15);
    }

    /// Get packed light value (for vertex encoding).
    #[inline]
    pub fn get_packed(&self, x: usize, y: usize, z: usize) -> u8 {
        self.data[Self::index(x, y, z)]
    }

    /// Safe get for out-of-bounds coordinates.
    /// Returns full sky light for any out-of-bounds position (cross-chunk faces should
    /// not go dark at chunk boundaries).
    #[inline]
    pub fn get_packed_safe(&self, x: i32, y: i32, z: i32) -> u8 {
        if x < 0 || x >= CHUNK_SIZE as i32 || y < 0 || y >= CHUNK_HEIGHT as i32 || z < 0 || z >= CHUNK_SIZE as i32 {
            0xF0 // full sky light — avoids black seams at chunk edges
        } else {
            self.data[Self::index(x as usize, y as usize, z as usize)]
        }
    }
}

impl Default for LightMap {
    fn default() -> Self { Self::new() }
}

/// Initialize sky light for a chunk by propagating from top to bottom.
/// Columns with only transparent blocks get full sky light (15) all the way down.
/// Opaque blocks block sky light propagation.
pub fn compute_initial_sky_light(chunk: &ChunkData, light: &mut LightMap) {
    for x in 0..CHUNK_SIZE {
        for z in 0..CHUNK_SIZE {
            let mut sky = 15u8;
            for y in (0..CHUNK_HEIGHT).rev() {
                let block = chunk.get(x, y, z);
                if block.is_transparent() {
                    light.set_sky(x, y, z, sky);
                } else {
                    sky = 0;
                    light.set_sky(x, y, z, 0);
                }
            }
        }
    }
}

/// Spread sky light horizontally and downward through transparent blocks via BFS.
/// After the initial top-down pass, sky-lit air blocks spread their light to neighbors
/// with a decay of 1 per block. This illuminates alcoves, overhangs, and cave entrances
/// that aren't directly open to the sky.
pub fn propagate_sky_light(chunk: &ChunkData, light: &mut LightMap) {
    let mut queue: VecDeque<(usize, usize, usize)> = VecDeque::new();

    // Enqueue all transparent blocks that already have sky light from the vertical pass
    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_HEIGHT {
            for z in 0..CHUNK_SIZE {
                let sky = light.get_sky(x, y, z);
                if sky > 1 && chunk.get(x, y, z).is_transparent() {
                    queue.push_back((x, y, z));
                }
            }
        }
    }

    let neighbors: [(i32, i32, i32); 6] = [
        (1, 0, 0), (-1, 0, 0),
        (0, 1, 0), (0, -1, 0),
        (0, 0, 1), (0, 0, -1),
    ];

    while let Some((x, y, z)) = queue.pop_front() {
        let current = light.get_sky(x, y, z);
        if current <= 1 { continue; }
        let spread = current - 1;

        for (dx, dy, dz) in neighbors {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            let nz = z as i32 + dz;

            if nx < 0 || nx >= CHUNK_SIZE as i32 || ny < 0 || ny >= CHUNK_HEIGHT as i32 || nz < 0 || nz >= CHUNK_SIZE as i32 {
                continue;
            }
            let (nx, ny, nz) = (nx as usize, ny as usize, nz as usize);

            let neighbor_block = chunk.get(nx, ny, nz);
            if !neighbor_block.is_transparent() { continue; }

            if light.get_sky(nx, ny, nz) < spread {
                light.set_sky(nx, ny, nz, spread);
                queue.push_back((nx, ny, nz));
            }
        }
    }
}

/// Propagate block light from all light-emitting blocks in the chunk using BFS.
pub fn compute_block_light(chunk: &ChunkData, light: &mut LightMap) {
    let mut queue: VecDeque<(usize, usize, usize)> = VecDeque::new();

    // Find all light-emitting blocks
    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_HEIGHT {
            for z in 0..CHUNK_SIZE {
                let block = chunk.get(x, y, z);
                let emission = block_light_emission(block);
                if emission > 0 {
                    light.set_block_light(x, y, z, emission);
                    queue.push_back((x, y, z));
                }
            }
        }
    }

    // BFS flood fill
    while let Some((x, y, z)) = queue.pop_front() {
        let current = light.get_block_light(x, y, z);
        if current <= 1 { continue; }
        let spread = current - 1;

        let neighbors: [(i32, i32, i32); 6] = [
            (1, 0, 0), (-1, 0, 0),
            (0, 1, 0), (0, -1, 0),
            (0, 0, 1), (0, 0, -1),
        ];

        for (dx, dy, dz) in neighbors {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            let nz = z as i32 + dz;

            if nx < 0 || nx >= CHUNK_SIZE as i32 || ny < 0 || ny >= CHUNK_HEIGHT as i32 || nz < 0 || nz >= CHUNK_SIZE as i32 {
                continue;
            }
            let (nx, ny, nz) = (nx as usize, ny as usize, nz as usize);

            let neighbor_block = chunk.get(nx, ny, nz);
            if !neighbor_block.is_transparent() && neighbor_block != BlockType::Air {
                continue;
            }

            if light.get_block_light(nx, ny, nz) < spread {
                light.set_block_light(nx, ny, nz, spread);
                queue.push_back((nx, ny, nz));
            }
        }
    }
}

/// Get light emission level for a block type.
fn block_light_emission(block: BlockType) -> u8 {
    match block {
        BlockType::Glowstone => 15,
        BlockType::Torch => 14,
        BlockType::Lantern => 13,
        BlockType::LightStone => 12,
        BlockType::RedstoneOre => 7,
        _ => 0,
    }
}

/// Compute a per-vertex light value by averaging the 4 adjacent block light values
/// (similar to smooth AO). Returns a packed f32: `sky * 16.0 + block`.
/// The shader unpacks and applies: `max(sky/15 * day_factor, block/15)`.
pub fn vertex_light(light_map: &LightMap, x: i32, y: i32, z: i32) -> f32 {
    let packed = light_map.get_packed_safe(x, y, z);
    let sky = (packed >> 4) as f32;
    let block = (packed & 0x0F) as f32;
    sky * 16.0 + block
}

/// Average light of 4 blocks sharing a vertex (for smooth lighting).
pub fn smooth_vertex_light(light_map: &LightMap, samples: &[(i32, i32, i32); 4]) -> f32 {
    let mut sky_sum = 0.0_f32;
    let mut block_sum = 0.0_f32;
    for &(x, y, z) in samples {
        let packed = light_map.get_packed_safe(x, y, z);
        sky_sum += (packed >> 4) as f32;
        block_sum += (packed & 0x0F) as f32;
    }
    let sky = sky_sum / 4.0;
    let block = block_sum / 4.0;
    sky * 16.0 + block
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_light_map_get_set() {
        let mut lm = LightMap::new();
        lm.set_sky(5, 10, 3, 12);
        lm.set_block_light(5, 10, 3, 7);
        assert_eq!(lm.get_sky(5, 10, 3), 12);
        assert_eq!(lm.get_block_light(5, 10, 3), 7);
    }

    #[test]
    fn test_light_map_nibble_independence() {
        let mut lm = LightMap::new();
        lm.set_sky(0, 0, 0, 15);
        assert_eq!(lm.get_block_light(0, 0, 0), 0);
        lm.set_block_light(0, 0, 0, 8);
        assert_eq!(lm.get_sky(0, 0, 0), 15);
        assert_eq!(lm.get_block_light(0, 0, 0), 8);
    }

    #[test]
    fn test_sky_light_propagation() {
        let mut chunk = ChunkData::new();
        let mut lm = LightMap::new();

        // Air column → full sky light
        compute_initial_sky_light(&chunk, &mut lm);
        assert_eq!(lm.get_sky(0, 200, 0), 15);
        assert_eq!(lm.get_sky(0, 0, 0), 15);

        // Place opaque block mid-column
        chunk.set(5, 50, 5, BlockType::Stone);
        let mut lm2 = LightMap::new();
        compute_initial_sky_light(&chunk, &mut lm2);
        assert_eq!(lm2.get_sky(5, 51, 5), 15); // above stone
        assert_eq!(lm2.get_sky(5, 50, 5), 0);  // at stone
        assert_eq!(lm2.get_sky(5, 49, 5), 0);  // below stone
    }

    #[test]
    fn test_sky_light_horizontal_spread() {
        let mut chunk = ChunkData::new();
        // Create a 1-block roof at y=50 spanning x=0..32, z=0..32
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                chunk.set(x, 50, z, BlockType::Stone);
            }
        }
        // Poke a hole at (16, 50) — remove the stone so sky light enters
        chunk.set(16, 50, 16, BlockType::Air);

        let mut lm = LightMap::new();
        compute_initial_sky_light(&chunk, &mut lm);

        // Before horizontal spread: block under roof at (15,49,16) has sky=0
        assert_eq!(lm.get_sky(15, 49, 16), 0);

        propagate_sky_light(&chunk, &mut lm);

        // After spread: the hole at (16,50,16) lets sky=15 through,
        // (16,49,16) gets 15 from vertical pass, then (15,49,16) gets 14
        assert_eq!(lm.get_sky(16, 49, 16), 15); // directly below hole
        assert_eq!(lm.get_sky(15, 49, 16), 14); // 1 block sideways from hole
        assert_eq!(lm.get_sky(14, 49, 16), 13); // 2 blocks sideways
    }

    #[test]
    fn test_block_light_propagation() {
        let mut chunk = ChunkData::new();
        let mut lm = LightMap::new();

        // Place a torch (emission=14) in open air
        chunk.set(16, 50, 16, BlockType::Torch);
        compute_block_light(&chunk, &mut lm);

        // Torch position should have light 14
        assert_eq!(lm.get_block_light(16, 50, 16), 14);
        // One block away should have 13
        assert_eq!(lm.get_block_light(17, 50, 16), 13);
        // Two blocks away should have 12
        assert_eq!(lm.get_block_light(18, 50, 16), 12);
    }

    #[test]
    fn test_vertex_light_encoding() {
        let mut lm = LightMap::new();
        lm.set_sky(0, 0, 0, 10);
        lm.set_block_light(0, 0, 0, 5);
        let v = vertex_light(&lm, 0, 0, 0);
        // sky=10, block=5 → 10*16+5 = 165
        assert!((v - 165.0).abs() < 0.01);
    }
}
