//! # AnvilKit 音频系统
//!
//! 基于 rodio 的跨平台音频播放模块。
//!
//! ## 使用示例
//!
//! ```rust,no_run
//! use anvilkit_ecs::prelude::*;
//! use anvilkit_ecs::audio::AudioSource;
//! use anvilkit_audio::AudioPlugin;
//!
//! let mut app = App::new();
//! app.add_plugins(AudioPlugin);
//! ```

#![warn(missing_docs)]

pub mod engine;
pub mod systems;

use anvilkit_ecs::prelude::*;
use anvilkit_ecs::schedule::AnvilKitSchedule;
use engine::AudioEngine;
use systems::audio_playback_system;

/// 音频插件
///
/// 初始化 rodio 音频引擎并注册播放系统。
pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        if let Some(engine) = AudioEngine::new() {
            app.insert_resource(engine);
        }
        app.add_systems(AnvilKitSchedule::PostUpdate, audio_playback_system);
    }
}
