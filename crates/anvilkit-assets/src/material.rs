//! # 材质和纹理数据
//!
//! 定义从 glTF 文件提取的材质和纹理数据结构。

/// CPU 侧纹理数据
///
/// 包含从 glTF 文件提取的 RGBA 图像数据。
///
/// # 示例
///
/// ```rust
/// use anvilkit_assets::material::TextureData;
///
/// let texture = TextureData {
///     width: 256,
///     height: 256,
///     data: vec![255u8; 256 * 256 * 4],
/// };
/// assert_eq!(texture.width, 256);
/// ```
#[derive(Debug, Clone)]
pub struct TextureData {
    /// 图像宽度（像素）
    pub width: u32,
    /// 图像高度（像素）
    pub height: u32,
    /// RGBA 像素数据（每像素 4 字节）
    pub data: Vec<u8>,
}

/// CPU 侧材质数据
///
/// 包含完整 PBR 材质属性：基础色、法线、金属度/粗糙度、AO、自发光。
///
/// # 示例
///
/// ```rust
/// use anvilkit_assets::material::MaterialData;
///
/// let mat = MaterialData::default();
/// assert_eq!(mat.base_color_factor, [1.0, 1.0, 1.0, 1.0]);
/// assert_eq!(mat.metallic_factor, 1.0);
/// assert_eq!(mat.roughness_factor, 1.0);
/// assert_eq!(mat.normal_scale, 1.0);
/// assert_eq!(mat.emissive_factor, [0.0, 0.0, 0.0]);
/// ```
#[derive(Debug, Clone)]
pub struct MaterialData {
    /// 基础色纹理（可选）
    pub base_color_texture: Option<TextureData>,
    /// 基础色因子 [R, G, B, A]（无纹理时作为纯色使用）
    pub base_color_factor: [f32; 4],
    /// 金属度因子 [0.0 = 非金属, 1.0 = 完全金属]
    pub metallic_factor: f32,
    /// 粗糙度因子 [0.0 = 光滑镜面, 1.0 = 完全粗糙]
    pub roughness_factor: f32,
    /// 法线贴图纹理（可选，tangent-space）
    pub normal_texture: Option<TextureData>,
    /// 法线贴图强度缩放 [default=1.0]
    pub normal_scale: f32,
    /// 金属度/粗糙度纹理（可选，glTF: G=roughness, B=metallic）
    pub metallic_roughness_texture: Option<TextureData>,
    /// 环境光遮蔽纹理（可选，R 通道）
    pub occlusion_texture: Option<TextureData>,
    /// 自发光纹理（可选）
    pub emissive_texture: Option<TextureData>,
    /// 自发光因子 [R, G, B]
    pub emissive_factor: [f32; 3],
    /// 双面渲染（glTF doubleSided → 禁用背面剔除）
    pub double_sided: bool,
}

impl Default for MaterialData {
    fn default() -> Self {
        Self {
            base_color_texture: None,
            base_color_factor: [1.0, 1.0, 1.0, 1.0],
            metallic_factor: 1.0,
            roughness_factor: 1.0,
            normal_texture: None,
            normal_scale: 1.0,
            metallic_roughness_texture: None,
            occlusion_texture: None,
            emissive_texture: None,
            emissive_factor: [0.0, 0.0, 0.0],
            double_sided: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_texture_data() {
        let tex = TextureData {
            width: 2,
            height: 2,
            data: vec![255; 16],
        };
        assert_eq!(tex.data.len(), (tex.width * tex.height * 4) as usize);
    }

    #[test]
    fn test_material_default() {
        let mat = MaterialData::default();
        assert_eq!(mat.base_color_factor, [1.0, 1.0, 1.0, 1.0]);
        assert!(mat.base_color_texture.is_none());
        assert!(mat.normal_texture.is_none());
        assert_eq!(mat.normal_scale, 1.0);
        assert!(mat.metallic_roughness_texture.is_none());
        assert!(mat.occlusion_texture.is_none());
        assert!(mat.emissive_texture.is_none());
        assert_eq!(mat.emissive_factor, [0.0, 0.0, 0.0]);
    }
}
