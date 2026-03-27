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
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::WindowId;

use anvilkit_ecs::app::App;
use anvilkit_render::window::events::RenderApp;
use anvilkit_render::window::window::WindowConfig;

mod window_size;
pub use window_size::WindowSize;

/// Game configuration for [`AnvilKitApp::run()`].
#[derive(Debug, Clone)]
pub struct GameConfig {
    /// Window title.
    pub title: String,
    /// Initial window width.
    pub width: u32,
    /// Initial window height.
    pub height: u32,
    /// Enable VSync.
    pub vsync: bool,
    /// Whether to enable raw mouse input (for FPS cameras).
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
}

impl<'a> GameContext<'a> {
    /// Get the ECS world.
    pub fn world(&self) -> &World {
        &self.app.world
    }

    /// Get the ECS world mutably.
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.app.world
    }
}

/// Trait for game-specific logic. Implement this and pass it to [`AnvilKitApp::run()`].
///
/// All methods have default no-op implementations, so you only need to override
/// what your game requires.
pub trait GameCallbacks: 'static {
    /// Called once after the GPU device and window are initialized.
    /// Use this to create pipelines, load assets, spawn initial entities.
    fn init(&mut self, _ctx: &mut GameContext) {}

    /// Called each frame after ECS update, before render.
    /// Use for game-specific post-update logic (e.g., chunk loading, AI ticks).
    fn post_update(&mut self, _ctx: &mut GameContext) {}

    /// Called each frame to render. The swapchain texture is available via `ctx.render_app`.
    fn render(&mut self, _ctx: &mut GameContext) {}

    /// Called when the window is resized.
    /// `width` and `height` are the new physical pixel dimensions.
    fn on_resize(&mut self, _ctx: &mut GameContext, _width: u32, _height: u32) {}

    /// Called for each window event before the engine processes it.
    /// Return `true` to indicate the event was consumed (engine will not process it).
    fn on_window_event(&mut self, _ctx: &mut GameContext, _event: &WindowEvent) -> bool {
        false
    }
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
                self.app.world.insert_resource(WindowSize::new(w as f32, h as f32));
            } else {
                self.app.world.insert_resource(WindowSize::new(
                    self.config.width as f32,
                    self.config.height as f32,
                ));
            }

            let mut ctx = GameContext {
                app: &mut self.app,
                render_app: &mut self.render_app,
            };
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
        // Let game handle event first
        {
            let mut ctx = GameContext {
                app: &mut self.app,
                render_app: &mut self.render_app,
            };
            if self.game.on_window_event(&mut ctx, &event) {
                return; // consumed by game
            }
        }

        match &event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
                return;
            }
            WindowEvent::Resized(size) => {
                let (w, h) = (size.width, size.height);
                if w > 0 && h > 0 {
                    self.app.world.insert_resource(WindowSize::new(w as f32, h as f32));
                    let mut ctx = GameContext {
                        app: &mut self.app,
                        render_app: &mut self.render_app,
                    };
                    self.game.on_resize(&mut ctx, w, h);
                }
            }
            WindowEvent::RedrawRequested => {
                let mut ctx = GameContext {
                    app: &mut self.app,
                    render_app: &mut self.render_app,
                };
                self.game.render(&mut ctx);
            }
            _ => {}
        }

        // Forward input to InputState
        RenderApp::forward_input(&mut self.app, &event);

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
        // Frame tick: DeltaTime → ECS update → end_frame → request_redraw
        self.render_app.tick(&mut self.app);

        // Game post-update hook
        let mut ctx = GameContext {
            app: &mut self.app,
            render_app: &mut self.render_app,
        };
        self.game.post_update(&mut ctx);

        // Check if the app wants to exit (e.g., game called app.exit())
        if self.app.should_exit() {
            event_loop.exit();
        }
    }
}

/// Prelude for convenient imports.
pub mod prelude {
    pub use crate::{AnvilKitApp, GameCallbacks, GameConfig, GameContext, WindowSize};
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
