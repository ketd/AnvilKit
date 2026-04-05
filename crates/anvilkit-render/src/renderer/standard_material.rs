//! # 标准 PBR 材质组件
//!
//! 提供高级材质抽象，无需手动创建 GPU pipeline 和 bind group。
//! 与 `MeshHandle` + `Transform` 搭配使用即可自动参与渲染。
//!
//! ## 使用示例
//!
//! ```rust
//! use anvilkit_render::renderer::standard_material::StandardMaterial;
//!
//! let material = StandardMaterial::new()
//!     .with_base_color([1.0, 0.0, 0.0, 1.0])
//!     .with_metallic(0.0)
//!     .with_roughness(0.5);
//! ```

use bevy_ecs::prelude::*;
use anvilkit_describe::Describe;
use crate::renderer::assets::MaterialHandle;

/// 默认 PBR 材质句柄资源
///
/// 在渲染初始化时创建，指向使用 1x1 白色 fallback 纹理的默认 PBR 材质。
/// `StandardMaterial` 组件的实体在没有显式 `MaterialHandle` 时使用此默认材质。
#[derive(Resource, Debug, Clone, Copy)]
pub struct DefaultMaterialHandle(pub MaterialHandle);

/// 标准 PBR 材质组件
///
/// 高级材质抽象，包含 PBR 渲染所需的所有参数。
/// 实体同时具有 `MeshHandle` + `StandardMaterial` + `Transform` 时，
/// 会被 `render_extract_system` 自动提取到绘制列表。
///
/// 使用默认 PBR 管线渲染（场景初始化时创建）。
/// 材质参数通过 `MaterialParams` 传递给 GPU uniform。
///
/// # 示例
///
/// ```rust
/// use anvilkit_render::renderer::standard_material::StandardMaterial;
///
/// // 红色金属球
/// let metal = StandardMaterial::new()
///     .with_base_color([1.0, 0.2, 0.2, 1.0])
///     .with_metallic(1.0)
///     .with_roughness(0.3);
///
/// // 白色塑料
/// let plastic = StandardMaterial::new()
///     .with_base_color([0.9, 0.9, 0.9, 1.0])
///     .with_metallic(0.0)
///     .with_roughness(0.7);
/// ```
#[derive(Debug, Clone, Component, Describe)]
/// Standard PBR material component with base color, metallic, roughness, and emissive.
pub struct StandardMaterial {
    /// 基础颜色 (linear RGBA)
    #[describe(hint = "Base color in linear RGBA", default = "[1.0, 1.0, 1.0, 1.0]")]
    pub base_color: [f32; 4],
    /// 金属度 [0.0 = 电介质, 1.0 = 金属]
    #[describe(hint = "0 = dielectric, 1 = full metal", range = "0.0..1.0", default = "0.0")]
    pub metallic: f32,
    /// 粗糙度 [0.0 = 镜面, 1.0 = 粗糙]
    #[describe(hint = "0 = mirror-smooth, 1 = fully rough", range = "0.0..1.0", default = "0.5")]
    pub roughness: f32,
    /// 法线贴图强度
    #[describe(hint = "Normal map intensity multiplier", range = "0.0..2.0", default = "1.0")]
    pub normal_scale: f32,
    /// 自发光因子 (linear RGB)
    #[describe(hint = "Emissive color factor [R,G,B]", default = "[0.0, 0.0, 0.0]")]
    pub emissive_factor: [f32; 3],
}

impl Default for StandardMaterial {
    fn default() -> Self {
        Self {
            base_color: [1.0, 1.0, 1.0, 1.0],
            metallic: 0.0,
            roughness: 0.5,
            normal_scale: 1.0,
            emissive_factor: [0.0, 0.0, 0.0],
        }
    }
}

impl StandardMaterial {
    /// 创建默认白色材质
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置基础颜色 (linear RGBA)
    pub fn with_base_color(mut self, color: [f32; 4]) -> Self {
        self.base_color = color;
        self
    }

    /// 设置金属度
    pub fn with_metallic(mut self, metallic: f32) -> Self {
        self.metallic = metallic;
        self
    }

    /// 设置粗糙度
    pub fn with_roughness(mut self, roughness: f32) -> Self {
        self.roughness = roughness;
        self
    }

    /// 设置法线贴图强度
    pub fn with_normal_scale(mut self, scale: f32) -> Self {
        self.normal_scale = scale;
        self
    }

    /// 设置自发光因子
    pub fn with_emissive(mut self, emissive: [f32; 3]) -> Self {
        self.emissive_factor = emissive;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_material() {
        let mat = StandardMaterial::default();
        assert_eq!(mat.base_color, [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(mat.metallic, 0.0);
        assert_eq!(mat.roughness, 0.5);
        assert_eq!(mat.normal_scale, 1.0);
        assert_eq!(mat.emissive_factor, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_builder_pattern() {
        let mat = StandardMaterial::new()
            .with_base_color([1.0, 0.0, 0.0, 1.0])
            .with_metallic(0.8)
            .with_roughness(0.2)
            .with_emissive([1.0, 0.5, 0.0]);

        assert_eq!(mat.base_color, [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(mat.metallic, 0.8);
        assert_eq!(mat.roughness, 0.2);
        assert_eq!(mat.emissive_factor, [1.0, 0.5, 0.0]);
    }
}
