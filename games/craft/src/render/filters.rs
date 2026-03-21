use bevy_ecs::prelude::*;
use bytemuck::{Pod, Zeroable};

/// Post-processing filter types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostFilter {
    None = 0,
    Underwater = 1,
    Vignette = 2,
    NightVision = 3,
}

impl PostFilter {
    pub fn cycle(self) -> Self {
        match self {
            PostFilter::None => PostFilter::Underwater,
            PostFilter::Underwater => PostFilter::Vignette,
            PostFilter::Vignette => PostFilter::NightVision,
            PostFilter::NightVision => PostFilter::None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            PostFilter::None => "None",
            PostFilter::Underwater => "Underwater",
            PostFilter::Vignette => "Vignette",
            PostFilter::NightVision => "Night Vision",
        }
    }
}

/// GPU uniform for filter parameters (passed to tonemap shader).
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct FilterUniform {
    pub filter_type: u32,
    pub intensity: f32,
    pub time: f32,
    /// 1.0 = apply manual gamma (linear swapchain), 0.0 = skip (sRGB swapchain)
    pub apply_gamma: f32,
}

impl Default for FilterUniform {
    fn default() -> Self {
        Self {
            filter_type: 0,
            intensity: 1.0,
            time: 0.0,
            apply_gamma: 1.0, // default: apply gamma (conservative)
        }
    }
}

/// Active filter state resource.
#[derive(Debug, Resource)]
pub struct ActiveFilter {
    pub filter: PostFilter,
    pub auto_underwater: bool,
}

impl Default for ActiveFilter {
    fn default() -> Self {
        Self {
            filter: PostFilter::None,
            auto_underwater: true,
        }
    }
}
