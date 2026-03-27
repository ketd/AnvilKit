//! # 阴影映射
//!
//! 点光源 cubemap shadow 和聚光灯 2D shadow 的数据结构。
//! 实际 GPU 渲染在 `events.rs` 的 shadow pass 中完成。

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shadow_caster_data() {
        let data = ShadowCasterData {
            light_view_proj: glam::Mat4::IDENTITY,
            bias: 0.005,
            normal_bias: 0.02,
        };
        assert_eq!(data.bias, 0.005);
        assert_eq!(data.normal_bias, 0.02);
    }
}
