//! 视锥体剔除

use glam::{Mat4, Vec3, Vec4};

/// 视锥体 (6 个裁剪平面)
///
/// 从 view-projection 矩阵提取，用于快速剔除不可见物体。
/// 每个平面以 (normal.xyz, distance) 格式存储，法线指向锥体内部。
#[derive(Debug, Clone, Copy)]
pub struct Frustum {
    /// 6 个裁剪平面: left, right, bottom, top, near, far
    pub planes: [Vec4; 6],
}

impl Frustum {
    /// 从 view-projection 矩阵提取视锥体平面
    ///
    /// 使用 Gribb/Hartmann 方法从组合矩阵提取平面。
    pub fn from_view_proj(vp: &Mat4) -> Self {
        let m = vp.to_cols_array_2d();
        let row = |r: usize| -> Vec4 {
            Vec4::new(m[0][r], m[1][r], m[2][r], m[3][r])
        };
        let r0 = row(0);
        let r1 = row(1);
        let r2 = row(2);
        let r3 = row(3);

        let mut planes = [
            r3 + r0,  // left
            r3 - r0,  // right
            r3 + r1,  // bottom
            r3 - r1,  // top
            r2,       // near (LH: z >= 0)
            r3 - r2,  // far
        ];

        // 归一化每个平面
        for p in &mut planes {
            let len = Vec3::new(p.x, p.y, p.z).length();
            if len > 0.0 {
                *p /= len;
            }
        }

        Self { planes }
    }

    /// 测试世界空间 AABB 是否与视锥体相交
    ///
    /// 使用 AABB 的中心+半尺寸与每个平面的有符号距离测试。
    /// 如果 AABB 完全在任一平面外侧，返回 false（不可见）。
    pub fn intersects_aabb(&self, center: Vec3, half_extents: Vec3) -> bool {
        for plane in &self.planes {
            let normal = Vec3::new(plane.x, plane.y, plane.z);
            let d = plane.w;
            let r = half_extents.x * normal.x.abs()
                + half_extents.y * normal.y.abs()
                + half_extents.z * normal.z.abs();
            let dist = normal.dot(center) + d;
            if dist < -r {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frustum_contains_origin() {
        let view = Mat4::look_at_lh(Vec3::new(0.0, 0.0, -5.0), Vec3::ZERO, Vec3::Y);
        let proj = Mat4::perspective_lh(60.0_f32.to_radians(), 1.0, 0.1, 100.0);
        let frustum = Frustum::from_view_proj(&(proj * view));

        assert!(frustum.intersects_aabb(Vec3::ZERO, Vec3::splat(0.5)));
        assert!(!frustum.intersects_aabb(Vec3::new(0.0, 0.0, -100.0), Vec3::splat(0.5)));
    }
}
