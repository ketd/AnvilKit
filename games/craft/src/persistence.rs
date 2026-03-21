use std::collections::HashSet;
use std::fs;
use std::io::{self, Read, Cursor};
use std::path::PathBuf;

use crate::chunk::{ChunkData, CHUNK_SIZE, CHUNK_HEIGHT};
use crate::resources::VoxelWorld;

const MAGIC: &[u8; 8] = b"CRFTSAVE";
const VERSION: u32 = 1;

/// Get the save file path (next to the executable).
pub fn save_path() -> PathBuf {
    let mut path = std::env::current_dir().unwrap_or_default();
    path.push("craft_world.sav");
    path
}

/// RLE-compress chunk block data.
/// Format: repeated [block_type: u8, count: u16] pairs.
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

/// Save modified chunks to disk.
/// Format: [magic:8][version:4][seed:4][chunk_count:4] + per chunk [cx:4][cz:4][rle_len:4][rle_data]
pub fn save_world(
    world: &VoxelWorld,
    modified_chunks: &HashSet<(i32, i32)>,
    seed: u32,
) -> io::Result<usize> {
    if modified_chunks.is_empty() {
        return Ok(0);
    }

    let path = save_path();
    let mut buf = Vec::new();

    // Header
    buf.extend_from_slice(MAGIC);
    buf.extend_from_slice(&VERSION.to_le_bytes());
    buf.extend_from_slice(&seed.to_le_bytes());

    // Count only chunks that actually exist in the world
    let chunks_to_save: Vec<(i32, i32)> = modified_chunks
        .iter()
        .copied()
        .filter(|k| world.chunks.contains_key(k))
        .collect();
    buf.extend_from_slice(&(chunks_to_save.len() as u32).to_le_bytes());

    for (cx, cz) in &chunks_to_save {
        buf.extend_from_slice(&cx.to_le_bytes());
        buf.extend_from_slice(&cz.to_le_bytes());
        let chunk = &world.chunks[&(*cx, *cz)];
        let rle = rle_compress(&chunk.blocks[..]);
        buf.extend_from_slice(&(rle.len() as u32).to_le_bytes());
        buf.extend_from_slice(&rle);
    }

    fs::write(&path, &buf)?;
    Ok(chunks_to_save.len())
}

/// Load modified chunks from disk. Returns seed and the set of loaded chunk keys.
pub fn load_world(world: &mut VoxelWorld) -> io::Result<(u32, HashSet<(i32, i32)>)> {
    let path = save_path();
    if !path.exists() {
        return Ok((42, HashSet::new()));
    }

    let data = fs::read(&path)?;
    if data.len() < 20 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Save file too small"));
    }

    let mut cursor = Cursor::new(&data);

    // Magic
    let mut magic = [0u8; 8];
    cursor.read_exact(&mut magic)?;
    if &magic != MAGIC {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid save file magic"));
    }

    // Version
    let mut version_bytes = [0u8; 4];
    cursor.read_exact(&mut version_bytes)?;
    let version = u32::from_le_bytes(version_bytes);
    if version != VERSION {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Unsupported save version {}", version),
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
        world.chunks.insert((cx, cz), chunk);
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
        // Fill with patterns
        for i in 0..100 {
            original[i] = BlockType::Stone as u8;
        }
        for i in 100..500 {
            original[i] = BlockType::Dirt as u8;
        }
        let compressed = rle_compress(&original);
        let decompressed = rle_decompress(&compressed, original.len()).unwrap();
        assert_eq!(original, decompressed);
        // Compressed should be much smaller than original
        assert!(compressed.len() < original.len());
    }

    #[test]
    fn rle_single_block() {
        let data = vec![5u8; 10000];
        let compressed = rle_compress(&data);
        assert_eq!(compressed.len(), 3); // 1 byte type + 2 bytes count
        let decompressed = rle_decompress(&compressed, 10000).unwrap();
        assert_eq!(data, decompressed);
    }
}
