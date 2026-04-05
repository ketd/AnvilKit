//! # AnvilKit Game Engine
//!
//! A modular, ECS-based game engine built with Rust.
//!
//! Instead of depending on individual `anvilkit-*` crates, add `anvilkit` to
//! your `Cargo.toml` and access everything through a single namespace:
//!
//! ```rust,ignore
//! use anvilkit::prelude::*;
//!
//! // Or import specific modules:
//! use anvilkit::app::ecs_app::App;
//! use anvilkit::render::WindowConfig;
//! use anvilkit::assets::MeshData;
//! ```

pub use anvilkit_core as core;
pub use anvilkit_render as render;
pub use anvilkit_assets as assets;
pub use anvilkit_input as input;
pub use anvilkit_audio as audio;
pub use anvilkit_app as app;
pub use anvilkit_describe as describe;
#[cfg(feature = "mcp")]
pub use anvilkit_mcp as mcp;

pub mod default_plugins;
pub use default_plugins::DefaultPlugins;

/// Convenient re-exports of the most commonly used types and traits.
pub mod prelude {
    pub use anvilkit_core::prelude::*;
    pub use anvilkit_render::prelude::*;
    pub use anvilkit_assets::prelude::*;
    pub use anvilkit_input::prelude::*;
    pub use anvilkit_audio::AudioPlugin;
    pub use anvilkit_audio::components::{AudioSource, AudioListener, PlaybackState, AudioBus};
    pub use anvilkit_app::prelude::{
        AnvilKitApp, GameCallbacks, GameConfig, GameContext, WindowSize,
        CursorMode, ScreenPlugin, EguiTextures,
        App, Plugin, DeltaTime, AppExt,
        AnvilKitEcsPlugin,
        AnvilKitSchedule, AnvilKitSystemSet, ScheduleBuilder, common_conditions,
        AutoInputPlugin, AutoDeltaTimePlugin,
        egui,
    };
    pub use anvilkit_describe::{Describe, ComponentSchema, FieldSchema};
    pub use crate::DefaultPlugins;

    // Re-export bevy_ecs prelude for games
    pub use bevy_ecs::prelude::*;
}
