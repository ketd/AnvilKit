use super::CraftScreen;

/// Render the pause overlay. Returns the screen to transition to, if any.
pub fn draw(ctx: &egui::Context) -> Option<CraftScreen> {
    let mut result = None;

    // Semi-transparent background
    egui::Area::new(egui::Id::new("pause_overlay"))
        .fixed_pos(egui::pos2(0.0, 0.0))
        .show(ctx, |ui| {
            let screen = ctx.screen_rect();
            ui.painter().rect_filled(
                screen,
                0.0,
                egui::Color32::from_rgba_unmultiplied(0, 0, 0, 160),
            );
        });

    egui::Window::new("Paused")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .title_bar(false)
        .fixed_size(egui::vec2(260.0, 220.0))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(12.0);
                ui.label(
                    egui::RichText::new("PAUSED")
                        .size(28.0)
                        .color(egui::Color32::WHITE)
                        .strong(),
                );
                ui.add_space(16.0);

                let btn = egui::vec2(200.0, 38.0);

                if ui.add_sized(btn, egui::Button::new(
                    egui::RichText::new("Resume").size(18.0),
                )).clicked() {
                    result = Some(CraftScreen::Playing);
                }

                if ui.add_sized(btn, egui::Button::new(
                    egui::RichText::new("Settings").size(16.0),
                )).clicked() {
                    result = Some(CraftScreen::Settings);
                }

                if ui.add_sized(btn, egui::Button::new(
                    egui::RichText::new("Save & Quit").size(16.0),
                )).clicked() {
                    result = Some(CraftScreen::SaveAndQuit);
                }
            });
        });

    result
}
