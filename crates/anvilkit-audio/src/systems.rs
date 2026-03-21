//! # 音频播放系统
//!
//! ECS 系统：监听 AudioSource 组件状态变化，驱动 rodio 播放。

use bevy_ecs::prelude::*;
use anvilkit_ecs::audio::{AudioSource, PlaybackState};
use log::{debug, error};
use std::io::BufReader;
use std::fs::File;

use crate::engine::AudioEngine;

/// 音频播放状态追踪组件
#[derive(Component)]
pub struct AudioPlaybackTracker {
    pub last_state: PlaybackState,
}

impl Default for AudioPlaybackTracker {
    fn default() -> Self {
        Self {
            last_state: PlaybackState::Stopped,
        }
    }
}

/// 音频播放系统
///
/// 检测 AudioSource 状态变化并驱动 rodio 播放。
pub fn audio_playback_system(
    mut commands: Commands,
    query: Query<(Entity, &AudioSource, Option<&AudioPlaybackTracker>)>,
    engine: Option<ResMut<AudioEngine>>,
) {
    let Some(mut engine) = engine else { return };

    for (entity, source, tracker) in query.iter() {
        let last_state = tracker.map(|t| t.last_state).unwrap_or(PlaybackState::Stopped);

        if source.state == last_state {
            continue;
        }

        match source.state {
            PlaybackState::Playing if last_state != PlaybackState::Playing => {
                if last_state == PlaybackState::Paused {
                    engine.resume(entity);
                } else {
                    // Start new playback
                    match File::open(&source.path) {
                        Ok(file) => {
                            let reader = BufReader::new(file);
                            match rodio::Decoder::new(reader) {
                                Ok(decoder) => {
                                    match engine.get_or_create_sink(entity) {
                                        Ok(sink) => {
                                            sink.set_volume(source.volume);
                                            sink.append(decoder);
                                            debug!("播放音频: {}", source.path);
                                        }
                                        Err(e) => error!("创建 sink 失败 {}: {}", source.path, e),
                                    }
                                }
                                Err(e) => error!("解码音频失败 {}: {}", source.path, e),
                            }
                        }
                        Err(e) => error!("打开音频文件失败 {}: {}", source.path, e),
                    }
                }
            }
            PlaybackState::Paused => {
                engine.pause(entity);
            }
            PlaybackState::Stopped => {
                engine.stop(entity);
            }
            _ => {}
        }

        // Update tracker
        commands.entity(entity).insert(AudioPlaybackTracker {
            last_state: source.state,
        });
    }

    engine.cleanup_finished();
}
