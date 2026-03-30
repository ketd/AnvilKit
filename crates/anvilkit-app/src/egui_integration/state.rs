use egui_winit::EventResponse;
use winit::event::WindowEvent;
use winit::window::Window;

/// Wraps `egui_winit::State` — handles input forwarding from winit to egui.
pub struct EguiInputState {
    pub(crate) state: egui_winit::State,
}

impl EguiInputState {
    pub fn new(ctx: egui::Context, window: &Window) -> Self {
        let state = egui_winit::State::new(
            ctx,
            egui::ViewportId::ROOT,
            window,
            Some(window.scale_factor() as f32),
            None, // theme
            None, // max_texture_side
        );
        Self { state }
    }

    /// Forward a winit event to egui. Returns whether egui consumed it.
    pub fn on_window_event(&mut self, window: &Window, event: &WindowEvent) -> EventResponse {
        self.state.on_window_event(window, event)
    }

    /// Begin an egui frame — call before any UI drawing.
    pub fn take_input(&mut self, window: &Window) -> egui::RawInput {
        self.state.take_egui_input(window)
    }

    /// After ending the egui frame, handle platform output (cursor, IME, clipboard).
    pub fn handle_output(&mut self, window: &Window, output: egui::PlatformOutput) {
        self.state.handle_platform_output(window, output);
    }
}
