//! # 应用框架
//!
//! Re-exports `bevy_app::App` as the primary application container.
//! Provides `AppExt` trait with engine-ergonomic exit helpers on top of bevy's
//! native `AppExit` event mechanism.

/// Re-export bevy_app::App as the primary application type.
pub use bevy_app::App;

/// Re-export bevy_app::Plugin for convenience.
pub use bevy_app::Plugin;

/// Re-export DeltaTime from core.
pub use anvilkit_core::time::DeltaTime;

/// Extension trait for `bevy_app::App` providing engine-ergonomic exit control.
///
/// Wraps bevy's native `AppExit` event mechanism with simpler method calls.
pub trait AppExt {
    /// Mark the application for exit by sending `AppExit::Success` event.
    fn exit_game(&mut self);
}

impl AppExt for App {
    fn exit_game(&mut self) {
        self.world_mut().send_event(bevy_app::AppExit::Success);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_creation() {
        let app = App::new();
        assert!(app.should_exit().is_none());
    }

    #[test]
    fn test_delta_time_default() {
        let dt = DeltaTime::default();
        assert!((dt.0 - 1.0 / 60.0).abs() < 0.001);
    }
}
