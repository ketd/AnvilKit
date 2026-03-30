use egui::{Color32, Rounding, Stroke, Visuals, style::WidgetVisuals};

/// Apply the Craft game dark theme to egui.
pub fn apply_craft_theme(ctx: &egui::Context) {
    let mut visuals = Visuals::dark();

    // Panel backgrounds
    visuals.window_fill = Color32::from_rgba_unmultiplied(18, 18, 24, 240);
    visuals.window_rounding = Rounding::same(10.0);
    visuals.window_stroke = Stroke::new(1.0, Color32::from_rgb(50, 50, 65));
    visuals.panel_fill = Color32::from_rgba_unmultiplied(12, 12, 16, 245);

    // Widgets — inactive
    visuals.widgets.inactive = WidgetVisuals {
        bg_fill: Color32::from_rgb(35, 35, 45),
        weak_bg_fill: Color32::from_rgb(30, 30, 40),
        bg_stroke: Stroke::new(1.0, Color32::from_rgb(55, 55, 70)),
        fg_stroke: Stroke::new(1.0, Color32::from_rgb(200, 200, 210)),
        rounding: Rounding::same(6.0),
        expansion: 0.0,
    };

    // Widgets — hovered
    visuals.widgets.hovered = WidgetVisuals {
        bg_fill: Color32::from_rgb(50, 50, 70),
        weak_bg_fill: Color32::from_rgb(45, 45, 60),
        bg_stroke: Stroke::new(1.0, Color32::from_rgb(80, 80, 110)),
        fg_stroke: Stroke::new(1.0, Color32::WHITE),
        rounding: Rounding::same(6.0),
        expansion: 1.0,
    };

    // Widgets — active (pressed)
    visuals.widgets.active = WidgetVisuals {
        bg_fill: Color32::from_rgb(70, 70, 100),
        weak_bg_fill: Color32::from_rgb(60, 60, 85),
        bg_stroke: Stroke::new(1.0, Color32::from_rgb(100, 100, 140)),
        fg_stroke: Stroke::new(1.0, Color32::WHITE),
        rounding: Rounding::same(6.0),
        expansion: 0.0,
    };

    // Non-interactive (labels, separators)
    visuals.widgets.noninteractive.bg_fill = Color32::TRANSPARENT;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, Color32::from_rgb(180, 180, 190));

    ctx.set_visuals(visuals);

    // Spacing
    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(8.0, 10.0);
    style.spacing.button_padding = egui::vec2(16.0, 8.0);
    style.spacing.window_margin = egui::Margin::same(20.0);
    ctx.set_style(style);
}
