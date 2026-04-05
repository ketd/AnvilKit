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
    // --- Phase 0: New blocks ---
    CoalOre = 23,
    IronOre = 24,
    GoldOre = 25,
    DiamondOre = 26,
    RedstoneOre = 27,
    LapisOre = 28,
    Torch = 29,
    Glowstone = 30,
    Lantern = 31,
    Workbench = 32,
    Furnace = 33,
    Sandstone = 34,
    Gravel = 35,
    Cactus = 36,
    BirchWood = 37,
    BirchLeaves = 38,
    SpruceWood = 39,
    SpruceLeaves = 40,
    SnowBlock = 41,
    Ice = 42,
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
            23 => Self::CoalOre,
            24 => Self::IronOre,
            25 => Self::GoldOre,
            26 => Self::DiamondOre,
            27 => Self::RedstoneOre,
            28 => Self::LapisOre,
            29 => Self::Torch,
            30 => Self::Glowstone,
            31 => Self::Lantern,
            32 => Self::Workbench,
            33 => Self::Furnace,
            34 => Self::Sandstone,
            35 => Self::Gravel,
            36 => Self::Cactus,
            37 => Self::BirchWood,
            38 => Self::BirchLeaves,
            39 => Self::SpruceWood,
            40 => Self::SpruceLeaves,
            41 => Self::SnowBlock,
            42 => Self::Ice,
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
                | Self::Torch
                | Self::Lantern
                | Self::Cactus
                | Self::BirchLeaves
                | Self::SpruceLeaves
                | Self::Ice
        )
    }

    pub fn is_plant(self) -> bool {
        matches!(self, Self::TallGrass | Self::YellowFlower | Self::RedFlower
            | Self::Cactus | Self::Torch | Self::Lantern)
    }

    /// Torch-shaped blocks use a thin 3D cuboid model instead of a cross billboard.
    pub fn is_torch_shape(self) -> bool {
        matches!(self, Self::Torch | Self::Lantern)
    }

    pub fn is_water(self) -> bool {
        matches!(self, Self::Water)
    }

    pub fn is_obstacle(self) -> bool {
        !matches!(
            self,
            Self::Air | Self::TallGrass | Self::YellowFlower | Self::RedFlower
                | Self::Cloud | Self::Water | Self::Torch | Self::Lantern | Self::Cactus
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
    pub fn face_tile(self, face: Face) -> u16 {
        // [left, right, top, bottom, front, back]
        // Tile indices use 32-col encoding: idx = col + row * 32
        // Original tiles (left half of atlas): old_col + old_row * 32
        let tiles: [u16; 6] = match self {
            Self::Grass      => [448, 448, 416, 486, 448, 448], // grass_side, grass_top, dirt
            Self::Sand       => [481; 6],
            Self::Stone      => [482; 6],
            Self::Brick      => [483; 6],
            Self::Wood       => [480, 480, 420, 420, 480, 480], // wood_log, wood_top
            Self::Cement     => [485; 6],
            Self::Dirt       => [486; 6],
            Self::Plank      => [484; 6],
            Self::Snow       => [456, 456, 424, 486, 456, 456], // snow_side, snow_top, dirt
            Self::Glass      => [65; 6],  // old 33 → (33%16) + (33/16)*32 = 1+64 = 65
            Self::Cobble     => [490; 6],
            Self::LightStone => [491; 6],
            Self::DarkStone  => [492; 6],
            Self::Chest      => [493; 6],
            Self::Leaves     => [494; 6],
            Self::Cloud      => [495; 6],
            Self::TallGrass    => [384; 6],
            Self::YellowFlower => [385; 6],
            Self::RedFlower    => [386; 6],
            Self::Purple       => [387; 6],
            Self::Sun          => [388; 6],
            Self::Water        => [397; 6],
            // New blocks — Minetest textures (tile = col + row*32, atlas 32x16)
            Self::CoalOre      => [48; 6],                     // mt_coal_ore
            Self::IronOre      => [49; 6],                     // mt_iron_ore
            Self::GoldOre      => [50; 6],                     // mt_gold_ore
            Self::DiamondOre   => [51; 6],                     // mt_diamond_ore
            Self::RedstoneOre  => [48; 6],                     // reuse coal_ore (no redstone texture yet)
            Self::LapisOre     => [49; 6],                     // reuse iron_ore
            Self::Torch        => [118; 6],                    // mt_torch
            Self::Glowstone    => [118; 6],                    // reuse torch
            Self::Lantern      => [118; 6],                    // reuse torch
            Self::Workbench    => [82, 82, 81, 82, 82, 82],   // mt_oak_planks sides, oak_log_top top
            Self::Furnace      => [113, 113, 114, 113, 112, 113], // side/side/top/side/front/side
            Self::Sandstone    => [22; 6],                     // mt_sandstone
            Self::Gravel       => [23; 6],                     // mt_gravel
            Self::Cactus       => [119, 119, 120, 120, 119, 119], // cactus side/top
            Self::BirchWood    => [84, 84, 85, 85, 84, 84],   // birch log side/top
            Self::BirchLeaves  => [87; 6],                     // mt_birch_leaves
            Self::SpruceWood   => [88, 88, 89, 89, 88, 88],   // spruce log side/top
            Self::SpruceLeaves => [83; 6],                     // reuse oak_leaves (tinted)
            Self::SnowBlock    => [26; 6],                     // mt_snow_top
            Self::Ice          => [28; 6],                     // mt_ice
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

/// Atlas grid dimensions (32 columns x 16 rows after expansion).
pub const ATLAS_COLS: u32 = 32;
pub const ATLAS_ROWS: u32 = 16;
/// Size of one tile in UV space (per axis).
pub const TILE_UV_X: f32 = 1.0 / ATLAS_COLS as f32; // 1/32
pub const TILE_UV_Y: f32 = 1.0 / ATLAS_ROWS as f32; // 1/16
/// Legacy alias — used where only one dimension matters (square grids).
pub const TILE_UV: f32 = TILE_UV_Y; // 1/16 for backward compat
/// Small inset to prevent texture bleeding at tile edges.
pub const TILE_INSET: f32 = 1.0 / 4096.0;

/// Convert flat tile index to UV min corner.
/// Layout: 32 columns x 16 rows. index = col + row * 32.
pub fn tile_uv(tile: u16) -> (f32, f32) {
    let col = (tile % ATLAS_COLS as u16) as f32;
    let row = (tile / ATLAS_COLS as u16) as f32;
    (col * TILE_UV_X, row * TILE_UV_Y)
}

impl BlockType {
    /// Approximate preview color (linear RGB) for HUD display.
    /// These are rough averages of each block's dominant texture color.
    pub fn preview_color(self) -> [f32; 3] {
        match self {
            Self::Air        => [0.0, 0.0, 0.0],
            Self::Grass      => [0.36, 0.55, 0.18],
            Self::Sand       => [0.85, 0.78, 0.55],
            Self::Stone      => [0.50, 0.50, 0.50],
            Self::Brick      => [0.60, 0.30, 0.25],
            Self::Wood       => [0.55, 0.37, 0.15],
            Self::Cement     => [0.65, 0.65, 0.65],
            Self::Dirt       => [0.50, 0.35, 0.20],
            Self::Plank      => [0.72, 0.56, 0.30],
            Self::Snow       => [0.92, 0.95, 0.98],
            Self::Glass      => [0.75, 0.85, 0.95],
            Self::Cobble     => [0.45, 0.45, 0.45],
            Self::LightStone => [0.80, 0.78, 0.70],
            Self::DarkStone  => [0.30, 0.30, 0.30],
            Self::Chest      => [0.55, 0.40, 0.20],
            Self::Leaves     => [0.20, 0.50, 0.12],
            Self::Cloud      => [0.95, 0.95, 0.98],
            Self::TallGrass  => [0.30, 0.60, 0.15],
            Self::YellowFlower => [0.90, 0.85, 0.15],
            Self::RedFlower  => [0.85, 0.15, 0.15],
            Self::Purple     => [0.55, 0.20, 0.70],
            Self::Sun        => [0.95, 0.90, 0.30],
            Self::Water      => [0.15, 0.40, 0.70],
            Self::CoalOre    => [0.35, 0.35, 0.35],
            Self::IronOre    => [0.60, 0.55, 0.50],
            Self::GoldOre    => [0.80, 0.70, 0.30],
            Self::DiamondOre => [0.40, 0.75, 0.80],
            Self::RedstoneOre => [0.60, 0.20, 0.20],
            Self::LapisOre   => [0.20, 0.30, 0.65],
            Self::Torch      => [0.90, 0.75, 0.25],
            Self::Glowstone  => [0.90, 0.80, 0.40],
            Self::Lantern    => [0.85, 0.70, 0.30],
            Self::Workbench  => [0.60, 0.45, 0.25],
            Self::Furnace    => [0.55, 0.55, 0.55],
            Self::Sandstone  => [0.85, 0.80, 0.60],
            Self::Gravel     => [0.55, 0.52, 0.50],
            Self::Cactus     => [0.20, 0.50, 0.15],
            Self::BirchWood  => [0.82, 0.80, 0.75],
            Self::BirchLeaves => [0.35, 0.60, 0.25],
            Self::SpruceWood => [0.35, 0.22, 0.10],
            Self::SpruceLeaves => [0.15, 0.35, 0.15],
            Self::SnowBlock  => [0.95, 0.97, 1.0],
            Self::Ice        => [0.65, 0.80, 0.95],
        }
    }

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
            Self::CoalOre => "block.coal_ore",
            Self::IronOre => "block.iron_ore",
            Self::GoldOre => "block.gold_ore",
            Self::DiamondOre => "block.diamond_ore",
            Self::RedstoneOre => "block.redstone_ore",
            Self::LapisOre => "block.lapis_ore",
            Self::Torch => "block.torch",
            Self::Glowstone => "block.glowstone",
            Self::Lantern => "block.lantern",
            Self::Workbench => "block.workbench",
            Self::Furnace => "block.furnace",
            Self::Sandstone => "block.sandstone",
            Self::Gravel => "block.gravel",
            Self::Cactus => "block.cactus",
            Self::BirchWood => "block.birch_wood",
            Self::BirchLeaves => "block.birch_leaves",
            Self::SpruceWood => "block.spruce_wood",
            Self::SpruceLeaves => "block.spruce_leaves",
            Self::SnowBlock => "block.snow_block",
            Self::Ice => "block.ice",
        }
    }
}

fn default_hardness() -> f32 { 1.0 }

/// Data-driven block property definition, loaded from blocks.ron.
#[derive(Debug, Clone, Deserialize)]
pub struct BlockDef {
    pub name: String,
    pub transparent: bool,
    pub plant: bool,
    pub obstacle: bool,
    pub water: bool,
    pub face_tiles: [u16; 6],
    /// Mining time multiplier (1.0 = normal, higher = harder). Default: 1.0
    #[serde(default = "default_hardness")]
    pub hardness: f32,
    /// Light emission level 0-15 (0 = no light). Default: 0
    #[serde(default)]
    pub light_emission: u8,
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

    pub fn face_tile(&self, block_id: u8, face_idx: usize) -> u16 {
        self.entries[block_id as usize]
            .as_ref()
            .map_or(0, |d| d.face_tiles[face_idx.min(5)])
    }

    pub fn hardness(&self, block_id: u8) -> f32 {
        self.entries[block_id as usize]
            .as_ref()
            .map_or(1.0, |d| d.hardness)
    }

    pub fn light_emission(&self, block_id: u8) -> u8 {
        self.entries[block_id as usize]
            .as_ref()
            .map_or(0, |d| d.light_emission)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_type_roundtrip() {
        for i in 0..=42u8 {
            let b = BlockType::from_u8(i);
            assert_eq!(b as u8, i);
        }
    }

    #[test]
    fn grass_faces() {
        let top = BlockType::Grass.face_tile(Face::Top);
        let bot = BlockType::Grass.face_tile(Face::Bottom);
        let side = BlockType::Grass.face_tile(Face::Right);
        assert_eq!(top, 416);  // grass_top (32-col: col=0, row=13)
        assert_eq!(bot, 486);  // dirt (32-col: col=6, row=15)
        assert_eq!(side, 448); // grass_side (32-col: col=0, row=14)
        assert_ne!(top, bot);
        assert_ne!(top, side);
    }
}
