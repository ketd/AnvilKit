//! Item registry, tool system, crafting recipes, smelting, hunger.

use bevy_ecs::prelude::*;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Item definitions
// ---------------------------------------------------------------------------

/// Tool material tier — determines mining level and durability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ToolTier {
    Wood,
    Stone,
    Iron,
    Gold,
    Diamond,
}

impl ToolTier {
    pub fn mining_level(self) -> u8 {
        match self {
            Self::Wood => 1,
            Self::Stone => 2,
            Self::Iron => 3,
            Self::Gold => 1, // fast but low level
            Self::Diamond => 4,
        }
    }

    pub fn speed_multiplier(self) -> f32 {
        match self {
            Self::Wood => 2.0,
            Self::Stone => 4.0,
            Self::Iron => 6.0,
            Self::Gold => 12.0,
            Self::Diamond => 8.0,
        }
    }

    pub fn max_durability(self) -> u32 {
        match self {
            Self::Wood => 59,
            Self::Stone => 131,
            Self::Iron => 250,
            Self::Gold => 32,
            Self::Diamond => 1561,
        }
    }
}

/// Tool type — determines which blocks it's effective against.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ToolType {
    Pickaxe,
    Axe,
    Shovel,
    Hoe,
    Sword,
}

/// Category of item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemCategory {
    Block,
    Tool,
    Weapon,
    Armor,
    Food,
    Material,
}

/// Extended item definition for the Craft game.
#[derive(Debug, Clone)]
pub struct CraftItemDef {
    pub id: u32,
    pub name: String,
    pub category: ItemCategory,
    pub max_stack: u32,
    pub tool_type: Option<ToolType>,
    pub tool_tier: Option<ToolTier>,
    pub damage: f32,
    pub armor_value: f32,
    pub food_value: u32,      // hunger points restored
    pub saturation: f32,      // saturation restored
    pub durability: u32,      // 0 = no durability
    /// Block ID this item places (if category == Block).
    pub block_id: Option<u8>,
    /// Required mining level to drop (0 = any tool).
    pub required_mining_level: u8,
}

impl CraftItemDef {
    /// Create a simple block item.
    pub fn block(id: u32, name: &str, block_id: u8) -> Self {
        Self {
            id,
            name: name.to_string(),
            category: ItemCategory::Block,
            max_stack: 64,
            tool_type: None,
            tool_tier: None,
            damage: 0.0,
            armor_value: 0.0,
            food_value: 0,
            saturation: 0.0,
            durability: 0,
            block_id: Some(block_id),
            required_mining_level: 0,
        }
    }

    /// Create a tool item.
    pub fn tool(id: u32, name: &str, tool_type: ToolType, tier: ToolTier) -> Self {
        Self {
            id,
            name: name.to_string(),
            category: ItemCategory::Tool,
            max_stack: 1,
            tool_type: Some(tool_type),
            tool_tier: Some(tier),
            damage: match tool_type {
                ToolType::Sword => 4.0 + tier.mining_level() as f32,
                _ => 1.0 + tier.mining_level() as f32 * 0.5,
            },
            armor_value: 0.0,
            food_value: 0,
            saturation: 0.0,
            durability: tier.max_durability(),
            block_id: None,
            required_mining_level: 0,
        }
    }

    /// Create a food item.
    pub fn food(id: u32, name: &str, food_value: u32, saturation: f32) -> Self {
        Self {
            id,
            name: name.to_string(),
            category: ItemCategory::Food,
            max_stack: 64,
            tool_type: None,
            tool_tier: None,
            damage: 0.0,
            armor_value: 0.0,
            food_value,
            saturation,
            durability: 0,
            block_id: None,
            required_mining_level: 0,
        }
    }

    /// Create a material item.
    pub fn material(id: u32, name: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            category: ItemCategory::Material,
            max_stack: 64,
            tool_type: None,
            tool_tier: None,
            damage: 0.0,
            armor_value: 0.0,
            food_value: 0,
            saturation: 0.0,
            durability: 0,
            block_id: None,
            required_mining_level: 0,
        }
    }
}

/// Global item registry resource.
#[derive(Resource)]
pub struct ItemRegistry {
    items: HashMap<u32, CraftItemDef>,
}

impl ItemRegistry {
    pub fn new() -> Self {
        Self { items: HashMap::new() }
    }

    pub fn register(&mut self, item: CraftItemDef) {
        self.items.insert(item.id, item);
    }

    pub fn get(&self, id: u32) -> Option<&CraftItemDef> {
        self.items.get(&id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&u32, &CraftItemDef)> {
        self.items.iter()
    }
}

impl Default for ItemRegistry {
    fn default() -> Self {
        let mut reg = Self::new();
        register_default_items(&mut reg);
        reg
    }
}

/// Register all built-in items.
fn register_default_items(reg: &mut ItemRegistry) {
    use crate::block::BlockType;

    // Block items (id = block_id for simplicity)
    let block_items = [
        (BlockType::Grass, "Grass"), (BlockType::Dirt, "Dirt"),
        (BlockType::Stone, "Stone"), (BlockType::Sand, "Sand"),
        (BlockType::Cobble, "Cobblestone"), (BlockType::Wood, "Oak Log"),
        (BlockType::Plank, "Oak Planks"), (BlockType::Brick, "Bricks"),
        (BlockType::Glass, "Glass"), (BlockType::Leaves, "Oak Leaves"),
        (BlockType::CoalOre, "Coal Ore"), (BlockType::IronOre, "Iron Ore"),
        (BlockType::GoldOre, "Gold Ore"), (BlockType::DiamondOre, "Diamond Ore"),
        (BlockType::Sandstone, "Sandstone"), (BlockType::Gravel, "Gravel"),
        (BlockType::Torch, "Torch"), (BlockType::Glowstone, "Glowstone"),
        (BlockType::Workbench, "Crafting Table"), (BlockType::Furnace, "Furnace"),
        (BlockType::BirchWood, "Birch Log"), (BlockType::SpruceWood, "Spruce Log"),
        (BlockType::SnowBlock, "Snow Block"), (BlockType::Ice, "Ice"),
    ];
    for (bt, name) in block_items {
        reg.register(CraftItemDef::block(bt as u32, name, bt as u8));
    }

    // Materials (IDs 100+)
    reg.register(CraftItemDef::material(100, "Stick"));
    reg.register(CraftItemDef::material(101, "Coal"));
    reg.register(CraftItemDef::material(102, "Iron Ingot"));
    reg.register(CraftItemDef::material(103, "Gold Ingot"));
    reg.register(CraftItemDef::material(104, "Diamond"));
    reg.register(CraftItemDef::material(105, "Redstone Dust"));
    reg.register(CraftItemDef::material(106, "Lapis Lazuli"));
    reg.register(CraftItemDef::material(107, "Leather"));
    reg.register(CraftItemDef::material(108, "Feather"));
    reg.register(CraftItemDef::material(109, "String"));

    // Tools (IDs 200+)
    let tiers = [
        (ToolTier::Wood, "Wooden", 200),
        (ToolTier::Stone, "Stone", 210),
        (ToolTier::Iron, "Iron", 220),
        (ToolTier::Gold, "Golden", 230),
        (ToolTier::Diamond, "Diamond", 240),
    ];
    for (tier, prefix, base_id) in tiers {
        reg.register(CraftItemDef::tool(base_id, &format!("{prefix} Pickaxe"), ToolType::Pickaxe, tier));
        reg.register(CraftItemDef::tool(base_id + 1, &format!("{prefix} Axe"), ToolType::Axe, tier));
        reg.register(CraftItemDef::tool(base_id + 2, &format!("{prefix} Shovel"), ToolType::Shovel, tier));
        reg.register(CraftItemDef::tool(base_id + 3, &format!("{prefix} Hoe"), ToolType::Hoe, tier));
        reg.register(CraftItemDef::tool(base_id + 4, &format!("{prefix} Sword"), ToolType::Sword, tier));
    }

    // Food (IDs 300+)
    reg.register(CraftItemDef::food(300, "Raw Porkchop", 3, 1.8));
    reg.register(CraftItemDef::food(301, "Cooked Porkchop", 8, 12.8));
    reg.register(CraftItemDef::food(302, "Raw Beef", 3, 1.8));
    reg.register(CraftItemDef::food(303, "Steak", 8, 12.8));
    reg.register(CraftItemDef::food(304, "Raw Chicken", 2, 1.2));
    reg.register(CraftItemDef::food(305, "Cooked Chicken", 6, 7.2));
    reg.register(CraftItemDef::food(306, "Bread", 5, 6.0));
    reg.register(CraftItemDef::food(307, "Apple", 4, 2.4));
}

// ---------------------------------------------------------------------------
// Crafting recipes
// ---------------------------------------------------------------------------

/// A crafting recipe (shaped or shapeless).
#[derive(Debug, Clone)]
pub struct CraftingRecipe {
    /// None = shapeless. Some = shaped grid (row-major, 0 = empty).
    pub grid: Option<Vec<Vec<u32>>>,
    /// Shapeless ingredients (item_id list). Empty if shaped.
    pub shapeless: Vec<u32>,
    /// Output item_id.
    pub output_id: u32,
    /// Output count.
    pub output_count: u32,
}

/// All registered recipes.
#[derive(Resource)]
pub struct RecipeRegistry {
    pub recipes: Vec<CraftingRecipe>,
}

impl Default for RecipeRegistry {
    fn default() -> Self {
        let mut reg = Self { recipes: Vec::new() };
        register_default_recipes(&mut reg);
        reg
    }
}

impl RecipeRegistry {
    /// Find a matching recipe for the given crafting grid (3x3 or 2x2).
    /// Grid is row-major, 0 = empty slot.
    pub fn find_match(&self, grid: &[Vec<u32>]) -> Option<(u32, u32)> {
        'recipe: for recipe in &self.recipes {
            if let Some(ref pattern) = recipe.grid {
                // Shaped match: try all offsets
                if shaped_match(pattern, grid) {
                    return Some((recipe.output_id, recipe.output_count));
                }
            } else {
                // Shapeless: check all ingredients present
                let mut remaining: Vec<u32> = recipe.shapeless.clone();
                for row in grid {
                    for &cell in row {
                        if cell == 0 { continue; }
                        if let Some(pos) = remaining.iter().position(|&r| r == cell) {
                            remaining.remove(pos);
                        } else {
                            continue 'recipe;
                        }
                    }
                }
                if remaining.is_empty() {
                    return Some((recipe.output_id, recipe.output_count));
                }
            }
        }
        None
    }
}

/// Check if a shaped pattern matches anywhere in the grid.
fn shaped_match(pattern: &[Vec<u32>], grid: &[Vec<u32>]) -> bool {
    let ph = pattern.len();
    let pw = pattern.iter().map(|r| r.len()).max().unwrap_or(0);
    let gh = grid.len();
    let gw = grid.iter().map(|r| r.len()).max().unwrap_or(0);

    if ph > gh || pw > gw { return false; }

    for oy in 0..=(gh - ph) {
        for ox in 0..=(gw - pw) {
            let mut ok = true;
            // Check pattern cells match
            for py in 0..ph {
                for px in 0..pw {
                    let p = pattern.get(py).and_then(|r| r.get(px)).copied().unwrap_or(0);
                    let g = grid.get(oy + py).and_then(|r| r.get(ox + px)).copied().unwrap_or(0);
                    if p != g { ok = false; break; }
                }
                if !ok { break; }
            }
            if !ok { continue; }
            // Check non-pattern cells are empty
            for gy in 0..gh {
                for gx in 0..gw {
                    let in_pattern = gy >= oy && gy < oy + ph && gx >= ox && gx < ox + pw;
                    if !in_pattern {
                        let g = grid.get(gy).and_then(|r| r.get(gx)).copied().unwrap_or(0);
                        if g != 0 { ok = false; break; }
                    }
                }
                if !ok { break; }
            }
            if ok { return true; }
        }
    }
    false
}

fn register_default_recipes(reg: &mut RecipeRegistry) {
    use crate::block::BlockType;
    let plank = BlockType::Plank as u32;
    let wood = BlockType::Wood as u32;
    let cobble = BlockType::Cobble as u32;
    let stick = 100u32;
    let coal = 101u32;
    let iron = 102u32;
    let gold = 103u32;
    let diamond = 104u32;

    // Wood → 4 Planks
    reg.recipes.push(CraftingRecipe {
        grid: None, shapeless: vec![wood], output_id: plank, output_count: 4,
    });
    // Planks → 4 Sticks (2 planks vertical)
    reg.recipes.push(CraftingRecipe {
        grid: Some(vec![vec![plank], vec![plank]]),
        shapeless: vec![], output_id: stick, output_count: 4,
    });
    // Crafting Table
    reg.recipes.push(CraftingRecipe {
        grid: Some(vec![vec![plank, plank], vec![plank, plank]]),
        shapeless: vec![], output_id: BlockType::Workbench as u32, output_count: 1,
    });
    // Furnace
    reg.recipes.push(CraftingRecipe {
        grid: Some(vec![
            vec![cobble, cobble, cobble],
            vec![cobble, 0,      cobble],
            vec![cobble, cobble, cobble],
        ]),
        shapeless: vec![], output_id: BlockType::Furnace as u32, output_count: 1,
    });
    // Torch
    reg.recipes.push(CraftingRecipe {
        grid: Some(vec![vec![coal], vec![stick]]),
        shapeless: vec![], output_id: BlockType::Torch as u32, output_count: 4,
    });

    // Tool recipes: pickaxe/axe/shovel/sword for each tier
    let tier_materials = [
        (plank, 200), (cobble, 210), (iron, 220), (gold, 230), (diamond, 240),
    ];
    for (mat, base_id) in tier_materials {
        // Pickaxe: MMM / _S_ / _S_
        reg.recipes.push(CraftingRecipe {
            grid: Some(vec![
                vec![mat, mat, mat],
                vec![0, stick, 0],
                vec![0, stick, 0],
            ]),
            shapeless: vec![], output_id: base_id, output_count: 1,
        });
        // Axe: MM_ / MS_ / _S_
        reg.recipes.push(CraftingRecipe {
            grid: Some(vec![
                vec![mat, mat], vec![mat, stick], vec![0, stick],
            ]),
            shapeless: vec![], output_id: base_id + 1, output_count: 1,
        });
        // Shovel: M / S / S
        reg.recipes.push(CraftingRecipe {
            grid: Some(vec![vec![mat], vec![stick], vec![stick]]),
            shapeless: vec![], output_id: base_id + 2, output_count: 1,
        });
        // Sword: M / M / S
        reg.recipes.push(CraftingRecipe {
            grid: Some(vec![vec![mat], vec![mat], vec![stick]]),
            shapeless: vec![], output_id: base_id + 4, output_count: 1,
        });
    }
}

// ---------------------------------------------------------------------------
// Smelting
// ---------------------------------------------------------------------------

/// A smelting recipe: input → output + experience.
#[derive(Debug, Clone)]
pub struct SmeltingRecipe {
    pub input_id: u32,
    pub output_id: u32,
    pub cook_time: f32, // seconds
}

/// All smelting recipes.
#[derive(Resource)]
pub struct SmeltingRegistry {
    pub recipes: Vec<SmeltingRecipe>,
}

impl Default for SmeltingRegistry {
    fn default() -> Self {
        use crate::block::BlockType;
        Self {
            recipes: vec![
                SmeltingRecipe { input_id: BlockType::IronOre as u32, output_id: 102, cook_time: 10.0 },
                SmeltingRecipe { input_id: BlockType::GoldOre as u32, output_id: 103, cook_time: 10.0 },
                SmeltingRecipe { input_id: BlockType::Sand as u32, output_id: BlockType::Glass as u32, cook_time: 10.0 },
                SmeltingRecipe { input_id: BlockType::CoalOre as u32, output_id: 101, cook_time: 10.0 },
                SmeltingRecipe { input_id: BlockType::Cobble as u32, output_id: BlockType::Stone as u32, cook_time: 10.0 },
                // Raw food → cooked
                SmeltingRecipe { input_id: 300, output_id: 301, cook_time: 10.0 }, // porkchop
                SmeltingRecipe { input_id: 302, output_id: 303, cook_time: 10.0 }, // beef
                SmeltingRecipe { input_id: 304, output_id: 305, cook_time: 10.0 }, // chicken
            ],
        }
    }
}

impl SmeltingRegistry {
    pub fn find(&self, input_id: u32) -> Option<&SmeltingRecipe> {
        self.recipes.iter().find(|r| r.input_id == input_id)
    }
}

/// Furnace state for a placed furnace block.
#[derive(Debug, Clone, Component)]
pub struct FurnaceState {
    pub input_id: u32,
    pub input_count: u32,
    pub fuel_remaining: f32,
    pub cook_progress: f32,
    pub output_id: u32,
    pub output_count: u32,
}

impl Default for FurnaceState {
    fn default() -> Self {
        Self {
            input_id: 0, input_count: 0,
            fuel_remaining: 0.0, cook_progress: 0.0,
            output_id: 0, output_count: 0,
        }
    }
}

// ---------------------------------------------------------------------------
// Tool / mining
// ---------------------------------------------------------------------------

/// Block hardness table — maps block_id to (hardness, required_tool, required_level).
pub fn block_mining_info(block_id: u8) -> (f32, Option<ToolType>, u8) {
    use crate::block::BlockType;
    let bt = BlockType::from_u8(block_id);
    match bt {
        BlockType::Stone | BlockType::Cobble | BlockType::Brick => (1.5, Some(ToolType::Pickaxe), 1),
        BlockType::IronOre => (3.0, Some(ToolType::Pickaxe), 2),
        BlockType::GoldOre => (3.0, Some(ToolType::Pickaxe), 3),
        BlockType::DiamondOre => (3.0, Some(ToolType::Pickaxe), 3),
        BlockType::CoalOre | BlockType::LapisOre | BlockType::RedstoneOre => (3.0, Some(ToolType::Pickaxe), 1),
        BlockType::Wood | BlockType::BirchWood | BlockType::SpruceWood | BlockType::Plank => (2.0, Some(ToolType::Axe), 0),
        BlockType::Dirt | BlockType::Grass | BlockType::Sand | BlockType::Gravel => (0.5, Some(ToolType::Shovel), 0),
        BlockType::Leaves | BlockType::BirchLeaves | BlockType::SpruceLeaves => (0.2, None, 0),
        BlockType::Glass | BlockType::Ice => (0.3, None, 0),
        BlockType::Sandstone => (0.8, Some(ToolType::Pickaxe), 1),
        _ => (1.0, None, 0),
    }
}

/// Calculate mining time given block hardness, tool type/tier.
pub fn mining_time(block_id: u8, held_tool: Option<(ToolType, ToolTier)>) -> f32 {
    let (hardness, preferred_tool, required_level) = block_mining_info(block_id);
    if hardness <= 0.0 { return 0.0; }

    let (speed, level) = match held_tool {
        Some((tool_type, tier)) => {
            if preferred_tool == Some(tool_type) {
                (tier.speed_multiplier(), tier.mining_level())
            } else {
                (1.0, tier.mining_level())
            }
        }
        None => (1.0, 0),
    };

    if level < required_level {
        return hardness * 5.0; // wrong tier: very slow and no drop
    }

    hardness / speed
}

// ---------------------------------------------------------------------------
// Tool durability component
// ---------------------------------------------------------------------------

/// Tracks durability for a held tool item.
#[derive(Debug, Clone, Component)]
pub struct Durability {
    pub current: u32,
    pub max: u32,
}

impl Durability {
    pub fn new(max: u32) -> Self {
        Self { current: max, max }
    }

    pub fn use_once(&mut self) -> bool {
        if self.current > 0 {
            self.current -= 1;
            true
        } else {
            false // broken
        }
    }

    pub fn is_broken(&self) -> bool {
        self.current == 0
    }
}

// ---------------------------------------------------------------------------
// Hunger system
// ---------------------------------------------------------------------------

/// Player hunger state.
#[derive(Debug, Clone, Component, Resource)]
pub struct Hunger {
    pub level: u32,        // 0-20
    pub saturation: f32,   // 0-20
    pub exhaustion: f32,   // accumulated, drains saturation then hunger
}

impl Default for Hunger {
    fn default() -> Self {
        Self { level: 20, saturation: 5.0, exhaustion: 0.0 }
    }
}

impl Hunger {
    pub fn add_exhaustion(&mut self, amount: f32) {
        self.exhaustion += amount;
        while self.exhaustion >= 4.0 {
            self.exhaustion -= 4.0;
            if self.saturation > 0.0 {
                self.saturation = (self.saturation - 1.0).max(0.0);
            } else if self.level > 0 {
                self.level -= 1;
            }
        }
    }

    pub fn eat(&mut self, food_value: u32, sat: f32) {
        self.level = (self.level + food_value).min(20);
        self.saturation = (self.saturation + sat).min(self.level as f32);
    }

    pub fn is_starving(&self) -> bool {
        self.level == 0
    }

    pub fn can_regen(&self) -> bool {
        self.level >= 18
    }
}

/// Hunger tick system: starvation damage. Disabled in Creative mode.
pub fn hunger_tick_system(
    mode: Res<crate::world_manager::GameMode>,
    dt: Res<anvilkit_core::time::DeltaTime>,
    hunger_query: Query<(Entity, &Hunger), With<crate::components::FpsCamera>>,
    mut damage_events: EventWriter<anvilkit_gameplay::health::DamageEvent>,
    mut starvation_timer: Local<f32>,
) {
    if *mode == crate::world_manager::GameMode::Creative { return; }
    for (entity, hunger) in &hunger_query {
        if hunger.is_starving() {
            *starvation_timer += dt.0;
            if *starvation_timer >= 4.0 {
                *starvation_timer -= 4.0;
                damage_events.send(anvilkit_gameplay::health::DamageEvent {
                    target: entity,
                    amount: 1.0,
                    source: None,
                });
            }
        } else {
            *starvation_timer = 0.0;
        }
    }
}

// ---------------------------------------------------------------------------
// Equipment (armor slots)
// ---------------------------------------------------------------------------

/// Armor slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArmorSlot {
    Helmet,
    Chestplate,
    Leggings,
    Boots,
}

/// Player equipment: 4 armor slots.
#[derive(Debug, Clone, Default, Component)]
pub struct Equipment {
    pub helmet: Option<u32>,     // item_id
    pub chestplate: Option<u32>,
    pub leggings: Option<u32>,
    pub boots: Option<u32>,
}

impl Equipment {
    /// Total armor points from all equipped pieces.
    pub fn total_armor(&self, registry: &ItemRegistry) -> f32 {
        let slots = [self.helmet, self.chestplate, self.leggings, self.boots];
        slots.iter().filter_map(|s| *s).map(|id| {
            registry.get(id).map_or(0.0, |def| def.armor_value)
        }).sum()
    }

    /// Damage reduction factor: 1.0 - (armor * 0.04), min 0.2.
    pub fn damage_reduction(&self, registry: &ItemRegistry) -> f32 {
        let armor = self.total_armor(registry);
        (1.0 - armor * 0.04).max(0.2)
    }
}

// ---------------------------------------------------------------------------
// Furnace tick system
// ---------------------------------------------------------------------------

/// Tick all active furnaces: consume fuel, advance cooking, produce output.
pub fn furnace_tick_system(
    dt: Res<anvilkit_core::time::DeltaTime>,
    smelting: Res<SmeltingRegistry>,
    mut furnaces: Query<&mut FurnaceState>,
) {
    for mut furnace in &mut furnaces {
        if furnace.input_id == 0 || furnace.fuel_remaining <= 0.0 {
            continue;
        }
        furnace.fuel_remaining -= dt.0;

        if let Some(recipe) = smelting.find(furnace.input_id) {
            furnace.cook_progress += dt.0;
            if furnace.cook_progress >= recipe.cook_time {
                furnace.cook_progress = 0.0;
                furnace.input_count = furnace.input_count.saturating_sub(1);
                if furnace.output_id == recipe.output_id {
                    furnace.output_count += 1;
                } else {
                    furnace.output_id = recipe.output_id;
                    furnace.output_count = 1;
                }
                if furnace.input_count == 0 {
                    furnace.input_id = 0;
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Mining progress
// ---------------------------------------------------------------------------

/// Tracks mining progress on a targeted block.
#[derive(Debug, Clone, Resource)]
pub struct MiningProgress {
    /// World position of the block being mined.
    pub target: Option<[i32; 3]>,
    /// Accumulated mining time.
    pub progress: f32,
    /// Required time to break.
    pub required: f32,
}

impl Default for MiningProgress {
    fn default() -> Self {
        Self { target: None, progress: 0.0, required: 1.0 }
    }
}

impl MiningProgress {
    pub fn fraction(&self) -> f32 {
        if self.required <= 0.0 { return 1.0; }
        (self.progress / self.required).clamp(0.0, 1.0)
    }

    pub fn is_complete(&self) -> bool {
        self.progress >= self.required
    }

    pub fn reset(&mut self) {
        self.target = None;
        self.progress = 0.0;
        self.required = 1.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item_registry_default() {
        let reg = ItemRegistry::default();
        // Should have block items + materials + tools + food
        assert!(reg.get(3).is_some(), "Stone block item"); // BlockType::Stone = 3
        assert!(reg.get(100).is_some(), "Stick material");
        assert!(reg.get(200).is_some(), "Wooden Pickaxe");
        assert!(reg.get(300).is_some(), "Raw Porkchop");
    }

    #[test]
    fn test_recipe_match_shapeless() {
        let reg = RecipeRegistry::default();
        // Wood → 4 Planks (shapeless)
        let grid = vec![vec![5, 0, 0], vec![0, 0, 0], vec![0, 0, 0]]; // Wood=5
        let result = reg.find_match(&grid);
        assert!(result.is_some());
        let (id, count) = result.unwrap();
        assert_eq!(id, 8); // Plank = 8
        assert_eq!(count, 4);
    }

    #[test]
    fn test_recipe_match_shaped() {
        let reg = RecipeRegistry::default();
        // Crafting table: 2x2 planks
        let grid = vec![vec![8, 8, 0], vec![8, 8, 0], vec![0, 0, 0]]; // Plank=8
        let result = reg.find_match(&grid);
        assert!(result.is_some());
        let (id, _count) = result.unwrap();
        assert_eq!(id, 32); // Workbench = 32
    }

    #[test]
    fn test_recipe_no_match() {
        let reg = RecipeRegistry::default();
        let grid = vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]];
        assert!(reg.find_match(&grid).is_none());
    }

    #[test]
    fn test_mining_time() {
        // Stone with wooden pickaxe
        let time = mining_time(3, Some((ToolType::Pickaxe, ToolTier::Wood)));
        assert!(time < 1.0, "Pickaxe should mine stone fast: {}", time);

        // Stone bare-handed
        let time_bare = mining_time(3, None);
        assert!(time_bare > time, "Bare hand should be slower than pickaxe");

        // Dirt with shovel
        let dirt_time = mining_time(7, Some((ToolType::Shovel, ToolTier::Wood)));
        assert!(dirt_time < 0.5, "Shovel on dirt should be fast: {}", dirt_time);
    }

    #[test]
    fn test_durability() {
        let mut d = Durability::new(10);
        assert!(!d.is_broken());
        for _ in 0..10 {
            d.use_once();
        }
        assert!(d.is_broken());
        assert!(!d.use_once()); // can't use broken tool
    }

    #[test]
    fn test_hunger() {
        let mut h = Hunger::default();
        assert_eq!(h.level, 20);

        // Exhaust to drain saturation then hunger
        h.add_exhaustion(20.0); // 5 drains
        assert!(h.saturation < 5.0 || h.level < 20);

        // Eat
        h.level = 10;
        h.saturation = 0.0;
        h.eat(5, 3.0);
        assert_eq!(h.level, 15);
        assert!((h.saturation - 3.0).abs() < 0.01);
    }

    #[test]
    fn test_equipment_default() {
        let eq = Equipment::default();
        assert!(eq.helmet.is_none());
        assert!(eq.chestplate.is_none());
    }

    #[test]
    fn test_equipment_damage_reduction() {
        let reg = ItemRegistry::default();
        let eq = Equipment::default(); // no armor
        let red = eq.damage_reduction(&reg);
        assert!((red - 1.0).abs() < 0.01, "No armor = no reduction");
    }

    #[test]
    fn test_furnace_state_default() {
        let f = FurnaceState::default();
        assert_eq!(f.input_id, 0);
        assert_eq!(f.cook_progress, 0.0);
    }

    #[test]
    fn test_smelting_registry() {
        let reg = SmeltingRegistry::default();
        let iron = reg.find(24); // IronOre = 24
        assert!(iron.is_some());
        assert_eq!(iron.unwrap().output_id, 102); // Iron Ingot
    }

    #[test]
    fn test_mining_progress() {
        let mut mp = MiningProgress::default();
        mp.target = Some([10, 50, 10]);
        mp.required = 1.0;
        mp.progress = 0.5;
        assert!(!mp.is_complete());
        assert!((mp.fraction() - 0.5).abs() < 0.01);
        mp.progress = 1.0;
        assert!(mp.is_complete());
    }
}
