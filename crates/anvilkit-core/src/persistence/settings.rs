//! # 玩家设置持久化
//!
//! 基于 RON 格式的类型化设置系统。

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// 图形设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphicsSettings {
    /// Window width.
    pub width: u32,
    /// Window height.
    pub height: u32,
    /// Fullscreen mode.
    pub fullscreen: bool,
    /// VSync enabled.
    pub vsync: bool,
    /// MSAA sample count (1, 2, 4).
    pub msaa: u32,
    /// Bloom enabled.
    pub bloom: bool,
    /// SSAO enabled.
    pub ssao: bool,
    /// Shadow quality (0=off, 1=low, 2=medium, 3=high).
    pub shadow_quality: u32,
}

impl Default for GraphicsSettings {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 720,
            fullscreen: false,
            vsync: true,
            msaa: 4,
            bloom: true,
            ssao: true,
            shadow_quality: 2,
        }
    }
}

/// 音频设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSettings {
    /// Master volume (0.0 - 1.0).
    pub master_volume: f32,
    /// Music volume (0.0 - 1.0).
    pub music_volume: f32,
    /// SFX volume (0.0 - 1.0).
    pub sfx_volume: f32,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            music_volume: 0.8,
            sfx_volume: 1.0,
        }
    }
}

/// 输入设置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InputSettings {
    /// Mouse sensitivity.
    pub mouse_sensitivity: f32,
    /// Invert Y axis.
    pub invert_y: bool,
    /// Action key overrides: action_name → key_name.
    pub action_overrides: HashMap<String, String>,
}

/// 游戏设置（所有分区）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Graphics configuration.
    pub graphics: GraphicsSettings,
    /// Audio configuration.
    pub audio: AudioSettings,
    /// Input configuration.
    pub input: InputSettings,
    /// Game-specific custom settings (RON values stored as strings).
    pub custom: HashMap<String, String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            graphics: GraphicsSettings::default(),
            audio: AudioSettings::default(),
            input: InputSettings {
                mouse_sensitivity: 0.003,
                invert_y: false,
                action_overrides: HashMap::new(),
            },
            custom: HashMap::new(),
        }
    }
}

impl Settings {
    /// 从 RON 文件加载设置。文件不存在时返回默认值。
    pub fn load(path: &Path) -> Self {
        match std::fs::read_to_string(path) {
            Ok(data) => ron::from_str(&data).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// 保存设置到 RON 文件。
    pub fn save(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create settings dir: {}", e))?;
        }
        let ron_str = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
            .map_err(|e| format!("Settings serialize failed: {}", e))?;
        std::fs::write(path, ron_str)
            .map_err(|e| format!("Failed to write settings: {}", e))?;
        Ok(())
    }

    /// 默认设置文件路径: config/settings.ron
    pub fn default_path() -> PathBuf {
        PathBuf::from("config/settings.ron")
    }
}
