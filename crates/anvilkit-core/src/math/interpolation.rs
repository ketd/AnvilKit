//! # 插值和动画支持
//! 
//! 提供各种插值算法，用于动画、过渡效果和数值平滑。
//! 
//! ## 核心 Trait
//! 
//! - [`Lerp`]: 线性插值
//! - [`Slerp`]: 球面线性插值（用于旋转）
//! - [`Interpolate`]: 通用插值接口
//! 
//! ## 缓动函数
//! 
//! 提供常用的缓动函数，用于创建自然的动画效果。
//! 
//! ## 使用示例
//! 
//! ```rust
//! use anvilkit_core::math::interpolation::{Lerp, Slerp, ease_in_out_cubic};
//! use glam::{Vec3, Quat};
//! 
//! // 线性插值
//! let start = Vec3::ZERO;
//! let end = Vec3::new(10.0, 20.0, 30.0);
//! let mid = start.lerp(end, 0.5);
//! 
//! // 旋转插值
//! let rot1 = Quat::IDENTITY;
//! let rot2 = Quat::from_rotation_y(std::f32::consts::PI);
//! let mid_rot = rot1.slerp(rot2, 0.5);
//! 
//! // 使用缓动函数
//! let t = ease_in_out_cubic(0.3);
//! let smooth_pos = start.lerp(end, t);
//! ```

use glam::{Vec2, Vec3, Vec4, Quat};

/// 线性插值 trait
/// 
/// 为支持线性插值的类型提供统一接口。
pub trait Lerp<T = Self> {
    /// 在两个值之间进行线性插值
    /// 
    /// # 参数
    /// 
    /// - `other`: 目标值
    /// - `t`: 插值参数，通常在 [0, 1] 范围内
    ///   - `t = 0.0` 返回 `self`
    ///   - `t = 1.0` 返回 `other`
    ///   - `t = 0.5` 返回中点
    /// 
    /// # 注意
    /// 
    /// `t` 可以超出 [0, 1] 范围进行外推。
    fn lerp(&self, other: T, t: f32) -> Self;
}

/// 球面线性插值 trait
/// 
/// 主要用于旋转插值，提供更自然的旋转过渡。
pub trait Slerp<T = Self> {
    /// 在两个值之间进行球面线性插值
    /// 
    /// # 参数
    /// 
    /// - `other`: 目标值
    /// - `t`: 插值参数，通常在 [0, 1] 范围内
    fn slerp(&self, other: T, t: f32) -> Self;
}

/// 通用插值接口
/// 
/// 提供多种插值方法的统一接口。
pub trait Interpolate<T = Self> {
    /// 线性插值
    fn interpolate_linear(&self, other: T, t: f32) -> Self;
    
    /// 平滑插值（使用 smoothstep 函数）
    fn interpolate_smooth(&self, other: T, t: f32) -> Self;
    
    /// 使用自定义缓动函数插值
    fn interpolate_eased(&self, other: T, t: f32, ease_fn: fn(f32) -> f32) -> Self;
}

// 为基础数值类型实现 Lerp
impl Lerp for f32 {
    fn lerp(&self, other: f32, t: f32) -> f32 {
        self + (other - self) * t
    }
}

impl Lerp for f64 {
    fn lerp(&self, other: f64, t: f32) -> f64 {
        self + (other - self) * t as f64
    }
}

// 为 glam 向量类型实现 Lerp
impl Lerp for Vec2 {
    fn lerp(&self, other: Vec2, t: f32) -> Vec2 {
        *self + (other - *self) * t
    }
}

impl Lerp for Vec3 {
    fn lerp(&self, other: Vec3, t: f32) -> Vec3 {
        *self + (other - *self) * t
    }
}

impl Lerp for Vec4 {
    fn lerp(&self, other: Vec4, t: f32) -> Vec4 {
        *self + (other - *self) * t
    }
}

// 为四元数实现 Slerp
impl Slerp for Quat {
    fn slerp(&self, other: Quat, t: f32) -> Quat {
        Quat::slerp(*self, other, t)
    }
}

// 为四元数实现 Lerp（使用 nlerp）
impl Lerp for Quat {
    fn lerp(&self, other: Quat, t: f32) -> Quat {
        Quat::lerp(*self, other, t).normalize()
    }
}

// 为支持 Lerp 的类型实现 Interpolate
impl<T: Lerp + Copy> Interpolate for T {
    fn interpolate_linear(&self, other: T, t: f32) -> T {
        self.lerp(other, t)
    }
    
    fn interpolate_smooth(&self, other: T, t: f32) -> T {
        let smooth_t = smoothstep(t);
        self.lerp(other, smooth_t)
    }
    
    fn interpolate_eased(&self, other: T, t: f32, ease_fn: fn(f32) -> f32) -> T {
        let eased_t = ease_fn(t);
        self.lerp(other, eased_t)
    }
}

/// Smoothstep 函数，提供平滑的 S 形曲线插值
/// 
/// 在 t ∈ [0, 1] 范围内，提供比线性插值更自然的过渡。
/// 
/// # 公式
/// 
/// `smoothstep(t) = 3t² - 2t³`
/// 
/// # 示例
/// 
/// ```rust
/// use anvilkit_core::math::interpolation::smoothstep;
/// 
/// assert_eq!(smoothstep(0.0), 0.0);
/// assert_eq!(smoothstep(1.0), 1.0);
/// assert_eq!(smoothstep(0.5), 0.5); // S 形曲线在中点与线性插值相同
/// assert!(smoothstep(0.25) < 0.25); // 前半段比线性插值慢
/// assert!(smoothstep(0.75) > 0.75); // 后半段比线性插值快
/// ```
pub fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Smootherstep 函数，提供更平滑的过渡
/// 
/// 比 smoothstep 提供更平滑的一阶和二阶导数。
/// 
/// # 公式
/// 
/// `smootherstep(t) = 6t⁵ - 15t⁴ + 10t³`
pub fn smootherstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

/// 将值从一个范围重映射到另一个范围
/// 
/// # 参数
/// 
/// - `value`: 输入值
/// - `from_min`, `from_max`: 输入范围
/// - `to_min`, `to_max`: 输出范围
/// 
/// # 示例
/// 
/// ```rust
/// use anvilkit_core::math::interpolation::remap;
/// 
/// // 将 [0, 100] 范围的值映射到 [0, 1] 范围
/// let normalized = remap(50.0, 0.0, 100.0, 0.0, 1.0);
/// assert_eq!(normalized, 0.5);
/// ```
pub fn remap(value: f32, from_min: f32, from_max: f32, to_min: f32, to_max: f32) -> f32 {
    let t = (value - from_min) / (from_max - from_min);
    to_min + t * (to_max - to_min)
}

// 缓动函数
// 这些函数提供各种动画缓动效果

/// 二次缓入函数
/// 
/// 动画开始时较慢，然后加速。
pub fn ease_in_quad(t: f32) -> f32 {
    t * t
}

/// 二次缓出函数
/// 
/// 动画开始时较快，然后减速。
pub fn ease_out_quad(t: f32) -> f32 {
    1.0 - (1.0 - t) * (1.0 - t)
}

/// 二次缓入缓出函数
/// 
/// 动画开始和结束时较慢，中间较快。
pub fn ease_in_out_quad(t: f32) -> f32 {
    if t < 0.5 {
        2.0 * t * t
    } else {
        1.0 - 2.0 * (1.0 - t) * (1.0 - t)
    }
}

/// 三次缓入函数
pub fn ease_in_cubic(t: f32) -> f32 {
    t * t * t
}

/// 三次缓出函数
pub fn ease_out_cubic(t: f32) -> f32 {
    let t = 1.0 - t;
    1.0 - t * t * t
}

/// 三次缓入缓出函数
pub fn ease_in_out_cubic(t: f32) -> f32 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        let t = 1.0 - t;
        1.0 - 4.0 * t * t * t
    }
}

/// 四次缓入函数
pub fn ease_in_quart(t: f32) -> f32 {
    t * t * t * t
}

/// 四次缓出函数
pub fn ease_out_quart(t: f32) -> f32 {
    let t = 1.0 - t;
    1.0 - t * t * t * t
}

/// 四次缓入缓出函数
pub fn ease_in_out_quart(t: f32) -> f32 {
    if t < 0.5 {
        8.0 * t * t * t * t
    } else {
        let t = 1.0 - t;
        1.0 - 8.0 * t * t * t * t
    }
}

/// 弹性缓出函数
/// 
/// 创建弹性效果，超出目标值然后回弹。
pub fn ease_out_elastic(t: f32) -> f32 {
    if t == 0.0 {
        0.0
    } else if t == 1.0 {
        1.0
    } else {
        let p = 0.3;
        let s = p / 4.0;
        2.0_f32.powf(-10.0 * t) * ((t - s) * (2.0 * std::f32::consts::PI) / p).sin() + 1.0
    }
}

/// 回弹缓出函数
/// 
/// 创建回弹效果，模拟球落地的弹跳。
pub fn ease_out_bounce(t: f32) -> f32 {
    if t < 1.0 / 2.75 {
        7.5625 * t * t
    } else if t < 2.0 / 2.75 {
        let t = t - 1.5 / 2.75;
        7.5625 * t * t + 0.75
    } else if t < 2.5 / 2.75 {
        let t = t - 2.25 / 2.75;
        7.5625 * t * t + 0.9375
    } else {
        let t = t - 2.625 / 2.75;
        7.5625 * t * t + 0.984375
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 自定义的近似相等比较函数
    fn vec3_approx_eq(a: Vec3, b: Vec3, epsilon: f32) -> bool {
        (a - b).length() < epsilon
    }

    #[allow(dead_code)]
    fn quat_approx_eq(a: glam::Quat, b: glam::Quat, epsilon: f32) -> bool {
        // 四元数 q 和 -q 表示相同的旋转，所以我们需要检查两种情况
        let dot = a.dot(b).abs();
        (dot - 1.0).abs() < epsilon
    }

    #[test]
    fn test_f32_lerp() {
        assert_eq!(0.0.lerp(10.0, 0.0), 0.0);
        assert_eq!(0.0.lerp(10.0, 1.0), 10.0);
        assert_eq!(0.0.lerp(10.0, 0.5), 5.0);
    }

    #[test]
    fn test_vec3_lerp() {
        let start = Vec3::ZERO;
        let end = Vec3::new(10.0, 20.0, 30.0);
        let mid = start.lerp(end, 0.5);
        
        assert!(vec3_approx_eq(mid, Vec3::new(5.0, 10.0, 15.0), 1e-6));
    }

    #[test]
    fn test_quat_slerp() {
        let start = Quat::IDENTITY;
        let end = Quat::from_rotation_y(std::f32::consts::PI);
        let mid = start.slerp(end, 0.5);

        // 验证插值结果是有效的四元数
        assert!(mid.is_finite());
        assert!((mid.length() - 1.0).abs() < 1e-6, "Quaternion should be normalized");

        // 验证旋转角度是否正确（应该是90度）
        let angle = 2.0 * mid.w.abs().acos();
        let expected_angle = std::f32::consts::PI * 0.5;
        assert!((angle - expected_angle).abs() < 1e-3,
                "Expected angle {}, got {}", expected_angle, angle);
    }

    #[test]
    fn test_smoothstep() {
        assert_eq!(smoothstep(0.0), 0.0);
        assert_eq!(smoothstep(1.0), 1.0);
        assert!((smoothstep(0.5) - 0.5).abs() < 1e-6);
        
        // 验证 S 形曲线特性
        assert!(smoothstep(0.25) < 0.25);
        assert!(smoothstep(0.75) > 0.75);
    }

    #[test]
    fn test_remap() {
        assert_eq!(remap(50.0, 0.0, 100.0, 0.0, 1.0), 0.5);
        assert_eq!(remap(0.0, 0.0, 100.0, -1.0, 1.0), -1.0);
        assert_eq!(remap(100.0, 0.0, 100.0, -1.0, 1.0), 1.0);
    }

    #[test]
    fn test_ease_functions() {
        // 测试缓动函数的边界值
        assert_eq!(ease_in_quad(0.0), 0.0);
        assert_eq!(ease_in_quad(1.0), 1.0);
        assert_eq!(ease_out_quad(0.0), 0.0);
        assert_eq!(ease_out_quad(1.0), 1.0);
        
        // 测试缓入缓出的对称性
        let t = 0.3;
        let ease_in = ease_in_out_cubic(t);
        let ease_out = ease_in_out_cubic(1.0 - t);
        assert!((ease_in - (1.0 - ease_out)).abs() < 1e-6);
    }

    #[test]
    fn test_interpolate_trait() {
        let start = Vec3::ZERO;
        let end = Vec3::new(10.0, 20.0, 30.0);
        
        let linear = start.interpolate_linear(end, 0.5);
        let _smooth = start.interpolate_smooth(end, 0.5);
        let _eased = start.interpolate_eased(end, 0.5, ease_in_out_cubic);
        
        assert!(vec3_approx_eq(linear, Vec3::new(5.0, 10.0, 15.0), 1e-6));

        // 在 t=0.25 时测试差异，因为在 t=0.5 时平滑插值可能与线性插值相同
        let linear_quarter = start.interpolate_linear(end, 0.25);
        let smooth_quarter = start.interpolate_smooth(end, 0.25);
        let eased_quarter = start.interpolate_eased(end, 0.25, ease_in_out_cubic);

        assert!(!vec3_approx_eq(smooth_quarter, linear_quarter, 1e-6)); // 平滑插值应该不同于线性插值
        assert!(!vec3_approx_eq(eased_quarter, linear_quarter, 1e-6)); // 缓动插值应该不同于线性插值
    }

    #[test]
    fn test_elastic_and_bounce() {
        // 测试弹性和回弹函数的边界值
        assert_eq!(ease_out_elastic(0.0), 0.0);
        assert!((ease_out_elastic(1.0) - 1.0).abs() < 1e-6);
        assert_eq!(ease_out_bounce(0.0), 0.0);
        assert!((ease_out_bounce(1.0) - 1.0).abs() < 1e-6);
        
        // 弹性函数应该在某些点超出 [0, 1] 范围
        let elastic_mid = ease_out_elastic(0.5);
        assert!(elastic_mid > 1.0 || elastic_mid < 0.0);
    }

    #[test]
    fn test_extrapolation() {
        // 测试超出 [0, 1] 范围的插值（外推）
        let start = 0.0;
        let end = 10.0;
        
        assert_eq!(start.lerp(end, -0.5), -5.0); // 向后外推
        assert_eq!(start.lerp(end, 1.5), 15.0);  // 向前外推
    }
}
