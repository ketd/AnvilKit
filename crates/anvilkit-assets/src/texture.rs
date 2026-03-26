//! # 独立纹理加载
//!
//! 直接从 PNG/JPEG 文件加载纹理数据到 `TextureData`，不依赖 glTF 容器。
//!
//! ## 使用示例
//!
//! ```rust,no_run
//! use anvilkit_assets::texture::load_texture;
//!
//! let texture = load_texture("assets/textures/grass.png").expect("加载纹理失败");
//! println!("纹理尺寸: {}x{}, 数据大小: {} bytes", texture.width, texture.height, texture.data.len());
//! ```

use std::path::Path;
use crate::material::TextureData;
use anvilkit_core::error::{AnvilKitError, Result};

/// 从文件加载纹理数据
///
/// 支持 PNG 和 JPEG 格式。所有格式统一转换为 RGBA8（每像素 4 字节）。
///
/// # 参数
///
/// - `path`: 纹理文件路径
///
/// # 返回
///
/// `TextureData` 包含宽度、高度和 RGBA8 像素数据。
///
/// # 示例
///
/// ```rust,no_run
/// use anvilkit_assets::texture::load_texture;
///
/// let tex = load_texture("assets/wall.png").unwrap();
/// assert_eq!(tex.data.len(), (tex.width * tex.height * 4) as usize);
/// ```
pub fn load_texture(path: impl AsRef<Path>) -> Result<TextureData> {
    let path = path.as_ref();
    let img = image::open(path).map_err(|e| {
        AnvilKitError::asset(format!("无法加载纹理 {:?}: {}", path, e))
    })?;

    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();

    Ok(TextureData {
        width,
        height,
        data: rgba.into_raw(),
    })
}

/// 从内存字节解码纹理数据
///
/// 自动检测格式（PNG/JPEG 等），转换为 RGBA8。
///
/// # 参数
///
/// - `bytes`: 图片文件的原始字节
///
/// # 返回
///
/// `TextureData` 包含宽度、高度和 RGBA8 像素数据。
pub fn load_texture_from_memory(bytes: &[u8]) -> Result<TextureData> {
    let img = image::load_from_memory(bytes).map_err(|e| {
        AnvilKitError::asset(format!("无法解码纹理: {}", e))
    })?;

    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();

    Ok(TextureData {
        width,
        height,
        data: rgba.into_raw(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_nonexistent_file() {
        let result = load_texture("nonexistent.png");
        assert!(result.is_err());
    }

    #[test]
    fn test_load_from_memory_png() {
        // 使用 image crate 生成 1x1 PNG 内存数据
        use image::{RgbaImage, Rgba};
        let img = RgbaImage::from_pixel(1, 1, Rgba([255, 0, 0, 255]));
        let mut buf = std::io::Cursor::new(Vec::new());
        img.write_to(&mut buf, image::ImageFormat::Png).unwrap();
        let png_bytes = buf.into_inner();

        let result = load_texture_from_memory(&png_bytes);
        assert!(result.is_ok());
        let tex = result.unwrap();
        assert_eq!(tex.width, 1);
        assert_eq!(tex.height, 1);
        assert_eq!(tex.data.len(), 4); // 1 pixel * 4 bytes (RGBA)
        assert_eq!(tex.data, vec![255, 0, 0, 255]); // red
    }

    #[test]
    fn test_load_from_memory_invalid() {
        let result = load_texture_from_memory(b"not a valid image");
        assert!(result.is_err());
    }
}
