use std::collections::{HashMap, HashSet};
use std::io::{self, Read, Cursor};
use std::path::PathBuf;

use serde::{Serialize, Deserialize};
use anvilkit_core::persistence::{SaveManager, WorldStorage};

use crate::chunk::{ChunkData, CHUNK_SIZE, CHUNK_HEIGHT};
use crate::resources::VoxelWorld;

// --- Engine-based persistence (new format) ---

const GAME_VERSION: &str = "0.1.0";
const DEFAULT_SLOT: &str = "quick";

/// Get (or create) the SaveManager for Craft.
fn save_manager() -> Result<SaveManager, io::Error> {
    let saves_dir = saves_dir();
    SaveManager::new(&saves_dir, GAME_VERSION)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))
}

/// Root saves directory.
pub fn saves_dir() -> PathBuf {
    let mut path = std::env::current_dir().unwrap_or_default();
    path.push("saves");
    path
}

/// Legacy save path (for backward compat import).
pub fn legacy_save_path() -> PathBuf {
    let mut path = std::env::current_dir().unwrap_or_default();
    path.push("craft_world.sav");
    path
}

/// Kept for backward compat — returns the saves directory.
pub fn save_path() -> PathBuf {
    saves_dir()
}

/// RLE-compress chunk block data.
/// Format: repeated [block_type: u8, count: u16 LE] pairs.
fn rle_compress(blocks: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    if blocks.is_empty() {
        return out;
    }

    let mut current = blocks[0];
    let mut count: u16 = 1;

    for &b in &blocks[1..] {
        if b == current && count < u16::MAX {
            count += 1;
        } else {
            out.push(current);
            out.extend_from_slice(&count.to_le_bytes());
            current = b;
            count = 1;
        }
    }
    out.push(current);
    out.extend_from_slice(&count.to_le_bytes());
    out
}

/// RLE-decompress into a block array.
fn rle_decompress(data: &[u8], expected_len: usize) -> io::Result<Vec<u8>> {
    let mut out = Vec::with_capacity(expected_len);
    let mut cursor = Cursor::new(data);

    while (cursor.position() as usize) < data.len() {
        let mut block = [0u8; 1];
        cursor.read_exact(&mut block)?;
        let mut count_bytes = [0u8; 2];
        cursor.read_exact(&mut count_bytes)?;
        let count = u16::from_le_bytes(count_bytes) as usize;
        out.extend(std::iter::repeat(block[0]).take(count));
    }

    if out.len() != expected_len {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("RLE decompressed {} bytes, expected {}", out.len(), expected_len),
        ));
    }
    Ok(out)
}

/// Save modified chunks using engine SaveManager + WorldStorage.
///
/// Each chunk is stored as key `chunk/{cx}/{cz}` with RLE-compressed block data.
/// Metadata (seed) is stored in the save slot's meta.ron.
pub fn save_world(
    world: &VoxelWorld,
    modified_chunks: &HashSet<(i32, i32)>,
    seed: u32,
) -> io::Result<usize> {
    if modified_chunks.is_empty() {
        return Ok(0);
    }

    let mgr = save_manager()?;

    // Build metadata
    let mut metadata = HashMap::new();
    metadata.insert("seed".to_string(), seed.to_string());

    // Create/update save slot and get data path
    let data_path = mgr.save(DEFAULT_SLOT, 0.0, metadata)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    let storage = WorldStorage::open(&data_path)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    // Batch all chunk writes
    let chunks_to_save: Vec<(i32, i32)> = modified_chunks
        .iter()
        .copied()
        .filter(|k| world.chunks.contains_key(k))
        .collect();

    let entries: Vec<(String, Vec<u8>)> = chunks_to_save
        .iter()
        .map(|(cx, cz)| {
            let key = format!("chunk/{}/{}", cx, cz);
            let chunk = &world.chunks[&(*cx, *cz)];
            let rle = rle_compress(&chunk.blocks[..]);
            (key, rle)
        })
        .collect();

    let batch: Vec<(&str, &[u8])> = entries
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_slice()))
        .collect();

    storage.batch_put(&batch)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    Ok(chunks_to_save.len())
}

// ---------------------------------------------------------------------------
// Player state persistence
// ---------------------------------------------------------------------------

/// Serializable player state for save/load.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerSaveData {
    pub position: [f32; 3],
    pub health: f32,
    pub max_health: f32,
    pub flying: bool,
    pub day_night_time: f32,
    /// Inventory slots: Vec of Option<(item_id, quantity)>.
    pub inventory: Vec<Option<(u32, u32)>>,
    pub selected_slot: usize,
}

/// Save player state to the current save slot.
pub fn save_player(data: &PlayerSaveData) -> io::Result<()> {
    let mgr = save_manager()?;
    let data_path = mgr.slot_data_path(DEFAULT_SLOT);
    let storage = WorldStorage::open(&data_path)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    let ron_str = ron::ser::to_string_pretty(data, ron::ser::PrettyConfig::default())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    storage.put("player", ron_str.as_bytes())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    Ok(())
}

/// Load player state from the current save slot.
pub fn load_player() -> io::Result<Option<PlayerSaveData>> {
    let mgr = save_manager()?;
    if mgr.get_save_info(DEFAULT_SLOT).is_none() {
        return Ok(None);
    }

    let data_path = mgr.slot_data_path(DEFAULT_SLOT);
    let storage = WorldStorage::open(&data_path)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    match storage.get("player") {
        Some(bytes) => {
            let ron_str = std::str::from_utf8(&bytes)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
            let data: PlayerSaveData = ron::from_str(ron_str)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
            Ok(Some(data))
        }
        None => Ok(None),
    }
}

/// Load world from engine SaveManager + WorldStorage.
/// Falls back to legacy binary format if no engine save exists.
pub fn load_world(world: &mut VoxelWorld) -> io::Result<(u32, HashSet<(i32, i32)>)> {
    // Try engine format first
    let mgr = save_manager()?;
    if let Some(info) = mgr.get_save_info(DEFAULT_SLOT) {
        return load_engine_save(world, &mgr, &info);
    }

    // Fall back to legacy binary format
    let legacy = legacy_save_path();
    if legacy.exists() {
        println!("Migrating legacy save to engine format...");
        let result = load_legacy(world)?;
        // Auto-migrate: save in new format
        let (seed, ref loaded) = result;
        if !loaded.is_empty() {
            if let Err(e) = save_world(world, loaded, seed) {
                println!("Auto-migration save failed: {}", e);
            } else {
                println!("Legacy save migrated successfully ({} chunks)", loaded.len());
            }
        }
        return Ok(result);
    }

    // No save file at all
    Ok((42, HashSet::new()))
}

/// Load from engine SaveManager + WorldStorage format.
fn load_engine_save(
    world: &mut VoxelWorld,
    mgr: &SaveManager,
    info: &anvilkit_core::persistence::SaveSlotInfo,
) -> io::Result<(u32, HashSet<(i32, i32)>)> {
    let seed: u32 = info.metadata
        .get("seed")
        .and_then(|s| s.parse().ok())
        .unwrap_or(42);

    let data_path = mgr.slot_data_path(DEFAULT_SLOT);
    let storage = WorldStorage::open(&data_path)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    let chunk_keys = storage.keys_with_prefix("chunk/");
    let block_count = CHUNK_SIZE * CHUNK_HEIGHT * CHUNK_SIZE;
    let mut loaded_keys = HashSet::new();

    for key in &chunk_keys {
        // Parse key "chunk/{cx}/{cz}"
        let parts: Vec<&str> = key.split('/').collect();
        if parts.len() != 3 {
            continue;
        }
        let cx: i32 = match parts[1].parse() {
            Ok(v) => v,
            Err(_) => continue,
        };
        let cz: i32 = match parts[2].parse() {
            Ok(v) => v,
            Err(_) => continue,
        };

        if let Some(rle_data) = storage.get(key) {
            let blocks = rle_decompress(&rle_data, block_count)?;
            let mut chunk = ChunkData::new();
            chunk.blocks[..].copy_from_slice(&blocks);
            // Compute lighting for loaded chunk
            let mut light = crate::lighting::LightMap::new();
            crate::lighting::compute_initial_sky_light(&chunk, &mut light);
            crate::lighting::compute_block_light(&chunk, &mut light);
            world.chunks.insert((cx, cz), chunk);
            world.light_maps.insert((cx, cz), light);
            loaded_keys.insert((cx, cz));
        }
    }

    Ok((seed, loaded_keys))
}

// --- Legacy binary format loader (backward compat) ---

const LEGACY_MAGIC: &[u8; 8] = b"CRFTSAVE";
const LEGACY_VERSION: u32 = 1;

/// Load from old binary format.
fn load_legacy(world: &mut VoxelWorld) -> io::Result<(u32, HashSet<(i32, i32)>)> {
    let path = legacy_save_path();
    let data = std::fs::read(&path)?;
    if data.len() < 20 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Save file too small"));
    }

    let mut cursor = Cursor::new(&data);

    // Magic
    let mut magic = [0u8; 8];
    cursor.read_exact(&mut magic)?;
    if &magic != LEGACY_MAGIC {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid save file magic"));
    }

    // Version
    let mut version_bytes = [0u8; 4];
    cursor.read_exact(&mut version_bytes)?;
    let version = u32::from_le_bytes(version_bytes);
    if version != LEGACY_VERSION {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Unsupported legacy save version {}", version),
        ));
    }

    // Seed
    let mut seed_bytes = [0u8; 4];
    cursor.read_exact(&mut seed_bytes)?;
    let seed = u32::from_le_bytes(seed_bytes);

    // Chunk count
    let mut count_bytes = [0u8; 4];
    cursor.read_exact(&mut count_bytes)?;
    let chunk_count = u32::from_le_bytes(count_bytes) as usize;

    let block_count = CHUNK_SIZE * CHUNK_HEIGHT * CHUNK_SIZE;
    let mut loaded_keys = HashSet::new();

    for _ in 0..chunk_count {
        let mut cx_bytes = [0u8; 4];
        cursor.read_exact(&mut cx_bytes)?;
        let cx = i32::from_le_bytes(cx_bytes);

        let mut cz_bytes = [0u8; 4];
        cursor.read_exact(&mut cz_bytes)?;
        let cz = i32::from_le_bytes(cz_bytes);

        let mut rle_len_bytes = [0u8; 4];
        cursor.read_exact(&mut rle_len_bytes)?;
        let rle_len = u32::from_le_bytes(rle_len_bytes) as usize;

        let pos = cursor.position() as usize;
        if pos + rle_len > data.len() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Truncated chunk data"));
        }
        let rle_data = &data[pos..pos + rle_len];
        cursor.set_position((pos + rle_len) as u64);

        let blocks = rle_decompress(rle_data, block_count)?;
        let mut chunk = ChunkData::new();
        chunk.blocks[..].copy_from_slice(&blocks);
        let mut light = crate::lighting::LightMap::new();
        crate::lighting::compute_initial_sky_light(&chunk, &mut light);
        crate::lighting::compute_block_light(&chunk, &mut light);
        world.chunks.insert((cx, cz), chunk);
        world.light_maps.insert((cx, cz), light);
        loaded_keys.insert((cx, cz));
    }

    Ok((seed, loaded_keys))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::BlockType;

    #[test]
    fn rle_roundtrip() {
        let mut original = vec![0u8; 1000];
        for i in 0..100 {
            original[i] = BlockType::Stone as u8;
        }
        for i in 100..500 {
            original[i] = BlockType::Dirt as u8;
        }
        let compressed = rle_compress(&original);
        let decompressed = rle_decompress(&compressed, original.len()).unwrap();
        assert_eq!(original, decompressed);
        assert!(compressed.len() < original.len());
    }

    #[test]
    fn rle_single_block() {
        let data = vec![5u8; 10000];
        let compressed = rle_compress(&data);
        assert_eq!(compressed.len(), 3);
        let decompressed = rle_decompress(&compressed, 10000).unwrap();
        assert_eq!(data, decompressed);
    }
}
