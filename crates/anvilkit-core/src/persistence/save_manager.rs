//! # 存档管理器
//!
//! 管理多个存档槽位，支持元数据、列举和自动存档。

use serde::{Serialize, Deserialize};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// 存档槽位信息（元数据，不含完整数据）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveSlotInfo {
    /// 槽位名称 (e.g., "quick", "slot_1", "_autosave").
    pub name: String,
    /// 存档创建时的 Unix 时间戳（秒）。
    pub timestamp: u64,
    /// 累计游玩时长（秒）。
    pub play_time_secs: f64,
    /// 游戏版本字符串。
    pub game_version: String,
    /// 游戏特定的元数据。
    pub metadata: std::collections::HashMap<String, String>,
}

/// 存档管理器
///
/// 在指定目录中管理多个存档槽位。每个槽位包含一个 `meta.ron` 元数据文件
/// 和一个 `data` 目录供 `WorldStorage` 使用。
///
/// # 目录结构
/// ```text
/// saves/
/// ├── quick/
/// │   ├── meta.ron
/// │   └── data/       ← WorldStorage root
/// ├── slot_1/
/// │   ├── meta.ron
/// │   └── data/
/// └── _autosave/
///     ├── meta.ron
///     └── data/
/// ```
pub struct SaveManager {
    saves_dir: PathBuf,
    game_version: String,
}

impl SaveManager {
    /// 创建存档管理器。
    pub fn new(saves_dir: impl AsRef<Path>, game_version: &str) -> Result<Self, String> {
        let saves_dir = saves_dir.as_ref().to_path_buf();
        std::fs::create_dir_all(&saves_dir)
            .map_err(|e| format!("Failed to create saves dir: {}", e))?;
        Ok(Self {
            saves_dir,
            game_version: game_version.to_string(),
        })
    }

    /// 列出所有可用存档的元数据。
    pub fn list_saves(&self) -> Vec<SaveSlotInfo> {
        let mut saves = Vec::new();
        let entries = match std::fs::read_dir(&self.saves_dir) {
            Ok(e) => e,
            Err(_) => return saves,
        };
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                let meta_path = entry.path().join("meta.ron");
                if let Ok(data) = std::fs::read_to_string(&meta_path) {
                    if let Ok(info) = ron::from_str::<SaveSlotInfo>(&data) {
                        saves.push(info);
                    }
                }
            }
        }
        saves.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        saves
    }

    /// 写入存档元数据。返回槽位的 data 目录路径（供 WorldStorage 使用）。
    pub fn save(
        &self,
        slot_name: &str,
        play_time_secs: f64,
        metadata: std::collections::HashMap<String, String>,
    ) -> Result<PathBuf, String> {
        let slot_dir = self.saves_dir.join(slot_name);
        let data_dir = slot_dir.join("data");
        std::fs::create_dir_all(&data_dir)
            .map_err(|e| format!("Failed to create save slot dir: {}", e))?;

        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let info = SaveSlotInfo {
            name: slot_name.to_string(),
            timestamp,
            play_time_secs,
            game_version: self.game_version.clone(),
            metadata,
        };

        let ron_str = ron::ser::to_string_pretty(&info, ron::ser::PrettyConfig::default())
            .map_err(|e| format!("Save meta serialize failed: {}", e))?;
        std::fs::write(slot_dir.join("meta.ron"), ron_str)
            .map_err(|e| format!("Failed to write save meta: {}", e))?;

        Ok(data_dir)
    }

    /// 获取存档槽位的 data 目录路径（用于 WorldStorage::open）。
    pub fn slot_data_path(&self, slot_name: &str) -> PathBuf {
        self.saves_dir.join(slot_name).join("data")
    }

    /// 获取存档槽位的元数据。
    pub fn get_save_info(&self, slot_name: &str) -> Option<SaveSlotInfo> {
        let meta_path = self.saves_dir.join(slot_name).join("meta.ron");
        let data = std::fs::read_to_string(&meta_path).ok()?;
        ron::from_str(&data).ok()
    }

    /// 删除存档槽位（包括所有数据）。
    pub fn delete(&self, slot_name: &str) -> Result<(), String> {
        let slot_dir = self.saves_dir.join(slot_name);
        if slot_dir.exists() {
            std::fs::remove_dir_all(&slot_dir)
                .map_err(|e| format!("Failed to delete save '{}': {}", slot_name, e))?;
        }
        Ok(())
    }

    /// 存档根目录路径。
    pub fn saves_dir(&self) -> &Path {
        &self.saves_dir
    }
}
