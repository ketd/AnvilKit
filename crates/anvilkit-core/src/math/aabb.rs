//! 轴对齐包围盒 (Axis-Aligned Bounding Box)

use glam::Vec3;

/// 轴对齐包围盒 (Axis-Aligned Bounding Box)
///
/// 用于快速视锥体剔除。附加到实体上表示其局部空间包围盒。
///
/// # 示例
///
/// ```rust
/// use anvilkit_core::math::Aabb;
/// use glam::Vec3;
///
/// let aabb = Aabb::from_min_max(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
/// assert_eq!(aabb.center(), Vec3::ZERO);
/// assert_eq!(aabb.half_extents(), Vec3::ONE);
/// ```
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "bevy_ecs", derive(bevy_ecs::prelude::Component))]
pub struct Aabb {
    /// Minimum corner of the bounding box.
    pub min: Vec3,
    /// Maximum corner of the bounding box.
    pub max: Vec3,
}

impl Aabb {
    /// 从最小/最大点创建 AABB
    pub fn from_min_max(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    /// 从顶点位置列表计算 AABB
    ///
    /// 如果 `points` 为空，返回 `None`。
    pub fn from_points(points: &[Vec3]) -> Option<Self> {
        if points.is_empty() {
            return None;
        }
        let mut min = Vec3::splat(f32::MAX);
        let mut max = Vec3::splat(f32::MIN);
        for &p in points {
            min = min.min(p);
            max = max.max(p);
        }
        Some(Self { min, max })
    }

    /// 中心点
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    /// 半尺寸
    pub fn half_extents(&self) -> Vec3 {
        (self.max - self.min) * 0.5
    }

    /// 测试两个 AABB 是否相交
    pub fn intersects(&self, other: &Aabb) -> bool {
        self.min.x <= other.max.x && self.max.x >= other.min.x
            && self.min.y <= other.max.y && self.max.y >= other.min.y
            && self.min.z <= other.max.z && self.max.z >= other.min.z
    }

    /// 将 AABB 按偏移量平移
    pub fn translated(&self, offset: Vec3) -> Aabb {
        Aabb {
            min: self.min + offset,
            max: self.max + offset,
        }
    }
}

impl Default for Aabb {
    fn default() -> Self {
        Self {
            min: Vec3::splat(-0.5),
            max: Vec3::splat(0.5),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aabb_from_points() {
        let aabb = Aabb::from_points(&[
            Vec3::new(-1.0, -2.0, -3.0),
            Vec3::new(4.0, 5.0, 6.0),
        ]).expect("non-empty points should return Some");
        assert_eq!(aabb.min, Vec3::new(-1.0, -2.0, -3.0));
        assert_eq!(aabb.max, Vec3::new(4.0, 5.0, 6.0));
        assert_eq!(aabb.center(), Vec3::new(1.5, 1.5, 1.5));
        assert_eq!(aabb.half_extents(), Vec3::new(2.5, 3.5, 4.5));
    }

    #[test]
    fn test_aabb_from_points_empty() {
        assert!(Aabb::from_points(&[]).is_none());
    }

    #[test]
    fn test_aabb_intersects() {
        let a = Aabb::from_min_max(Vec3::ZERO, Vec3::ONE);
        let b = Aabb::from_min_max(Vec3::splat(0.5), Vec3::splat(1.5));
        assert!(a.intersects(&b));

        let c = Aabb::from_min_max(Vec3::splat(2.0), Vec3::splat(3.0));
        assert!(!a.intersects(&c));
    }

    #[test]
    fn test_aabb_translated() {
        let aabb = Aabb::from_min_max(Vec3::ZERO, Vec3::ONE);
        let moved = aabb.translated(Vec3::new(5.0, 0.0, 0.0));
        assert_eq!(moved.min, Vec3::new(5.0, 0.0, 0.0));
        assert_eq!(moved.max, Vec3::new(6.0, 1.0, 1.0));
    }
}
