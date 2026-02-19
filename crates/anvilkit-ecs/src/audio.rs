//! # 音频组件
//!
//! 定义 ECS 音频组件类型，用于与 kira 音频引擎集成。
//! 组件定义不依赖 kira，实际音频播放由 AudioPlugin 提供。
//!
//! ## 使用示例
//!
//! ```rust
//! use anvilkit_ecs::audio::{AudioSource, AudioListener, PlaybackState};
//!
//! let source = AudioSource::new("sounds/explosion.ogg");
//! assert_eq!(source.volume, 1.0);
//! assert_eq!(source.state, PlaybackState::Stopped);
//!
//! let listener = AudioListener::default();
//! assert!(listener.is_active);
//! ```

use bevy_ecs::prelude::*;
/// 播放状态
///
/// # 示例
///
/// ```rust
/// use anvilkit_ecs::audio::PlaybackState;
/// let state = PlaybackState::Playing;
/// assert_ne!(state, PlaybackState::Stopped);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
}

/// 音频源组件
///
/// 附加到实体上表示该实体可播放音频。
///
/// # 示例
///
/// ```rust
/// use anvilkit_ecs::audio::AudioSource;
///
/// let mut source = AudioSource::new("music/bgm.ogg");
/// source.volume = 0.8;
/// source.looping = true;
/// source.spatial = true;
/// ```
#[derive(Debug, Clone, Component)]
pub struct AudioSource {
    /// 音频文件路径
    pub path: String,
    /// 音量 [0.0, 1.0+]
    pub volume: f32,
    /// 播放速率（1.0 = 正常）
    pub pitch: f32,
    /// 是否循环
    pub looping: bool,
    /// 是否空间化音频（3D 定位）
    pub spatial: bool,
    /// 空间音频衰减距离
    pub spatial_range: f32,
    /// 当前播放状态
    pub state: PlaybackState,
}

impl AudioSource {
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
            volume: 1.0,
            pitch: 1.0,
            looping: false,
            spatial: false,
            spatial_range: 20.0,
            state: PlaybackState::Stopped,
        }
    }

    /// 标记为播放请求
    pub fn play(&mut self) { self.state = PlaybackState::Playing; }

    /// 标记为暂停
    pub fn pause(&mut self) { self.state = PlaybackState::Paused; }

    /// 标记为停止
    pub fn stop(&mut self) { self.state = PlaybackState::Stopped; }
}

/// 音频监听器组件
///
/// 附加到相机或玩家实体上，表示 3D 音频的收听位置。
///
/// # 示例
///
/// ```rust
/// use anvilkit_ecs::audio::AudioListener;
///
/// let listener = AudioListener::default();
/// assert!(listener.is_active);
/// ```
#[derive(Debug, Clone, Component)]
pub struct AudioListener {
    /// 是否激活（场景中应只有一个激活的 listener）
    pub is_active: bool,
}

impl Default for AudioListener {
    fn default() -> Self {
        Self { is_active: true }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_source() {
        let mut src = AudioSource::new("test.ogg");
        assert_eq!(src.state, PlaybackState::Stopped);
        assert_eq!(src.volume, 1.0);
        assert!(!src.looping);

        src.play();
        assert_eq!(src.state, PlaybackState::Playing);

        src.pause();
        assert_eq!(src.state, PlaybackState::Paused);

        src.stop();
        assert_eq!(src.state, PlaybackState::Stopped);
    }

    #[test]
    fn test_audio_listener() {
        let listener = AudioListener::default();
        assert!(listener.is_active);
    }
}
