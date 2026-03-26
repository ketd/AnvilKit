//! # Parsed Asset
//!
//! 统一的解析结果类型，用于 AssetServer 后台解析。

use crate::mesh::MeshData;
use crate::material::TextureData;
use crate::audio_asset::AudioAsset;

/// 解析后的资产数据
///
/// AssetServer 在 worker thread 中将原始字节解析为具体类型，
/// main thread 的 `process_completed` 只需插入到对应 storage。
#[derive(Debug)]
pub enum ParsedAsset {
    /// 网格数据（来自 glTF）
    Mesh(Vec<MeshData>),
    /// 纹理数据（来自 PNG/JPEG）
    Texture(TextureData),
    /// 音频数据（原始字节）
    Audio(AudioAsset),
    /// 原始字节（通用格式）
    Raw(Vec<u8>),
}

impl ParsedAsset {
    /// 尝试获取网格数据
    pub fn as_meshes(&self) -> Option<&[MeshData]> {
        match self {
            ParsedAsset::Mesh(meshes) => Some(meshes),
            _ => None,
        }
    }

    /// 尝试获取纹理数据
    pub fn as_texture(&self) -> Option<&TextureData> {
        match self {
            ParsedAsset::Texture(tex) => Some(tex),
            _ => None,
        }
    }

    /// 尝试获取音频数据
    pub fn as_audio(&self) -> Option<&AudioAsset> {
        match self {
            ParsedAsset::Audio(audio) => Some(audio),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parsed_asset_raw() {
        let asset = ParsedAsset::Raw(vec![1, 2, 3]);
        assert!(asset.as_meshes().is_none());
        assert!(asset.as_texture().is_none());
    }

    #[test]
    fn test_parsed_asset_audio() {
        let audio = AudioAsset::new(vec![0xFF], "test.wav");
        let asset = ParsedAsset::Audio(audio);
        assert!(asset.as_audio().is_some());
        assert_eq!(asset.as_audio().unwrap().path, "test.wav");
    }
}
