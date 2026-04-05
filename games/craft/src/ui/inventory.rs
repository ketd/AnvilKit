use super::CraftScreen;
use crate::block::{BlockType, Face, tile_uv, TILE_UV_X, TILE_UV_Y};

/// All placeable block types for Creative inventory.
const ALL_BLOCKS: &[BlockType] = &[
    BlockType::Grass, BlockType::Dirt, BlockType::Stone, BlockType::Sand,
    BlockType::Brick, BlockType::Wood, BlockType::Cement, BlockType::Plank,
    BlockType::Snow, BlockType::Glass, BlockType::Cobble, BlockType::LightStone,
    BlockType::DarkStone, BlockType::Chest, BlockType::Leaves, BlockType::Cloud,
    BlockType::TallGrass, BlockType::YellowFlower, BlockType::RedFlower, BlockType::Purple,
    BlockType::Water,
    BlockType::CoalOre, BlockType::IronOre, BlockType::GoldOre, BlockType::DiamondOre,
    BlockType::RedstoneOre, BlockType::LapisOre,
    BlockType::Torch, BlockType::Glowstone, BlockType::Lantern,
    BlockType::Workbench, BlockType::Furnace,
    BlockType::Sandstone, BlockType::Gravel, BlockType::Cactus,
    BlockType::BirchWood, BlockType::BirchLeaves, BlockType::SpruceWood, BlockType::SpruceLeaves,
    BlockType::SnowBlock, BlockType::Ice,
];

/// Creative inventory — full block selection grid with atlas texture icons.
pub fn draw(
    ctx: &egui::Context,
    locale: Option<&anvilkit_data::Locale>,
    selected_block: BlockType,
    atlas_tex_id: Option<egui::TextureId>,
) -> (Option<CraftScreen>, Option<BlockType>) {
    let mut picked: Option<BlockType> = None;

    // Dim background
    egui::Area::new(egui::Id::new("inventory_overlay"))
        .fixed_pos(egui::pos2(0.0, 0.0))
        .show(ctx, |ui| {
            let screen = ctx.screen_rect();
            ui.painter().rect_filled(
                screen,
                0.0,
                egui::Color32::from_rgba_unmultiplied(0, 0, 0, 160),
            );
        });

    let cols = 9;
    let icon_size = 48.0;

    egui::Window::new("Inventory")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .title_bar(false)
        .fixed_size(egui::vec2(620.0, 460.0))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new("Creative Inventory")
                        .size(22.0)
                        .color(egui::Color32::WHITE)
                        .strong(),
                );
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(4.0);

            // Scrollable grid of all blocks
            egui::ScrollArea::vertical()
                .max_height(350.0)
                .show(ui, |ui| {
                    egui::Grid::new("creative_grid")
                        .spacing([4.0, 4.0])
                        .show(ui, |ui| {
                            for (i, &block) in ALL_BLOCKS.iter().enumerate() {
                                let name = locale.map_or_else(
                                    || format!("{:?}", block),
                                    |l| l.t(block.locale_key()).to_string(),
                                );
                                let is_selected = block == selected_block;

                                let stroke = if is_selected {
                                    egui::Stroke::new(2.0, egui::Color32::YELLOW)
                                } else {
                                    egui::Stroke::new(1.0, egui::Color32::from_rgb(80, 80, 80))
                                };

                                // Each slot is a framed vertical layout with icon + name
                                let response = ui.allocate_ui(egui::vec2(62.0, 76.0), |ui| {
                                    let (rect, resp) = ui.allocate_exact_size(
                                        egui::vec2(62.0, 76.0),
                                        egui::Sense::click(),
                                    );

                                    // Background
                                    let bg = if is_selected {
                                        egui::Color32::from_rgba_unmultiplied(60, 60, 40, 200)
                                    } else if resp.hovered() {
                                        egui::Color32::from_rgba_unmultiplied(50, 50, 50, 200)
                                    } else {
                                        egui::Color32::from_rgba_unmultiplied(35, 35, 35, 180)
                                    };
                                    ui.painter().rect(rect, 4.0, bg, stroke);

                                    // Block texture icon
                                    let icon_rect = egui::Rect::from_min_size(
                                        egui::pos2(rect.min.x + 7.0, rect.min.y + 4.0),
                                        egui::vec2(icon_size, icon_size),
                                    );

                                    if let Some(tex_id) = atlas_tex_id {
                                        // Compute UV rect for this block's front face tile
                                        let tile = block.face_tile(Face::Front);
                                        let (u0, v0) = tile_uv(tile);
                                        let u1 = u0 + TILE_UV_X;
                                        let v1 = v0 + TILE_UV_Y;
                                        let uv = egui::Rect::from_min_max(
                                            egui::pos2(u0, v0),
                                            egui::pos2(u1, v1),
                                        );
                                        ui.painter().image(
                                            tex_id,
                                            icon_rect,
                                            uv,
                                            egui::Color32::WHITE,
                                        );
                                    } else {
                                        // Fallback: colored rectangle
                                        let [r, g, b] = block.preview_color();
                                        let color = egui::Color32::from_rgb(
                                            (r * 255.0) as u8,
                                            (g * 255.0) as u8,
                                            (b * 255.0) as u8,
                                        );
                                        ui.painter().rect_filled(icon_rect, 2.0, color);
                                    }

                                    // Block name below icon
                                    let text_color = if is_selected {
                                        egui::Color32::YELLOW
                                    } else {
                                        egui::Color32::from_rgb(200, 200, 200)
                                    };
                                    let text_pos = egui::pos2(
                                        rect.center().x,
                                        rect.min.y + icon_size + 7.0,
                                    );
                                    // Truncate long names
                                    let short: String = name.chars().take(8).collect();
                                    ui.painter().text(
                                        text_pos,
                                        egui::Align2::CENTER_TOP,
                                        &short,
                                        egui::FontId::proportional(9.0),
                                        text_color,
                                    );

                                    resp
                                });

                                if response.inner.clicked() {
                                    picked = Some(block);
                                }

                                if (i + 1) % cols == 0 {
                                    ui.end_row();
                                }
                            }
                        });
                });

            ui.add_space(4.0);
            ui.separator();
            ui.add_space(2.0);

            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new("Click a block to select it for the hotbar. Press E or ESC to close.")
                        .size(11.0)
                        .color(egui::Color32::from_rgb(100, 100, 110)),
                );
            });
        });

    (None, picked)
}
