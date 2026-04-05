//! # AnvilKit App Runner
//!
//! Eliminates game boilerplate by handling the winit event loop, input forwarding,
//! DeltaTime management, and frame lifecycle. Games implement [`GameCallbacks`] and
//! call [`AnvilKitApp::run()`].
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use anvilkit_app::prelude::*;
//!
//! struct MyGame;
//!
//! impl GameCallbacks for MyGame {
//!     fn init(&mut self, ctx: &mut GameContext) {
//!         // Initialize GPU resources, spawn entities
//!     }
//!     fn render(&mut self, ctx: &mut GameContext) {
//!         // Draw the frame
//!     }
//! }
//!
//! // AnvilKitApp::run(GameConfig::default(), MyGame);
//! ```

use bevy_ecs::prelude::*;
use anvilkit_describe::Describe;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::WindowId;

use crate::ecs_app::App;
use anvilkit_render::window::events::RenderApp;
use anvilkit_render::window::window::WindowConfig;

// --- Modules migrated from anvilkit-ecs ---
pub mod ecs_app;
pub mod ecs_plugin;
pub mod schedule;
pub mod auto_plugins;
pub mod state;

mod window_size;
pub mod screen;
pub mod egui_integration;

pub use window_size::WindowSize;
pub use screen::{CursorMode, ScreenPlugin};
pub use egui_integration::{EguiIntegration, EguiTextures};

/// Game configuration for [`AnvilKitApp::run()`].
#[derive(Debug, Clone, Describe)]
/// Top-level game window and input configuration.
pub struct GameConfig {
    /// Window title.
    #[describe(hint = "Text shown in the window title bar", default = "AnvilKit Game")]
    pub title: String,
    /// Initial window width.
    #[describe(hint = "Window width in physical pixels", range = "320..7680", default = "1280")]
    pub width: u32,
    /// Initial window height.
    #[describe(hint = "Window height in physical pixels", range = "240..4320", default = "720")]
    pub height: u32,
    /// Enable VSync.
    #[describe(hint = "Synchronize frame presentation with display refresh", default = "true")]
    pub vsync: bool,
    /// Whether to enable raw mouse input (for FPS cameras).
    #[describe(hint = "Use raw/unfiltered mouse input for FPS cameras", default = "true")]
    pub raw_mouse_input: bool,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            title: "AnvilKit Game".to_string(),
            width: 1280,
            height: 720,
            vsync: true,
            raw_mouse_input: true,
        }
    }
}

impl GameConfig {
    /// Create a new config with the given title.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            ..Default::default()
        }
    }

    /// Set window dimensions.
    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    fn to_window_config(&self) -> WindowConfig {
        WindowConfig::new()
            .with_title(&self.title)
            .with_size(self.width, self.height)
            .with_vsync(self.vsync)
    }
}

/// Context passed to [`GameCallbacks`] methods, providing access to the ECS world
/// and render infrastructure.
pub struct GameContext<'a> {
    /// The ECS application (world + schedules).
    pub app: &'a mut App,
    /// The render application (device, surface, window).
    pub render_app: &'a mut RenderApp,
    /// egui integration (available after init).
    pub egui: Option<&'a mut EguiIntegration>,
}

impl<'a> GameContext<'a> {
    /// Get the ECS world.
    pub fn world(&self) -> &World {
        self.app.world()
    }

    /// Get the ECS world mutably.
    pub fn world_mut(&mut self) -> &mut World {
        self.app.world_mut()
    }
}

/// Trait for game-specific logic. Implement this and pass it to [`AnvilKitApp::run()`].
///
/// All methods have default no-op implementations, so you only need to override
/// what your game requires.
///
/// ## Lifecycle order per frame
///
/// 1. `update()` — game logic before ECS schedules
/// 2. ECS schedules run (PreUpdate → Update → PostUpdate)
/// 3. `post_update()` — game logic after ECS schedules
/// 4. Cursor sync / input forwarding
/// 5. `render()` — draw the frame
/// 6. `ui()` — draw egui UI
pub trait GameCallbacks: 'static {
    /// Called once after the GPU device and window are initialized.
    /// Use this to create pipelines, load assets, spawn initial entities.
    fn init(&mut self, _ctx: &mut GameContext) {}

    /// Called each frame before ECS schedules run.
    /// Use for game logic that should execute before systems (e.g., block interaction, chunk loading).
    fn update(&mut self, _ctx: &mut GameContext) {}

    /// Called each frame after ECS update, before render.
    /// Use for game-specific post-update logic (e.g., chunk loading, AI ticks).
    fn post_update(&mut self, _ctx: &mut GameContext) {}

    /// Called each frame to render. The swapchain texture is available via `ctx.render_app`.
    fn render(&mut self, _ctx: &mut GameContext) {}

    /// Optional egui UI hook. NOT automatically called by the framework — games
    /// call this manually from `render()` when they have an active egui frame.
    /// Provided as a convention for separating render and UI logic.
    fn ui(&mut self, _ctx: &mut GameContext, _egui_ctx: &egui::Context) {}

    /// Called when the window is resized.
    /// `width` and `height` are the new physical pixel dimensions.
    fn on_resize(&mut self, _ctx: &mut GameContext, _width: u32, _height: u32) {}

    /// Called for each window event before the engine processes it.
    /// Return `true` to indicate the event was consumed (engine will not process it).
    fn on_window_event(&mut self, _ctx: &mut GameContext, _event: &WindowEvent) -> bool {
        false
    }

    /// Called when the application is about to exit (window close, `app.exit()`).
    /// Use for cleanup, auto-save, etc.
    fn on_shutdown(&mut self, _ctx: &mut GameContext) {}
}

/// The main application runner.
///
/// Handles the winit event loop, input forwarding, DeltaTime, and frame lifecycle.
/// Games provide a [`GameConfig`] and a [`GameCallbacks`] implementation.
pub struct AnvilKitApp<G: GameCallbacks> {
    render_app: RenderApp,
    app: App,
    game: G,
    config: GameConfig,
    initialized: bool,
    egui: Option<EguiIntegration>,
}

/// Helper macro to construct GameContext from AnvilKitApp fields.
/// Works around Rust's split borrow limitations with struct fields.
macro_rules! game_ctx {
    ($self:ident) => {
        GameContext {
            app: &mut $self.app,
            render_app: &mut $self.render_app,
            egui: None,
        }
    };
    ($self:ident, egui) => {
        GameContext {
            app: &mut $self.app,
            render_app: &mut $self.render_app,
            egui: $self.egui.as_mut(),
        }
    };
}

impl<G: GameCallbacks> AnvilKitApp<G> {
    /// Run the game. This blocks until the window is closed.
    pub fn run(config: GameConfig, app: App, game: G) {
        let event_loop = EventLoop::new().expect("Failed to create event loop");

        let wconfig = config.to_window_config();

        let mut runner = AnvilKitApp {
            render_app: RenderApp::new(wconfig),
            app,
            game,
            config,
            initialized: false,
            egui: None,
        };

        event_loop.run_app(&mut runner).expect("Event loop error");
    }
}

impl<G: GameCallbacks> ApplicationHandler for AnvilKitApp<G> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.render_app.resumed(event_loop);

        if !self.initialized {
            // Insert WindowSize resource
            if let Some((_w, _h)) = {
                let st = self.render_app.window_state();
                let s = st.size();
                if s.0 > 0 && s.1 > 0 { Some(s) } else { None }
            } {
                let (w, h) = self.render_app.window_state().size();
                self.app.world_mut().insert_resource(WindowSize::new(w as f32, h as f32));
            } else {
                self.app.world_mut().insert_resource(WindowSize::new(
                    self.config.width as f32,
                    self.config.height as f32,
                ));
            }

            // Initialize egui
            if self.egui.is_none() {
                if let (Some(device), Some(window), Some(format)) = (
                    self.render_app.render_device(),
                    self.render_app.window(),
                    self.render_app.surface_format(),
                ) {
                    self.egui = Some(EguiIntegration::new(
                        device.device(),
                        format,
                        window,
                    ));
                    self.app.world_mut().insert_resource(EguiTextures::default());
                }
            }

            let mut ctx = game_ctx!(self);
            self.game.init(&mut ctx);
            self.initialized = true;
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        // Forward to egui first
        let egui_consumed = if let Some(ref mut egui) = self.egui {
            if let Some(window) = self.render_app.window() {
                egui.handle_event(window, &event)
            } else {
                false
            }
        } else {
            false
        };

        // Let game handle event (skip if egui consumed it)
        if !egui_consumed {
            let mut ctx = game_ctx!(self);
            if self.game.on_window_event(&mut ctx, &event) {
                return; // consumed by game
            }
        }

        match &event {
            WindowEvent::CloseRequested => {
                let mut ctx = game_ctx!(self);
                self.game.on_shutdown(&mut ctx);
                event_loop.exit();
                return;
            }
            WindowEvent::Resized(size) => {
                let (w, h) = (size.width, size.height);
                if w > 0 && h > 0 {
                    self.app.world_mut().insert_resource(WindowSize::new(w as f32, h as f32));
                    let mut ctx = game_ctx!(self);
                    self.game.on_resize(&mut ctx, w, h);
                }
            }
            WindowEvent::RedrawRequested => {
                // Begin egui frame BEFORE game render, so ui() can be called during render
                if let Some(ref mut egui) = self.egui {
                    if let Some(window) = self.render_app.window().cloned() {
                        egui.begin_frame(&window);
                    }
                }

                // Game render — gets egui integration so it can call ui() + render egui
                let mut ctx = game_ctx!(self, egui);
                self.game.render(&mut ctx);
            }
            _ => {}
        }

        // Forward input to InputState (skip if egui is consuming input)
        let egui_wants = self.egui.as_ref().map_or(false, |e| {
            e.wants_pointer_input() || e.wants_keyboard_input()
        });
        if !egui_wants {
            RenderApp::forward_input(&mut self.app, &event);
        }

        // Let RenderApp handle window management (resize surface, etc.)
        self.render_app.window_event(event_loop, window_id, event);
    }

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: DeviceId,
        event: DeviceEvent,
    ) {
        if self.config.raw_mouse_input {
            RenderApp::forward_device_input(&mut self.app, &event);
        }
        self.render_app.device_event(event_loop, device_id, event);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // 1. Game update hook (before ECS schedules)
        {
            let mut ctx = game_ctx!(self);
            self.game.update(&mut ctx);
        }

        // 2. Frame tick: DeltaTime → ECS update → end_frame → request_redraw
        self.render_app.tick(&mut self.app);

        // Apply cursor mode from ECS resource (set by ScreenPlugin's cursor_sync_system)
        if let Some(cursor_mode) = self.app.world().get_resource::<screen::CursorMode>() {
            if let Some(window) = self.render_app.window() {
                match cursor_mode {
                    screen::CursorMode::Free => {
                        let _ = window.set_cursor_grab(winit::window::CursorGrabMode::None);
                        window.set_cursor_visible(true);
                    }
                    screen::CursorMode::Locked => {
                        let _ = window
                            .set_cursor_grab(winit::window::CursorGrabMode::Confined)
                            .or_else(|_| {
                                window.set_cursor_grab(winit::window::CursorGrabMode::Locked)
                            });
                        window.set_cursor_visible(false);
                    }
                }
            }
        }

        // Game post-update hook
        {
            let mut ctx = game_ctx!(self);
            self.game.post_update(&mut ctx);
        }

        // Check if the app wants to exit (e.g., game called app.exit_game())
        if self.app.should_exit().is_some() {
            let mut ctx = game_ctx!(self);
            self.game.on_shutdown(&mut ctx);
            event_loop.exit();
        }
    }
}

/// Prelude for convenient imports.
pub mod prelude {
    pub use crate::{AnvilKitApp, GameCallbacks, GameConfig, GameContext, WindowSize};
    pub use crate::screen::{CursorMode, ScreenPlugin};
    pub use crate::egui_integration::EguiTextures;
    pub use crate::ecs_app::{App, Plugin, DeltaTime, AppExt};
    pub use crate::ecs_plugin::AnvilKitEcsPlugin;
    pub use crate::schedule::{AnvilKitSchedule, AnvilKitSystemSet, ScheduleBuilder, common_conditions};
    pub use crate::auto_plugins::{AutoInputPlugin, AutoDeltaTimePlugin};
    pub use crate::state::{GameState, NextGameState, StateTransitionEvent, StateValue, in_state, state_transition_system};
    pub use bevy_ecs::prelude::*;
    pub use egui;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_config_default() {
        let config = GameConfig::default();
        assert_eq!(config.width, 1280);
        assert_eq!(config.height, 720);
        assert!(config.vsync);
    }

    #[test]
    fn test_game_config_builder() {
        let config = GameConfig::new("Test Game").with_size(1920, 1080);
        assert_eq!(config.title, "Test Game");
        assert_eq!(config.width, 1920);
        assert_eq!(config.height, 1080);
    }

    #[test]
    fn test_window_size() {
        let ws = WindowSize::new(800.0, 600.0);
        assert_eq!(ws.width, 800.0);
        assert_eq!(ws.height, 600.0);
    }
}
