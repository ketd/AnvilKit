use serde::Deserialize;
use bevy_ecs::prelude::Resource;
use anvilkit_data::DataTable;

/// Block types matching Craft's item definitions.
///
/// Tile indices reference the 16×16 texture atlas (256×256 pixels, 16px per tile).
/// Row 0 is at the top of the atlas image.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockType {
    Air = 0,
    Grass = 1,
    Sand = 2,
    Stone = 3,
    Brick = 4,
    Wood = 5,
    Cement = 6,
    Dirt = 7,
    Plank = 8,
    Snow = 9,
    Glass = 10,
    Cobble = 11,
    LightStone = 12,
    DarkStone = 13,
    Chest = 14,
    Leaves = 15,
    Cloud = 16,
    TallGrass = 17,
    YellowFlower = 18,
    RedFlower = 19,
    Purple = 20,
    Sun = 21,
    Water = 22,
}

impl BlockType {
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::Grass,
            2 => Self::Sand,
            3 => Self::Stone,
            4 => Self::Brick,
            5 => Self::Wood,
            6 => Self::Cement,
            7 => Self::Dirt,
            8 => Self::Plank,
            9 => Self::Snow,
            10 => Self::Glass,
            11 => Self::Cobble,
            12 => Self::LightStone,
            13 => Self::DarkStone,
            14 => Self::Chest,
            15 => Self::Leaves,
            16 => Self::Cloud,
            17 => Self::TallGrass,
            18 => Self::YellowFlower,
            19 => Self::RedFlower,
            20 => Self::Purple,
            21 => Self::Sun,
            22 => Self::Water,
            _ => Self::Air,
        }
    }

    pub fn is_transparent(self) -> bool {
        matches!(
            self,
            Self::Air
                | Self::Glass
                | Self::Leaves
                | Self::TallGrass
                | Self::YellowFlower
                | Self::RedFlower
                | Self::Cloud
                | Self::Water
        )
    }

    pub fn is_plant(self) -> bool {
        matches!(self, Self::TallGrass | Self::YellowFlower | Self::RedFlower)
    }

    pub fn is_water(self) -> bool {
        matches!(self, Self::Water)
    }

    pub fn is_obstacle(self) -> bool {
        !matches!(
            self,
            Self::Air | Self::TallGrass | Self::YellowFlower | Self::RedFlower | Self::Cloud | Self::Water
        )
    }

    /// Returns flat tile index for each face.
    ///
    /// The actual Craft texture.png atlas layout (verified by pixel inspection):
    ///   Row 15: block faces (wood=240, sand=241, stone=242, brick=243, plank=244,
    ///           cement=245, dirt=246, wood_planks=247, cobble=250, light_stone=251,
    ///           dark_stone=252, chest=253, leaves=254, cloud=255)
    ///   Row 14: grass_side=224, snow_side=232
    ///   Row 13: grass_top=208, wood_top=212, snow_top=216
    ///   Row 12: plants (tallgrass=192, yellow_flower=193, etc.)
    ///
    /// Tile index → (col, row) = (index % 16, index / 16)
    pub fn face_tile(self, face: Face) -> u8 {
        // [left, right, top, bottom, front, back]
        let tiles: [u8; 6] = match self {
            Self::Grass      => [224, 224, 208, 246, 224, 224], // side=grass_side, top=grass_top, bottom=dirt
            Self::Sand       => [241; 6],
            Self::Stone      => [242; 6],
            Self::Brick      => [243; 6],
            Self::Wood       => [240, 240, 212, 212, 240, 240], // side=wood_log, top/bottom=wood_top
            Self::Cement     => [245; 6],
            Self::Dirt       => [246; 6],
            Self::Plank      => [244; 6],
            Self::Snow       => [232, 232, 216, 246, 232, 232], // side=snow_side, top=snow_top, bottom=dirt
            Self::Glass      => [17; 6],  // transparent lighter tile in row 1
            Self::Cobble     => [250; 6],
            Self::LightStone => [251; 6],
            Self::DarkStone  => [252; 6],
            Self::Chest      => [253; 6],
            Self::Leaves     => [254; 6],
            Self::Cloud      => [255; 6],
            Self::TallGrass    => [192; 6],
            Self::YellowFlower => [193; 6],
            Self::RedFlower    => [194; 6],
            Self::Purple       => [195; 6],
            Self::Sun          => [196; 6],
            Self::Water        => [205; 6],
            Self::Air => [0; 6],
        };
        let idx = match face {
            Face::Left   => 0,
            Face::Right  => 1,
            Face::Top    => 2,
            Face::Bottom => 3,
            Face::Front  => 4,
            Face::Back   => 5,
        };
        tiles[idx]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Face {
    Top,    // +Y
    Bottom, // -Y
    Right,  // +X
    Left,   // -X
    Front,  // +Z
    Back,   // -Z
}

impl Face {
    pub fn normal(self) -> [f32; 3] {
        match self {
            Self::Top => [0.0, 1.0, 0.0],
            Self::Bottom => [0.0, -1.0, 0.0],
            Self::Right => [1.0, 0.0, 0.0],
            Self::Left => [-1.0, 0.0, 0.0],
            Self::Front => [0.0, 0.0, 1.0],
            Self::Back => [0.0, 0.0, -1.0],
        }
    }
}

/// Size of one tile in UV space.
pub const TILE_UV: f32 = 1.0 / 16.0;
/// Small inset to prevent texture bleeding at tile edges.
pub const TILE_INSET: f32 = 1.0 / 2048.0;

/// Convert flat tile index to UV min corner (col, row) = (index % 16, index / 16).
pub fn tile_uv(tile: u8) -> (f32, f32) {
    let col = (tile % 16) as f32;
    let row = (tile / 16) as f32;
    (col * TILE_UV, row * TILE_UV)
}

impl BlockType {
    /// Convert block type to item ID for inventory.
    pub fn item_id(self) -> u32 {
        self as u32
    }

    /// Convert item ID back to block type.
    pub fn from_item_id(id: u32) -> Self {
        Self::from_u8(id as u8)
    }

    /// Locale key for this block type (matches blocks.ron name field).
    pub fn locale_key(self) -> &'static str {
        match self {
            Self::Air => "block.air",
            Self::Grass => "block.grass",
            Self::Sand => "block.sand",
            Self::Stone => "block.stone",
            Self::Brick => "block.brick",
            Self::Wood => "block.wood",
            Self::Cement => "block.cement",
            Self::Dirt => "block.dirt",
            Self::Plank => "block.plank",
            Self::Snow => "block.snow",
            Self::Glass => "block.glass",
            Self::Cobble => "block.cobble",
            Self::LightStone => "block.light_stone",
            Self::DarkStone => "block.dark_stone",
            Self::Chest => "block.chest",
            Self::Leaves => "block.leaves",
            Self::Cloud => "block.cloud",
            Self::TallGrass => "block.tall_grass",
            Self::YellowFlower => "block.yellow_flower",
            Self::RedFlower => "block.red_flower",
            Self::Purple => "block.purple",
            Self::Sun => "block.sun",
            Self::Water => "block.water",
        }
    }
}

/// Data-driven block property definition, loaded from blocks.ron.
#[derive(Debug, Clone, Deserialize)]
pub struct BlockDef {
    pub name: String,
    pub transparent: bool,
    pub plant: bool,
    pub obstacle: bool,
    pub water: bool,
    pub face_tiles: [u8; 6],
}

/// Type alias for the block data table.
pub type BlockTable = DataTable<u8, BlockDef>;

/// Fast O(1) lookup cache for block properties, indexed by block ID (u8).
/// Built from BlockTable at startup, used by mesh builder in tight loops.
#[derive(Resource)]
pub struct BlockDefCache {
    entries: [Option<BlockDef>; 256],
}

impl BlockDefCache {
    pub fn from_table(table: &BlockTable) -> Self {
        let mut entries: [Option<BlockDef>; 256] = std::array::from_fn(|_| None);
        for (&id, def) in table.iter() {
            entries[id as usize] = Some(def.clone());
        }
        Self { entries }
    }

    pub fn get(&self, block_id: u8) -> Option<&BlockDef> {
        self.entries[block_id as usize].as_ref()
    }

    pub fn is_transparent(&self, block_id: u8) -> bool {
        self.entries[block_id as usize]
            .as_ref()
            .map_or(block_id == 0, |d| d.transparent)
    }

    pub fn is_plant(&self, block_id: u8) -> bool {
        self.entries[block_id as usize]
            .as_ref()
            .map_or(false, |d| d.plant)
    }

    pub fn is_obstacle(&self, block_id: u8) -> bool {
        self.entries[block_id as usize]
            .as_ref()
            .map_or(false, |d| d.obstacle)
    }

    pub fn is_water(&self, block_id: u8) -> bool {
        self.entries[block_id as usize]
            .as_ref()
            .map_or(false, |d| d.water)
    }

    pub fn face_tile(&self, block_id: u8, face_idx: usize) -> u8 {
        self.entries[block_id as usize]
            .as_ref()
            .map_or(0, |d| d.face_tiles[face_idx.min(5)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_type_roundtrip() {
        for i in 0..=22u8 {
            let b = BlockType::from_u8(i);
            assert_eq!(b as u8, i);
        }
    }

    #[test]
    fn grass_faces() {
        let top = BlockType::Grass.face_tile(Face::Top);
        let bot = BlockType::Grass.face_tile(Face::Bottom);
        let side = BlockType::Grass.face_tile(Face::Right);
        assert_eq!(top, 208);  // grass_top
        assert_eq!(bot, 246);  // dirt
        assert_eq!(side, 224); // grass_side
        assert_ne!(top, bot);
        assert_ne!(top, side);
    }
}
