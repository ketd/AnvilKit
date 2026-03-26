//! # 渲染插件系统
//!
//! 提供与 AnvilKit ECS 系统的集成，实现渲染功能的插件化。

use anvilkit_ecs::prelude::*;
use anvilkit_ecs::physics::DeltaTime;
use anvilkit_input::prelude::InputState;
use log::info;

use crate::window::WindowConfig;
use crate::renderer::assets::{MeshHandle, MaterialHandle, RenderAssets};
use crate::renderer::draw::{ActiveCamera, Aabb, DrawCommand, DrawCommandList, Frustum, SceneLights, MaterialParams};
use crate::renderer::state::RenderState;

/// 渲染插件
///
/// 将渲染系统集成到 AnvilKit ECS 应用中的插件。
///
/// # 示例
///
/// ```rust,no_run
/// use anvilkit_render::prelude::*;
/// use anvilkit_ecs::prelude::*;
///
/// // 创建应用并添加渲染插件
/// let mut app = App::new();
/// app.add_plugins(RenderPlugin::default())
///    .run();
/// ```
#[derive(Debug, Clone)]
pub struct RenderPlugin {
    /// 窗口配置
    window_config: WindowConfig,
}

impl Default for RenderPlugin {
    fn default() -> Self {
        Self {
            window_config: WindowConfig::default(),
        }
    }
}

impl RenderPlugin {
    /// 创建新的渲染插件
    ///
    /// # 示例
    ///
    /// ```rust
    /// use anvilkit_render::plugin::RenderPlugin;
    ///
    /// let plugin = RenderPlugin::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置窗口配置
    ///
    /// # 示例
    ///
    /// ```rust
    /// use anvilkit_render::prelude::*;
    ///
    /// let config = WindowConfig::new()
    ///     .with_title("我的游戏")
    ///     .with_size(1920, 1080);
    ///
    /// let plugin = RenderPlugin::new().with_window_config(config);
    /// ```
    pub fn with_window_config(mut self, config: WindowConfig) -> Self {
        self.window_config = config;
        self
    }

    /// 获取窗口配置
    ///
    /// # 示例
    ///
    /// ```rust
    /// use anvilkit_render::plugin::RenderPlugin;
    ///
    /// let plugin = RenderPlugin::new();
    /// let config = plugin.window_config();
    /// assert_eq!(config.title, "AnvilKit Application");
    /// ```
    pub fn window_config(&self) -> &WindowConfig {
        &self.window_config
    }
}

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        info!("构建渲染插件");

        // 添加渲染配置资源
        app.insert_resource(RenderConfig {
            window_config: self.window_config.clone(),
            ..Default::default()
        });

        // 注册 ECS 资源
        app.init_resource::<ActiveCamera>();
        app.init_resource::<DrawCommandList>();
        app.init_resource::<RenderAssets>();
        app.init_resource::<SceneLights>();
        app.insert_resource(InputState::new());
        app.init_resource::<DeltaTime>();

        // 帧捕获资源（capture feature）
        #[cfg(feature = "capture")]
        {
            app.init_resource::<crate::renderer::capture::CaptureState>();
        }

        // 添加真实 ECS 渲染系统到 PostUpdate 阶段
        app.add_systems(
            AnvilKitSchedule::PostUpdate,
            (
                camera_system,
                render_extract_system.after(camera_system),
            ),
        );

        info!("渲染插件构建完成");
    }
}

/// 渲染配置资源
///
/// 存储渲染系统的全局配置参数。
///
/// # 示例
///
/// ```rust
/// use anvilkit_render::plugin::RenderConfig;
///
/// let config = RenderConfig::default();
/// assert_eq!(config.msaa_samples, 4);
/// ```
#[derive(Debug, Clone, Resource)]
pub struct RenderConfig {
    /// 窗口配置
    pub window_config: WindowConfig,
    /// MSAA 采样数（默认 4，设为 1 禁用）
    pub msaa_samples: u32,
    /// 场景清除颜色 (linear RGBA)
    pub clear_color: [f32; 4],
    /// 默认背面剔除模式
    pub default_cull_mode: wgpu::Face,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            window_config: WindowConfig::default(),
            msaa_samples: 4,
            clear_color: [0.15, 0.3, 0.6, 1.0],
            default_cull_mode: wgpu::Face::Back,
        }
    }
}

/// 相机组件
///
/// 定义渲染视角和投影参数的组件。
///
/// # 示例
///
/// ```rust
/// use anvilkit_render::plugin::CameraComponent;
/// use anvilkit_core::math::Transform;
/// use glam::Vec3;
///
/// let camera = CameraComponent {
///     fov: 60.0,
///     near: 0.1,
///     far: 1000.0,
///     is_active: true,
///     aspect_ratio: 16.0 / 9.0,
///     ..Default::default()
/// };
/// ```
/// 相机投影模式
#[derive(Debug, Clone)]
pub enum Projection {
    /// 透视投影（3D 场景默认）
    Perspective {
        /// 垂直视野角度（度）
        fov: f32,
    },
    /// 正交投影（2D 场景、UI 等）
    Orthographic {
        /// 左边界
        left: f32,
        /// 右边界
        right: f32,
        /// 下边界
        bottom: f32,
        /// 上边界
        top: f32,
    },
}

impl Default for Projection {
    fn default() -> Self {
        Projection::Perspective { fov: 60.0 }
    }
}

#[derive(Debug, Clone, Component)]
pub struct CameraComponent {
    /// 投影模式
    pub projection: Projection,
    /// 视野角度（度）— 向后兼容，perspective 模式下等同于 projection.fov
    pub fov: f32,
    /// 近裁剪面
    pub near: f32,
    /// 远裁剪面
    pub far: f32,
    /// 是否激活
    pub is_active: bool,
    /// 宽高比（由 RenderApp 在 resize 时更新，或用户手动设置）
    pub aspect_ratio: f32,
    /// 渲染优先级（多相机时按优先级排序，高优先级先渲染）
    pub priority: i32,
}

impl Default for CameraComponent {
    fn default() -> Self {
        Self {
            projection: Projection::default(),
            fov: 60.0,
            near: 0.1,
            far: 1000.0,
            is_active: true,
            aspect_ratio: 16.0 / 9.0,
            priority: 0,
        }
    }
}

// ---------------------------------------------------------------------------
//  ECS 系统
// ---------------------------------------------------------------------------

/// 相机系统 (PostUpdate)
///
/// 查询 (CameraComponent, Transform) → 计算 view_proj → 写入 ActiveCamera
fn camera_system(
    camera_query: Query<(&CameraComponent, &Transform)>,
    render_state: Option<Res<RenderState>>,
    mut active_camera: ResMut<ActiveCamera>,
) {
    let Some((camera, transform)) = camera_query.iter().find(|(c, _)| c.is_active) else {
        return;
    };

    // 如果 RenderState 存在，用实际 surface size 计算 aspect ratio
    let aspect = if let Some(ref rs) = render_state {
        let (w, h) = rs.surface_size;
        w as f32 / h.max(1) as f32
    } else {
        camera.aspect_ratio
    };

    let eye = transform.translation;
    // LH 坐标系中，前方是 +Z
    let forward = transform.rotation * glam::Vec3::Z;
    let target = eye + forward;

    let view = glam::Mat4::look_at_lh(eye, target, glam::Vec3::Y);
    let proj = match &camera.projection {
        Projection::Perspective { fov } => {
            glam::Mat4::perspective_lh(fov.to_radians(), aspect, camera.near, camera.far)
        }
        Projection::Orthographic { left, right, bottom, top } => {
            glam::Mat4::orthographic_lh(*left, *right, *bottom, *top, camera.near, camera.far)
        }
    };

    active_camera.view_proj = proj * view;
    active_camera.camera_pos = eye;
    active_camera.fov_radians = match &camera.projection {
        Projection::Perspective { fov } => fov.to_radians(),
        Projection::Orthographic { .. } => std::f32::consts::FRAC_PI_4, // default for ortho
    };
}

/// 渲染提取系统 (PostUpdate, after camera_system)
///
/// 查询 (MeshHandle, MaterialHandle, GlobalTransform, Option<MaterialParams>, Option<Aabb>)
/// → 视锥体剔除 → 填充 DrawCommandList
///
/// Uses `GlobalTransform` (world-space) rather than local `Transform`,
/// so entities in a parent-child hierarchy render at their correct world position.
fn render_extract_system(
    query: Query<(&MeshHandle, &MaterialHandle, &GlobalTransform, Option<&MaterialParams>, Option<&Aabb>)>,
    std_mat_query: Query<(&MeshHandle, &crate::renderer::standard_material::StandardMaterial, &GlobalTransform, Option<&Aabb>), Without<MaterialHandle>>,
    active_camera: Res<ActiveCamera>,
    default_material: Option<Res<crate::renderer::standard_material::DefaultMaterialHandle>>,
    mut draw_list: ResMut<DrawCommandList>,
) {
    draw_list.clear();

    let frustum = Frustum::from_view_proj(&active_camera.view_proj);

    // Path 1: 传统 MaterialHandle 实体
    for (mesh, material, global_transform, mat_params, aabb) in query.iter() {
        let model = global_transform.0;

        if let Some(aabb) = aabb {
            let local_center = aabb.center();
            let world_center = model.transform_point3(local_center);
            let scale = global_transform.scale();
            let world_half = aabb.half_extents() * scale;

            if !frustum.intersects_aabb(world_center, world_half) {
                continue;
            }
        }

        let default_params = MaterialParams::default();
        let p = mat_params.unwrap_or(&default_params);

        draw_list.push(DrawCommand {
            mesh: *mesh,
            material: *material,
            model_matrix: model,
            metallic: p.metallic,
            roughness: p.roughness,
            normal_scale: p.normal_scale,
            emissive_factor: p.emissive_factor,
        });
    }

    // Path 2: StandardMaterial 实体（使用默认 PBR 管线）
    if let Some(default_mat) = default_material {
        for (mesh, std_mat, global_transform, aabb) in std_mat_query.iter() {
            let model = global_transform.0;

            if let Some(aabb) = aabb {
                let local_center = aabb.center();
                let world_center = model.transform_point3(local_center);
                let scale = global_transform.scale();
                let world_half = aabb.half_extents() * scale;

                if !frustum.intersects_aabb(world_center, world_half) {
                    continue;
                }
            }

            draw_list.push(DrawCommand {
                mesh: *mesh,
                material: default_mat.0,
                model_matrix: model,
                metallic: std_mat.metallic,
                roughness: std_mat.roughness,
                normal_scale: std_mat.normal_scale,
                emissive_factor: std_mat.emissive_factor,
            });
        }
    }

    // Sort for batching: group by material → mesh to minimize state changes
    draw_list.sort_for_batching();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_plugin_creation() {
        let plugin = RenderPlugin::new();
        assert_eq!(plugin.window_config().title, "AnvilKit Application");
    }

    #[test]
    fn test_render_plugin_with_config() {
        let config = WindowConfig::new()
            .with_title("Test Game")
            .with_size(800, 600);

        let plugin = RenderPlugin::new().with_window_config(config);
        assert_eq!(plugin.window_config().title, "Test Game");
        assert_eq!(plugin.window_config().width, 800);
        assert_eq!(plugin.window_config().height, 600);
    }

    #[test]
    fn test_camera_component_default() {
        let camera = CameraComponent::default();
        assert_eq!(camera.fov, 60.0);
        assert_eq!(camera.near, 0.1);
        assert_eq!(camera.far, 1000.0);
        assert!(camera.is_active);
    }

    #[test]
    fn test_render_plugin_default_config() {
        let plugin = RenderPlugin::new();
        assert_eq!(plugin.window_config().title, "AnvilKit Application");
        assert_eq!(plugin.window_config().width, 1280);
    }

    #[test]
    fn test_render_plugin_custom_window() {
        let config = WindowConfig::new()
            .with_title("Custom Window")
            .with_size(800, 600);
        let plugin = RenderPlugin::new().with_window_config(config);

        assert_eq!(plugin.window_config().title, "Custom Window");
        assert_eq!(plugin.window_config().width, 800);
        assert_eq!(plugin.window_config().height, 600);
    }

    #[test]
    fn test_render_config_default() {
        let config = RenderConfig::default();
        assert!(config.window_config.vsync);
        assert_eq!(config.msaa_samples, 4);
        assert_eq!(config.clear_color, [0.15, 0.3, 0.6, 1.0]);
    }

    #[test]
    fn test_camera_component_fields() {
        let camera = CameraComponent::default();
        assert!(camera.fov > 0.0);
        assert!(camera.near > 0.0);
        assert!(camera.far > camera.near);
    }
}
