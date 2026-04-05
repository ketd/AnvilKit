use noise::{NoiseFn, SuperSimplex};

use crate::block::BlockType;
use crate::biome::{Biome, BiomeMap};
use crate::chunk::{ChunkData, CHUNK_SIZE, CHUNK_HEIGHT};
use crate::config;

pub struct WorldGenerator {
    noise_height: SuperSimplex,
    noise_detail: SuperSimplex,
    noise_tree: SuperSimplex,
    noise_plant: SuperSimplex,
    noise_cloud: SuperSimplex,
    noise_cave: SuperSimplex,
    noise_ore: SuperSimplex,
    biome_map: BiomeMap,
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
            noise_ore: SuperSimplex::new(seed.wrapping_add(6)),
            biome_map: BiomeMap::new(seed),
        }
    }

    /// Generate a full chunk at chunk coordinates (cx, cz).
    /// World X = cx * CHUNK_SIZE + local_x, World Z = cz * CHUNK_SIZE + local_z.
    pub fn generate_chunk(&self, cx: i32, cz: i32) -> ChunkData {
        let mut chunk = ChunkData::new();
        let base_x = cx * CHUNK_SIZE as i32;
        let base_z = cz * CHUNK_SIZE as i32;

        // Pre-compute height map + biome map
        let mut height_map = [[0i32; CHUNK_SIZE]; CHUNK_SIZE];
        let mut biome_map = [[Biome::Plains; CHUNK_SIZE]; CHUNK_SIZE];
        for lx in 0..CHUNK_SIZE {
            for lz in 0..CHUNK_SIZE {
                let wx = (base_x + lx as i32) as f64;
                let wz = (base_z + lz as i32) as f64;

                let biome = self.biome_map.get_biome(wx, wz);
                biome_map[lx][lz] = biome;

                // Two-octave simplex with biome modulation (smoothed at boundaries)
                let h1 = self.noise_height.get([wx * config::NOISE_HEIGHT_SCALE, wz * config::NOISE_HEIGHT_SCALE]);
                let amp = self.noise_detail.get([wx * config::NOISE_DETAIL_SCALE, wz * config::NOISE_DETAIL_SCALE]);
                let (height_offset, height_scale) = self.biome_map.smoothed_height_params(wx, wz);
                let base_h = config::BASE_HEIGHT + height_offset;
                let h = base_h + (h1 * config::HEIGHT_AMP1 + amp * config::HEIGHT_AMP2) * height_scale;
                height_map[lx][lz] = h as i32;
            }
        }

        let water_level = config::WATER_LEVEL;

        // Fill terrain (biome-aware surface/fill blocks)
        for lx in 0..CHUNK_SIZE {
            for lz in 0..CHUNK_SIZE {
                let h = height_map[lx][lz];
                let biome = biome_map[lx][lz];
                let (surface, fill) = biome_surface_blocks(biome, h, water_level);

                for y in 0..CHUNK_HEIGHT {
                    let yi = y as i32;
                    let block = if yi == 0 {
                        BlockType::Stone
                    } else if yi < h - 4 {
                        BlockType::Stone
                    } else if yi < h {
                        fill
                    } else if yi == h {
                        surface
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

        // Ore generation
        self.generate_ores(&mut chunk, &height_map, base_x, base_z);

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
                    let cave_val = self.noise_cave.get([wx * config::CAVE_NOISE_SCALE, wy * config::CAVE_NOISE_SCALE, wz * config::CAVE_NOISE_SCALE]);
                    if cave_val > config::CAVE_THRESHOLD {
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

        // Trees (biome-filtered)
        for lx in 2..CHUNK_SIZE - 2 {
            for lz in 2..CHUNK_SIZE - 2 {
                let biome = biome_map[lx][lz];
                if !biome.has_trees() { continue; }
                let wx = (base_x + lx as i32) as f64;
                let wz = (base_z + lz as i32) as f64;
                let h = height_map[lx][lz];
                if h <= water_level || h >= (CHUNK_HEIGHT as i32 - 12) {
                    continue;
                }

                let tree_val = self.noise_tree.get([wx * config::TREE_NOISE_SCALE, wz * config::TREE_NOISE_SCALE]);
                if tree_val > config::TREE_THRESHOLD {
                    let (trunk, leaves) = biome_tree_blocks(biome);
                    self.place_tree_typed(&mut chunk, lx, (h + 1) as usize, lz, trunk, leaves);
                }
            }
        }

        // Plants (biome-filtered)
        for lx in 0..CHUNK_SIZE {
            for lz in 0..CHUNK_SIZE {
                let biome = biome_map[lx][lz];
                if !biome.has_plants() { continue; }
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
                let cloud_val = self.noise_cloud.get([wx * config::CLOUD_NOISE_SCALE, wz * config::CLOUD_NOISE_SCALE]);
                if cloud_val > config::CLOUD_THRESHOLD {
                    for cy in config::CLOUD_Y_MIN..config::CLOUD_Y_MAX {
                        chunk.set(lx, cy, lz, BlockType::Cloud);
                    }
                }
            }
        }

        chunk
    }

    fn place_tree_typed(&self, chunk: &mut ChunkData, x: usize, base_y: usize, z: usize, trunk: BlockType, leaves: BlockType) {
        let trunk_height = config::TRUNK_HEIGHT;
        for dy in 0..trunk_height {
            let y = base_y + dy;
            if y < CHUNK_HEIGHT {
                chunk.set(x, y, z, trunk);
            }
        }
        let center_y = base_y + trunk_height;
        let r = config::LEAF_RADIUS;
        for dx in -r..=r {
            for dy in -r..=r {
                for dz in -r..=r {
                    if dx * dx + dy * dy + dz * dz > r * r + 1 {
                        continue;
                    }
                    let lx = x as i32 + dx;
                    let ly = center_y as i32 + dy;
                    let lz = z as i32 + dz;
                    if lx < 0 || lx >= CHUNK_SIZE as i32 || ly < 0 || ly >= CHUNK_HEIGHT as i32 || lz < 0 || lz >= CHUNK_SIZE as i32 {
                        continue;
                    }
                    if chunk.get(lx as usize, ly as usize, lz as usize) == BlockType::Air {
                        chunk.set(lx as usize, ly as usize, lz as usize, leaves);
                    }
                }
            }
        }
    }

    fn generate_ores(&self, chunk: &mut ChunkData, height_map: &[[i32; CHUNK_SIZE]; CHUNK_SIZE], base_x: i32, base_z: i32) {
        // (block_type, y_min, y_max, noise_threshold, noise_scale)
        let ore_configs: [(BlockType, i32, i32, f64, f64); 6] = [
            (BlockType::CoalOre,     5, 80, 0.70, 0.08),
            (BlockType::IronOre,     5, 64, 0.75, 0.07),
            (BlockType::GoldOre,     5, 32, 0.82, 0.06),
            (BlockType::DiamondOre,  1, 16, 0.88, 0.05),
            (BlockType::RedstoneOre, 1, 16, 0.85, 0.06),
            (BlockType::LapisOre,    1, 32, 0.84, 0.06),
        ];

        for lx in 0..CHUNK_SIZE {
            for lz in 0..CHUNK_SIZE {
                let wx = (base_x + lx as i32) as f64;
                let wz = (base_z + lz as i32) as f64;
                let surface = height_map[lx][lz];

                for &(block_type, y_min, y_max, threshold, scale) in &ore_configs {
                    let y_end = (y_max as usize).min(surface as usize);
                    for y in (y_min as usize)..y_end {
                        if chunk.get(lx, y, lz) != BlockType::Stone {
                            continue;
                        }
                        let wy = y as f64;
                        let val = self.noise_ore.get([wx * scale, wy * scale, wz * scale]);
                        if val > threshold {
                            chunk.set(lx, y, lz, block_type);
                        }
                    }
                }
            }
        }
    }
}

/// Select surface and fill blocks based on biome.
fn biome_surface_blocks(biome: Biome, h: i32, water_level: i32) -> (BlockType, BlockType) {
    match biome {
        Biome::Desert => (BlockType::Sand, BlockType::Sandstone),
        Biome::Tundra => {
            if h <= water_level { (BlockType::Sand, BlockType::Gravel) }
            else { (BlockType::SnowBlock, BlockType::Stone) }
        }
        Biome::Ocean => (BlockType::Sand, BlockType::Sand),
        Biome::Mountains => (BlockType::Stone, BlockType::Stone),
        Biome::Swamp => {
            if h <= water_level + 1 { (BlockType::Sand, BlockType::Dirt) }
            else { (BlockType::Grass, BlockType::Dirt) }
        }
        _ => {
            // Plains, Forest — default behavior
            if h <= water_level { (BlockType::Sand, BlockType::Sand) }
            else { (BlockType::Grass, BlockType::Dirt) }
        }
    }
}

/// Select tree trunk/leaves blocks based on biome.
fn biome_tree_blocks(biome: Biome) -> (BlockType, BlockType) {
    match biome {
        Biome::Tundra => (BlockType::SpruceWood, BlockType::SpruceLeaves),
        Biome::Forest => (BlockType::BirchWood, BlockType::BirchLeaves), // mix birch in forests
        _ => (BlockType::Wood, BlockType::Leaves),
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
        // Some height should have a surface block (varies by biome)
        let mut found_surface = false;
        for y in 10..80 {
            let b = chunk.get(16, y, 16);
            if matches!(b, BlockType::Grass | BlockType::Sand | BlockType::Stone
                | BlockType::SnowBlock | BlockType::Sandstone) {
                // Check there's air or water above — confirms surface
                let above = chunk.get(16, y + 1, 16);
                if above == BlockType::Air || above == BlockType::Water {
                    found_surface = true;
                    break;
                }
            }
        }
        assert!(found_surface, "Should find a surface block");
    }

    #[test]
    fn generates_ores() {
        let gen = WorldGenerator::new(42);
        let chunk = gen.generate_chunk(0, 0);
        // Check that at least some ore blocks were placed
        let mut ore_count = 0;
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                for y in 1..60 {
                    let b = chunk.get(x, y, z);
                    if matches!(b, BlockType::CoalOre | BlockType::IronOre | BlockType::GoldOre
                        | BlockType::DiamondOre | BlockType::RedstoneOre | BlockType::LapisOre) {
                        ore_count += 1;
                    }
                }
            }
        }
        assert!(ore_count > 0, "Should generate some ore blocks");
    }
}
