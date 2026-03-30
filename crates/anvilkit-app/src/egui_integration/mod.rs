//! egui integration for AnvilKit — handles input, rendering, and texture management.
//!
//! Games use egui through the `GameCallbacks::ui()` method. The integration is
//! fully managed by `AnvilKitApp` — no manual setup needed.

mod state;
mod renderer;

use bevy_ecs::system::Resource;
use std::collections::HashMap;
use winit::event::WindowEvent;
use winit::window::Window;

pub use self::renderer::EguiRenderer;
pub use self::state::EguiInputState;

/// Complete egui integration: input state + GPU renderer + egui context.
pub struct EguiIntegration {
    pub ctx: egui::Context,
    pub input_state: EguiInputState,
    pub renderer: EguiRenderer,
}

impl EguiIntegration {
    /// Create a new egui integration.
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        window: &Window,
    ) -> Self {
        let ctx = egui::Context::default();
        let input_state = EguiInputState::new(ctx.clone(), window);
        let renderer = EguiRenderer::new(device, surface_format);
        Self {
            ctx,
            input_state,
            renderer,
        }
    }

    /// Forward a winit event to egui. Returns true if egui consumed it.
    pub fn handle_event(&mut self, window: &Window, event: &WindowEvent) -> bool {
        let response = self.input_state.on_window_event(window, event);
        response.consumed
    }

    /// Whether egui wants exclusive pointer input (hovering a widget).
    pub fn wants_pointer_input(&self) -> bool {
        self.ctx.wants_pointer_input()
    }

    /// Whether egui wants exclusive keyboard input (typing in a text field).
    pub fn wants_keyboard_input(&self) -> bool {
        self.ctx.wants_keyboard_input()
    }

    /// Begin an egui frame. Call before `GameCallbacks::ui()`.
    pub fn begin_frame(&mut self, window: &Window) {
        let raw_input = self.input_state.take_input(window);
        self.ctx.begin_pass(raw_input);
    }

    /// End the egui frame and render to the target. Call after `GameCallbacks::ui()`.
    pub fn end_frame_and_render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        window: &Window,
        screen_w: u32,
        screen_h: u32,
    ) {
        let output = self.ctx.end_pass();

        // Handle platform output (cursor changes, clipboard, IME)
        self.input_state
            .handle_output(window, output.platform_output);

        // Tessellate shapes into GPU-ready primitives
        let primitives = self
            .ctx
            .tessellate(output.shapes, self.ctx.pixels_per_point());

        // Render
        let screen = renderer::ScreenDescriptor {
            size_in_pixels: [screen_w, screen_h],
            pixels_per_point: self.ctx.pixels_per_point(),
        };
        self.renderer.render(
            device,
            queue,
            encoder,
            target,
            &primitives,
            &output.textures_delta,
            &screen,
        );
    }

    /// Register a wgpu texture for use in egui widgets (Image, ImageButton).
    pub fn register_texture(
        &mut self,
        device: &wgpu::Device,
        view: &wgpu::TextureView,
    ) -> egui::TextureId {
        self.renderer.register_native_texture(device, view)
    }
}

/// ECS Resource that maps named textures to egui TextureIds.
/// Register textures in `GameCallbacks::init()`, use them in `GameCallbacks::ui()`.
#[derive(Resource, Default)]
pub struct EguiTextures {
    textures: HashMap<String, egui::TextureId>,
}

impl EguiTextures {
    /// Get a registered texture by name.
    pub fn get(&self, name: &str) -> Option<egui::TextureId> {
        self.textures.get(name).copied()
    }

    /// Register a texture with a name. Overwrites any existing texture with the same name.
    pub fn insert(&mut self, name: impl Into<String>, id: egui::TextureId) {
        self.textures.insert(name.into(), id);
    }
}
