use winit::dpi::PhysicalSize;
use log::{error, debug};

use super::render_app::RenderApp;
use super::lighting::{pack_lights, compute_cascade_matrices};
use crate::renderer::draw::{ActiveCamera, DrawCommandList, SceneLights, UniformBatchBuffer};
use crate::renderer::assets::RenderAssets;
use crate::renderer::state::{RenderState, PbrSceneUniform, CSM_CASCADE_COUNT};
use crate::renderer::buffer::SHADOW_MAP_SIZE;
use crate::renderer::bloom::BloomSettings;

impl RenderApp {
    /// 处理窗口大小变化
    pub(super) fn handle_resize(&mut self, new_size: PhysicalSize<u32>) {
        debug!("窗口大小变化: {}x{}", new_size.width, new_size.height);

        self.window_state.set_size(new_size.width, new_size.height);

        if let (Some(device), Some(surface)) = (&self.render_device, &mut self.render_surface) {
            if let Err(e) = surface.resize(device, new_size.width, new_size.height) {
                error!("调整渲染表面大小失败: {}", e);
            }
        }

        // 通过 SceneRenderer 重建所有 size-dependent GPU 资源
        if self.gpu_initialized && new_size.width > 0 && new_size.height > 0 {
            if let (Some(app), Some(device)) = (&mut self.app, &self.render_device) {
                let bloom_mip_count: u32 = app.world().get_resource::<BloomSettings>()
                    .map(|s| s.mip_count)
                    .unwrap_or(5u32);
                if let Some(mut rs) = app.world_mut().get_resource_mut::<RenderState>() {
                    crate::renderer::scene_renderer::SceneRenderer::handle_resize(
                        device, &mut rs, new_size.width, new_size.height, bloom_mip_count,
                    );
                }
            }
        }
    }

    /// 处理缩放因子变化
    pub(super) fn handle_scale_factor_changed(&mut self, scale_factor: f64) {
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

        // 延迟初始化后处理 GPU 资源（通过 SceneRenderer）
        {
            let pp_settings = app.world().get_resource::<crate::renderer::post_process::PostProcessSettings>()
                .cloned()
                .unwrap_or_default();
            if let Some(mut rs) = app.world_mut().get_resource_mut::<RenderState>() {
                crate::renderer::scene_renderer::SceneRenderer::ensure_post_process_resources(
                    device, &mut rs, &pp_settings,
                );
            }
        }

        let Some(active_camera) = app.world().get_resource::<ActiveCamera>() else { return };
        let Some(draw_list) = app.world().get_resource::<DrawCommandList>() else { return };
        let Some(render_assets) = app.world().get_resource::<RenderAssets>() else { return };
        let Some(render_state) = app.world().get_resource::<RenderState>() else { return };

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
        let scene_lights = app.world().get_resource::<SceneLights>()
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

        // --- Batch all uniform data into a single CPU buffer, then upload once ---
        // Alignment: 256 bytes. PbrSceneUniform is 992 bytes -> stride = 1024 bytes.
        let alignment = 256usize;
        let mut batch = UniformBatchBuffer::new(alignment);

        // --- Pass 0: CSM Shadow passes (one render pass per cascade, batched draws) ---
        // Accumulate shadow uniforms for all cascades x objects.
        // shadow_draw_info[cascade_idx] = vec of (offset, cmd_idx) for draws in that cascade.
        let num_cascades = render_state.shadow_cascade_views.len().min(CSM_CASCADE_COUNT);
        let mut shadow_draw_info: Vec<Vec<(u32, usize)>> = vec![Vec::new(); num_cascades];

        for cascade_idx in 0..num_cascades {
            let cascade_vp = cascade_matrices[cascade_idx];

            for (cmd_idx, cmd) in draw_list.commands.iter().enumerate() {
                if render_assets.get_mesh(&cmd.mesh).is_none() { continue; }

                let shadow_uniform = PbrSceneUniform {
                    model: cmd.model_matrix.to_cols_array_2d(),
                    view_proj: cascade_vp.to_cols_array_2d(),
                    ..Default::default()
                };
                let offset = batch.push(bytemuck::bytes_of(&shadow_uniform));
                shadow_draw_info[cascade_idx].push((offset, cmd_idx));
            }
        }

        // Scene pass uniforms -- accumulate after shadow uniforms in the same batch buffer.
        // scene_draw_info = vec of (offset, cmd_idx) for draws that have valid mesh+material.
        let mut scene_draw_info: Vec<(u32, usize)> = Vec::new();

        for (cmd_idx, cmd) in draw_list.commands.iter().enumerate() {
            if render_assets.get_mesh(&cmd.mesh).is_none() { continue; }
            if render_assets.get_material(&cmd.material).is_none() { continue; }

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
            let offset = batch.push(bytemuck::bytes_of(&uniform));
            scene_draw_info.push((offset, cmd_idx));
        }

        // Single write_buffer uploads ALL uniform data for shadow + scene passes
        if !batch.as_bytes().is_empty() {
            device.queue().write_buffer(
                &render_state.scene_uniform_buffer, 0, batch.as_bytes(),
            );
        }

        // --- Shadow render passes: one per cascade, all draws inside ---
        for cascade_idx in 0..num_cascades {
            let cascade_view = &render_state.shadow_cascade_views[cascade_idx];
            let draws = &shadow_draw_info[cascade_idx];
            if draws.is_empty() { continue; }

            let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("CSM Shadow Pass"),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: cascade_view,
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

            for &(offset, cmd_idx) in draws {
                let cmd = &draw_list.commands[cmd_idx];
                let gpu_mesh = render_assets.get_mesh(&cmd.mesh).unwrap();
                rp.set_bind_group(0, &render_state.scene_bind_group, &[offset]);
                rp.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
                rp.set_index_buffer(gpu_mesh.index_buffer.slice(..), gpu_mesh.index_format);
                rp.draw_indexed(0..gpu_mesh.index_count, 0, 0..1);
            }
        }

        // --- Pass 1: Scene -> HDR render target (single render pass, all draws) ---
        if !scene_draw_info.is_empty() {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ECS HDR Scene Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &render_state.hdr_msaa_texture_view,
                    resolve_target: Some(&render_state.hdr_texture_view),
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.15, g: 0.3, b: 0.6, a: 1.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &render_state.depth_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            for &(offset, cmd_idx) in &scene_draw_info {
                let cmd = &draw_list.commands[cmd_idx];
                let gpu_mesh = render_assets.get_mesh(&cmd.mesh).unwrap();
                let gpu_material = render_assets.get_material(&cmd.material).unwrap();

                let pipeline = match render_assets.get_pipeline(&gpu_material.pipeline_handle) {
                    Some(p) => p,
                    None => {
                        log::error!("材质引用了不存在的管线");
                        continue;
                    }
                };

                render_pass.set_pipeline(pipeline);
                render_pass.set_bind_group(0, &render_state.scene_bind_group, &[offset]);
                render_pass.set_bind_group(1, &gpu_material.bind_group, &[]);
                render_pass.set_bind_group(2, &render_state.ibl_shadow_bind_group, &[]);
                render_pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
                render_pass.set_index_buffer(gpu_mesh.index_buffer.slice(..), gpu_mesh.index_format);
                render_pass.draw_indexed(0..gpu_mesh.index_count, 0, 0..1);
            }
        }

        // --- 后处理管线 (顺序: SSAO → DOF → MotionBlur → Bloom → ColorGrading) ---
        {
            let pp_settings = app.world().get_resource::<crate::renderer::post_process::PostProcessSettings>()
                .cloned()
                .unwrap_or_default();

            // 1. SSAO
            #[cfg(feature = "advanced-render")]
            if let (Some(ref ssao_settings), Some(ref ssao_res)) = (&pp_settings.ssao, &render_state.post_process.ssao) {
                let proj = active_camera.view_proj; // 近似投影矩阵
                ssao_res.execute(device, &mut encoder, &render_state.depth_texture_view, &proj, ssao_settings);
            }

            // 2. DOF
            #[cfg(feature = "advanced-render")]
            if let (Some(ref dof_settings), Some(ref dof_res)) = (&pp_settings.dof, &render_state.post_process.dof) {
                dof_res.execute(device, &mut encoder, &render_state.hdr_texture_view, &render_state.depth_texture_view, dof_settings);
            }

            // 3. Motion Blur
            #[cfg(feature = "advanced-render")]
            if let (Some(ref mb_settings), Some(ref mb_res)) = (&pp_settings.motion_blur, &render_state.post_process.motion_blur) {
                // 使用上一帧的 VP 矩阵；首帧回退到当前帧（运动模糊=0）
                let prev_vp = render_state.post_process.prev_view_proj
                    .unwrap_or_else(|| view_proj.to_cols_array_2d());
                let curr_inv_vp = view_proj.inverse().to_cols_array_2d();
                mb_res.execute(device, &mut encoder, &render_state.hdr_texture_view, &render_state.depth_texture_view, mb_settings, prev_vp, curr_inv_vp);
            }

            // 4. Bloom
            if let Some(ref bloom) = render_state.bloom {
                let bloom_settings = pp_settings.bloom.as_ref()
                    .or_else(|| app.world().get_resource::<BloomSettings>());
                let default_settings = BloomSettings::default();
                let settings = bloom_settings.unwrap_or(&default_settings);
                bloom.execute(device, &mut encoder, &render_state.hdr_texture_view, settings);
            }

            // 5. Color Grading（通过中间纹理避免 src == dst 读写冲突）
            #[cfg(feature = "advanced-render")]
            if let (Some(ref cg_settings), Some(ref cg_res)) = (&pp_settings.color_grading, &render_state.post_process.color_grading) {
                cg_res.execute(device, &mut encoder, &render_state.hdr_texture_view, &render_state.hdr_texture, &render_state.hdr_texture_view, cg_settings);
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

            let should_capture = app.world().get_resource::<CaptureState>()
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
                let output_path = app.world().get_resource::<crate::renderer::capture::CaptureState>()
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

        // 存储当前帧 VP 矩阵供下帧 Motion Blur 使用
        if let Some(mut rs) = app.world_mut().get_resource_mut::<RenderState>() {
            rs.post_process.prev_view_proj = Some(view_proj.to_cols_array_2d());
        }

        // 更新 CaptureState（需要 &mut self.app）
        #[cfg(feature = "capture")]
        if capture_active {
            if let Some(ref mut app) = self.app {
                if let Some(mut state) = app.world_mut().get_resource_mut::<crate::renderer::capture::CaptureState>() {
                    state.on_frame_captured();
                }
            }
        }
    }

    /// 执行渲染（ECS 路径）
    pub(super) fn render(&mut self) {
        if self.app.is_some() && self.gpu_initialized {
            self.render_ecs();
        }
    }
}
