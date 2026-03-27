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
//! use anvilkit::ecs::App;
//! use anvilkit::render::WindowConfig;
//! use anvilkit::assets::MeshData;
//! ```

pub use anvilkit_core as core;
pub use anvilkit_ecs as ecs;
pub use anvilkit_render as render;
pub use anvilkit_assets as assets;
pub use anvilkit_input as input;
pub use anvilkit_audio as audio;
pub use anvilkit_camera as camera;
pub use anvilkit_app as app;
pub use anvilkit_ui as ui;
pub use anvilkit_gameplay as gameplay;

pub mod default_plugins;
pub use default_plugins::DefaultPlugins;

/// Convenient re-exports of the most commonly used types and traits.
pub mod prelude {
    pub use anvilkit_core::prelude::*;
    pub use anvilkit_ecs::prelude::*;
    pub use anvilkit_render::prelude::*;
    pub use anvilkit_assets::prelude::*;
    pub use anvilkit_input::prelude::*;
    pub use anvilkit_camera::prelude::*;
    pub use anvilkit_audio::AudioPlugin;
    pub use anvilkit_ecs::audio::{AudioSource, AudioListener, PlaybackState, AudioBus};
    pub use anvilkit_app::prelude::*;
    pub use crate::DefaultPlugins;
}
