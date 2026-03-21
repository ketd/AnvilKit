use noise::{NoiseFn, SuperSimplex};

use crate::block::BlockType;
use crate::chunk::{ChunkData, CHUNK_SIZE, CHUNK_HEIGHT};
use crate::config;

pub struct WorldGenerator {
    noise_height: SuperSimplex,
    noise_detail: SuperSimplex,
    noise_tree: SuperSimplex,
    noise_plant: SuperSimplex,
    noise_cloud: SuperSimplex,
    noise_cave: SuperSimplex,
}

impl WorldGenerator {
    pub fn new(seed: u32) -> Self {
        Self {
            noise_height: SuperSimplex::new(seed),
            noise_detail: SuperSimplex::new(seed.wrapping_add(1)),
            noise_tree: SuperSimplex::new(seed.wrapping_add(2)),
            noise_plant: SuperSimplex::new(seed.wrapping_add(3)),
            noise_cloud: SuperSimplex::new(seed.wrapping_add(4)),
            noise_cave: SuperSimplex::new(seed.wrapping_add(5)),
        }
    }

    /// Generate a full chunk at chunk coordinates (cx, cz).
    /// World X = cx * CHUNK_SIZE + local_x, World Z = cz * CHUNK_SIZE + local_z.
    pub fn generate_chunk(&self, cx: i32, cz: i32) -> ChunkData {
        let mut chunk = ChunkData::new();
        let base_x = cx * CHUNK_SIZE as i32;
        let base_z = cz * CHUNK_SIZE as i32;

        // Pre-compute height map
        let mut height_map = [[0i32; CHUNK_SIZE]; CHUNK_SIZE];
        for lx in 0..CHUNK_SIZE {
            for lz in 0..CHUNK_SIZE {
                let wx = (base_x + lx as i32) as f64;
                let wz = (base_z + lz as i32) as f64;

                // Two-octave simplex: base height + amplitude modulation
                let h1 = self.noise_height.get([wx * 0.005, wz * 0.005]);
                let amp = self.noise_detail.get([wx * 0.01, wz * 0.01]);
                let h = 32.0 + h1 * 16.0 + amp * 8.0;
                height_map[lx][lz] = h as i32;
            }
        }

        let water_level = config::WATER_LEVEL;

        // Fill terrain
        for lx in 0..CHUNK_SIZE {
            for lz in 0..CHUNK_SIZE {
                let h = height_map[lx][lz];

                for y in 0..CHUNK_HEIGHT {
                    let yi = y as i32;
                    let block = if yi == 0 {
                        BlockType::Stone
                    } else if yi < h - 4 {
                        BlockType::Stone
                    } else if yi < h {
                        if h <= water_level + 1 {
                            BlockType::Sand
                        } else {
                            BlockType::Dirt
                        }
                    } else if yi == h {
                        if h <= water_level {
                            BlockType::Sand
                        } else {
                            BlockType::Grass
                        }
                    } else if yi <= water_level {
                        BlockType::Water
                    } else {
                        BlockType::Air
                    };
                    if block != BlockType::Air {
                        chunk.set(lx, y, lz, block);
                    }
                }
            }
        }

        // Cave carving: 3D noise removes blocks to create underground caverns
        for lx in 0..CHUNK_SIZE {
            for lz in 0..CHUNK_SIZE {
                let wx = (base_x + lx as i32) as f64;
                let wz = (base_z + lz as i32) as f64;
                let h = height_map[lx][lz];

                for y in 2..CHUNK_HEIGHT {
                    let yi = y as i32;
                    // Only carve below terrain surface
                    if yi >= h {
                        break;
                    }
                    // Preserve bedrock layer (y=0..1)
                    let wy = y as f64;
                    let cave_val = self.noise_cave.get([wx * 0.05, wy * 0.05, wz * 0.05]);
                    if cave_val > 0.3 {
                        // Carve cave: fill with water if below water level, else air
                        if yi <= water_level {
                            chunk.set(lx, y, lz, BlockType::Water);
                        } else {
                            chunk.set(lx, y, lz, BlockType::Air);
                        }
                    }
                }
            }
        }

        // Trees
        for lx in 2..CHUNK_SIZE - 2 {
            for lz in 2..CHUNK_SIZE - 2 {
                let wx = (base_x + lx as i32) as f64;
                let wz = (base_z + lz as i32) as f64;
                let h = height_map[lx][lz];
                if h <= water_level || h >= (CHUNK_HEIGHT as i32 - 12) {
                    continue;
                }

                let tree_val = self.noise_tree.get([wx * 0.3, wz * 0.3]);
                if tree_val > 0.84 {
                    self.place_tree(&mut chunk, lx, (h + 1) as usize, lz);
                }
            }
        }

        // Plants
        for lx in 0..CHUNK_SIZE {
            for lz in 0..CHUNK_SIZE {
                let wx = (base_x + lx as i32) as f64;
                let wz = (base_z + lz as i32) as f64;
                let h = height_map[lx][lz];
                if h <= water_level {
                    continue;
                }
                let plant_val = self.noise_plant.get([wx * 0.5, wz * 0.5]);
                if plant_val > 0.7 {
                    let h_above = (h + 1) as usize;
                    if h_above < CHUNK_HEIGHT && chunk.get(lx, h_above, lz) == BlockType::Air {
                        chunk.set(lx, h_above, lz, BlockType::YellowFlower);
                    }
                } else if plant_val > 0.65 {
                    let h_above = (h + 1) as usize;
                    if h_above < CHUNK_HEIGHT && chunk.get(lx, h_above, lz) == BlockType::Air {
                        chunk.set(lx, h_above, lz, BlockType::RedFlower);
                    }
                } else if plant_val > 0.55 {
                    let h_above = (h + 1) as usize;
                    if h_above < CHUNK_HEIGHT && chunk.get(lx, h_above, lz) == BlockType::Air {
                        chunk.set(lx, h_above, lz, BlockType::TallGrass);
                    }
                }
            }
        }

        // Clouds
        for lx in 0..CHUNK_SIZE {
            for lz in 0..CHUNK_SIZE {
                let wx = (base_x + lx as i32) as f64;
                let wz = (base_z + lz as i32) as f64;
                let cloud_val = self.noise_cloud.get([wx * 0.01, wz * 0.01]);
                if cloud_val > 0.75 {
                    for cy in config::CLOUD_Y_MIN..config::CLOUD_Y_MAX {
                        chunk.set(lx, cy, lz, BlockType::Cloud);
                    }
                }
            }
        }

        chunk
    }

    fn place_tree(&self, chunk: &mut ChunkData, x: usize, base_y: usize, z: usize) {
        let trunk_height = 5;
        // Trunk
        for dy in 0..trunk_height {
            let y = base_y + dy;
            if y < CHUNK_HEIGHT {
                chunk.set(x, y, z, BlockType::Wood);
            }
        }
        // Leaves sphere (radius 3, centered at top of trunk)
        let center_y = base_y + trunk_height;
        let r = 3i32;
        for dx in -r..=r {
            for dy in -r..=r {
                for dz in -r..=r {
                    if dx * dx + dy * dy + dz * dz > r * r + 1 {
                        continue;
                    }
                    let lx = x as i32 + dx;
                    let ly = center_y as i32 + dy;
                    let lz = z as i32 + dz;
                    if lx < 0
                        || lx >= CHUNK_SIZE as i32
                        || ly < 0
                        || ly >= CHUNK_HEIGHT as i32
                        || lz < 0
                        || lz >= CHUNK_SIZE as i32
                    {
                        continue;
                    }
                    if chunk.get(lx as usize, ly as usize, lz as usize) == BlockType::Air {
                        chunk.set(lx as usize, ly as usize, lz as usize, BlockType::Leaves);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_terrain() {
        let gen = WorldGenerator::new(42);
        let chunk = gen.generate_chunk(0, 0);
        // Bottom should be stone
        assert_eq!(chunk.get(0, 0, 0), BlockType::Stone);
        // Some height should have grass/sand/dirt
        let mut found_surface = false;
        for y in 20..50 {
            let b = chunk.get(16, y, 16);
            if b == BlockType::Grass || b == BlockType::Sand {
                found_surface = true;
                break;
            }
        }
        assert!(found_surface, "Should find a surface block");
    }
}
