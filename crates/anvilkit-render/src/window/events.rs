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
use crate::renderer::state::{RenderState, PbrSceneUniform, GpuLight, MAX_LIGHTS};
use crate::renderer::buffer::{
    create_uniform_buffer, create_depth_texture_msaa,
    create_hdr_render_target, create_hdr_msaa_texture,
    create_sampler, create_texture_linear, create_shadow_map, create_shadow_sampler,
    Vertex, PbrVertex, SHADOW_MAP_SIZE,
};
use crate::renderer::{RenderPipelineBuilder, DEPTH_FORMAT};
use crate::renderer::ibl::generate_brdf_lut;
use anvilkit_core::error::{AnvilKitError, Result};

/// 将 SceneLights 打包为 GPU 光源数组
///
/// 返回 (lights_array, light_count)。方向光占 slot 0，其余填充点光和聚光。
fn pack_lights(scene_lights: &SceneLights) -> ([GpuLight; MAX_LIGHTS], u32) {
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

/// 计算方向光的光空间矩阵（正交投影）
///
/// 生成一个从光源方向看向原点的 view-projection 矩阵，
/// 用于 shadow pass 的深度渲染。
fn compute_light_space_matrix(light_direction: &glam::Vec3) -> glam::Mat4 {
    let light_dir = light_direction.normalize();
    // 光源位置设在场景中心的反方向
    let light_pos = -light_dir * 15.0;
    let light_view = glam::Mat4::look_at_lh(light_pos, glam::Vec3::ZERO, glam::Vec3::Y);
    // 正交投影覆盖场景范围
    let light_proj = glam::Mat4::orthographic_lh(-10.0, 10.0, -10.0, 10.0, 0.1, 30.0);
    light_proj * light_view
}

/// Shadow pass shader (depth-only, reads model + view_proj from scene uniform)
const SHADOW_SHADER: &str = include_str!("../../../../shaders/shadow.wgsl");

/// ACES Filmic tone mapping post-process shader (fullscreen triangle)
const TONEMAP_SHADER: &str = include_str!("../../../../shaders/tonemap.wgsl");

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

        let (_, depth_texture_view) = create_depth_texture_msaa(device, w, h, "ECS Depth MSAA");

        // HDR render target (resolve target, sample_count=1) + MSAA color attachment
        let (_, hdr_texture_view) = create_hdr_render_target(device, w, h, "ECS HDR RT");
        let (_, hdr_msaa_texture_view) = create_hdr_msaa_texture(device, w, h, "ECS HDR MSAA");
        let sampler = create_sampler(device, "ECS Tonemap Sampler");

        // Tonemap bind group layout + bind group
        let tonemap_bind_group_layout = device.device().create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("ECS Tonemap BGL"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0, visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        }, count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1, visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            },
        );

        let tonemap_bind_group = device.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ECS Tonemap BG"),
            layout: &tonemap_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&hdr_texture_view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sampler) },
            ],
        });

        // Tonemap pipeline needs its own bind group layout (consumed by builder)
        let tonemap_pipeline_bgl = device.device().create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("ECS Tonemap Pipeline BGL"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0, visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        }, count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1, visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            },
        );

        let tonemap_pipeline = RenderPipelineBuilder::new()
            .with_vertex_shader(TONEMAP_SHADER)
            .with_fragment_shader(TONEMAP_SHADER)
            .with_format(format)
            .with_vertex_layouts(vec![])
            .with_bind_group_layouts(vec![tonemap_pipeline_bgl])
            .with_label("ECS Tonemap Pipeline")
            .build(device)
            .expect("创建 Tonemap 管线失败")
            .into_pipeline();

        // IBL + Shadow: bind group 2 (BRDF LUT + shadow map)
        let brdf_lut_data = generate_brdf_lut(256);
        let (_, brdf_lut_view) = create_texture_linear(device, 256, 256, &brdf_lut_data, "ECS BRDF LUT");
        let (_, shadow_map_view) = create_shadow_map(device, SHADOW_MAP_SIZE, "ECS Shadow Map");
        let shadow_sampler = create_shadow_sampler(device, "ECS Shadow Sampler");

        let ibl_shadow_bind_group_layout = device.device().create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("ECS IBL+Shadow BGL"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0, visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        }, count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1, visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2, visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Depth,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        }, count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3, visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                        count: None,
                    },
                ],
            },
        );

        let ibl_shadow_bind_group = device.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ECS IBL+Shadow BG"),
            layout: &ibl_shadow_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&brdf_lut_view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sampler) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&shadow_map_view) },
                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::Sampler(&shadow_sampler) },
            ],
        });

        // Shadow pass pipeline (depth-only, uses PbrVertex layout for position)
        let shadow_scene_bgl = device.device().create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("Shadow Scene BGL"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            },
        );

        let shadow_pipeline = RenderPipelineBuilder::new()
            .with_vertex_shader(SHADOW_SHADER)
            .with_format(wgpu::TextureFormat::Rgba8Unorm) // dummy, no color output
            .with_vertex_layouts(vec![PbrVertex::layout()])
            .with_depth_format(DEPTH_FORMAT)
            .with_bind_group_layouts(vec![shadow_scene_bgl])
            .with_label("ECS Shadow Pipeline")
            .build_depth_only(device)
            .expect("创建 Shadow 管线失败")
            .into_pipeline();

        app.insert_resource(RenderState {
            surface_format: format,
            surface_size: (w, h),
            scene_uniform_buffer,
            scene_bind_group,
            scene_bind_group_layout,
            depth_texture_view,
            hdr_texture_view,
            tonemap_pipeline,
            tonemap_bind_group,
            tonemap_bind_group_layout,
            ibl_shadow_bind_group,
            ibl_shadow_bind_group_layout,
            shadow_pipeline,
            shadow_map_view,
            hdr_msaa_texture_view,
        });

        self.gpu_initialized = true;
        info!("RenderState (HDR + IBL + Shadow) 已注入 ECS World");
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

        // 更新 ECS RenderState 中的深度纹理、HDR RT 和 surface_size
        if self.gpu_initialized && new_size.width > 0 && new_size.height > 0 {
            if let (Some(app), Some(device)) = (&mut self.app, &self.render_device) {
                if let Some(mut rs) = app.world.get_resource_mut::<RenderState>() {
                    rs.surface_size = (new_size.width, new_size.height);
                    let (_, depth_view) = create_depth_texture_msaa(device, new_size.width, new_size.height, "ECS Depth MSAA");
                    rs.depth_texture_view = depth_view;

                    // Recreate HDR RT (resolve), MSAA color, and tonemap bind group
                    let (_, hdr_view) = create_hdr_render_target(device, new_size.width, new_size.height, "ECS HDR RT");
                    let (_, hdr_msaa_view) = create_hdr_msaa_texture(device, new_size.width, new_size.height, "ECS HDR MSAA");
                    let sampler = create_sampler(device, "ECS Sampler");
                    let new_bg = device.device().create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("ECS Tonemap BG"),
                        layout: &rs.tonemap_bind_group_layout,
                        entries: &[
                            wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&hdr_view) },
                            wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sampler) },
                        ],
                    });
                    rs.hdr_texture_view = hdr_view;
                    rs.hdr_msaa_texture_view = hdr_msaa_view;
                    rs.tonemap_bind_group = new_bg;
                }
            }
        }
    }

    /// 处理缩放因子变化
    fn handle_scale_factor_changed(&mut self, scale_factor: f64) {
        debug!("缩放因子变化: {}", scale_factor);
        self.window_state.set_scale_factor(scale_factor);
    }

    /// 执行 ECS 多物体 HDR PBR 渲染
    ///
    /// Pass 1: 场景渲染到 HDR RT (Rgba16Float)
    /// Pass 2: Tone mapping HDR → Swapchain (ACES Filmic)
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

        let swapchain_view = frame.texture.create_view(&Default::default());
        let view_proj = active_camera.view_proj;
        let camera_pos = active_camera.camera_pos;

        // 获取场景灯光并打包为 GPU 数组
        let default_lights = SceneLights::default();
        let scene_lights = app.world.get_resource::<SceneLights>()
            .unwrap_or(&default_lights);
        let (gpu_lights, light_count) = pack_lights(scene_lights);
        let light = &scene_lights.directional;

        // Compute light-space matrix for shadow mapping (directional light)
        let shadow_view_proj = compute_light_space_matrix(&light.direction);

        // === Pass 0: Shadow pass → shadow depth map ===
        for cmd in draw_list.commands.iter() {
            let Some(gpu_mesh) = render_assets.get_mesh(&cmd.mesh) else { continue };

            let model = cmd.model_matrix;
            let shadow_uniform = PbrSceneUniform {
                model: model.to_cols_array_2d(),
                view_proj: shadow_view_proj.to_cols_array_2d(),
                ..Default::default()
            };
            device.queue().write_buffer(
                &render_state.scene_uniform_buffer, 0, bytemuck::bytes_of(&shadow_uniform),
            );

            let mut encoder = device.device().create_command_encoder(
                &wgpu::CommandEncoderDescriptor { label: Some("Shadow Encoder") },
            );
            {
                let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Shadow Pass"),
                    color_attachments: &[],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &render_state.shadow_map_view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
                rp.set_pipeline(&render_state.shadow_pipeline);
                rp.set_bind_group(0, &render_state.scene_bind_group, &[]);
                rp.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
                rp.set_index_buffer(gpu_mesh.index_buffer.slice(..), gpu_mesh.index_format);
                rp.draw_indexed(0..gpu_mesh.index_count, 0, 0..1);
            }
            device.queue().submit(std::iter::once(encoder.finish()));
        }

        // === Pass 1: Scene → HDR render target ===
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
                material_params: [cmd.metallic, cmd.roughness, cmd.normal_scale, light_count as f32],
                lights: gpu_lights,
                shadow_view_proj: shadow_view_proj.to_cols_array_2d(),
                emissive_factor: [cmd.emissive_factor[0], cmd.emissive_factor[1], cmd.emissive_factor[2], 0.0],
            };
            device.queue().write_buffer(
                &render_state.scene_uniform_buffer,
                0,
                bytemuck::bytes_of(&uniform),
            );

            let mut encoder = device.device().create_command_encoder(
                &wgpu::CommandEncoderDescriptor { label: Some("ECS HDR Scene Encoder") },
            );

            {
                let color_load = if i == 0 {
                    wgpu::LoadOp::Clear(wgpu::Color { r: 0.15, g: 0.3, b: 0.6, a: 1.0 })
                } else {
                    wgpu::LoadOp::Load
                };
                let depth_load = if i == 0 {
                    wgpu::LoadOp::Clear(1.0)
                } else {
                    wgpu::LoadOp::Load
                };

                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("ECS HDR Scene Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &render_state.hdr_msaa_texture_view,
                        resolve_target: Some(&render_state.hdr_texture_view),
                        ops: wgpu::Operations { load: color_load, store: wgpu::StoreOp::Discard },
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
                render_pass.set_bind_group(2, &render_state.ibl_shadow_bind_group, &[]);
                render_pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
                render_pass.set_index_buffer(gpu_mesh.index_buffer.slice(..), gpu_mesh.index_format);
                render_pass.draw_indexed(0..gpu_mesh.index_count, 0, 0..1);
            }

            device.queue().submit(std::iter::once(encoder.finish()));
        }

        // === Pass 2: Tone mapping HDR → Swapchain ===
        {
            let mut encoder = device.device().create_command_encoder(
                &wgpu::CommandEncoderDescriptor { label: Some("ECS Tonemap Encoder") },
            );

            {
                let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("ECS Tonemap Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &swapchain_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

                rp.set_pipeline(&render_state.tonemap_pipeline);
                rp.set_bind_group(0, &render_state.tonemap_bind_group, &[]);
                rp.draw(0..3, 0..1); // Fullscreen triangle
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
