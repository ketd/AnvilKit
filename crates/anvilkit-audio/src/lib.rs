//! # AnvilKit 音频系统
//!
//! 基于 rodio 的跨平台音频播放模块。
//!
//! ## 使用示例
//!
//! ```rust,no_run
//! use bevy_app::App;
//! use anvilkit_audio::components::AudioSource;
//! use anvilkit_audio::AudioPlugin;
//!
//! let mut app = App::new();
//! app.add_plugins(AudioPlugin);
//! ```

#![warn(missing_docs)]

pub mod engine;
pub mod systems;
pub mod components;

use bevy_ecs::prelude::*;
use bevy_app::{App, Plugin};
use engine::AudioEngine;
use systems::{audio_playback_system, audio_cleanup_system, spatial_audio_system};

/// 音频插件
///
/// 初始化 rodio 音频引擎并注册播放系统。
pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        if let Some(engine) = AudioEngine::new() {
            app.insert_non_send_resource(engine);
        }
        app.add_systems(bevy_app::PostUpdate, (
            audio_playback_system,
            audio_cleanup_system.after(audio_playback_system),
            spatial_audio_system.after(audio_playback_system),
        ));
    }
}
