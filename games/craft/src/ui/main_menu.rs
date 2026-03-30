use super::CraftScreen;

/// Render the main menu. Returns the screen to transition to, if any.
pub fn draw(ctx: &egui::Context) -> Option<CraftScreen> {
    let mut result = None;

    egui::CentralPanel::default()
        .frame(egui::Frame::none().fill(egui::Color32::from_rgb(10, 10, 15)))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(ui.available_height() * 0.25);

                ui.label(
                    egui::RichText::new("C R A F T")
                        .size(48.0)
                        .color(egui::Color32::WHITE)
                        .strong(),
                );

                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new("A voxel sandbox")
                        .size(14.0)
                        .color(egui::Color32::from_rgb(120, 120, 140)),
                );

                ui.add_space(40.0);

                let btn_size = egui::vec2(220.0, 44.0);

                if ui
                    .add_sized(btn_size, egui::Button::new(
                        egui::RichText::new("Play").size(20.0),
                    ))
                    .clicked()
                {
                    result = Some(CraftScreen::Playing);
                }

                ui.add_space(4.0);

                if ui
                    .add_sized(btn_size, egui::Button::new(
                        egui::RichText::new("Settings").size(18.0),
                    ))
                    .clicked()
                {
                    result = Some(CraftScreen::Settings);
                }

                ui.add_space(4.0);

                if ui
                    .add_sized(btn_size, egui::Button::new(
                        egui::RichText::new("Quit").size(18.0),
                    ))
                    .clicked()
                {
                    result = Some(CraftScreen::Quit);
                }
            });
        });

    result
}
