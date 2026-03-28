use crate::renderer::draw::SceneLights;
use crate::renderer::state::{GpuLight, MAX_LIGHTS};

/// 将 SceneLights 打包为 GPU 光源数组
///
/// 返回 (lights_array, light_count)。方向光占 slot 0，其余填充点光和聚光。
/// 可被游戏和示例直接调用，不必复制此函数。
pub fn pack_lights(scene_lights: &SceneLights) -> ([GpuLight; MAX_LIGHTS], u32) {
    let mut lights = [GpuLight::default(); MAX_LIGHTS];
    let mut count = 0u32;

    // Slot 0: directional light (type=0)
    let dir = &scene_lights.directional;
    lights[0] = GpuLight {
        position_type: [0.0, 0.0, 0.0, 0.0], // type=0 directional
        direction_range: [dir.direction.x, dir.direction.y, dir.direction.z, 0.0],
        color_intensity: [dir.color.x, dir.color.y, dir.color.z, dir.intensity],
        params: [0.0; 4],
    };
    count += 1;

    // Point lights (type=1)
    for pl in &scene_lights.point_lights {
        if count as usize >= MAX_LIGHTS { break; }
        lights[count as usize] = GpuLight {
            position_type: [pl.position.x, pl.position.y, pl.position.z, 1.0],
            direction_range: [0.0, 0.0, 0.0, pl.range],
            color_intensity: [pl.color.x, pl.color.y, pl.color.z, pl.intensity],
            params: [0.0; 4],
        };
        count += 1;
    }

    // Spot lights (type=2)
    for sl in &scene_lights.spot_lights {
        if count as usize >= MAX_LIGHTS { break; }
        lights[count as usize] = GpuLight {
            position_type: [sl.position.x, sl.position.y, sl.position.z, 2.0],
            direction_range: [sl.direction.x, sl.direction.y, sl.direction.z, sl.range],
            color_intensity: [sl.color.x, sl.color.y, sl.color.z, sl.intensity],
            params: [sl.inner_cone_angle.cos(), sl.outer_cone_angle.cos(), 0.0, 0.0],
        };
        count += 1;
    }

    (lights, count)
}

/// Cascade Shadow Maps 默认分割比例（视锥体远平面百分比）
const CSM_SPLIT_RATIOS: [f32; 3] = [0.1, 0.3, 1.0];

/// 计算 CSM 各级 cascade 的光空间矩阵
///
/// 将相机视锥体按 `CSM_SPLIT_RATIOS` 分割，每个子锥体紧密包围一个正交投影。
/// 返回 (cascade_matrices, cascade_split_distances)。
pub fn compute_cascade_matrices(
    light_direction: &glam::Vec3,
    view: &glam::Mat4,
    fov: f32,
    aspect: f32,
    near: f32,
    far: f32,
) -> ([glam::Mat4; 3], [f32; 3]) {
    let light_dir = light_direction.normalize();
    let _inv_view = view.inverse();

    let mut matrices = [glam::Mat4::IDENTITY; 3];
    let mut splits = [0.0f32; 3];
    let mut prev_split = near;

    for (i, &ratio) in CSM_SPLIT_RATIOS.iter().enumerate() {
        let split_far = near + (far - near) * ratio;
        splits[i] = split_far;

        // Compute frustum corners for this cascade slice
        let proj = glam::Mat4::perspective_lh(fov, aspect, prev_split, split_far);
        let inv_vp = (proj * *view).inverse();

        // NDC corners → world-space
        let ndc_corners = [
            glam::Vec3::new(-1.0, -1.0, 0.0), glam::Vec3::new(1.0, -1.0, 0.0),
            glam::Vec3::new(-1.0,  1.0, 0.0), glam::Vec3::new(1.0,  1.0, 0.0),
            glam::Vec3::new(-1.0, -1.0, 1.0), glam::Vec3::new(1.0, -1.0, 1.0),
            glam::Vec3::new(-1.0,  1.0, 1.0), glam::Vec3::new(1.0,  1.0, 1.0),
        ];

        let mut world_corners = [glam::Vec3::ZERO; 8];
        let mut center = glam::Vec3::ZERO;
        for (j, ndc) in ndc_corners.iter().enumerate() {
            let clip = inv_vp * glam::Vec4::new(ndc.x, ndc.y, ndc.z, 1.0);
            world_corners[j] = clip.truncate() / clip.w;
            center += world_corners[j];
        }
        center /= 8.0;

        // Build light view looking at the center of the frustum slice
        let light_pos = center - light_dir * 50.0;
        let up = if light_dir.y.abs() > 0.99 { glam::Vec3::Z } else { glam::Vec3::Y };
        let light_view = glam::Mat4::look_at_lh(light_pos, center, up);

        // Find bounding box in light space
        let mut min_ls = glam::Vec3::splat(f32::MAX);
        let mut max_ls = glam::Vec3::splat(f32::MIN);
        for c in &world_corners {
            let ls = (light_view * glam::Vec4::new(c.x, c.y, c.z, 1.0)).truncate();
            min_ls = min_ls.min(ls);
            max_ls = max_ls.max(ls);
        }

        // Add margin to avoid edge clipping
        let margin = (max_ls - min_ls).max_element() * 0.1;
        min_ls -= glam::Vec3::splat(margin);
        max_ls += glam::Vec3::splat(margin);

        let light_proj = glam::Mat4::orthographic_lh(
            min_ls.x, max_ls.x, min_ls.y, max_ls.y,
            min_ls.z - 50.0, max_ls.z + 50.0,
        );

        matrices[i] = light_proj * light_view;
        prev_split = split_far;
    }

    (matrices, splits)
}

/// Legacy: compute a single light-space matrix (for backward compatibility).
pub fn compute_light_space_matrix(light_direction: &glam::Vec3) -> glam::Mat4 {
    let light_dir = light_direction.normalize();
    let light_pos = -light_dir * 15.0;
    let light_view = glam::Mat4::look_at_lh(light_pos, glam::Vec3::ZERO, glam::Vec3::Y);
    let light_proj = glam::Mat4::orthographic_lh(-10.0, 10.0, -10.0, 10.0, 0.1, 30.0);
    light_proj * light_view
}
