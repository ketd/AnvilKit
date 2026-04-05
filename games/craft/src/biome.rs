//! Biome system — temperature/humidity noise → biome selection.

use noise::{NoiseFn, SuperSimplex};
use bevy_ecs::prelude::Resource;

/// Biome type determined by temperature and humidity noise.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Biome {
    Plains,
    Forest,
    Desert,
    Tundra,
    Ocean,
    Mountains,
    Swamp,
}

/// Biome map resource — dual-noise lookup.
#[derive(Resource)]
pub struct BiomeMap {
    noise_temp: SuperSimplex,
    noise_humidity: SuperSimplex,
}

impl BiomeMap {
    pub fn new(seed: u32) -> Self {
        Self {
            noise_temp: SuperSimplex::new(seed.wrapping_add(100)),
            noise_humidity: SuperSimplex::new(seed.wrapping_add(101)),
        }
    }

    /// Get biome at world coordinates.
    pub fn get_biome(&self, wx: f64, wz: f64) -> Biome {
        let temp = self.noise_temp.get([wx * 0.002, wz * 0.002]);
        let humidity = self.noise_humidity.get([wx * 0.003, wz * 0.003]);

        if temp < -0.3 {
            Biome::Tundra
        } else if temp > 0.4 {
            if humidity < -0.2 { Biome::Desert } else { Biome::Swamp }
        } else if humidity > 0.3 {
            Biome::Forest
        } else if humidity < -0.4 {
            Biome::Ocean
        } else if temp > 0.2 {
            Biome::Plains
        } else {
            Biome::Mountains
        }
    }
}

impl Biome {
    /// Height offset added to base terrain height.
    pub fn height_offset(self) -> f64 {
        match self {
            Self::Ocean => -15.0,
            Self::Mountains => 20.0,
            Self::Swamp => -5.0,
            _ => 0.0,
        }
    }

    /// Height noise scale multiplier.
    pub fn height_scale(self) -> f64 {
        match self {
            Self::Mountains => 2.0,
            Self::Ocean => 0.5,
            Self::Swamp => 0.3,
            _ => 1.0,
        }
    }

    /// Whether trees can spawn in this biome.
    pub fn has_trees(self) -> bool {
        !matches!(self, Self::Desert | Self::Ocean)
    }

    /// Whether plants (grass/flowers) can spawn.
    pub fn has_plants(self) -> bool {
        !matches!(self, Self::Desert | Self::Tundra | Self::Ocean)
    }
}

impl BiomeMap {
    /// Get smoothed height parameters at a world position by blending nearby biomes.
    /// Samples a 5x5 grid around the position and averages height_offset and height_scale.
    pub fn smoothed_height_params(&self, wx: f64, wz: f64) -> (f64, f64) {
        let mut offset_sum = 0.0;
        let mut scale_sum = 0.0;
        let samples = 5;
        let step = 4.0; // sample every 4 blocks
        let total = (samples * samples) as f64;

        for dx in 0..samples {
            for dz in 0..samples {
                let sx = wx + (dx as f64 - 2.0) * step;
                let sz = wz + (dz as f64 - 2.0) * step;
                let biome = self.get_biome(sx, sz);
                offset_sum += biome.height_offset();
                scale_sum += biome.height_scale();
            }
        }

        (offset_sum / total, scale_sum / total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_biome_deterministic() {
        let map = BiomeMap::new(42);
        let b1 = map.get_biome(100.0, 200.0);
        let b2 = map.get_biome(100.0, 200.0);
        assert_eq!(b1, b2);
    }

    #[test]
    fn test_biome_coverage() {
        let map = BiomeMap::new(42);
        let mut found = std::collections::HashSet::new();
        // Sample a large area to find all biomes
        for x in -500..500 {
            for z in -500..500 {
                found.insert(map.get_biome(x as f64 * 10.0, z as f64 * 10.0));
                if found.len() == 7 { break; }
            }
            if found.len() == 7 { break; }
        }
        assert!(found.len() >= 5, "Expected at least 5 different biomes in a 10000x10000 area, found {}", found.len());
    }

    #[test]
    fn test_biome_smoothing() {
        let map = BiomeMap::new(42);
        let (offset, scale) = map.smoothed_height_params(100.0, 100.0);
        // Smoothed values should be finite and reasonable
        assert!(offset.is_finite());
        assert!(scale.is_finite());
        assert!(scale > 0.0);
    }

    #[test]
    fn test_biome_properties() {
        assert!(!Biome::Desert.has_trees());
        assert!(!Biome::Ocean.has_trees());
        assert!(Biome::Forest.has_trees());
        assert!(!Biome::Desert.has_plants());
        assert!(Biome::Plains.has_plants());
    }
}
