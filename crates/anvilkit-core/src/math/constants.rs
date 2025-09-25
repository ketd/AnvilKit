//! # 数学常量和工具函数
//! 
//! 提供游戏开发中常用的数学常量、转换函数和工具函数。
//! 
//! ## 常量分类
//! 
//! - **角度转换**: 弧度和角度之间的转换常量
//! - **物理常量**: 重力、阻尼等物理模拟常量
//! - **几何常量**: 常用的几何数值
//! - **颜色常量**: 预定义的颜色值
//! 
//! ## 使用示例
//! 
//! ```rust
//! use anvilkit_core::math::constants::*;
//! use glam::Vec3;
//! 
//! // 角度转换
//! let radians = 90.0 * DEG_TO_RAD;
//! let degrees = std::f32::consts::PI * RAD_TO_DEG;
//! 
//! // 物理模拟
//! let gravity = Vec3::new(0.0, -GRAVITY_EARTH, 0.0);
//! 
//! // 几何计算
//! let radius = 5.0;
//! let circle_area = std::f32::consts::PI * radius * radius;
//! ```

use glam::{Vec2, Vec3, Vec4};

// ============================================================================
// 角度和弧度转换常量
// ============================================================================

/// 将角度转换为弧度的乘数 (π/180)
pub const DEG_TO_RAD: f32 = std::f32::consts::PI / 180.0;

/// 将弧度转换为角度的乘数 (180/π)
pub const RAD_TO_DEG: f32 = 180.0 / std::f32::consts::PI;

/// 半圆的弧度值 (π)
pub const HALF_CIRCLE_RAD: f32 = std::f32::consts::PI;

/// 四分之一圆的弧度值 (π/2)
pub const QUARTER_CIRCLE_RAD: f32 = std::f32::consts::FRAC_PI_2;

/// 完整圆的弧度值 (2π)
pub const FULL_CIRCLE_RAD: f32 = std::f32::consts::TAU;

// ============================================================================
// 物理常量
// ============================================================================

/// 地球表面重力加速度 (m/s²)
pub const GRAVITY_EARTH: f32 = 9.80665;

/// 月球表面重力加速度 (m/s²)
pub const GRAVITY_MOON: f32 = 1.625;

/// 火星表面重力加速度 (m/s²)
pub const GRAVITY_MARS: f32 = 3.711;

/// 标准大气压 (Pa)
pub const ATMOSPHERIC_PRESSURE: f32 = 101325.0;

/// 空气密度 (kg/m³) - 海平面，15°C
pub const AIR_DENSITY: f32 = 1.225;

/// 水的密度 (kg/m³) - 4°C
pub const WATER_DENSITY: f32 = 1000.0;

/// 光速 (m/s)
pub const SPEED_OF_LIGHT: f32 = 299_792_458.0;

/// 声速 (m/s) - 20°C，干燥空气
pub const SPEED_OF_SOUND: f32 = 343.0;

// ============================================================================
// 几何和数学常量
// ============================================================================

/// 黄金比例 (φ = (1 + √5) / 2)
pub const GOLDEN_RATIO: f32 = 1.618033988749;

/// 黄金比例的倒数 (1/φ)
pub const GOLDEN_RATIO_INVERSE: f32 = 0.618033988749;

/// 欧拉数 (e)
pub const EULER: f32 = std::f32::consts::E;

/// 自然对数的底数 (ln(2))
pub const LN_2: f32 = std::f32::consts::LN_2;

/// 自然对数的底数 (ln(10))
pub const LN_10: f32 = std::f32::consts::LN_10;

/// 平方根 2
pub const SQRT_2: f32 = std::f32::consts::SQRT_2;

/// 平方根 3
pub const SQRT_3: f32 = 1.732050807569;

/// 平方根 5
pub const SQRT_5: f32 = 2.236067977499;

// ============================================================================
// 浮点数精度常量
// ============================================================================

/// 单精度浮点数的机器精度
pub const F32_EPSILON: f32 = f32::EPSILON;

/// 用于比较的小数值
pub const SMALL_NUMBER: f32 = 1e-6;

/// 用于比较的极小数值
pub const TINY_NUMBER: f32 = 1e-8;

/// 用于角度比较的小数值（约 0.01 度）
pub const ANGLE_EPSILON: f32 = 0.0001745329;

// ============================================================================
// 常用向量常量
// ============================================================================

/// 2D 向量常量
pub mod vec2 {
    use super::*;
    
    /// 零向量
    pub const ZERO: Vec2 = Vec2::ZERO;
    
    /// 单位向量
    pub const ONE: Vec2 = Vec2::ONE;
    
    /// X 轴单位向量
    pub const X: Vec2 = Vec2::X;
    
    /// Y 轴单位向量
    pub const Y: Vec2 = Vec2::Y;
    
    /// 负 X 轴单位向量
    pub const NEG_X: Vec2 = Vec2::NEG_X;
    
    /// 负 Y 轴单位向量
    pub const NEG_Y: Vec2 = Vec2::NEG_Y;
    
    /// 右方向（正 X）
    pub const RIGHT: Vec2 = Vec2::X;
    
    /// 左方向（负 X）
    pub const LEFT: Vec2 = Vec2::NEG_X;
    
    /// 上方向（正 Y）
    pub const UP: Vec2 = Vec2::Y;
    
    /// 下方向（负 Y）
    pub const DOWN: Vec2 = Vec2::NEG_Y;
}

/// 3D 向量常量
pub mod vec3 {
    use super::*;
    
    /// 零向量
    pub const ZERO: Vec3 = Vec3::ZERO;
    
    /// 单位向量
    pub const ONE: Vec3 = Vec3::ONE;
    
    /// X 轴单位向量
    pub const X: Vec3 = Vec3::X;
    
    /// Y 轴单位向量
    pub const Y: Vec3 = Vec3::Y;
    
    /// Z 轴单位向量
    pub const Z: Vec3 = Vec3::Z;
    
    /// 负 X 轴单位向量
    pub const NEG_X: Vec3 = Vec3::NEG_X;
    
    /// 负 Y 轴单位向量
    pub const NEG_Y: Vec3 = Vec3::NEG_Y;
    
    /// 负 Z 轴单位向量
    pub const NEG_Z: Vec3 = Vec3::NEG_Z;
    
    /// 右方向（正 X）
    pub const RIGHT: Vec3 = Vec3::X;
    
    /// 左方向（负 X）
    pub const LEFT: Vec3 = Vec3::NEG_X;
    
    /// 上方向（正 Y）
    pub const UP: Vec3 = Vec3::Y;
    
    /// 下方向（负 Y）
    pub const DOWN: Vec3 = Vec3::NEG_Y;
    
    /// 前方向（负 Z，右手坐标系）
    pub const FORWARD: Vec3 = Vec3::NEG_Z;
    
    /// 后方向（正 Z）
    pub const BACKWARD: Vec3 = Vec3::Z;
}

// ============================================================================
// 颜色常量 (RGBA)
// ============================================================================

/// 颜色常量模块
pub mod colors {
    use super::*;
    
    /// 透明色
    pub const TRANSPARENT: Vec4 = Vec4::new(0.0, 0.0, 0.0, 0.0);
    
    /// 白色
    pub const WHITE: Vec4 = Vec4::new(1.0, 1.0, 1.0, 1.0);
    
    /// 黑色
    pub const BLACK: Vec4 = Vec4::new(0.0, 0.0, 0.0, 1.0);
    
    /// 红色
    pub const RED: Vec4 = Vec4::new(1.0, 0.0, 0.0, 1.0);
    
    /// 绿色
    pub const GREEN: Vec4 = Vec4::new(0.0, 1.0, 0.0, 1.0);
    
    /// 蓝色
    pub const BLUE: Vec4 = Vec4::new(0.0, 0.0, 1.0, 1.0);
    
    /// 黄色
    pub const YELLOW: Vec4 = Vec4::new(1.0, 1.0, 0.0, 1.0);
    
    /// 青色
    pub const CYAN: Vec4 = Vec4::new(0.0, 1.0, 1.0, 1.0);
    
    /// 洋红色
    pub const MAGENTA: Vec4 = Vec4::new(1.0, 0.0, 1.0, 1.0);
    
    /// 橙色
    pub const ORANGE: Vec4 = Vec4::new(1.0, 0.5, 0.0, 1.0);
    
    /// 紫色
    pub const PURPLE: Vec4 = Vec4::new(0.5, 0.0, 1.0, 1.0);
    
    /// 灰色
    pub const GRAY: Vec4 = Vec4::new(0.5, 0.5, 0.5, 1.0);
    
    /// 浅灰色
    pub const LIGHT_GRAY: Vec4 = Vec4::new(0.75, 0.75, 0.75, 1.0);
    
    /// 深灰色
    pub const DARK_GRAY: Vec4 = Vec4::new(0.25, 0.25, 0.25, 1.0);
}

// ============================================================================
// 工具函数
// ============================================================================

/// 将角度转换为弧度
/// 
/// # 示例
/// 
/// ```rust
/// use anvilkit_core::math::constants::degrees_to_radians;
/// 
/// let radians = degrees_to_radians(90.0);
/// assert!((radians - std::f32::consts::FRAC_PI_2).abs() < 1e-6);
/// ```
pub fn degrees_to_radians(degrees: f32) -> f32 {
    degrees * DEG_TO_RAD
}

/// 将弧度转换为角度
/// 
/// # 示例
/// 
/// ```rust
/// use anvilkit_core::math::constants::radians_to_degrees;
/// 
/// let degrees = radians_to_degrees(std::f32::consts::PI);
/// assert!((degrees - 180.0).abs() < 1e-6);
/// ```
pub fn radians_to_degrees(radians: f32) -> f32 {
    radians * RAD_TO_DEG
}

/// 将角度标准化到 [0, 360) 范围
/// 
/// # 示例
/// 
/// ```rust
/// use anvilkit_core::math::constants::normalize_degrees;
/// 
/// assert_eq!(normalize_degrees(450.0), 90.0);
/// assert_eq!(normalize_degrees(-90.0), 270.0);
/// ```
pub fn normalize_degrees(degrees: f32) -> f32 {
    let mut result = degrees % 360.0;
    if result < 0.0 {
        result += 360.0;
    }
    result
}

/// 将弧度标准化到 [0, 2π) 范围
/// 
/// # 示例
/// 
/// ```rust
/// use anvilkit_core::math::constants::normalize_radians;
/// 
/// let normalized = normalize_radians(3.0 * std::f32::consts::PI);
/// assert!((normalized - std::f32::consts::PI).abs() < 1e-6);
/// ```
pub fn normalize_radians(radians: f32) -> f32 {
    let mut result = radians % FULL_CIRCLE_RAD;
    if result < 0.0 {
        result += FULL_CIRCLE_RAD;
    }
    result
}

/// 检查两个浮点数是否近似相等
/// 
/// # 参数
/// 
/// - `a`, `b`: 要比较的数值
/// - `epsilon`: 容差值，默认使用 `SMALL_NUMBER`
/// 
/// # 示例
/// 
/// ```rust
/// use anvilkit_core::math::constants::approximately_equal;
/// 
/// assert!(approximately_equal(0.1 + 0.2, 0.3, None));
/// assert!(!approximately_equal(1.0, 2.0, None));
/// ```
pub fn approximately_equal(a: f32, b: f32, epsilon: Option<f32>) -> bool {
    let eps = epsilon.unwrap_or(SMALL_NUMBER);
    (a - b).abs() <= eps
}

/// 检查浮点数是否接近零
/// 
/// # 示例
/// 
/// ```rust
/// use anvilkit_core::math::constants::is_nearly_zero;
/// 
/// assert!(is_nearly_zero(1e-7));
/// assert!(!is_nearly_zero(0.1));
/// ```
pub fn is_nearly_zero(value: f32) -> bool {
    value.abs() <= SMALL_NUMBER
}

/// 安全的平方根函数，确保输入非负
/// 
/// # 示例
/// 
/// ```rust
/// use anvilkit_core::math::constants::safe_sqrt;
/// 
/// assert_eq!(safe_sqrt(4.0), 2.0);
/// assert_eq!(safe_sqrt(-1.0), 0.0); // 负数返回 0
/// ```
pub fn safe_sqrt(value: f32) -> f32 {
    if value < 0.0 {
        0.0
    } else {
        value.sqrt()
    }
}

/// 计算两点之间的距离
/// 
/// # 示例
/// 
/// ```rust
/// use anvilkit_core::math::constants::distance_2d;
/// use glam::Vec2;
/// 
/// let dist = distance_2d(Vec2::ZERO, Vec2::new(3.0, 4.0));
/// assert_eq!(dist, 5.0); // 3-4-5 三角形
/// ```
pub fn distance_2d(a: Vec2, b: Vec2) -> f32 {
    (b - a).length()
}

/// 计算两点之间的距离（3D）
pub fn distance_3d(a: Vec3, b: Vec3) -> f32 {
    (b - a).length()
}

/// 计算两点之间的距离的平方（避免开方运算）
/// 
/// 用于性能敏感的距离比较。
pub fn distance_squared_2d(a: Vec2, b: Vec2) -> f32 {
    (b - a).length_squared()
}

/// 计算两点之间的距离的平方（3D）
pub fn distance_squared_3d(a: Vec3, b: Vec3) -> f32 {
    (b - a).length_squared()
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_angle_conversion() {
        assert_relative_eq!(degrees_to_radians(90.0), std::f32::consts::FRAC_PI_2, epsilon = 1e-6);
        assert_relative_eq!(degrees_to_radians(180.0), std::f32::consts::PI, epsilon = 1e-6);
        assert_relative_eq!(radians_to_degrees(std::f32::consts::PI), 180.0, epsilon = 1e-6);
    }

    #[test]
    fn test_angle_normalization() {
        assert_eq!(normalize_degrees(450.0), 90.0);
        assert_eq!(normalize_degrees(-90.0), 270.0);
        assert_eq!(normalize_degrees(0.0), 0.0);
        assert_eq!(normalize_degrees(360.0), 0.0);
        
        let normalized = normalize_radians(3.0 * std::f32::consts::PI);
        assert_relative_eq!(normalized, std::f32::consts::PI, epsilon = 1e-6);
    }

    #[test]
    fn test_approximately_equal() {
        assert!(approximately_equal(0.1 + 0.2, 0.3, None));
        assert!(approximately_equal(1.0, 1.0000001, Some(1e-5)));
        assert!(!approximately_equal(1.0, 2.0, None));
    }

    #[test]
    fn test_is_nearly_zero() {
        assert!(is_nearly_zero(1e-7));
        assert!(is_nearly_zero(0.0));
        assert!(!is_nearly_zero(0.1));
        assert!(!is_nearly_zero(-0.1));
    }

    #[test]
    fn test_safe_sqrt() {
        assert_eq!(safe_sqrt(4.0), 2.0);
        assert_eq!(safe_sqrt(0.0), 0.0);
        assert_eq!(safe_sqrt(-1.0), 0.0);
    }

    #[test]
    fn test_distance_functions() {
        let a = Vec2::ZERO;
        let b = Vec2::new(3.0, 4.0);
        
        assert_eq!(distance_2d(a, b), 5.0);
        assert_eq!(distance_squared_2d(a, b), 25.0);
        
        let a3 = Vec3::ZERO;
        let b3 = Vec3::new(1.0, 2.0, 2.0);
        
        assert_eq!(distance_3d(a3, b3), 3.0);
        assert_eq!(distance_squared_3d(a3, b3), 9.0);
    }

    #[test]
    fn test_constants() {
        // 测试一些重要常量的值
        assert_relative_eq!(GOLDEN_RATIO, 1.618033988749, epsilon = 1e-6);
        assert_relative_eq!(GOLDEN_RATIO * GOLDEN_RATIO_INVERSE, 1.0, epsilon = 1e-6);
        assert!((DEG_TO_RAD * RAD_TO_DEG - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_vector_constants() {
        assert_eq!(vec2::RIGHT, Vec2::new(1.0, 0.0));
        assert_eq!(vec2::UP, Vec2::new(0.0, 1.0));
        assert_eq!(vec3::FORWARD, Vec3::new(0.0, 0.0, -1.0));
        assert_eq!(vec3::UP, Vec3::new(0.0, 1.0, 0.0));
    }

    #[test]
    fn test_color_constants() {
        assert_eq!(colors::WHITE, Vec4::new(1.0, 1.0, 1.0, 1.0));
        assert_eq!(colors::BLACK, Vec4::new(0.0, 0.0, 0.0, 1.0));
        assert_eq!(colors::TRANSPARENT, Vec4::new(0.0, 0.0, 0.0, 0.0));
    }
}
