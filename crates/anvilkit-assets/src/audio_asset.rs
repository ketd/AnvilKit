//! # Audio Asset
//!
//! 音频资产类型，用于 AssetServer 集成。

/// 音频资产数据
///
/// 存储从文件加载的原始音频字节，供 rodio 解码器使用。
#[derive(Debug, Clone)]
pub struct AudioAsset {
    /// 音频文件原始字节
    pub bytes: Vec<u8>,
    /// 文件路径（用于调试）
    pub path: String,
}

impl AudioAsset {
    /// 创建新的音频资产
    pub fn new(bytes: Vec<u8>, path: impl Into<String>) -> Self {
        Self {
            bytes,
            path: path.into(),
        }
    }

    /// 获取音频数据的 BufReader（供 rodio::Decoder 使用）
    pub fn cursor(&self) -> std::io::Cursor<Vec<u8>> {
        std::io::Cursor::new(self.bytes.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_asset_creation() {
        let asset = AudioAsset::new(vec![1, 2, 3], "test.wav");
        assert_eq!(asset.bytes.len(), 3);
        assert_eq!(asset.path, "test.wav");
    }

    #[test]
    fn test_audio_asset_cursor() {
        let asset = AudioAsset::new(vec![0xFF, 0xFE], "test.ogg");
        let cursor = asset.cursor();
        assert_eq!(cursor.into_inner().len(), 2);
    }
}
