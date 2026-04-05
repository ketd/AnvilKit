//! # 音频播放系统
//!
//! ECS 系统：监听 AudioSource 组件状态变化，驱动 rodio 播放。

use bevy_ecs::prelude::*;
use crate::components::{AudioSource, PlaybackState, AudioListener, AudioBus};
use anvilkit_core::math::Transform;
use log::{debug, error};
use std::io::BufReader;
use std::fs::File;
use rodio::Source;

use crate::engine::AudioEngine;

/// 音频播放状态追踪组件
#[derive(Component)]
pub struct AudioPlaybackTracker {
    /// The playback state observed on the previous frame.
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
    engine: Option<NonSendMut<AudioEngine>>,
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
                                            sink.set_speed(source.pitch);
                                            if source.looping {
                                                let buffered = decoder.buffered();
                                                sink.append(buffered.repeat_infinite());
                                            } else {
                                                sink.append(decoder);
                                            }
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

/// 音频清理系统 — 移除已 despawn 实体的 Sink，防止泄漏。
///
/// 通过 `RemovedComponents<AudioSource>` 检测实体移除事件。
pub fn audio_cleanup_system(
    mut removed: RemovedComponents<AudioSource>,
    engine: Option<NonSendMut<AudioEngine>>,
) {
    let Some(mut engine) = engine else { return };
    for entity in removed.read() {
        engine.stop(entity);
        debug!("清理已移除实体的音频 sink: {:?}", entity);
    }
}

/// 空间音频系统 — 基于距离的音量衰减 + 立体声平移
pub fn spatial_audio_system(
    query: Query<(Entity, &AudioSource, &Transform)>,
    listener_query: Query<&Transform, With<AudioListener>>,
    engine: Option<NonSend<AudioEngine>>,
    bus: Option<Res<AudioBus>>,
) {
    let Some(engine) = engine else { return };
    let default_bus = AudioBus::default();
    let bus = bus.as_deref().unwrap_or(&default_bus);

    let listener_transform = listener_query.iter().next().copied()
        .unwrap_or(Transform::IDENTITY);
    let listener_pos = listener_transform.translation;
    // Listener's right vector for stereo panning
    let listener_right = listener_transform.rotation * glam::Vec3::X;

    for (entity, source, transform) in query.iter() {
        if source.state != PlaybackState::Playing { continue; }

        let bus_vol = bus.effective_volume(source.bus);
        let effective_vol = if source.spatial && source.spatial_range > 0.0 {
            let distance = (transform.translation - listener_pos).length();
            let attenuation = (1.0 - distance / source.spatial_range).max(0.0);
            source.volume * attenuation * bus_vol
        } else {
            source.volume * bus_vol
        };

        // Stereo panning: project source direction onto listener's right axis.
        // pan in [-1, 1]: -1 = full left, 0 = center, +1 = full right
        let _panning = if source.spatial {
            let offset = transform.translation - listener_pos;
            let len = offset.length();
            if len > 1e-5 {
                let dir = offset / len;
                // Dot with right vector gives signed horizontal displacement
                dir.dot(listener_right).clamp(-1.0, 1.0)
            } else {
                0.0 // source at listener position → center
            }
        } else {
            0.0
        };

        // Derive per-channel volumes from the panning value.
        // Equal-power-ish linear pan law:
        //   left  = (1 - pan) * 0.5 * volume
        //   right = (1 + pan) * 0.5 * volume
        let _left_vol  = (1.0 - _panning) * 0.5 * effective_vol;
        let _right_vol = (1.0 + _panning) * 0.5 * effective_vol;

        // NOTE: rodio 0.19 `Sink` does not expose a stereo panning API.
        // When rodio gains `set_stereo_volume` or equivalent, replace the
        // single `set_volume` call below with per-channel volumes.
        // For now, apply the distance-attenuated mono volume.
        engine.set_volume(entity, effective_vol);
    }
}

#[cfg(test)]
mod tests {
    /// 距离衰减计算单元测试（与 spatial_audio_system 中的逻辑一致）
    #[test]
    fn test_distance_attenuation() {
        let spatial_range = 20.0_f32;

        // At distance 0: full volume
        let atten_0 = (1.0 - 0.0 / spatial_range).max(0.0);
        assert!((atten_0 - 1.0).abs() < f32::EPSILON);

        // At half range: half volume
        let atten_half = (1.0 - 10.0 / spatial_range).max(0.0);
        assert!((atten_half - 0.5).abs() < f32::EPSILON);

        // At full range: zero volume
        let atten_full = (1.0 - 20.0 / spatial_range).max(0.0);
        assert!((atten_full - 0.0).abs() < f32::EPSILON);

        // Beyond range: clamped to zero
        let atten_beyond = (1.0 - 30.0 / spatial_range).max(0.0);
        assert!((atten_beyond - 0.0).abs() < f32::EPSILON);
    }

    /// 立体声 panning 计算单元测试
    #[test]
    fn test_stereo_panning_calculation() {
        use glam::Vec3;

        let listener_right = Vec3::X; // facing +Z, right is +X

        // Source to the right
        let dir_right = Vec3::X;
        let pan_right = dir_right.dot(listener_right).clamp(-1.0, 1.0);
        assert!((pan_right - 1.0).abs() < f32::EPSILON);

        // Source to the left
        let dir_left = Vec3::NEG_X;
        let pan_left = dir_left.dot(listener_right).clamp(-1.0, 1.0);
        assert!((pan_left - (-1.0)).abs() < f32::EPSILON);

        // Source directly ahead
        let dir_ahead = Vec3::Z;
        let pan_ahead = dir_ahead.dot(listener_right).clamp(-1.0, 1.0);
        assert!((pan_ahead - 0.0).abs() < f32::EPSILON);
    }
}
