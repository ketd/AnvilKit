//! # 事件处理和应用生命周期
//!
//! 基于 winit 0.30 的 ApplicationHandler 实现应用生命周期管理和事件处理。

use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::{WindowEvent, DeviceEvent, DeviceId},
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
    dpi::PhysicalSize,
};
use log::{info, error, debug};

use anvilkit_ecs::app::App;
use crate::window::{WindowConfig, WindowState};
use crate::renderer::{RenderDevice, RenderSurface};
use crate::renderer::assets::RenderAssets;
use crate::renderer::draw::{ActiveCamera, DrawCommandList, SceneLights};
use crate::renderer::state::{RenderState, PbrSceneUniform};
use crate::renderer::buffer::{create_uniform_buffer, create_depth_texture};
use anvilkit_core::error::{AnvilKitError, Result};

/// 渲染应用
///
/// 实现 ApplicationHandler trait，管理窗口生命周期和渲染循环。
/// 使用 winit 0.30 API 设计，提供跨平台兼容性。
///
/// # 设计理念
///
/// - **延迟初始化**: 在 `resumed` 事件中创建窗口和渲染上下文
/// - **事件驱动**: 响应窗口事件和设备事件
/// - **ECS 集成**: 持有 App 并每帧调用 update()
///
/// # 示例
///
/// ```rust,no_run
/// use anvilkit_render::window::{RenderApp, WindowConfig};
/// use anvilkit_render::prelude::*;
/// use winit::event_loop::EventLoop;
///
/// let mut app = App::new();
/// app.add_plugins(RenderPlugin::default());
///
/// RenderApp::run(app);
/// ```
pub struct RenderApp {
    /// 窗口配置
    config: WindowConfig,
    /// 窗口实例（延迟初始化）
    window: Option<Arc<Window>>,
    /// 窗口状态
    window_state: WindowState,
    /// 渲染设备（延迟初始化）
    render_device: Option<RenderDevice>,
    /// 渲染表面（延迟初始化，持有对 window 的引用）
    render_surface: Option<RenderSurface<'static>>,

    /// 清除颜色
    clear_color: wgpu::Color,
    /// 是否请求退出
    exit_requested: bool,

    // --- ECS fields ---
    /// ECS App（当通过 RenderApp::run() 启动时持有）
    app: Option<App>,
    /// GPU 是否已初始化并注入到 ECS World
    gpu_initialized: bool,
}

impl RenderApp {
    /// 创建新的渲染应用
    ///
    /// # 参数
    ///
    /// - `config`: 窗口配置参数
    ///
    /// # 示例
    ///
    /// ```rust
    /// use anvilkit_render::window::{RenderApp, WindowConfig};
    ///
    /// let config = WindowConfig::new().with_title("我的应用");
    /// let app = RenderApp::new(config);
    /// ```
    pub fn new(config: WindowConfig) -> Self {
        info!("创建渲染应用: {}", config.title);

        Self {
            config,
            window: None,
            window_state: WindowState::new(),
            render_device: None,
            render_surface: None,
            clear_color: wgpu::Color {
                r: 0.1,
                g: 0.2,
                b: 0.3,
                a: 1.0,
            },
            exit_requested: false,
            app: None,
            gpu_initialized: false,
        }
    }

    /// ECS 驱动的入口点
    ///
    /// 创建 EventLoop、窗口，运行 winit 主循环。
    /// 每帧调用 `app.update()` 然后执行 GPU 渲染。
    ///
    /// # 参数
    ///
    /// - `app`: 已配置好 RenderPlugin 和系统的 ECS App
    pub fn run(app: App) {
        let event_loop = winit::event_loop::EventLoop::new().unwrap();

        // 从 App 中读取 RenderConfig 获取 WindowConfig
        let window_config = app.world.get_resource::<crate::plugin::RenderConfig>()
            .map(|c| c.window_config.clone())
            .unwrap_or_default();

        let mut render_app = Self::new(window_config);
        render_app.app = Some(app);

        event_loop.run_app(&mut render_app).unwrap();
    }

    /// 获取窗口配置
    pub fn config(&self) -> &WindowConfig {
        &self.config
    }

    /// 获取窗口状态
    pub fn window_state(&self) -> &WindowState {
        &self.window_state
    }

    /// 获取窗口实例
    pub fn window(&self) -> Option<&Arc<Window>> {
        self.window.as_ref()
    }

    /// 请求退出应用
    pub fn request_exit(&mut self) {
        info!("请求退出应用");
        self.exit_requested = true;
    }

    /// 检查是否请求退出
    pub fn is_exit_requested(&self) -> bool {
        self.exit_requested
    }

    /// 获取渲染设备（初始化后可用）
    pub fn render_device(&self) -> Option<&RenderDevice> {
        self.render_device.as_ref()
    }

    /// 获取渲染表面格式（初始化后可用）
    pub fn surface_format(&self) -> Option<wgpu::TextureFormat> {
        self.render_surface.as_ref().map(|s| s.format())
    }

    /// 获取当前帧的 SurfaceTexture（用于外部渲染）
    pub fn get_current_frame(&self) -> Option<wgpu::SurfaceTexture> {
        self.render_surface.as_ref().and_then(|s| s.get_current_frame().ok())
    }

    // --- Internal methods ---

    /// 创建窗口
    fn create_window(&mut self, event_loop: &ActiveEventLoop) -> Result<()> {
        if self.window.is_some() {
            return Ok(());
        }

        info!("创建窗口: {} ({}x{})",
              self.config.title, self.config.width, self.config.height);

        let attributes = self.config.to_window_attributes();
        let window = event_loop.create_window(attributes)
            .map_err(|e| AnvilKitError::render(format!("创建窗口失败: {}", e)))?;

        let size = window.inner_size();
        self.window_state.set_size(size.width, size.height);
        self.window_state.set_scale_factor(window.scale_factor());

        self.window = Some(Arc::new(window));

        info!("窗口创建成功");
        Ok(())
    }

    /// 初始化渲染资源
    async fn init_render(&mut self) -> Result<()> {
        if self.render_device.is_some() {
            return Ok(());
        }

        let window = self.window.as_ref()
            .ok_or_else(|| AnvilKitError::render("窗口未创建".to_string()))?;

        info!("初始化渲染设备和表面");

        let device = RenderDevice::new(window).await?;

        // SAFETY: Arc<Window> 保证了 window 的生命周期至少与 self 相同。
        let window_ref: &Arc<Window> = unsafe {
            &*(window as *const Arc<Window>)
        };
        let surface = RenderSurface::new(&device, window_ref)?;

        self.render_device = Some(device);
        self.render_surface = Some(surface);

        info!("渲染设备和表面初始化成功");
        Ok(())
    }

    /// GPU 初始化后，将共享资源注入 ECS World
    fn inject_render_state_to_ecs(&mut self) {
        if self.gpu_initialized {
            return;
        }

        let Some(app) = &mut self.app else { return };
        let Some(device) = &self.render_device else { return };
        let Some(surface) = &self.render_surface else { return };

        let format = surface.format();
        let (w, h) = self.window_state.size();

        // 创建 PBR 场景 Uniform 缓冲区 (256 字节)
        let initial_uniform = PbrSceneUniform::default();
        let scene_uniform_buffer = create_uniform_buffer(
            device,
            "ECS Scene Uniform",
            bytemuck::bytes_of(&initial_uniform),
        );

        let scene_bind_group_layout = device.device().create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("ECS Scene BGL"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            },
        );

        let scene_bind_group = device.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ECS Scene BG"),
            layout: &scene_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: scene_uniform_buffer.as_entire_binding(),
            }],
        });

        let (_, depth_texture_view) = create_depth_texture(device, w, h, "ECS Depth");

        app.insert_resource(RenderState {
            surface_format: format,
            surface_size: (w, h),
            scene_uniform_buffer,
            scene_bind_group,
            scene_bind_group_layout,
            depth_texture_view,
        });

        self.gpu_initialized = true;
        info!("RenderState 已注入 ECS World");
    }

    /// 处理窗口大小变化
    fn handle_resize(&mut self, new_size: PhysicalSize<u32>) {
        debug!("窗口大小变化: {}x{}", new_size.width, new_size.height);

        self.window_state.set_size(new_size.width, new_size.height);

        if let (Some(device), Some(surface)) = (&self.render_device, &mut self.render_surface) {
            if let Err(e) = surface.resize(device, new_size.width, new_size.height) {
                error!("调整渲染表面大小失败: {}", e);
            }
        }

        // 更新 ECS RenderState 中的深度纹理和 surface_size
        if self.gpu_initialized && new_size.width > 0 && new_size.height > 0 {
            if let (Some(app), Some(device)) = (&mut self.app, &self.render_device) {
                if let Some(mut rs) = app.world.get_resource_mut::<RenderState>() {
                    rs.surface_size = (new_size.width, new_size.height);
                    let (_, view) = create_depth_texture(
                        device,
                        new_size.width,
                        new_size.height,
                        "ECS Depth",
                    );
                    rs.depth_texture_view = view;
                }
            }
        }
    }

    /// 处理缩放因子变化
    fn handle_scale_factor_changed(&mut self, scale_factor: f64) {
        debug!("缩放因子变化: {}", scale_factor);
        self.window_state.set_scale_factor(scale_factor);
    }

    /// 执行 ECS 多物体 PBR 渲染
    ///
    /// 每个物体独立 write_buffer + encoder + render_pass + submit，
    /// 确保 per-object uniform 正确生效。
    fn render_ecs(&mut self) {
        let (Some(device), Some(surface)) = (&self.render_device, &self.render_surface) else {
            return;
        };

        let Some(app) = &self.app else { return };

        let Some(active_camera) = app.world.get_resource::<ActiveCamera>() else { return };
        let Some(draw_list) = app.world.get_resource::<DrawCommandList>() else { return };
        let Some(render_assets) = app.world.get_resource::<RenderAssets>() else { return };
        let Some(render_state) = app.world.get_resource::<RenderState>() else { return };

        if draw_list.commands.is_empty() {
            return;
        }

        let frame = match surface.get_current_frame() {
            Ok(frame) => frame,
            Err(e) => {
                error!("获取当前帧失败: {}", e);
                return;
            }
        };

        let view = frame.texture.create_view(&Default::default());
        let view_proj = active_camera.view_proj;
        let camera_pos = active_camera.camera_pos;

        // 获取场景灯光（如果有）
        let default_lights = SceneLights::default();
        let scene_lights = app.world.get_resource::<SceneLights>()
            .unwrap_or(&default_lights);
        let light = &scene_lights.directional;

        for (i, cmd) in draw_list.commands.iter().enumerate() {
            let Some(gpu_mesh) = render_assets.get_mesh(&cmd.mesh) else { continue };
            let Some(gpu_material) = render_assets.get_material(&cmd.material) else { continue };

            let model = cmd.model_matrix;
            let normal_matrix = model.inverse().transpose();

            let uniform = PbrSceneUniform {
                model: model.to_cols_array_2d(),
                view_proj: view_proj.to_cols_array_2d(),
                normal_matrix: normal_matrix.to_cols_array_2d(),
                camera_pos: [camera_pos.x, camera_pos.y, camera_pos.z, 0.0],
                light_dir: [light.direction.x, light.direction.y, light.direction.z, 0.0],
                light_color: [light.color.x, light.color.y, light.color.z, light.intensity],
                material_params: [cmd.metallic, cmd.roughness, cmd.normal_scale, 0.0],
            };
            device.queue().write_buffer(
                &render_state.scene_uniform_buffer,
                0,
                bytemuck::bytes_of(&uniform),
            );

            let mut encoder = device.device().create_command_encoder(
                &wgpu::CommandEncoderDescriptor { label: Some("ECS Render Encoder") },
            );

            {
                let color_load = if i == 0 {
                    wgpu::LoadOp::Clear(self.clear_color)
                } else {
                    wgpu::LoadOp::Load
                };
                let depth_load = if i == 0 {
                    wgpu::LoadOp::Clear(1.0)
                } else {
                    wgpu::LoadOp::Load
                };

                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("ECS Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations { load: color_load, store: wgpu::StoreOp::Store },
                    })],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &render_state.depth_texture_view,
                        depth_ops: Some(wgpu::Operations { load: depth_load, store: wgpu::StoreOp::Store }),
                        stencil_ops: None,
                    }),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

                render_pass.set_pipeline(&gpu_material.pipeline);
                render_pass.set_bind_group(0, &render_state.scene_bind_group, &[]);
                render_pass.set_bind_group(1, &gpu_material.bind_group, &[]);
                render_pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
                render_pass.set_index_buffer(gpu_mesh.index_buffer.slice(..), gpu_mesh.index_format);
                render_pass.draw_indexed(0..gpu_mesh.index_count, 0, 0..1);
            }

            device.queue().submit(std::iter::once(encoder.finish()));
        }

        frame.present();
    }

    /// 执行渲染（ECS 路径）
    fn render(&mut self) {
        if self.app.is_some() && self.gpu_initialized {
            self.render_ecs();
        }
    }
}

impl ApplicationHandler for RenderApp {
    /// 应用恢复事件
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        info!("应用恢复");

        if let Err(e) = self.create_window(event_loop) {
            error!("创建窗口失败: {}", e);
            event_loop.exit();
            return;
        }

        if let Err(e) = pollster::block_on(self.init_render()) {
            error!("初始化渲染失败: {}", e);
            event_loop.exit();
            return;
        }

        // 如果持有 ECS App，注入 RenderState
        self.inject_render_state_to_ecs();

        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    /// 窗口事件处理
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                info!("收到窗口关闭请求");
                self.request_exit();
                event_loop.exit();
            }

            WindowEvent::Resized(new_size) => {
                self.handle_resize(new_size);
            }

            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.handle_scale_factor_changed(scale_factor);
            }

            WindowEvent::Focused(focused) => {
                debug!("窗口焦点变化: {}", focused);
                self.window_state.set_focused(focused);
            }

            WindowEvent::Occluded(occluded) => {
                debug!("窗口遮挡状态: {}", occluded);
                self.window_state.set_minimized(occluded);
            }

            WindowEvent::RedrawRequested => {
                self.render();
            }

            _ => {}
        }
    }

    /// 设备事件处理
    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        _event: DeviceEvent,
    ) {
        // 后续可以添加输入处理
    }

    /// 即将等待事件
    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // 如果持有 ECS App，每帧调用 update() 运行 ECS 系统
        if let Some(app) = &mut self.app {
            app.update();
        }

        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_app_creation() {
        let config = WindowConfig::new().with_title("Test App");
        let app = RenderApp::new(config);

        assert_eq!(app.config().title, "Test App");
        assert!(app.window().is_none());
        assert!(!app.is_exit_requested());
    }

    #[test]
    fn test_exit_request() {
        let mut app = RenderApp::new(WindowConfig::default());

        assert!(!app.is_exit_requested());
        app.request_exit();
        assert!(app.is_exit_requested());
    }

    #[test]
    fn test_window_state_updates() {
        let mut app = RenderApp::new(WindowConfig::default());

        let new_size = PhysicalSize::new(1920, 1080);
        app.handle_resize(new_size);
        assert_eq!(app.window_state().size(), (1920, 1080));

        app.handle_scale_factor_changed(2.0);
        assert_eq!(app.window_state().scale_factor(), 2.0);
    }

    #[test]
    fn test_render_app_config() {
        let config = WindowConfig::new()
            .with_title("Test")
            .with_size(640, 480);
        let app = RenderApp::new(config);

        assert_eq!(app.config().title, "Test");
        assert_eq!(app.config().width, 640);
    }

    #[test]
    fn test_render_app_exit_request_toggle() {
        let mut app = RenderApp::new(WindowConfig::new());
        assert!(!app.is_exit_requested());

        app.request_exit();
        assert!(app.is_exit_requested());
    }

    #[test]
    fn test_render_app_window_state() {
        let config = WindowConfig::new().with_size(1024, 768);
        let app = RenderApp::new(config);

        // WindowState defaults to (1280, 720) since it's created via WindowState::new()
        let state = app.window_state();
        assert_eq!(state.size(), (1280, 720));
    }
}
