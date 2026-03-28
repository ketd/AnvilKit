//! # 屏幕→世界射线投射
//!
//! 提供鼠标拾取和射线测试所需的数学工具函数。
//!
//! ## 功能
//!
//! - [`screen_to_ray`] — 将屏幕坐标转换为世界空间射线
//! - [`ray_plane_intersection`] — 射线与水平平面相交测试
//! - [`ray_sphere_intersection`] — 射线与球体相交测试

use glam::{Mat4, Vec2, Vec3};

/// 将屏幕坐标转换为世界空间射线
///
/// 通过反投影变换将 2D 屏幕坐标映射为 3D 世界空间的射线原点和方向。
///
/// # 参数
///
/// - `mouse_pos`: 鼠标屏幕坐标 (像素)，左上角为 (0,0)
/// - `window_size`: 窗口尺寸 (宽, 高) 像素
/// - `view_proj`: 视图-投影矩阵 (projection * view)
///
/// # 返回
///
/// `(origin, direction)` — 射线起点和归一化方向向量
pub fn screen_to_ray(mouse_pos: Vec2, window_size: Vec2, view_proj: &Mat4) -> (Vec3, Vec3) {
    let ndc_x = 2.0 * mouse_pos.x / window_size.x - 1.0;
    let ndc_y = 1.0 - 2.0 * mouse_pos.y / window_size.y;

    let inv_vp = view_proj.inverse();

    let near_clip = inv_vp * glam::Vec4::new(ndc_x, ndc_y, 0.0, 1.0);
    let far_clip = inv_vp * glam::Vec4::new(ndc_x, ndc_y, 1.0, 1.0);

    let near_world = Vec3::new(
        near_clip.x / near_clip.w,
        near_clip.y / near_clip.w,
        near_clip.z / near_clip.w,
    );
    let far_world = Vec3::new(
        far_clip.x / far_clip.w,
        far_clip.y / far_clip.w,
        far_clip.z / far_clip.w,
    );

    let direction = (far_world - near_world).normalize();
    (near_world, direction)
}

/// 射线与水平平面相交测试
///
/// 测试从 `origin` 沿 `direction` 发射的射线是否与 y=`plane_y` 的水平平面相交。
///
/// # 返回
///
/// `Some(hit_point)` — 交点的世界坐标，`None` — 射线与平面平行或交点在射线背后
pub fn ray_plane_intersection(origin: Vec3, direction: Vec3, plane_y: f32) -> Option<Vec3> {
    if direction.y.abs() < 1e-7 {
        return None;
    }

    let t = (plane_y - origin.y) / direction.y;
    if t < 0.0 {
        return None;
    }

    Some(origin + direction * t)
}

/// 射线与球体相交测试
///
/// 使用解析法测试射线是否与球体相交，返回最近交点的参数 t 值。
///
/// # 返回
///
/// `Some(t)` — 最近交点的参数值 (hit = origin + direction * t)，`None` — 未命中
pub fn ray_sphere_intersection(
    origin: Vec3,
    direction: Vec3,
    center: Vec3,
    radius: f32,
) -> Option<f32> {
    let oc = origin - center;
    let a = direction.dot(direction);
    let b = 2.0 * oc.dot(direction);
    let c = oc.dot(oc) - radius * radius;
    let discriminant = b * b - 4.0 * a * c;

    if discriminant < 0.0 {
        return None;
    }

    let sqrt_disc = discriminant.sqrt();
    let inv_2a = 1.0 / (2.0 * a);

    let t1 = (-b - sqrt_disc) * inv_2a;
    if t1 >= 0.0 {
        return Some(t1);
    }

    let t2 = (-b + sqrt_disc) * inv_2a;
    if t2 >= 0.0 {
        return Some(t2);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ray_plane_straight_down() {
        let hit = ray_plane_intersection(
            Vec3::new(0.0, 10.0, 0.0),
            Vec3::new(0.0, -1.0, 0.0),
            0.0,
        );
        assert!(hit.is_some());
        let p = hit.unwrap();
        assert!((p.x).abs() < 1e-5);
        assert!((p.y).abs() < 1e-5);
        assert!((p.z).abs() < 1e-5);
    }

    #[test]
    fn test_ray_plane_diagonal() {
        let hit = ray_plane_intersection(
            Vec3::new(0.0, 10.0, 0.0),
            Vec3::new(1.0, -1.0, 0.0).normalize(),
            0.0,
        );
        assert!(hit.is_some());
        let p = hit.unwrap();
        assert!((p.x - 10.0).abs() < 1e-4);
        assert!((p.y).abs() < 1e-4);
    }

    #[test]
    fn test_ray_plane_parallel() {
        let hit = ray_plane_intersection(
            Vec3::new(0.0, 5.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            0.0,
        );
        assert!(hit.is_none());
    }

    #[test]
    fn test_ray_plane_behind() {
        let hit = ray_plane_intersection(
            Vec3::new(0.0, 5.0, 0.0),
            Vec3::new(0.0, -1.0, 0.0),
            10.0,
        );
        assert!(hit.is_none());
    }

    #[test]
    fn test_ray_sphere_hit() {
        let t = ray_sphere_intersection(
            Vec3::new(0.0, 0.0, -5.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::ZERO,
            1.0,
        );
        assert!(t.is_some());
        assert!((t.unwrap() - 4.0).abs() < 1e-5);
    }

    #[test]
    fn test_ray_sphere_miss() {
        let t = ray_sphere_intersection(
            Vec3::new(0.0, 5.0, -5.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::ZERO,
            1.0,
        );
        assert!(t.is_none());
    }

    #[test]
    fn test_ray_sphere_inside() {
        let t = ray_sphere_intersection(
            Vec3::ZERO,
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::ZERO,
            2.0,
        );
        assert!(t.is_some());
        assert!((t.unwrap() - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_ray_sphere_tangent() {
        let t = ray_sphere_intersection(
            Vec3::new(-5.0, 1.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::ZERO,
            1.0,
        );
        assert!(t.is_some());
        assert!((t.unwrap() - 5.0).abs() < 1e-4);
    }

    #[test]
    fn test_screen_to_ray_center() {
        let vp = Mat4::IDENTITY;
        let (origin, dir) = screen_to_ray(
            Vec2::new(640.0, 360.0),
            Vec2::new(1280.0, 720.0),
            &vp,
        );
        assert!((origin.x).abs() < 1e-3);
        assert!((origin.y).abs() < 1e-3);
        assert!(dir.z > 0.9);
    }

    #[test]
    fn test_screen_to_ray_with_perspective() {
        let view = Mat4::look_at_lh(
            Vec3::new(0.0, 10.0, 0.0),
            Vec3::ZERO,
            Vec3::Z,
        );
        let proj = Mat4::perspective_lh(60f32.to_radians(), 1.0, 0.1, 100.0);
        let vp = proj * view;

        let (origin, dir) = screen_to_ray(
            Vec2::new(400.0, 400.0),
            Vec2::new(800.0, 800.0),
            &vp,
        );

        assert!(dir.y < 0.0, "Expected downward ray, got dir.y={}", dir.y);
        let hit = ray_plane_intersection(origin, dir, 0.0);
        assert!(hit.is_some(), "Expected ray to hit y=0 plane");
    }
}
