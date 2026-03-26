//! # 事件处理和应用生命周期
//!
//! 基于 winit 0.30 的 ApplicationHandler 实现应用生命周期管理和事件处理。

use std::sync::Arc;
use std::time::Instant;
use winit::{
    application::ApplicationHandler,
    event::{WindowEvent, DeviceEvent, DeviceId},
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
    dpi::PhysicalSize,
};
use log::{info, error, debug};

use anvilkit_ecs::app::App;
use anvilkit_ecs::physics::DeltaTime;
use anvilkit_input::prelude::{InputState, KeyCode, MouseButton};
use crate::window::{WindowConfig, WindowState};
use crate::renderer::{RenderDevice, RenderSurface};
use crate::renderer::assets::RenderAssets;
use crate::renderer::draw::{ActiveCamera, DrawCommandList, SceneLights};
use crate::renderer::state::{RenderState, PbrSceneUniform, GpuLight, MAX_LIGHTS};
use crate::renderer::buffer::{
    create_uniform_buffer, create_depth_texture_msaa,
    create_hdr_render_target, create_hdr_msaa_texture,
    create_sampler, create_texture, create_texture_linear, create_shadow_sampler,
    create_csm_shadow_map,
    Vertex, PbrVertex, SHADOW_MAP_SIZE, HDR_FORMAT, MSAA_SAMPLE_COUNT,
};
use crate::renderer::state::CSM_CASCADE_COUNT;
use crate::renderer::{RenderPipelineBuilder, DEPTH_FORMAT};
use crate::renderer::ibl::generate_brdf_lut;
use crate::renderer::bloom::{BloomResources, BloomSettings};
use anvilkit_core::error::{AnvilKitError, Result};

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

/// Shadow pass shader (depth-only, reads model + view_proj from scene uniform)
const PBR_SHADER: &str = include_str!("../shaders/pbr.wgsl");
const SHADOW_SHADER: &str = include_str!("../shaders/shadow.wgsl");

/// ACES Filmic tone mapping post-process shader (fullscreen triangle)
const TONEMAP_SHADER: &str = include_str!("../shaders/tonemap.wgsl");

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
    /// 渲染表面（延迟初始化，内部持有 Arc<Window>）
    render_surface: Option<RenderSurface>,

    /// 是否请求退出
    exit_requested: bool,

    // --- ECS fields ---
    /// ECS App（当通过 RenderApp::run() 启动时持有）
    app: Option<App>,
    /// GPU 是否已初始化并注入到 ECS World
    gpu_initialized: bool,

    /// 上一帧时间戳，用于计算真实帧时间
    last_frame_time: Instant,

    /// 帧捕获资源（capture feature 启用时）
    #[cfg(feature = "capture")]
    capture_resources: Option<crate::renderer::capture::CaptureResources>,
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
            last_frame_time: Instant::now(),
            #[cfg(feature = "capture")]
            capture_resources: None,
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
        let surface = RenderSurface::new_with_vsync(&device, window, self.config.vsync)?;

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

        // 创建 PBR 场景 Uniform 缓冲区 (992 字节)
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

        // --- Bloom resources ---
        let bloom_settings = BloomSettings::default();
        let bloom = BloomResources::new(device, w, h, bloom_settings.mip_count);

        // Tonemap bind group layout + bind group (3 entries: HDR + sampler + bloom)
        let tonemap_bgl_entries = [
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
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                }, count: None,
            },
        ];

        let tonemap_bind_group_layout = device.device().create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("ECS Tonemap BGL"),
                entries: &tonemap_bgl_entries,
            },
        );

        let bloom_view_for_tonemap = if bloom.mip_views.is_empty() {
            &hdr_texture_view // fallback — shouldn't happen
        } else {
            &bloom.mip_views[0]
        };
        let tonemap_bind_group = device.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ECS Tonemap BG"),
            layout: &tonemap_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&hdr_texture_view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sampler) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(bloom_view_for_tonemap) },
            ],
        });

        // Tonemap pipeline BGL (consumed by builder — duplicate needed because builder takes ownership)
        let tonemap_pipeline_bgl = device.device().create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("ECS Tonemap Pipeline BGL"),
                entries: &tonemap_bgl_entries,
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

        // IBL + Shadow: bind group 2 (BRDF LUT + CSM shadow map array)
        let brdf_lut_data = generate_brdf_lut(256);
        let (_, brdf_lut_view) = create_texture_linear(device, 256, 256, &brdf_lut_data, "ECS BRDF LUT");
        let (_shadow_tex, shadow_map_view, shadow_cascade_views) =
            create_csm_shadow_map(device, SHADOW_MAP_SIZE, CSM_CASCADE_COUNT as u32, "ECS CSM Shadow Map");
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
                            view_dimension: wgpu::TextureViewDimension::D2Array,
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
            shadow_cascade_views,
            hdr_msaa_texture_view,
            bloom: Some(bloom),
            post_process: crate::renderer::post_process::PostProcessResources::new(),
        });
        app.insert_resource(bloom_settings);
        app.insert_resource(crate::renderer::post_process::PostProcessSettings::default());

        // --- 创建默认 PBR 管线 + 默认材质（StandardMaterial 使用） ---
        {
            use crate::renderer::standard_material::DefaultMaterialHandle;

            // 创建 1x1 fallback 纹理
            let white_pixel = [255u8, 255, 255, 255];
            let normal_pixel = [128u8, 128, 255, 255]; // 默认法线 (0,0,1) in tangent space
            let _black_pixel = [0u8, 0, 0, 255];

            let (_, default_base_view) = create_texture(device, 1, 1, &white_pixel, "Default Base Color");
            let (_, default_normal_view) = create_texture_linear(device, 1, 1, &normal_pixel, "Default Normal Map");
            let (_, default_mr_view) = create_texture_linear(device, 1, 1, &white_pixel, "Default MR");
            let (_, default_ao_view) = create_texture_linear(device, 1, 1, &white_pixel, "Default AO");
            let (_, default_emissive_view) = create_texture(device, 1, 1, &white_pixel, "Default Emissive");
            let default_sampler = create_sampler(device, "Default Material Sampler");

            // Material BGL: 5 textures + 1 sampler
            let tex_layout_entry = |binding: u32| -> wgpu::BindGroupLayoutEntry {
                wgpu::BindGroupLayoutEntry {
                    binding, visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    }, count: None,
                }
            };

            let mat_bgl = device.device().create_bind_group_layout(
                &wgpu::BindGroupLayoutDescriptor {
                    label: Some("Default Material BGL"),
                    entries: &[
                        tex_layout_entry(0), // base_color
                        tex_layout_entry(1), // normal_map
                        tex_layout_entry(2), // metallic_roughness
                        tex_layout_entry(3), // ao
                        tex_layout_entry(4), // emissive
                        wgpu::BindGroupLayoutEntry {
                            binding: 5, visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                },
            );

            let default_mat_bg = device.device().create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Default Material BG"),
                layout: &mat_bgl,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&default_base_view) },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&default_normal_view) },
                    wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&default_mr_view) },
                    wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(&default_ao_view) },
                    wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::TextureView(&default_emissive_view) },
                    wgpu::BindGroupEntry { binding: 5, resource: wgpu::BindingResource::Sampler(&default_sampler) },
                ],
            });

            // Scene BGL 和 IBL+Shadow BGL 用于 PBR 管线需要重新创建（builder 取走所有权）
            let pbr_scene_bgl = device.device().create_bind_group_layout(
                &wgpu::BindGroupLayoutDescriptor {
                    label: Some("PBR Scene BGL"),
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
            let pbr_ibl_bgl = device.device().create_bind_group_layout(
                &wgpu::BindGroupLayoutDescriptor {
                    label: Some("PBR IBL+Shadow BGL"),
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
                                view_dimension: wgpu::TextureViewDimension::D2Array,
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

            let pbr_pipeline = RenderPipelineBuilder::new()
                .with_vertex_shader(PBR_SHADER)
                .with_fragment_shader(PBR_SHADER)
                .with_format(HDR_FORMAT)
                .with_vertex_layouts(vec![PbrVertex::layout()])
                .with_depth_format(DEPTH_FORMAT)
                .with_bind_group_layouts(vec![pbr_scene_bgl, mat_bgl, pbr_ibl_bgl])
                .with_label("Default PBR Pipeline")
                .with_multisample_count(MSAA_SAMPLE_COUNT)
                .build(device)
                .expect("创建默认 PBR 管线失败")
                .into_pipeline();

            // 注册到 RenderAssets
            let mat_handle = {
                let mut assets = app.world.get_resource_mut::<RenderAssets>().expect("RenderAssets 必须已注册");
                assets.create_material(pbr_pipeline, default_mat_bg)
            };
            app.world.insert_resource(DefaultMaterialHandle(mat_handle));
            info!("默认 PBR 材质已创建: {:?}", mat_handle);
        }

        self.gpu_initialized = true;
        info!("RenderState (HDR + IBL + Shadow + Bloom + Default PBR) 已注入 ECS World");
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
                let bloom_mip_count: u32 = app.world.get_resource::<BloomSettings>()
                    .map(|s| s.mip_count)
                    .unwrap_or(5u32);
                if let Some(mut rs) = app.world.get_resource_mut::<RenderState>() {
                    rs.surface_size = (new_size.width, new_size.height);
                    let (_, depth_view) = create_depth_texture_msaa(device, new_size.width, new_size.height, "ECS Depth MSAA");
                    rs.depth_texture_view = depth_view;

                    // Recreate HDR RT (resolve), MSAA color, and tonemap bind group
                    let (_, hdr_view) = create_hdr_render_target(device, new_size.width, new_size.height, "ECS HDR RT");
                    let (_, hdr_msaa_view) = create_hdr_msaa_texture(device, new_size.width, new_size.height, "ECS HDR MSAA");
                    let sampler = create_sampler(device, "ECS Sampler");
                    // Resize bloom mip chain
                    if let Some(ref mut bloom) = rs.bloom {
                        bloom.resize(device, new_size.width, new_size.height, bloom_mip_count);
                    }

                    // Recreate tonemap bind group with new HDR + bloom views
                    let bloom_view = rs.bloom.as_ref()
                        .and_then(|b| b.mip_views.first());
                    let bloom_view_ref = bloom_view.unwrap_or(&hdr_view);
                    let new_bg = device.device().create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("ECS Tonemap BG"),
                        layout: &rs.tonemap_bind_group_layout,
                        entries: &[
                            wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&hdr_view) },
                            wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sampler) },
                            wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(bloom_view_ref) },
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

        let Some(app) = &mut self.app else { return };

        // 延迟初始化后处理 GPU 资源（需要 mutable 访问）
        {
            let pp_settings = app.world.get_resource::<crate::renderer::post_process::PostProcessSettings>()
                .cloned()
                .unwrap_or_default();
            if let Some(mut rs) = app.world.get_resource_mut::<RenderState>() {
                let (w, h) = rs.surface_size;
                rs.post_process.ensure_resources(device, w, h, &pp_settings);
            }
        }

        let Some(active_camera) = app.world.get_resource::<ActiveCamera>() else { return };
        let Some(draw_list) = app.world.get_resource::<DrawCommandList>() else { return };
        let Some(render_assets) = app.world.get_resource::<RenderAssets>() else { return };
        let Some(render_state) = app.world.get_resource::<RenderState>() else { return };

        if draw_list.commands.is_empty() {
            return;
        }

        let frame = match surface.get_current_frame_with_recovery(device) {
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

        // Compute CSM cascade matrices for shadow mapping
        let (sw, sh) = render_state.surface_size;
        let cam_aspect = sw as f32 / sh.max(1) as f32;
        let cam_fov = active_camera.fov_radians;
        // Approximate view matrix from camera position and forward direction
        let cam_view_approx = glam::Mat4::look_at_lh(
            camera_pos,
            camera_pos + (active_camera.view_proj.inverse() * glam::Vec4::new(0.0, 0.0, -1.0, 0.0)).truncate().normalize(),
            glam::Vec3::Y,
        );
        let (cascade_matrices, cascade_splits) =
            compute_cascade_matrices(&light.direction, &cam_view_approx, cam_fov, cam_aspect, 0.1, 200.0);

        // === Batched rendering: single encoder, multiple passes, single submit ===
        let mut encoder = device.device().create_command_encoder(
            &wgpu::CommandEncoderDescriptor { label: Some("ECS Frame Encoder") },
        );

        // --- Pass 0: CSM Shadow passes (one per cascade, one sub-pass per object) ---
        // Each object requires its own uniform data, so we use separate render passes
        // with write_buffer between them to ensure correct per-object transforms.
        for cascade_idx in 0..render_state.shadow_cascade_views.len().min(CSM_CASCADE_COUNT) {
            let cascade_view = &render_state.shadow_cascade_views[cascade_idx];
            let cascade_vp = cascade_matrices[cascade_idx];

            for (draw_idx, cmd) in draw_list.commands.iter().enumerate() {
                let Some(gpu_mesh) = render_assets.get_mesh(&cmd.mesh) else { continue };

                let shadow_uniform = PbrSceneUniform {
                    model: cmd.model_matrix.to_cols_array_2d(),
                    view_proj: cascade_vp.to_cols_array_2d(),
                    ..Default::default()
                };
                device.queue().write_buffer(
                    &render_state.scene_uniform_buffer, 0, bytemuck::bytes_of(&shadow_uniform),
                );

                let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("CSM Shadow Pass"),
                    color_attachments: &[],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: cascade_view,
                        depth_ops: Some(wgpu::Operations {
                            load: if draw_idx == 0 { wgpu::LoadOp::Clear(1.0) } else { wgpu::LoadOp::Load },
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
        }

        // --- Pass 1: Scene → HDR render target (one sub-pass per object) ---
        // Each object gets its own render pass so that write_buffer takes effect
        // before the draw call. First pass clears, subsequent passes load.
        for (draw_idx, cmd) in draw_list.commands.iter().enumerate() {
            let Some(gpu_mesh) = render_assets.get_mesh(&cmd.mesh) else { continue };
            let Some(gpu_material) = render_assets.get_material(&cmd.material) else { continue };

            let model = cmd.model_matrix;
            // Normal matrix: inverse transpose of the model matrix.
            // This correctly transforms normals for any scale (uniform or non-uniform).
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
                cascade_view_projs: [
                    cascade_matrices[0].to_cols_array_2d(),
                    cascade_matrices[1].to_cols_array_2d(),
                    cascade_matrices[2].to_cols_array_2d(),
                ],
                cascade_splits: [cascade_splits[0], cascade_splits[1], cascade_splits[2], 1.0 / SHADOW_MAP_SIZE as f32],
                emissive_factor: [cmd.emissive_factor[0], cmd.emissive_factor[1], cmd.emissive_factor[2], CSM_CASCADE_COUNT as f32],
            };
            device.queue().write_buffer(
                &render_state.scene_uniform_buffer, 0, bytemuck::bytes_of(&uniform),
            );

            let Some(pipeline) = render_assets.get_pipeline(&gpu_material.pipeline_handle) else {
                log::error!("材质引用了不存在的管线");
                continue;
            };

            let is_first = draw_idx == 0;
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ECS HDR Scene Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &render_state.hdr_msaa_texture_view,
                    resolve_target: Some(&render_state.hdr_texture_view),
                    ops: wgpu::Operations {
                        load: if is_first {
                            wgpu::LoadOp::Clear(wgpu::Color { r: 0.15, g: 0.3, b: 0.6, a: 1.0 })
                        } else {
                            wgpu::LoadOp::Load
                        },
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &render_state.depth_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        load: if is_first { wgpu::LoadOp::Clear(1.0) } else { wgpu::LoadOp::Load },
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(pipeline);
            render_pass.set_bind_group(0, &render_state.scene_bind_group, &[]);
            render_pass.set_bind_group(1, &gpu_material.bind_group, &[]);
            render_pass.set_bind_group(2, &render_state.ibl_shadow_bind_group, &[]);
            render_pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
            render_pass.set_index_buffer(gpu_mesh.index_buffer.slice(..), gpu_mesh.index_format);
            render_pass.draw_indexed(0..gpu_mesh.index_count, 0, 0..1);
        }

        // --- 后处理管线 (顺序: SSAO → DOF → MotionBlur → Bloom → ColorGrading) ---
        {
            let pp_settings = app.world.get_resource::<crate::renderer::post_process::PostProcessSettings>()
                .cloned()
                .unwrap_or_default();

            // 1. SSAO
            if let (Some(ref ssao_settings), Some(ref ssao_res)) = (&pp_settings.ssao, &render_state.post_process.ssao) {
                let proj = active_camera.view_proj; // 近似投影矩阵
                ssao_res.execute(device, &mut encoder, &render_state.depth_texture_view, &proj, ssao_settings);
            }

            // 2. DOF
            if let (Some(ref dof_settings), Some(ref dof_res)) = (&pp_settings.dof, &render_state.post_process.dof) {
                dof_res.execute(device, &mut encoder, &render_state.hdr_texture_view, &render_state.depth_texture_view, dof_settings);
            }

            // 3. Motion Blur
            if let (Some(ref mb_settings), Some(ref mb_res)) = (&pp_settings.motion_blur, &render_state.post_process.motion_blur) {
                let prev_vp = view_proj.to_cols_array_2d();
                let curr_inv_vp = view_proj.inverse().to_cols_array_2d();
                mb_res.execute(device, &mut encoder, &render_state.hdr_texture_view, &render_state.depth_texture_view, mb_settings, prev_vp, curr_inv_vp);
            }

            // 4. Bloom
            if let Some(ref bloom) = render_state.bloom {
                let bloom_settings = pp_settings.bloom.as_ref()
                    .or_else(|| app.world.get_resource::<BloomSettings>());
                let default_settings = BloomSettings::default();
                let settings = bloom_settings.unwrap_or(&default_settings);
                bloom.execute(device, &mut encoder, &render_state.hdr_texture_view, settings);
            }

            // 5. Color Grading
            if let (Some(ref cg_settings), Some(ref cg_res)) = (&pp_settings.color_grading, &render_state.post_process.color_grading) {
                // Color grading 需要 src 和 dst view；此处直接在 hdr_texture_view 上操作
                // 如果有独立的 intermediate texture 会更好，但当前架构下直接对 HDR 做 in-place 处理
                cg_res.execute(device, &mut encoder, &render_state.hdr_texture_view, &render_state.hdr_texture_view, cg_settings);
            }
        }

        // --- Pass 2: Tone mapping HDR + Bloom → Swapchain ---
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

        // --- Capture: 额外 tonemap pass → capture texture → staging buffer ---
        #[cfg(feature = "capture")]
        let capture_active = {
            use crate::renderer::capture::{CaptureState, CaptureResources};

            let should_capture = app.world.get_resource::<CaptureState>()
                .map(|s| s.should_capture())
                .unwrap_or(false);

            if should_capture {
                let (sw, sh) = render_state.surface_size;
                let fmt = surface.format();

                // 延迟初始化或 resize capture resources
                if self.capture_resources.is_none() {
                    self.capture_resources = Some(CaptureResources::new(device.device(), sw, sh, fmt));
                }
                if let Some(ref mut cr) = self.capture_resources {
                    cr.resize(device.device(), sw, sh);
                }

                if let Some(ref cr) = self.capture_resources {
                    // 额外 tonemap pass 写入 capture_view
                    {
                        let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some("Capture Tonemap Pass"),
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: &cr.capture_view,
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
                        rp.draw(0..3, 0..1);
                    }

                    // copy capture texture → staging buffer
                    cr.encode_copy(&mut encoder);
                }
            }

            should_capture
        };

        // Single submit for all passes
        device.queue().submit(std::iter::once(encoder.finish()));

        // --- Capture: 回读像素并保存 ---
        #[cfg(feature = "capture")]
        if capture_active {
            use crate::renderer::capture::save_png;

            if let Some(ref cr) = self.capture_resources {
                let output_path = app.world.get_resource::<crate::renderer::capture::CaptureState>()
                    .and_then(|s| s.current_output_path());

                match cr.read_pixels(device.device()) {
                    Ok(pixels) => {
                        if let Some(path) = output_path {
                            save_png(&pixels, cr.width, cr.height, &path);
                        }
                    }
                    Err(e) => {
                        log::error!("帧捕获像素回读失败: {}", e);
                    }
                }
            }
        }

        frame.present();

        // 更新 CaptureState（需要 &mut self.app）
        #[cfg(feature = "capture")]
        if capture_active {
            if let Some(ref mut app) = self.app {
                if let Some(mut state) = app.world.get_resource_mut::<crate::renderer::capture::CaptureState>() {
                    state.on_frame_captured();
                }
            }
        }
    }

    /// 执行渲染（ECS 路径）
    fn render(&mut self) {
        if self.app.is_some() && self.gpu_initialized {
            self.render_ecs();
        }
    }

    // --- Public helpers for games with custom ApplicationHandler ---

    /// Forward a window event to [`InputState`] (keyboard, mouse, cursor, scroll).
    ///
    /// Call this from your own [`ApplicationHandler::window_event`] implementation
    /// so the engine handles input state bookkeeping while you handle game-specific events.
    pub fn forward_input(app: &mut App, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput { event, .. } => {
                if let winit::keyboard::PhysicalKey::Code(code) = event.physical_key {
                    if let Some(mut input) = app.world.get_resource_mut::<InputState>() {
                        if let Some(key) = KeyCode::from_winit(code) {
                            if event.state.is_pressed() {
                                input.press_key(key);
                            } else {
                                input.release_key(key);
                            }
                        }
                    }
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if let Some(mut input) = app.world.get_resource_mut::<InputState>() {
                    if let Some(btn) = MouseButton::from_winit(*button) {
                        if state.is_pressed() {
                            input.press_mouse(btn);
                        } else {
                            input.release_mouse(btn);
                        }
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                if let Some(mut input) = app.world.get_resource_mut::<InputState>() {
                    input.set_mouse_position(glam::Vec2::new(position.x as f32, position.y as f32));
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                if let Some(mut input) = app.world.get_resource_mut::<InputState>() {
                    let scroll = match delta {
                        winit::event::MouseScrollDelta::LineDelta(_, y) => *y,
                        winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32 / 120.0,
                    };
                    input.add_scroll_delta(scroll);
                }
            }
            _ => {}
        }
    }

    /// Forward a device event to [`InputState`] (raw mouse motion delta).
    ///
    /// Call this from your own [`ApplicationHandler::device_event`] implementation.
    /// The accumulated delta is cleared automatically by [`InputState::end_frame`].
    pub fn forward_device_input(app: &mut App, event: &DeviceEvent) {
        if let DeviceEvent::MouseMotion { delta } = event {
            if let Some(mut input) = app.world.get_resource_mut::<InputState>() {
                input.add_mouse_delta(glam::Vec2::new(delta.0 as f32, delta.1 as f32));
            }
        }
    }

    /// Run a single frame tick: update DeltaTime, run `app.update()`, clear input state.
    ///
    /// Call this from your own [`ApplicationHandler::about_to_wait`] implementation.
    /// Handles the standard per-frame lifecycle so your game only needs to add
    /// pre-update and post-update logic around it.
    pub fn tick(&mut self, app: &mut App) {
        let now = Instant::now();
        let raw_dt = now.duration_since(self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;
        let dt = raw_dt.clamp(0.001, 0.1);
        app.world.insert_resource(DeltaTime(dt));

        app.update();

        if let Some(mut input) = app.world.get_resource_mut::<InputState>() {
            input.end_frame();
        }

        if let Some(window) = &self.window {
            window.request_redraw();
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

            WindowEvent::KeyboardInput { .. }
            | WindowEvent::MouseInput { .. }
            | WindowEvent::CursorMoved { .. }
            | WindowEvent::MouseWheel { .. } => {
                if let Some(app) = &mut self.app {
                    Self::forward_input(app, &event);
                }
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
        event: DeviceEvent,
    ) {
        if let Some(app) = &mut self.app {
            Self::forward_device_input(app, &event);
        }
    }

    /// 即将等待事件
    #[allow(unused_variables)]
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // 使用 tick() 统一处理：DeltaTime → app.update() → end_frame → request_redraw
        // 注意：需要临时取出 app 以满足借用检查（tick 需要 &mut self 和 &mut App）
        if let Some(mut app) = self.app.take() {
            self.tick(&mut app);

            // 检查 capture auto_exit
            #[cfg(feature = "capture")]
            {
                if let Some(state) = app.world.get_resource::<crate::renderer::capture::CaptureState>() {
                    if state.exit_requested {
                        info!("帧捕获完成，自动退出");
                        event_loop.exit();
                    }
                }
            }

            self.app = Some(app);
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
