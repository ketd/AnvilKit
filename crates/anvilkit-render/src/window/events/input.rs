use std::time::Instant;
use winit::{
    application::ApplicationHandler,
    event::{WindowEvent, DeviceEvent, DeviceId},
    event_loop::ActiveEventLoop,
    window::WindowId,
};
use log::{info, error, debug};

use bevy_app::App;
use anvilkit_core::time::DeltaTime;
use anvilkit_input::prelude::{InputState, KeyCode, MouseButton};

use super::render_app::RenderApp;

impl RenderApp {
    // --- Public helpers for games with custom ApplicationHandler ---

    /// Forward a window event to [`InputState`] (keyboard, mouse, cursor, scroll).
    ///
    /// Call this from your own [`ApplicationHandler::window_event`] implementation
    /// so the engine handles input state bookkeeping while you handle game-specific events.
    pub fn forward_input(app: &mut App, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput { event, .. } => {
                if let winit::keyboard::PhysicalKey::Code(code) = event.physical_key {
                    if let Some(mut input) = app.world_mut().get_resource_mut::<InputState>() {
                        if let Some(key) = KeyCode::from_winit(code) {
                            if event.state.is_pressed() {
                                input.press_key(key);
                            } else {
                                input.release_key(key);
                            }
                        }
                    }
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if let Some(mut input) = app.world_mut().get_resource_mut::<InputState>() {
                    if let Some(btn) = MouseButton::from_winit(*button) {
                        if state.is_pressed() {
                            input.press_mouse(btn);
                        } else {
                            input.release_mouse(btn);
                        }
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                if let Some(mut input) = app.world_mut().get_resource_mut::<InputState>() {
                    input.set_mouse_position(glam::Vec2::new(position.x as f32, position.y as f32));
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                if let Some(mut input) = app.world_mut().get_resource_mut::<InputState>() {
                    let scroll = match delta {
                        winit::event::MouseScrollDelta::LineDelta(_, y) => *y,
                        winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32 / 120.0,
                    };
                    input.add_scroll_delta(scroll);
                }
            }
            _ => {}
        }
    }

    /// Forward a device event to [`InputState`] (raw mouse motion delta).
    ///
    /// Call this from your own [`ApplicationHandler::device_event`] implementation.
    /// The accumulated delta is cleared automatically by [`InputState::end_frame`].
    pub fn forward_device_input(app: &mut App, event: &DeviceEvent) {
        if let DeviceEvent::MouseMotion { delta } = event {
            if let Some(mut input) = app.world_mut().get_resource_mut::<InputState>() {
                input.add_mouse_delta(glam::Vec2::new(delta.0 as f32, delta.1 as f32));
            }
        }
    }

    /// Run a single frame tick: update DeltaTime, run `app.update()`, clear input state.
    ///
    /// Call this from your own [`ApplicationHandler::about_to_wait`] implementation.
    /// Handles the standard per-frame lifecycle so your game only needs to add
    /// pre-update and post-update logic around it.
    pub fn tick(&mut self, app: &mut App) {
        let now = Instant::now();
        let raw_dt = now.duration_since(self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;
        let dt = raw_dt.clamp(0.001, 0.1);
        app.world_mut().insert_resource(DeltaTime(dt));

        app.update();

        if let Some(mut input) = app.world_mut().get_resource_mut::<InputState>() {
            input.end_frame();
        }

        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

impl ApplicationHandler for RenderApp {
    /// 应用恢复事件
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        info!("应用恢复");

        if let Err(e) = self.create_window(event_loop) {
            error!("创建窗口失败: {}", e);
            event_loop.exit();
            return;
        }

        if let Err(e) = pollster::block_on(self.init_render()) {
            error!("初始化渲染失败: {}", e);
            event_loop.exit();
            return;
        }

        // 如果持有 ECS App，注入 RenderState
        self.inject_render_state_to_ecs();

        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    /// 窗口事件处理
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                info!("收到窗口关闭请求");
                self.request_exit();
                event_loop.exit();
            }

            WindowEvent::Resized(new_size) => {
                self.handle_resize(new_size);
            }

            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.handle_scale_factor_changed(scale_factor);
            }

            WindowEvent::KeyboardInput { .. }
            | WindowEvent::MouseInput { .. }
            | WindowEvent::CursorMoved { .. }
            | WindowEvent::MouseWheel { .. } => {
                if let Some(app) = &mut self.app {
                    Self::forward_input(app, &event);
                }
            }

            WindowEvent::Focused(focused) => {
                debug!("窗口焦点变化: {}", focused);
                self.window_state.set_focused(focused);
            }

            WindowEvent::Occluded(occluded) => {
                debug!("窗口遮挡状态: {}", occluded);
                self.window_state.set_minimized(occluded);
            }

            WindowEvent::RedrawRequested => {
                self.render();
            }

            _ => {}
        }
    }

    /// 设备事件处理
    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        if let Some(app) = &mut self.app {
            Self::forward_device_input(app, &event);
        }
    }

    /// 即将等待事件
    #[allow(unused_variables)]
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // 使用 tick() 统一处理：DeltaTime → app.update() → end_frame → request_redraw
        // 注意：需要临时取出 app 以满足借用检查（tick 需要 &mut self 和 &mut App）
        if let Some(mut app) = self.app.take() {
            self.tick(&mut app);

            // 检查 capture auto_exit
            #[cfg(feature = "capture")]
            {
                if let Some(state) = app.world().get_resource::<crate::renderer::capture::CaptureState>() {
                    if state.exit_requested {
                        info!("帧捕获完成，自动退出");
                        event_loop.exit();
                    }
                }
            }

            self.app = Some(app);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::render_app::RenderApp;
    use crate::window::WindowConfig;
    use winit::dpi::PhysicalSize;

    #[test]
    fn test_render_app_creation() {
        let config = WindowConfig::new().with_title("Test App");
        let app = RenderApp::new(config);

        assert_eq!(app.config().title, "Test App");
        assert!(app.window().is_none());
        assert!(!app.is_exit_requested());
    }

    #[test]
    fn test_exit_request() {
        let mut app = RenderApp::new(WindowConfig::default());

        assert!(!app.is_exit_requested());
        app.request_exit();
        assert!(app.is_exit_requested());
    }

    #[test]
    fn test_window_state_updates() {
        let mut app = RenderApp::new(WindowConfig::default());

        let new_size = PhysicalSize::new(1920, 1080);
        app.handle_resize(new_size);
        assert_eq!(app.window_state().size(), (1920, 1080));

        app.handle_scale_factor_changed(2.0);
        assert_eq!(app.window_state().scale_factor(), 2.0);
    }

    #[test]
    fn test_render_app_config() {
        let config = WindowConfig::new()
            .with_title("Test")
            .with_size(640, 480);
        let app = RenderApp::new(config);

        assert_eq!(app.config().title, "Test");
        assert_eq!(app.config().width, 640);
    }

    #[test]
    fn test_render_app_exit_request_toggle() {
        let mut app = RenderApp::new(WindowConfig::new());
        assert!(!app.is_exit_requested());

        app.request_exit();
        assert!(app.is_exit_requested());
    }

    #[test]
    fn test_render_app_window_state() {
        let config = WindowConfig::new().with_size(1024, 768);
        let app = RenderApp::new(config);

        // WindowState defaults to (1280, 720) since it's created via WindowState::new()
        let state = app.window_state();
        assert_eq!(state.size(), (1280, 720));
    }
}
