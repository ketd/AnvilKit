//! # 渲染插件系统
//! 
//! 提供与 AnvilKit ECS 系统的集成，实现渲染功能的插件化。

use anvilkit_ecs::prelude::*;
use anvilkit_core::error::{AnvilKitError, Result};
use log::{info, warn, error, debug};

use crate::window::{RenderApp, WindowConfig};

/// 渲染插件
/// 
/// 将渲染系统集成到 AnvilKit ECS 应用中的插件。
/// 
/// # 设计理念
/// 
/// - **插件化**: 作为 ECS 插件提供渲染功能
/// - **资源管理**: 自动管理渲染相关的资源和组件
/// - **系统集成**: 与 ECS 系统调度器无缝集成
/// - **配置灵活**: 支持自定义窗口和渲染配置
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
    /// # 参数
    /// 
    /// - `config`: 窗口配置
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
    /// # 返回
    /// 
    /// 返回当前的窗口配置
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
        
        // 添加渲染相关的资源
        app.insert_resource(RenderConfig {
            window_config: self.window_config.clone(),
        });
        
        // 添加渲染相关的组件
        app.register_component::<RenderComponent>();
        app.register_component::<CameraComponent>();
        app.register_component::<MeshComponent>();
        app.register_component::<MaterialComponent>();
        
        // 添加渲染系统
        app.add_systems(
            AnvilKitSchedule::Update,
            (
                render_system,
                camera_system,
                mesh_system,
            ).in_set(RenderSystemSet::Render),
        );
        
        // 添加系统集合
        app.configure_sets(
            AnvilKitSchedule::Update,
            RenderSystemSet::Render.after(AnvilKitSystemSet::Update),
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

/// 渲染组件
/// 
/// 标记实体需要进行渲染的组件。
/// 
/// # 示例
/// 
/// ```rust
/// use anvilkit_render::plugin::RenderComponent;
/// use anvilkit_ecs::prelude::*;
/// 
/// // 创建带有渲染组件的实体
/// let mut world = World::new();
/// let entity = world.spawn(RenderComponent::default()).id();
/// ```
#[derive(Debug, Clone, Component)]
pub struct RenderComponent {
    /// 是否可见
    pub visible: bool,
    /// 渲染层级
    pub layer: u32,
}

impl Default for RenderComponent {
    fn default() -> Self {
        Self {
            visible: true,
            layer: 0,
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
}

impl Default for CameraComponent {
    fn default() -> Self {
        Self {
            fov: 60.0,
            near: 0.1,
            far: 1000.0,
            is_active: true,
        }
    }
}

/// 网格组件
/// 
/// 定义实体的几何网格数据。
/// 
/// # 示例
/// 
/// ```rust
/// use anvilkit_render::plugin::MeshComponent;
/// 
/// let mesh = MeshComponent {
///     mesh_id: "cube".to_string(),
///     vertex_count: 24,
///     index_count: 36,
/// };
/// ```
#[derive(Debug, Clone, Component)]
pub struct MeshComponent {
    /// 网格 ID
    pub mesh_id: String,
    /// 顶点数量
    pub vertex_count: u32,
    /// 索引数量
    pub index_count: u32,
}

/// 材质组件
/// 
/// 定义实体的材质和着色参数。
/// 
/// # 示例
/// 
/// ```rust
/// use anvilkit_render::plugin::MaterialComponent;
/// use glam::Vec3;
/// 
/// let material = MaterialComponent {
///     material_id: "default".to_string(),
///     color: Vec3::new(1.0, 1.0, 1.0),
///     metallic: 0.0,
///     roughness: 0.5,
/// };
/// ```
#[derive(Debug, Clone, Component)]
pub struct MaterialComponent {
    /// 材质 ID
    pub material_id: String,
    /// 基础颜色
    pub color: glam::Vec3,
    /// 金属度
    pub metallic: f32,
    /// 粗糙度
    pub roughness: f32,
}

/// 渲染系统集合
/// 
/// 定义渲染相关系统的执行顺序。
/// 
/// # 示例
/// 
/// ```rust
/// use anvilkit_render::plugin::RenderSystemSet;
/// use anvilkit_ecs::prelude::*;
/// 
/// // 配置系统集合
/// // app.configure_sets(
/// //     AnvilKitSchedule::Update,
/// //     RenderSystemSet::Render.after(AnvilKitSystemSet::Update),
/// // );
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub enum RenderSystemSet {
    /// 渲染系统
    Render,
    /// 相机系统
    Camera,
    /// 网格系统
    Mesh,
    /// 材质系统
    Material,
}

/// 渲染系统
/// 
/// 执行主要的渲染逻辑。
/// 
/// # 参数
/// 
/// - `render_query`: 查询需要渲染的实体
/// - `camera_query`: 查询相机实体
fn render_system(
    render_query: Query<(Entity, &RenderComponent, &Transform)>,
    camera_query: Query<(Entity, &CameraComponent, &Transform)>,
) {
    // 查找激活的相机
    let active_camera = camera_query
        .iter()
        .find(|(_, camera, _)| camera.is_active);
    
    if active_camera.is_none() {
        return; // 没有激活的相机，跳过渲染
    }
    
    // 渲染所有可见的实体
    for (_entity, render_comp, _transform) in render_query.iter() {
        if render_comp.visible {
            // 执行渲染逻辑
            debug!("渲染实体");
        }
    }
}

/// 相机系统
/// 
/// 更新相机相关的逻辑。
/// 
/// # 参数
/// 
/// - `camera_query`: 查询相机实体
fn camera_system(
    mut camera_query: Query<(Entity, &mut CameraComponent, &Transform)>,
) {
    for (_entity, mut _camera, _transform) in camera_query.iter_mut() {
        // 更新相机逻辑
        debug!("更新相机");
    }
}

/// 网格系统
/// 
/// 管理网格资源和渲染数据。
/// 
/// # 参数
/// 
/// - `mesh_query`: 查询网格实体
fn mesh_system(
    mesh_query: Query<(Entity, &MeshComponent)>,
) {
    for (_entity, _mesh) in mesh_query.iter() {
        // 更新网格逻辑
        debug!("更新网格");
    }
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
    fn test_render_component_default() {
        let component = RenderComponent::default();
        assert!(component.visible);
        assert_eq!(component.layer, 0);
    }
    
    #[test]
    fn test_camera_component_default() {
        let camera = CameraComponent::default();
        assert_eq!(camera.fov, 60.0);
        assert_eq!(camera.near, 0.1);
        assert_eq!(camera.far, 1000.0);
        assert!(camera.is_active);
    }
}
