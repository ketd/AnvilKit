//! # 绘制命令、相机资源和场景灯光
//!
//! 提供 ECS 渲染系统的中间表示：绘制命令列表、活动相机信息和场景灯光。

mod culling;
mod lighting;
mod commands;
mod gpu;

pub use culling::{Aabb, Frustum};
pub use lighting::{ActiveCamera, DirectionalLight, PointLight, SpotLight, SceneLights, MAX_SHADOW_LIGHTS};
pub use commands::{MaterialParams, DrawCommand, DrawCommandList};
pub use gpu::{UniformBatchBuffer, RenderTarget, InstanceData};

#[cfg(test)]
mod tests {
    use super::*;
    use glam::{Mat4, Vec3};

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

    #[test]
    fn test_active_camera_fov_default() {
        let camera = ActiveCamera::default();
        assert!((camera.fov_radians - std::f32::consts::FRAC_PI_4).abs() < 0.001);
    }

    #[test]
    fn test_active_camera_fov_custom() {
        let mut camera = ActiveCamera::default();
        camera.fov_radians = 60.0_f32.to_radians();
        assert!((camera.fov_radians - 60.0_f32.to_radians()).abs() < 0.001);
    }

    #[test]
    fn test_uniform_batch_buffer() {
        let mut buf = UniformBatchBuffer::new(256);
        assert_eq!(buf.count(), 0);

        let offset1 = buf.push(&[1u8; 128]);
        assert_eq!(offset1, 0);
        assert_eq!(buf.count(), 1);
        assert_eq!(buf.size(), 256); // padded to alignment

        let offset2 = buf.push(&[2u8; 64]);
        assert_eq!(offset2, 256);
        assert_eq!(buf.count(), 2);
        assert_eq!(buf.size(), 512);

        buf.clear();
        assert_eq!(buf.count(), 0);
    }

    #[test]
    fn test_render_target_default_is_screen() {
        let target = RenderTarget::default();
        assert!(matches!(target, RenderTarget::Screen));
    }
}
