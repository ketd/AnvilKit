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
    /// Audio is stopped (not playing).
    Stopped,
    /// Audio is currently playing.
    Playing,
    /// Audio is paused and can be resumed.
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
    /// 音频总线分类（用于混音音量控制）
    pub bus: AudioBusCategory,
}

impl AudioSource {
    /// Creates a new audio source with the given file path and default settings.
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
            volume: 1.0,
            pitch: 1.0,
            looping: false,
            spatial: false,
            spatial_range: 20.0,
            state: PlaybackState::Stopped,
            bus: AudioBusCategory::SFX,
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

/// 音频总线分类
///
/// 用于将音频源分组到不同的混音通道。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AudioBusCategory {
    /// 音效（默认）
    #[default]
    SFX,
    /// 背景音乐
    Music,
    /// 语音/对话
    Voice,
}

/// 音频总线资源 — 主音量 + 分类音量
///
/// # 示例
///
/// ```rust
/// use anvilkit_ecs::audio::AudioBus;
///
/// let bus = AudioBus::default();
/// assert_eq!(bus.master, 1.0);
/// ```
#[derive(Resource, Debug, Clone)]
pub struct AudioBus {
    /// 全局主音量 [0.0, 1.0]
    pub master: f32,
    /// 音乐音量 [0.0, 1.0]
    pub music: f32,
    /// 音效音量 [0.0, 1.0]
    pub sfx: f32,
    /// 语音音量 [0.0, 1.0]
    pub voice: f32,
}

impl Default for AudioBus {
    fn default() -> Self {
        Self { master: 1.0, music: 1.0, sfx: 1.0, voice: 1.0 }
    }
}

impl AudioBus {
    /// 获取指定分类的有效音量（含主音量）
    pub fn effective_volume(&self, category: AudioBusCategory) -> f32 {
        let cat_vol = match category {
            AudioBusCategory::SFX => self.sfx,
            AudioBusCategory::Music => self.music,
            AudioBusCategory::Voice => self.voice,
        };
        self.master * cat_vol
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

    #[test]
    fn test_audio_bus_effective_volume() {
        let mut bus = AudioBus::default();
        assert_eq!(bus.effective_volume(AudioBusCategory::SFX), 1.0);

        bus.master = 0.5;
        bus.sfx = 0.8;
        let vol = bus.effective_volume(AudioBusCategory::SFX);
        assert!((vol - 0.4).abs() < 0.001);

        bus.master = 0.0;
        assert_eq!(bus.effective_volume(AudioBusCategory::Music), 0.0);
    }

    #[test]
    fn test_audio_bus_category_default() {
        let src = AudioSource::new("test.ogg");
        assert_eq!(src.bus, AudioBusCategory::SFX);
    }

    #[test]
    fn test_audio_source_looping_default() {
        let source = AudioSource::new("test.ogg");
        assert!(!source.looping);
        assert!((source.pitch - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_audio_source_looping_and_pitch() {
        let mut source = AudioSource::new("sound.wav");
        source.looping = true;
        source.pitch = 2.0;
        assert!(source.looping);
        assert!((source.pitch - 2.0).abs() < 0.001);
    }
}
