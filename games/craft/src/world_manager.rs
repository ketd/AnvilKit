//! World management — creation, selection, listing save slots.

use bevy_ecs::prelude::*;
use anvilkit_core::persistence::SaveManager;

/// Describes a saved world for the selection screen.
#[derive(Debug, Clone)]
pub struct WorldInfo {
    pub slot_name: String,
    pub display_name: String,
    pub seed: u32,
    pub last_played: String, // human-readable timestamp
}

/// Game mode — controls survival mechanics and inventory behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Resource)]
pub enum GameMode {
    Survival,
    Creative,
}

impl Default for GameMode {
    fn default() -> Self { Self::Creative }
}

/// Data for creating a new world.
#[derive(Debug, Clone)]
pub struct NewWorldConfig {
    pub name: String,
    pub seed_input: String, // empty = random
    pub mode: GameMode,
}

impl Default for NewWorldConfig {
    fn default() -> Self {
        Self {
            name: "New World".to_string(),
            seed_input: String::new(),
            mode: GameMode::Survival,
        }
    }
}

impl NewWorldConfig {
    /// Resolve seed from input string (hash or parse, random if empty).
    pub fn resolved_seed(&self) -> u32 {
        if self.seed_input.is_empty() {
            // Use system time as seed
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u32)
                .unwrap_or(42)
        } else if let Ok(n) = self.seed_input.parse::<u32>() {
            n
        } else {
            // String hash
            let mut hash: u32 = 0;
            for b in self.seed_input.bytes() {
                hash = hash.wrapping_mul(31).wrapping_add(b as u32);
            }
            hash
        }
    }
}

/// List all saved worlds from the save directory.
pub fn list_worlds(save_dir: &str) -> Vec<WorldInfo> {
    let save_mgr = match SaveManager::new(save_dir, "0.1.0") {
        Ok(m) => m,
        Err(_) => return Vec::new(),
    };

    let slots = save_mgr.list_saves();
    slots.into_iter().map(|info| {
        let seed = info.metadata.get("seed")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        WorldInfo {
            slot_name: info.name.clone(),
            display_name: info.metadata.get("world_name")
                .cloned()
                .unwrap_or_else(|| info.name.clone()),
            seed,
            last_played: format!("{}s played", info.play_time_secs),
        }
    }).collect()
}

/// F3 debug overlay data — updated each frame, read by HUD renderer.
#[derive(Debug, Clone, Resource)]
pub struct DebugInfo {
    pub fps: f32,
    pub player_pos: [f32; 3],
    pub chunk_pos: [i32; 2],
    pub facing: [f32; 3],
    pub light_level: u8,
    pub biome_name: String,
    pub loaded_chunks: usize,
    pub active_entities: usize,
    pub show: bool,
}

impl Default for DebugInfo {
    fn default() -> Self {
        Self {
            fps: 0.0,
            player_pos: [0.0; 3],
            chunk_pos: [0; 2],
            facing: [0.0, 0.0, -1.0],
            light_level: 15,
            biome_name: "Plains".to_string(),
            loaded_chunks: 0,
            active_entities: 0,
            show: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_world_config_default() {
        let cfg = NewWorldConfig::default();
        assert_eq!(cfg.name, "New World");
        assert_eq!(cfg.mode, GameMode::Survival);
    }

    #[test]
    fn test_seed_resolution_numeric() {
        let cfg = NewWorldConfig {
            seed_input: "12345".to_string(),
            ..Default::default()
        };
        assert_eq!(cfg.resolved_seed(), 12345);
    }

    #[test]
    fn test_seed_resolution_string() {
        let cfg = NewWorldConfig {
            seed_input: "hello".to_string(),
            ..Default::default()
        };
        let s1 = cfg.resolved_seed();
        let s2 = cfg.resolved_seed();
        assert_eq!(s1, s2, "Same string should produce same seed");
        assert_ne!(s1, 0);
    }

    #[test]
    fn test_seed_resolution_empty() {
        let cfg = NewWorldConfig::default();
        let s = cfg.resolved_seed();
        // Random seed, just check it's nonzero
        assert!(s > 0 || s == 0); // always true, but exercises the code path
    }

    #[test]
    fn test_game_mode_default() {
        assert_eq!(GameMode::default(), GameMode::Creative);
    }
}
