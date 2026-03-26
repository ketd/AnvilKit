//! # 绘制命令、相机资源和场景灯光
//!
//! 提供 ECS 渲染系统的中间表示：绘制命令列表、活动相机信息和场景灯光。

use bevy_ecs::prelude::*;
use glam::{Mat4, Vec3};

use crate::renderer::assets::{MeshHandle, MaterialHandle};

/// 轴对齐包围盒 (Axis-Aligned Bounding Box)
///
/// 用于快速视锥体剔除。附加到实体上表示其局部空间包围盒。
///
/// # 示例
///
/// ```rust
/// use anvilkit_render::renderer::draw::Aabb;
/// use glam::Vec3;
///
/// let aabb = Aabb::from_min_max(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
/// assert_eq!(aabb.center(), Vec3::ZERO);
/// assert_eq!(aabb.half_extents(), Vec3::ONE);
/// ```
#[derive(Debug, Clone, Copy, Component)]
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

/// 视锥体 (6 个裁剪平面)
///
/// 从 view-projection 矩阵提取，用于快速剔除不可见物体。
/// 每个平面以 (normal.xyz, distance) 格式存储，法线指向锥体内部。
#[derive(Debug, Clone, Copy)]
pub struct Frustum {
    /// 6 个裁剪平面: left, right, bottom, top, near, far
    pub planes: [glam::Vec4; 6],
}

impl Frustum {
    /// 从 view-projection 矩阵提取视锥体平面
    ///
    /// 使用 Gribb/Hartmann 方法从组合矩阵提取平面。
    pub fn from_view_proj(vp: &Mat4) -> Self {
        let m = vp.to_cols_array_2d();
        let row = |r: usize| -> glam::Vec4 {
            glam::Vec4::new(m[0][r], m[1][r], m[2][r], m[3][r])
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
            let len = glam::Vec3::new(p.x, p.y, p.z).length();
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
            let normal = glam::Vec3::new(plane.x, plane.y, plane.z);
            let d = plane.w;
            // 计算 AABB 沿平面法线方向的最大投影半径
            let r = half_extents.x * normal.x.abs()
                + half_extents.y * normal.y.abs()
                + half_extents.z * normal.z.abs();
            // 中心到平面的有符号距离
            let dist = normal.dot(center) + d;
            if dist < -r {
                return false; // 完全在平面外侧
            }
        }
        true
    }
}

/// 活动相机资源
///
/// 由 camera_system 每帧写入，包含当前激活相机的视图投影矩阵。
#[derive(Resource)]
pub struct ActiveCamera {
    /// Combined view-projection matrix of the active camera.
    pub view_proj: Mat4,
    /// World-space position of the active camera.
    pub camera_pos: Vec3,
    /// Vertical field of view in radians (used by CSM shadow mapping).
    pub fov_radians: f32,
}

impl Default for ActiveCamera {
    fn default() -> Self {
        Self {
            view_proj: Mat4::IDENTITY,
            camera_pos: Vec3::ZERO,
            fov_radians: std::f32::consts::FRAC_PI_4,
        }
    }
}

/// 方向光
#[derive(Debug, Clone)]
pub struct DirectionalLight {
    /// 光照方向（从光源指向场景）
    pub direction: Vec3,
    /// 光照颜色 (linear RGB)
    pub color: Vec3,
    /// 光照强度
    pub intensity: f32,
}

impl Default for DirectionalLight {
    fn default() -> Self {
        Self {
            direction: Vec3::new(-0.5, -0.8, 0.3).normalize(),
            color: Vec3::new(1.0, 0.95, 0.9),
            intensity: 5.0,
        }
    }
}

/// 点光源
#[derive(Debug, Clone)]
pub struct PointLight {
    /// 世界空间位置
    pub position: Vec3,
    /// 光照颜色 (linear RGB)
    pub color: Vec3,
    /// 光照强度
    pub intensity: f32,
    /// 衰减距离（超出此距离光照为零）
    pub range: f32,
}

impl Default for PointLight {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 3.0, 0.0),
            color: Vec3::ONE,
            intensity: 5.0,
            range: 10.0,
        }
    }
}

/// 聚光灯
#[derive(Debug, Clone)]
pub struct SpotLight {
    /// 世界空间位置
    pub position: Vec3,
    /// 光照方向（从光源指向场景）
    pub direction: Vec3,
    /// 光照颜色 (linear RGB)
    pub color: Vec3,
    /// 光照强度
    pub intensity: f32,
    /// 衰减距离
    pub range: f32,
    /// 内锥角（弧度），全亮区域
    pub inner_cone_angle: f32,
    /// 外锥角（弧度），衰减到零
    pub outer_cone_angle: f32,
}

impl Default for SpotLight {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 3.0, 0.0),
            direction: Vec3::new(0.0, -1.0, 0.0),
            color: Vec3::ONE,
            intensity: 10.0,
            range: 15.0,
            inner_cone_angle: 0.35,  // ~20 degrees
            outer_cone_angle: 0.52,  // ~30 degrees
        }
    }
}

/// 场景灯光资源
///
/// 持有场景中所有灯光信息，最多 8 盏（1 方向光 + 点光/聚光组合）。
#[derive(Resource)]
pub struct SceneLights {
    /// The primary directional (sun) light.
    pub directional: DirectionalLight,
    /// All active point lights in the scene.
    pub point_lights: Vec<PointLight>,
    /// All active spot lights in the scene.
    pub spot_lights: Vec<SpotLight>,
}

impl Default for SceneLights {
    fn default() -> Self {
        Self {
            directional: DirectionalLight::default(),
            point_lights: Vec::new(),
            spot_lights: Vec::new(),
        }
    }
}

/// 材质参数组件
///
/// 附加到实体上以控制 PBR 材质参数。
/// 如果实体没有此组件，render_extract_system 使用默认值 (metallic=0, roughness=0.5, normal_scale=1.0)。
#[derive(Debug, Clone, Component)]
pub struct MaterialParams {
    /// Metalness factor (0 = dielectric, 1 = metal).
    pub metallic: f32,
    /// Surface roughness (0 = mirror, 1 = diffuse).
    pub roughness: f32,
    /// Normal map intensity multiplier.
    pub normal_scale: f32,
    /// Emissive color factor [R, G, B].
    pub emissive_factor: [f32; 3],
}

impl Default for MaterialParams {
    fn default() -> Self {
        Self {
            metallic: 0.0,
            roughness: 0.5,
            normal_scale: 1.0,
            emissive_factor: [0.0; 3],
        }
    }
}

/// 单个绘制命令
pub struct DrawCommand {
    /// Handle to the GPU mesh to draw.
    pub mesh: MeshHandle,
    /// Handle to the GPU material (pipeline + bind group).
    pub material: MaterialHandle,
    /// Object-to-world transform matrix.
    pub model_matrix: Mat4,
    /// PBR metalness factor for this draw.
    pub metallic: f32,
    /// PBR roughness factor for this draw.
    pub roughness: f32,
    /// Normal map intensity for this draw.
    pub normal_scale: f32,
    /// Emissive color factor [R, G, B] for this draw.
    pub emissive_factor: [f32; 3],
}

/// 每帧的绘制命令列表
///
/// 由 render_extract_system 填充，由 RenderApp::render_ecs() 消费。
/// 支持按 mesh+material 排序分组以减少管线状态切换。
#[derive(Resource, Default)]
pub struct DrawCommandList {
    /// Collected draw commands for the current frame.
    pub commands: Vec<DrawCommand>,
}

impl DrawCommandList {
    /// Removes all draw commands from the list.
    pub fn clear(&mut self) {
        self.commands.clear();
    }

    /// Appends a draw command to the list.
    pub fn push(&mut self, cmd: DrawCommand) {
        self.commands.push(cmd);
    }

    /// 按 (material, mesh) 排序以实现批处理
    ///
    /// 相同 material 的命令排在一起，减少管线状态切换。
    /// 相同 mesh 的命令排在一起，减少顶点缓冲区切换。
    pub fn sort_for_batching(&mut self) {
        self.commands.sort_by(|a, b| {
            a.material.index().cmp(&b.material.index())
                .then(a.mesh.index().cmp(&b.mesh.index()))
        });
    }
}

/// GPU 实例数据（per-instance，128 字节）
///
/// 包含每个实例的变换和材质参数。
/// 用于 GPU instancing 时通过 storage buffer 传递。
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceData {
    /// Object-to-world model matrix (64 bytes).
    pub model: [[f32; 4]; 4],
    /// Inverse-transpose model matrix for normals (64 bytes).
    pub normal_matrix: [[f32; 4]; 4],
}

impl Default for InstanceData {
    fn default() -> Self {
        Self {
            model: glam::Mat4::IDENTITY.to_cols_array_2d(),
            normal_matrix: glam::Mat4::IDENTITY.to_cols_array_2d(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_directional_light_default() {
        let light = DirectionalLight::default();
        assert!(light.direction.length() > 0.99);
        assert!(light.intensity > 0.0);
    }

    #[test]
    fn test_scene_lights_default() {
        let lights = SceneLights::default();
        assert!(lights.directional.intensity > 0.0);
    }

    #[test]
    fn test_material_params_default() {
        let params = MaterialParams::default();
        assert_eq!(params.metallic, 0.0);
        assert_eq!(params.roughness, 0.5);
        assert_eq!(params.normal_scale, 1.0);
    }

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
    fn test_frustum_contains_origin() {
        // A simple perspective-like VP that should contain the origin in front of the camera
        let view = Mat4::look_at_lh(Vec3::new(0.0, 0.0, -5.0), Vec3::ZERO, Vec3::Y);
        let proj = Mat4::perspective_lh(60.0_f32.to_radians(), 1.0, 0.1, 100.0);
        let frustum = Frustum::from_view_proj(&(proj * view));

        // Origin should be visible
        assert!(frustum.intersects_aabb(Vec3::ZERO, Vec3::splat(0.5)));

        // Far behind the camera should not be visible
        assert!(!frustum.intersects_aabb(Vec3::new(0.0, 0.0, -100.0), Vec3::splat(0.5)));
    }
}
