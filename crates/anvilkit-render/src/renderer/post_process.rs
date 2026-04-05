//! # 后处理管线配置
//!
//! 统一的后处理效果开关和参数管理。通过 `PostProcessSettings` 资源
//! 控制各效果的启用状态和参数。
//!
//! ## 使用示例
//!
//! ```rust,no_run
//! # #[cfg(feature = "advanced-render")] {
//! use anvilkit_render::renderer::post_process::PostProcessSettings;
//! use anvilkit_render::renderer::ssao::SsaoSettings;
//! use anvilkit_render::renderer::bloom::BloomSettings;
//!
//! let settings = PostProcessSettings {
//!     ssao: Some(SsaoSettings::default()),
//!     bloom: Some(BloomSettings::default()),
//!     ..Default::default()
//! };
//! # }
//! ```

use bevy_ecs::prelude::*;
use anvilkit_describe::Describe;
#[cfg(feature = "advanced-render")]
use crate::renderer::ssao::SsaoSettings;
#[cfg(feature = "advanced-render")]
use crate::renderer::dof::DofSettings;
#[cfg(feature = "advanced-render")]
use crate::renderer::motion_blur::MotionBlurSettings;
#[cfg(feature = "advanced-render")]
use crate::renderer::color_grading::ColorGradingSettings;
use crate::renderer::bloom::BloomSettings;

/// 后处理管线统一配置
///
/// 每个效果通过 `Option<Settings>` 控制：
/// - `None` = 禁用该效果
/// - `Some(settings)` = 启用并使用给定参数
///
/// 效果执行顺序（固定）：SSAO → DOF → Motion Blur → Bloom → Color Grading → Tonemap
#[derive(Resource, Default, Clone, Debug, Describe)]
/// Unified post-process pipeline configuration.
pub struct PostProcessSettings {
    /// SSAO 设置（如启用，tonemap shader 会采样 AO texture 调制环境光）
    #[cfg(feature = "advanced-render")]
    pub ssao: Option<SsaoSettings>,
    /// 景深模糊。`None` 禁用。
    #[cfg(feature = "advanced-render")]
    pub dof: Option<DofSettings>,
    /// 运动模糊。`None` 禁用。
    #[cfg(feature = "advanced-render")]
    pub motion_blur: Option<MotionBlurSettings>,
    /// Bloom 辉光。`None` 禁用。
    pub bloom: Option<BloomSettings>,
    /// 色彩分级（LUT 调色）。`None` 禁用。
    #[cfg(feature = "advanced-render")]
    pub color_grading: Option<ColorGradingSettings>,
    /// Tonemap 是否接受 AO 纹理输入
    ///
    /// 启用后，tonemap pass 的 fragment shader 会额外采样 SSAO 输出，
    /// 将环境遮蔽应用到最终颜色。需要 SSAO pass 先执行。
    pub ao_input_enabled: bool,
}

impl PostProcessSettings {
    /// 创建全部禁用的配置
    pub fn none() -> Self {
        Self::default()
    }

    /// 创建仅启用 Bloom 的配置
    pub fn bloom_only() -> Self {
        Self {
            bloom: Some(BloomSettings::default()),
            ..Default::default()
        }
    }

    /// 是否有任何效果启用
    pub fn any_enabled(&self) -> bool {
        #[allow(unused_mut)]
        let mut enabled = self.bloom.is_some();
        #[cfg(feature = "advanced-render")]
        {
            enabled = enabled
                || self.ssao.is_some()
                || self.dof.is_some()
                || self.motion_blur.is_some()
                || self.color_grading.is_some();
        }
        enabled
    }
}

/// 后处理 GPU 资源集合
///
/// 延迟创建：仅在对应效果首次启用时分配 GPU 资源。
pub struct PostProcessResources {
    /// SSAO GPU 资源（延迟初始化）
    #[cfg(feature = "advanced-render")]
    pub ssao: Option<crate::renderer::ssao::SsaoResources>,
    /// DOF GPU 资源（延迟初始化）
    #[cfg(feature = "advanced-render")]
    pub dof: Option<crate::renderer::dof::DofResources>,
    /// Motion Blur GPU 资源（延迟初始化）
    #[cfg(feature = "advanced-render")]
    pub motion_blur: Option<crate::renderer::motion_blur::MotionBlurResources>,
    /// Color Grading GPU 资源（延迟初始化）
    #[cfg(feature = "advanced-render")]
    pub color_grading: Option<crate::renderer::color_grading::ColorGradingResources>,
    /// 上一帧的 view-projection 矩阵，供 Motion Blur 使用。
    /// 首帧为 None，使用当前帧矩阵（运动模糊=0）。
    pub prev_view_proj: Option<[[f32; 4]; 4]>,
}

impl PostProcessResources {
    /// 创建空的资源集合（所有效果未初始化）
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "advanced-render")]
            ssao: None,
            #[cfg(feature = "advanced-render")]
            dof: None,
            #[cfg(feature = "advanced-render")]
            motion_blur: None,
            #[cfg(feature = "advanced-render")]
            color_grading: None,
            prev_view_proj: None,
        }
    }

    /// 根据 settings 延迟初始化/resize 需要的 GPU 资源
    pub fn ensure_resources(
        &mut self,
        device: &crate::renderer::RenderDevice,
        width: u32,
        height: u32,
        settings: &PostProcessSettings,
    ) {
        // 未启用 advanced-render 时，无 post-process 资源需要创建
        let _ = (device, width, height, settings);

        // SSAO
        #[cfg(feature = "advanced-render")]
        if settings.ssao.is_some() {
            if self.ssao.is_none() {
                self.ssao = Some(crate::renderer::ssao::SsaoResources::new(device, width, height, 1));
            }
        }

        // DOF
        #[cfg(feature = "advanced-render")]
        if settings.dof.is_some() {
            if self.dof.is_none() {
                self.dof = Some(crate::renderer::dof::DofResources::new(device, width, height));
            }
        }

        // Motion Blur
        #[cfg(feature = "advanced-render")]
        if settings.motion_blur.is_some() {
            if self.motion_blur.is_none() {
                self.motion_blur = Some(crate::renderer::motion_blur::MotionBlurResources::new(device, width, height));
            }
        }

        // Color Grading
        #[cfg(feature = "advanced-render")]
        if settings.color_grading.is_some() {
            if self.color_grading.is_none() {
                self.color_grading = Some(crate::renderer::color_grading::ColorGradingResources::new(device));
            }
        }
    }

    /// Resize 所有已创建的资源
    pub fn resize(&mut self, device: &crate::renderer::RenderDevice, width: u32, height: u32) {
        let _ = (device, width, height);

        #[cfg(feature = "advanced-render")]
        {
            if let Some(ref mut ssao) = self.ssao {
                ssao.resize(device, width, height);
            }
            if let Some(ref mut dof) = self.dof {
                dof.resize(device, width, height);
            }
            if let Some(ref mut mb) = self.motion_blur {
                mb.resize(device, width, height);
            }
            if let Some(ref mut cg) = self.color_grading {
                cg.resize(device, width, height);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_all_disabled() {
        let settings = PostProcessSettings::default();
        assert!(!settings.any_enabled());
        #[cfg(feature = "advanced-render")]
        assert!(settings.ssao.is_none());
        assert!(settings.bloom.is_none());
    }

    #[test]
    fn test_bloom_only() {
        let settings = PostProcessSettings::bloom_only();
        assert!(settings.any_enabled());
        assert!(settings.bloom.is_some());
        #[cfg(feature = "advanced-render")]
        assert!(settings.ssao.is_none());
    }

    #[cfg(feature = "advanced-render")]
    #[test]
    fn test_full_pipeline() {
        let settings = PostProcessSettings {
            ssao: Some(SsaoSettings::default()),
            dof: Some(DofSettings::default()),
            motion_blur: Some(MotionBlurSettings::default()),
            bloom: Some(BloomSettings::default()),
            color_grading: Some(ColorGradingSettings::default()),
            ao_input_enabled: false,
        };
        assert!(settings.any_enabled());
    }
}
