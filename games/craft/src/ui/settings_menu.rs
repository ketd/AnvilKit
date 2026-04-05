use super::{CraftScreen, SettingsReturnTo};

/// Persistent settings state — serialized to disk.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SettingsState {
    pub volume: f32,
    pub sensitivity: f32,
    pub fov: f32,
    pub view_distance: f32,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            volume: 0.8,
            sensitivity: 3.0,
            fov: 70.0,
            view_distance: 8.0,
        }
    }
}

impl SettingsState {
    const PATH: &'static str = "saves/settings.ron";

    /// Load from disk, or return defaults.
    pub fn load_or_default() -> Self {
        std::fs::read_to_string(Self::PATH)
            .ok()
            .and_then(|s| ron::from_str(&s).ok())
            .unwrap_or_default()
    }

    /// Save to disk.
    pub fn save(&self) {
        if let Some(parent) = std::path::Path::new(Self::PATH).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(s) = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default()) {
            let _ = std::fs::write(Self::PATH, s);
        }
    }
}

/// Render the settings screen. Returns the screen to transition to, if any.
pub fn draw(
    ctx: &egui::Context,
    state: &mut SettingsState,
    return_to: SettingsReturnTo,
) -> Option<CraftScreen> {
    let mut result = None;

    // Dim background
    egui::Area::new(egui::Id::new("settings_overlay"))
        .fixed_pos(egui::pos2(0.0, 0.0))
        .show(ctx, |ui| {
            let screen = ctx.screen_rect();
            ui.painter().rect_filled(
                screen,
                0.0,
                egui::Color32::from_rgba_unmultiplied(0, 0, 0, 160),
            );
        });

    egui::Window::new("Settings")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .title_bar(false)
        .fixed_size(egui::vec2(340.0, 300.0))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new("Settings")
                        .size(24.0)
                        .color(egui::Color32::WHITE)
                        .strong(),
                );
            });

            ui.add_space(12.0);
            ui.separator();
            ui.add_space(8.0);

            egui::Grid::new("settings_grid")
                .num_columns(2)
                .spacing([16.0, 10.0])
                .show(ui, |ui| {
                    ui.label("Volume");
                    ui.add(egui::Slider::new(&mut state.volume, 0.0..=1.0));
                    ui.end_row();

                    ui.label("Sensitivity");
                    ui.add(egui::Slider::new(&mut state.sensitivity, 0.5..=10.0));
                    ui.end_row();

                    ui.label("FOV");
                    ui.add(egui::Slider::new(&mut state.fov, 50.0..=120.0));
                    ui.end_row();

                    ui.label("View Distance");
                    ui.add(egui::Slider::new(&mut state.view_distance, 3.0..=16.0));
                    ui.end_row();
                });

            ui.add_space(12.0);
            ui.separator();
            ui.add_space(8.0);

            ui.vertical_centered(|ui| {
                if ui
                    .add_sized(
                        egui::vec2(180.0, 36.0),
                        egui::Button::new(egui::RichText::new("Back").size(16.0)),
                    )
                    .clicked()
                {
                    result = Some(match return_to {
                        SettingsReturnTo::MainMenu => CraftScreen::MainMenu,
                        SettingsReturnTo::Paused => CraftScreen::Paused,
                    });
                }
            });
        });

    result
}
