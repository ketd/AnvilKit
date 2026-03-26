//! # 阴影映射
//!
//! 点光源 cubemap shadow 和聚光灯 2D shadow 的数据结构。
//! 实际 GPU 渲染在 `events.rs` 的 shadow pass 中完成。

use super::draw::MAX_SHADOW_LIGHTS;

/// 点光源阴影配置
#[derive(Debug, Clone)]
pub struct PointShadowConfig {
    /// Cubemap 分辨率（每个面）
    pub resolution: u32,
    /// 近裁剪面
    pub near: f32,
    /// 远裁剪面
    pub far: f32,
    /// 阴影偏移（防止 shadow acne）
    pub bias: f32,
}

impl Default for PointShadowConfig {
    fn default() -> Self {
        Self {
            resolution: 512,
            near: 0.1,
            far: 50.0,
            bias: 0.005,
        }
    }
}

/// 聚光灯阴影配置
#[derive(Debug, Clone)]
pub struct SpotShadowConfig {
    /// Shadow map 分辨率
    pub resolution: u32,
    /// 阴影偏移
    pub bias: f32,
    /// 法线偏移
    pub normal_bias: f32,
}

impl Default for SpotShadowConfig {
    fn default() -> Self {
        Self {
            resolution: 1024,
            bias: 0.005,
            normal_bias: 0.02,
        }
    }
}

/// 阴影投射光源数据
///
/// PBR shader 采样此数据计算阴影。
/// 最多 [`MAX_SHADOW_LIGHTS`] 个光源同时投射阴影。
#[derive(Debug, Clone)]
pub struct ShadowCasterData {
    /// 光源空间 view-projection 矩阵
    pub light_view_proj: glam::Mat4,
    /// 阴影偏移
    pub bias: f32,
    /// 法线偏移
    pub normal_bias: f32,
}

/// 阴影 atlas 管理器
///
/// 管理所有阴影贴图的 GPU 资源分配。
pub struct ShadowAtlas {
    /// 最大阴影光源数
    pub max_lights: usize,
    /// 当前活跃的阴影光源数
    pub active_count: usize,
}

impl ShadowAtlas {
    /// 创建新的阴影 atlas
    pub fn new() -> Self {
        Self {
            max_lights: MAX_SHADOW_LIGHTS,
            active_count: 0,
        }
    }

    /// 是否还有空闲的阴影槽位
    pub fn has_capacity(&self) -> bool {
        self.active_count < self.max_lights
    }

    /// 分配一个阴影槽位，返回槽位索引
    pub fn allocate(&mut self) -> Option<usize> {
        if self.has_capacity() {
            let slot = self.active_count;
            self.active_count += 1;
            Some(slot)
        } else {
            None
        }
    }

    /// 重置所有分配（每帧调用）
    pub fn reset(&mut self) {
        self.active_count = 0;
    }
}

impl Default for ShadowAtlas {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_shadow_config_default() {
        let config = PointShadowConfig::default();
        assert_eq!(config.resolution, 512);
        assert!(config.bias > 0.0);
    }

    #[test]
    fn test_spot_shadow_config_default() {
        let config = SpotShadowConfig::default();
        assert_eq!(config.resolution, 1024);
    }

    #[test]
    fn test_shadow_atlas_allocation() {
        let mut atlas = ShadowAtlas::new();
        assert_eq!(atlas.max_lights, MAX_SHADOW_LIGHTS);
        assert!(atlas.has_capacity());

        for i in 0..MAX_SHADOW_LIGHTS {
            assert_eq!(atlas.allocate(), Some(i));
        }
        assert!(!atlas.has_capacity());
        assert_eq!(atlas.allocate(), None);

        atlas.reset();
        assert!(atlas.has_capacity());
        assert_eq!(atlas.allocate(), Some(0));
    }
}
