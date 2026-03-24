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
/// use anvilkit_render::window::WindowConfig;
///
/// let config = RenderConfig {
///     window_config: WindowConfig::default(),
/// };
/// ```
#[derive(Debug, Clone, Resource)]
pub struct RenderConfig {
    /// 窗口配置
    pub window_config: WindowConfig,
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
/// };
/// ```
#[derive(Debug, Clone, Component)]
pub struct CameraComponent {
    /// 视野角度（度）
    pub fov: f32,
    /// 近裁剪面
    pub near: f32,
    /// 远裁剪面
    pub far: f32,
    /// 是否激活
    pub is_active: bool,
    /// 宽高比（由 RenderApp 在 resize 时更新，或用户手动设置）
    pub aspect_ratio: f32,
}

impl Default for CameraComponent {
    fn default() -> Self {
        Self {
            fov: 60.0,
            near: 0.1,
            far: 1000.0,
            is_active: true,
            aspect_ratio: 16.0 / 9.0,
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
    let proj = glam::Mat4::perspective_lh(camera.fov.to_radians(), aspect, camera.near, camera.far);

    active_camera.view_proj = proj * view;
    active_camera.camera_pos = eye;
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
    active_camera: Res<ActiveCamera>,
    mut draw_list: ResMut<DrawCommandList>,
) {
    draw_list.clear();

    let frustum = Frustum::from_view_proj(&active_camera.view_proj);

    for (mesh, material, global_transform, mat_params, aabb) in query.iter() {
        let model = global_transform.0;

        // Frustum culling: if entity has an Aabb, test visibility
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
        let config = RenderConfig {
            window_config: WindowConfig::default(),
        };
        assert!(config.window_config.vsync);
    }

    #[test]
    fn test_camera_component_fields() {
        let camera = CameraComponent::default();
        assert!(camera.fov > 0.0);
        assert!(camera.near > 0.0);
        assert!(camera.far > camera.near);
    }
}
