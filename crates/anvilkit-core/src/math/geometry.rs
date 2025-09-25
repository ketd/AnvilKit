//! # 几何图形和边界框
//! 
//! 提供游戏开发中常用的几何图形类型和空间查询功能。
//! 
//! ## 核心类型
//! 
//! - [`Rect`]: 2D 矩形
//! - [`Circle`]: 2D 圆形
//! - [`Bounds2D`]: 2D 轴对齐边界框
//! - [`Bounds3D`]: 3D 轴对齐边界框
//! 
//! ## 使用示例
//! 
//! ```rust
//! use anvilkit_core::math::geometry::{Rect, Circle, Bounds3D};
//! use glam::{Vec2, Vec3};
//! 
//! // 创建 2D 矩形
//! let rect = Rect::from_center_size(Vec2::ZERO, Vec2::new(10.0, 20.0));
//! 
//! // 创建圆形
//! let circle = Circle::new(Vec2::new(5.0, 5.0), 3.0);
//! 
//! // 检查碰撞
//! if rect.intersects_circle(&circle) {
//!     println!("矩形和圆形相交！");
//! }
//! ```

use glam::{Vec2, Vec3};

/// 2D 矩形，用于边界检测和 UI 布局
/// 
/// 矩形使用最小点和最大点表示，确保 `min.x <= max.x` 和 `min.y <= max.y`。
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Rect {
    /// 最小点（左下角）
    pub min: Vec2,
    /// 最大点（右上角）
    pub max: Vec2,
}

impl Rect {
    /// 零大小的矩形
    pub const ZERO: Self = Self {
        min: Vec2::ZERO,
        max: Vec2::ZERO,
    };

    /// 创建新的矩形
    /// 
    /// # 参数
    /// 
    /// - `min`: 最小点
    /// - `max`: 最大点
    /// 
    /// # 注意
    /// 
    /// 如果 `min` 的某个分量大于 `max` 的对应分量，会自动交换以确保有效性。
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_core::math::geometry::Rect;
    /// use glam::Vec2;
    /// 
    /// let rect = Rect::new(Vec2::ZERO, Vec2::new(10.0, 20.0));
    /// assert_eq!(rect.width(), 10.0);
    /// assert_eq!(rect.height(), 20.0);
    /// ```
    pub fn new(min: Vec2, max: Vec2) -> Self {
        Self {
            min: min.min(max),
            max: min.max(max),
        }
    }

    /// 从中心点和大小创建矩形
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_core::math::geometry::Rect;
    /// use glam::Vec2;
    /// 
    /// let rect = Rect::from_center_size(Vec2::new(5.0, 5.0), Vec2::new(10.0, 20.0));
    /// assert_eq!(rect.center(), Vec2::new(5.0, 5.0));
    /// ```
    pub fn from_center_size(center: Vec2, size: Vec2) -> Self {
        let half_size = size * 0.5;
        Self::new(center - half_size, center + half_size)
    }

    /// 从位置和大小创建矩形（位置为左下角）
    pub fn from_position_size(position: Vec2, size: Vec2) -> Self {
        Self::new(position, position + size)
    }

    /// 获取矩形宽度
    pub fn width(&self) -> f32 {
        self.max.x - self.min.x
    }

    /// 获取矩形高度
    pub fn height(&self) -> f32 {
        self.max.y - self.min.y
    }

    /// 获取矩形大小
    pub fn size(&self) -> Vec2 {
        self.max - self.min
    }

    /// 获取矩形中心点
    pub fn center(&self) -> Vec2 {
        (self.min + self.max) * 0.5
    }

    /// 获取矩形面积
    pub fn area(&self) -> f32 {
        let size = self.size();
        size.x * size.y
    }

    /// 获取矩形周长
    pub fn perimeter(&self) -> f32 {
        let size = self.size();
        2.0 * (size.x + size.y)
    }

    /// 检查点是否在矩形内
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_core::math::geometry::Rect;
    /// use glam::Vec2;
    /// 
    /// let rect = Rect::new(Vec2::ZERO, Vec2::new(10.0, 10.0));
    /// assert!(rect.contains(Vec2::new(5.0, 5.0)));
    /// assert!(!rect.contains(Vec2::new(15.0, 5.0)));
    /// ```
    pub fn contains(&self, point: Vec2) -> bool {
        point.x >= self.min.x && point.x <= self.max.x &&
        point.y >= self.min.y && point.y <= self.max.y
    }

    /// 检查是否与另一个矩形相交
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_core::math::geometry::Rect;
    /// use glam::Vec2;
    /// 
    /// let rect1 = Rect::new(Vec2::ZERO, Vec2::new(10.0, 10.0));
    /// let rect2 = Rect::new(Vec2::new(5.0, 5.0), Vec2::new(15.0, 15.0));
    /// assert!(rect1.intersects(&rect2));
    /// ```
    pub fn intersects(&self, other: &Rect) -> bool {
        self.min.x <= other.max.x && self.max.x >= other.min.x &&
        self.min.y <= other.max.y && self.max.y >= other.min.y
    }

    /// 检查是否与圆形相交
    pub fn intersects_circle(&self, circle: &Circle) -> bool {
        // 找到矩形上距离圆心最近的点
        let closest_point = Vec2::new(
            circle.center.x.clamp(self.min.x, self.max.x),
            circle.center.y.clamp(self.min.y, self.max.y),
        );
        
        // 检查距离是否小于半径
        let distance_squared = (circle.center - closest_point).length_squared();
        distance_squared <= circle.radius * circle.radius
    }

    /// 计算与另一个矩形的交集
    /// 
    /// # 返回
    /// 
    /// 如果两个矩形相交，返回交集矩形；否则返回 `None`。
    pub fn intersection(&self, other: &Rect) -> Option<Rect> {
        if !self.intersects(other) {
            return None;
        }

        Some(Rect::new(
            self.min.max(other.min),
            self.max.min(other.max),
        ))
    }

    /// 计算包含两个矩形的最小矩形
    pub fn union(&self, other: &Rect) -> Rect {
        Rect::new(
            self.min.min(other.min),
            self.max.max(other.max),
        )
    }

    /// 扩展矩形以包含指定点
    pub fn expand_to_include(&mut self, point: Vec2) {
        self.min = self.min.min(point);
        self.max = self.max.max(point);
    }

    /// 按指定量扩展矩形
    /// 
    /// # 参数
    /// 
    /// - `amount`: 扩展量，正值扩展，负值收缩
    pub fn expand(&self, amount: f32) -> Rect {
        let expansion = Vec2::splat(amount);
        Rect::new(self.min - expansion, self.max + expansion)
    }

    /// 检查矩形是否有效（非负大小）
    pub fn is_valid(&self) -> bool {
        self.min.x <= self.max.x && self.min.y <= self.max.y
    }
}

/// 2D 圆形
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Circle {
    /// 圆心
    pub center: Vec2,
    /// 半径
    pub radius: f32,
}

impl Circle {
    /// 创建新的圆形
    /// 
    /// # 参数
    /// 
    /// - `center`: 圆心位置
    /// - `radius`: 半径（必须为非负数）
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_core::math::geometry::Circle;
    /// use glam::Vec2;
    /// 
    /// let circle = Circle::new(Vec2::new(5.0, 5.0), 3.0);
    /// assert_eq!(circle.area(), std::f32::consts::PI * 9.0);
    /// ```
    pub fn new(center: Vec2, radius: f32) -> Self {
        Self {
            center,
            radius: radius.max(0.0), // 确保半径非负
        }
    }

    /// 获取圆的面积
    pub fn area(&self) -> f32 {
        std::f32::consts::PI * self.radius * self.radius
    }

    /// 获取圆的周长
    pub fn circumference(&self) -> f32 {
        2.0 * std::f32::consts::PI * self.radius
    }

    /// 检查点是否在圆内
    pub fn contains(&self, point: Vec2) -> bool {
        (point - self.center).length_squared() <= self.radius * self.radius
    }

    /// 检查是否与另一个圆相交
    pub fn intersects(&self, other: &Circle) -> bool {
        let distance_squared = (self.center - other.center).length_squared();
        let radius_sum = self.radius + other.radius;
        distance_squared <= radius_sum * radius_sum
    }

    /// 检查是否与矩形相交
    pub fn intersects_rect(&self, rect: &Rect) -> bool {
        rect.intersects_circle(self)
    }

    /// 获取圆的边界矩形
    pub fn bounding_rect(&self) -> Rect {
        let radius_vec = Vec2::splat(self.radius);
        Rect::new(self.center - radius_vec, self.center + radius_vec)
    }
}

/// 2D 轴对齐边界框
pub type Bounds2D = Rect;

/// 3D 轴对齐边界框
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Bounds3D {
    /// 最小点
    pub min: Vec3,
    /// 最大点
    pub max: Vec3,
}

impl Bounds3D {
    /// 零大小的边界框
    pub const ZERO: Self = Self {
        min: Vec3::ZERO,
        max: Vec3::ZERO,
    };

    /// 创建新的 3D 边界框
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self {
            min: min.min(max),
            max: min.max(max),
        }
    }

    /// 从中心点和大小创建边界框
    pub fn from_center_size(center: Vec3, size: Vec3) -> Self {
        let half_size = size * 0.5;
        Self::new(center - half_size, center + half_size)
    }

    /// 获取边界框大小
    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }

    /// 获取边界框中心点
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    /// 获取边界框体积
    pub fn volume(&self) -> f32 {
        let size = self.size();
        size.x * size.y * size.z
    }

    /// 检查点是否在边界框内
    pub fn contains(&self, point: Vec3) -> bool {
        point.x >= self.min.x && point.x <= self.max.x &&
        point.y >= self.min.y && point.y <= self.max.y &&
        point.z >= self.min.z && point.z <= self.max.z
    }

    /// 检查是否与另一个边界框相交
    pub fn intersects(&self, other: &Bounds3D) -> bool {
        self.min.x <= other.max.x && self.max.x >= other.min.x &&
        self.min.y <= other.max.y && self.max.y >= other.min.y &&
        self.min.z <= other.max.z && self.max.z >= other.min.z
    }

    /// 扩展边界框以包含指定点
    pub fn expand_to_include(&mut self, point: Vec3) {
        self.min = self.min.min(point);
        self.max = self.max.max(point);
    }

    /// 计算与另一个边界框的交集
    pub fn intersection(&self, other: &Bounds3D) -> Option<Bounds3D> {
        if !self.intersects(other) {
            return None;
        }

        Some(Bounds3D::new(
            self.min.max(other.min),
            self.max.min(other.max),
        ))
    }

    /// 计算包含两个边界框的最小边界框
    pub fn union(&self, other: &Bounds3D) -> Bounds3D {
        Bounds3D::new(
            self.min.min(other.min),
            self.max.max(other.max),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_rect_creation() {
        let rect = Rect::new(Vec2::ZERO, Vec2::new(10.0, 20.0));
        assert_eq!(rect.width(), 10.0);
        assert_eq!(rect.height(), 20.0);
        assert_eq!(rect.area(), 200.0);
    }

    #[test]
    fn test_rect_from_center_size() {
        let rect = Rect::from_center_size(Vec2::new(5.0, 10.0), Vec2::new(10.0, 20.0));
        assert_eq!(rect.center(), Vec2::new(5.0, 10.0));
        assert_eq!(rect.size(), Vec2::new(10.0, 20.0));
    }

    #[test]
    fn test_rect_contains() {
        let rect = Rect::new(Vec2::ZERO, Vec2::new(10.0, 10.0));
        assert!(rect.contains(Vec2::new(5.0, 5.0)));
        assert!(rect.contains(Vec2::ZERO)); // 边界点
        assert!(rect.contains(Vec2::new(10.0, 10.0))); // 边界点
        assert!(!rect.contains(Vec2::new(15.0, 5.0)));
    }

    #[test]
    fn test_rect_intersection() {
        let rect1 = Rect::new(Vec2::ZERO, Vec2::new(10.0, 10.0));
        let rect2 = Rect::new(Vec2::new(5.0, 5.0), Vec2::new(15.0, 15.0));
        
        assert!(rect1.intersects(&rect2));
        
        let intersection = rect1.intersection(&rect2).unwrap();
        assert_eq!(intersection.min, Vec2::new(5.0, 5.0));
        assert_eq!(intersection.max, Vec2::new(10.0, 10.0));
    }

    #[test]
    fn test_rect_union() {
        let rect1 = Rect::new(Vec2::ZERO, Vec2::new(5.0, 5.0));
        let rect2 = Rect::new(Vec2::new(3.0, 3.0), Vec2::new(8.0, 8.0));
        
        let union = rect1.union(&rect2);
        assert_eq!(union.min, Vec2::ZERO);
        assert_eq!(union.max, Vec2::new(8.0, 8.0));
    }

    #[test]
    fn test_circle_creation() {
        let circle = Circle::new(Vec2::new(5.0, 5.0), 3.0);
        assert_eq!(circle.center, Vec2::new(5.0, 5.0));
        assert_eq!(circle.radius, 3.0);
        assert_relative_eq!(circle.area(), std::f32::consts::PI * 9.0, epsilon = 1e-6);
    }

    #[test]
    fn test_circle_contains() {
        let circle = Circle::new(Vec2::ZERO, 5.0);
        assert!(circle.contains(Vec2::new(3.0, 4.0))); // 3-4-5 三角形
        assert!(!circle.contains(Vec2::new(4.0, 4.0))); // 超出半径
    }

    #[test]
    fn test_circle_intersection() {
        let circle1 = Circle::new(Vec2::ZERO, 5.0);
        let circle2 = Circle::new(Vec2::new(8.0, 0.0), 5.0);
        
        assert!(circle1.intersects(&circle2)); // 相交
        
        let circle3 = Circle::new(Vec2::new(12.0, 0.0), 5.0);
        assert!(!circle1.intersects(&circle3)); // 不相交
    }

    #[test]
    fn test_rect_circle_intersection() {
        let rect = Rect::new(Vec2::ZERO, Vec2::new(10.0, 10.0));
        let circle = Circle::new(Vec2::new(5.0, 5.0), 3.0);
        
        assert!(rect.intersects_circle(&circle));
        assert!(circle.intersects_rect(&rect));
    }

    #[test]
    fn test_bounds3d() {
        let bounds = Bounds3D::from_center_size(Vec3::ZERO, Vec3::ONE);
        assert_eq!(bounds.center(), Vec3::ZERO);
        assert_eq!(bounds.volume(), 1.0);
        assert!(bounds.contains(Vec3::new(0.4, 0.4, 0.4)));
        assert!(!bounds.contains(Vec3::new(0.6, 0.6, 0.6)));
    }

    #[test]
    fn test_rect_expand() {
        let rect = Rect::new(Vec2::new(2.0, 2.0), Vec2::new(8.0, 8.0));
        let expanded = rect.expand(1.0);
        
        assert_eq!(expanded.min, Vec2::new(1.0, 1.0));
        assert_eq!(expanded.max, Vec2::new(9.0, 9.0));
    }

    #[test]
    fn test_rect_auto_correct() {
        // 测试自动纠正最小值和最大值
        let rect = Rect::new(Vec2::new(10.0, 10.0), Vec2::ZERO);
        assert_eq!(rect.min, Vec2::ZERO);
        assert_eq!(rect.max, Vec2::new(10.0, 10.0));
    }
}
