use crate::block::BlockType;

pub const CHUNK_SIZE: usize = 32;
pub const CHUNK_HEIGHT: usize = 256;

/// Flat array storage for a 32×256×32 chunk.
/// Index order: x + z * CHUNK_SIZE + y * CHUNK_SIZE * CHUNK_SIZE
pub struct ChunkData {
    pub blocks: Box<[u8; CHUNK_SIZE * CHUNK_HEIGHT * CHUNK_SIZE]>,
}

impl ChunkData {
    pub fn new() -> Self {
        Self {
            blocks: Box::new([0u8; CHUNK_SIZE * CHUNK_HEIGHT * CHUNK_SIZE]),
        }
    }

    #[inline]
    fn index(x: usize, y: usize, z: usize) -> usize {
        x + z * CHUNK_SIZE + y * CHUNK_SIZE * CHUNK_SIZE
    }

    #[inline]
    pub fn get(&self, x: usize, y: usize, z: usize) -> BlockType {
        BlockType::from_u8(self.blocks[Self::index(x, y, z)])
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, z: usize, block: BlockType) {
        self.blocks[Self::index(x, y, z)] = block as u8;
    }

    /// Safe get that returns Air for out-of-bounds coordinates.
    #[inline]
    pub fn get_safe(&self, x: i32, y: i32, z: i32) -> BlockType {
        if x < 0 || x >= CHUNK_SIZE as i32 || y < 0 || y >= CHUNK_HEIGHT as i32 || z < 0 || z >= CHUNK_SIZE as i32 {
            BlockType::Air
        } else {
            self.get(x as usize, y as usize, z as usize)
        }
    }
}

impl Default for ChunkData {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_get() {
        let mut chunk = ChunkData::new();
        chunk.set(5, 10, 3, BlockType::Stone);
        assert_eq!(chunk.get(5, 10, 3), BlockType::Stone);
        assert_eq!(chunk.get(0, 0, 0), BlockType::Air);
    }

    #[test]
    fn get_safe_oob() {
        let chunk = ChunkData::new();
        assert_eq!(chunk.get_safe(-1, 0, 0), BlockType::Air);
        assert_eq!(chunk.get_safe(0, -1, 0), BlockType::Air);
        assert_eq!(chunk.get_safe(0, 0, 32), BlockType::Air);
    }
}
