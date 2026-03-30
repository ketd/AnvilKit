use super::CraftScreen;

/// Render the inventory screen. Returns the screen to transition to, if any.
pub fn draw(
    ctx: &egui::Context,
    slots: &[(String, u32)],
    selected: &mut usize,
) -> Option<CraftScreen> {
    let result = None;

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

    egui::Window::new("Inventory")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .title_bar(false)
        .fixed_size(egui::vec2(310.0, 370.0))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new("Inventory")
                        .size(24.0)
                        .color(egui::Color32::WHITE)
                        .strong(),
                );
            });

            ui.add_space(12.0);
            ui.separator();
            ui.add_space(8.0);

            // 3x3 grid
            egui::Grid::new("inventory_grid")
                .spacing([6.0, 6.0])
                .show(ui, |ui| {
                    for i in 0..9.min(slots.len()) {
                        let (name, qty) = &slots[i];
                        let is_selected = i == *selected;

                        let btn_text = if name.is_empty() {
                            egui::RichText::new("--").size(14.0).color(egui::Color32::from_rgb(80, 80, 80))
                        } else {
                            let label = format!("{}\nx{}", name, qty);
                            let color = if is_selected {
                                egui::Color32::YELLOW
                            } else {
                                egui::Color32::WHITE
                            };
                            egui::RichText::new(label).size(13.0).color(color)
                        };

                        let btn = egui::Button::new(btn_text)
                            .min_size(egui::vec2(88.0, 88.0));

                        let btn = if is_selected {
                            btn.stroke(egui::Stroke::new(2.0, egui::Color32::YELLOW))
                        } else {
                            btn
                        };

                        if ui.add(btn).clicked() {
                            *selected = i;
                        }

                        if (i + 1) % 3 == 0 {
                            ui.end_row();
                        }
                    }
                });

            ui.add_space(12.0);
            ui.separator();
            ui.add_space(4.0);

            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new("Press E or ESC to close")
                        .size(12.0)
                        .color(egui::Color32::from_rgb(100, 100, 110)),
                );
            });
        });

    result
}
