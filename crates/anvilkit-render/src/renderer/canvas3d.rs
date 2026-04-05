//! # Canvas3D — Agent-Friendly 3D Rendering
//!
//! Zero-wgpu-knowledge 3D drawing API. Renders colored 3D primitives with basic
//! directional lighting. For full PBR, use the ECS pipeline with entity spawning.
//!
//! ## Usage
//!
//! ```rust,ignore
//! impl GameCallbacks for MyGame {
//!     fn render(&mut self, ctx: &mut GameContext) {
//!         let Some(mut c) = Canvas3D::begin(ctx.render_app, &mut self.renderer3d) else { return };
//!         c.set_camera(Vec3::new(0.0, 5.0, 10.0), Vec3::ZERO, 60.0);
//!         c.set_sun(Vec3::new(-1.0, -1.0, -1.0), [1.0, 1.0, 0.9]);
//!         c.clear([0.1, 0.1, 0.2, 1.0]);
//!         c.draw_cube(Vec3::ZERO, Vec3::ONE, [0.8, 0.2, 0.2, 1.0]);
//!         c.draw_sphere(Vec3::new(3.0, 1.0, 0.0), 1.0, [0.2, 0.5, 0.9, 1.0]);
//!         c.draw_ground(20.0, [0.3, 0.3, 0.3, 1.0]);
//!         c.finish();
//!     }
//! }
//! ```

use crate::renderer::RenderDevice;
use crate::renderer::text::TextRenderer;

const MAX_3D_VERTICES: usize = 262144; // 256K vertices

const CANVAS3D_SHADER: &str = r#"
struct Uniforms {
    view_proj: mat4x4<f32>,
    light_dir: vec4<f32>,
    light_color: vec4<f32>,
    ambient: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> u: Uniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) color: vec4<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_pos = u.view_proj * vec4<f32>(in.position, 1.0);
    out.world_normal = in.normal;
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let n = normalize(in.world_normal);
    let l = normalize(-u.light_dir.xyz);
    let ndotl = max(dot(n, l), 0.0);
    let diffuse = u.light_color.rgb * ndotl;
    let lit = in.color.rgb * (u.ambient.rgb + diffuse);
    return vec4<f32>(lit, in.color.a);
}
"#;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex3D {
    position: [f32; 3],
    normal: [f32; 3],
    color: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Canvas3DUniform {
    view_proj: [[f32; 4]; 4],
    light_dir: [f32; 4],
    light_color: [f32; 4],
    ambient: [f32; 4],
}

impl Vertex3D {
    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x3, offset: 0, shader_location: 0 },
                wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x3, offset: 12, shader_location: 1 },
                wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x4, offset: 24, shader_location: 2 },
            ],
        }
    }
}

/// GPU resources for Canvas3D. Created once in init().
pub struct Canvas3DRenderer {
    vertex_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    depth_texture: wgpu::TextureView,
    pipeline: wgpu::RenderPipeline,
    bgl: wgpu::BindGroupLayout,
    text_renderer: TextRenderer,
    surface_format: wgpu::TextureFormat,
    width: u32,
    height: u32,
    #[cfg(feature = "capture")]
    capture_resources: Option<crate::renderer::capture::CaptureResources>,
}

impl Canvas3DRenderer {
    pub fn new(device: &RenderDevice, format: wgpu::TextureFormat, width: u32, height: u32) -> Self {
        let vertex_buffer = device.device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("Canvas3D VB"),
            size: (MAX_3D_VERTICES * std::mem::size_of::<Vertex3D>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_buffer = device.device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("Canvas3D UB"),
            size: std::mem::size_of::<Canvas3DUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let depth_texture = device.device().create_texture(&wgpu::TextureDescriptor {
            label: Some("Canvas3D Depth"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1, sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        }).create_view(&Default::default());

        let bgl = device.device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Canvas3D BGL"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let shader = device.device().create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Canvas3D Shader"),
            source: wgpu::ShaderSource::Wgsl(CANVAS3D_SHADER.into()),
        });

        let pl = device.device().create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Canvas3D PL"),
            bind_group_layouts: &[&bgl],
            push_constant_ranges: &[],
        });

        let pipeline = device.device().create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Canvas3D Pipeline"),
            layout: Some(&pl),
            vertex: wgpu::VertexState {
                module: &shader, entry_point: "vs_main",
                buffers: &[Vertex3D::layout()],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader, entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format, blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });

        Self {
            vertex_buffer, uniform_buffer, depth_texture,
            pipeline, bgl, surface_format: format,
            text_renderer: TextRenderer::new(device, format),
            width, height,
            #[cfg(feature = "capture")]
            capture_resources: None,
        }
    }

    /// Resize depth buffer when window changes.
    pub fn resize(&mut self, device: &RenderDevice, width: u32, height: u32) {
        if width == self.width && height == self.height { return; }
        self.width = width;
        self.height = height;
        self.depth_texture = device.device().create_texture(&wgpu::TextureDescriptor {
            label: Some("Canvas3D Depth"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1, sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        }).create_view(&Default::default());
    }
}

/// Per-frame 3D drawing context.
pub struct Canvas3D<'a> {
    device: &'a RenderDevice,
    renderer: &'a mut Canvas3DRenderer,
    swapchain_view: wgpu::TextureView,
    frame: wgpu::SurfaceTexture,
    vertices: Vec<Vertex3D>,
    clear_color: [f32; 4],
    camera_pos: glam::Vec3,
    camera_target: glam::Vec3,
    camera_fov: f32,
    light_dir: glam::Vec3,
    light_color: [f32; 3],
    ambient: [f32; 3],
    width: f32,
    height: f32,
}

impl<'a> Canvas3D<'a> {
    pub fn begin(
        render_app: &'a mut crate::window::events::RenderApp,
        renderer: &'a mut Canvas3DRenderer,
    ) -> Option<Self> {
        let device = render_app.render_device()?;
        let (w, h) = render_app.window_state().size();
        renderer.resize(device, w, h);
        let frame = render_app.get_current_frame()?;
        let swapchain_view = frame.texture.create_view(&Default::default());

        Some(Canvas3D {
            device, renderer, swapchain_view, frame,
            vertices: Vec::new(),
            clear_color: [0.1, 0.1, 0.15, 1.0],
            camera_pos: glam::Vec3::new(0.0, 5.0, 10.0),
            camera_target: glam::Vec3::ZERO,
            camera_fov: 60.0,
            light_dir: glam::Vec3::new(-1.0, -1.0, -0.5).normalize(),
            light_color: [1.0, 0.95, 0.85],
            ambient: [0.15, 0.15, 0.2],
            width: w as f32, height: h as f32,
        })
    }

    pub fn width(&self) -> f32 { self.width }
    pub fn height(&self) -> f32 { self.height }
    pub fn clear(&mut self, color: [f32; 4]) { self.clear_color = color; }

    /// Set camera position, look-at target, and vertical FOV in degrees.
    pub fn set_camera(&mut self, pos: glam::Vec3, target: glam::Vec3, fov_degrees: f32) {
        self.camera_pos = pos;
        self.camera_target = target;
        self.camera_fov = fov_degrees;
    }

    /// Set directional light direction and color.
    pub fn set_sun(&mut self, direction: glam::Vec3, color: [f32; 3]) {
        self.light_dir = direction.normalize();
        self.light_color = color;
    }

    /// Set ambient light color.
    pub fn set_ambient(&mut self, color: [f32; 3]) { self.ambient = color; }

    /// Draw a filled cube at position with given size per axis.
    pub fn draw_cube(&mut self, pos: glam::Vec3, size: glam::Vec3, color: [f32; 4]) {
        let h = size * 0.5;
        let faces: [([f32; 3], [[f32; 3]; 4]); 6] = [
            ([0.0, 0.0, 1.0], [[pos.x-h.x,pos.y-h.y,pos.z+h.z],[pos.x+h.x,pos.y-h.y,pos.z+h.z],[pos.x+h.x,pos.y+h.y,pos.z+h.z],[pos.x-h.x,pos.y+h.y,pos.z+h.z]]),  // front
            ([0.0, 0.0,-1.0], [[pos.x+h.x,pos.y-h.y,pos.z-h.z],[pos.x-h.x,pos.y-h.y,pos.z-h.z],[pos.x-h.x,pos.y+h.y,pos.z-h.z],[pos.x+h.x,pos.y+h.y,pos.z-h.z]]),  // back
            ([0.0, 1.0, 0.0], [[pos.x-h.x,pos.y+h.y,pos.z+h.z],[pos.x+h.x,pos.y+h.y,pos.z+h.z],[pos.x+h.x,pos.y+h.y,pos.z-h.z],[pos.x-h.x,pos.y+h.y,pos.z-h.z]]),  // top
            ([0.0,-1.0, 0.0], [[pos.x-h.x,pos.y-h.y,pos.z-h.z],[pos.x+h.x,pos.y-h.y,pos.z-h.z],[pos.x+h.x,pos.y-h.y,pos.z+h.z],[pos.x-h.x,pos.y-h.y,pos.z+h.z]]),  // bottom
            ([1.0, 0.0, 0.0], [[pos.x+h.x,pos.y-h.y,pos.z+h.z],[pos.x+h.x,pos.y-h.y,pos.z-h.z],[pos.x+h.x,pos.y+h.y,pos.z-h.z],[pos.x+h.x,pos.y+h.y,pos.z+h.z]]),  // right
            ([-1.0,0.0, 0.0], [[pos.x-h.x,pos.y-h.y,pos.z-h.z],[pos.x-h.x,pos.y-h.y,pos.z+h.z],[pos.x-h.x,pos.y+h.y,pos.z+h.z],[pos.x-h.x,pos.y+h.y,pos.z-h.z]]),  // left
        ];
        for (normal, verts) in &faces {
            self.push_quad(*normal, verts, color);
        }
    }

    /// Draw a flat ground plane centered at origin.
    pub fn draw_ground(&mut self, size: f32, color: [f32; 4]) {
        let h = size * 0.5;
        let n = [0.0, 1.0, 0.0];
        let verts = [[-h, 0.0, h], [h, 0.0, h], [h, 0.0, -h], [-h, 0.0, -h]];
        self.push_quad(n, &verts, color);
    }

    /// Draw a sphere (icosphere approximation with 2 subdivisions).
    pub fn draw_sphere(&mut self, center: glam::Vec3, radius: f32, color: [f32; 4]) {
        let rings = 12u32;
        let sectors = 24u32;
        for r in 0..rings {
            let theta0 = std::f32::consts::PI * r as f32 / rings as f32;
            let theta1 = std::f32::consts::PI * (r + 1) as f32 / rings as f32;
            for s in 0..sectors {
                let phi0 = std::f32::consts::TAU * s as f32 / sectors as f32;
                let phi1 = std::f32::consts::TAU * (s + 1) as f32 / sectors as f32;

                let p = |t: f32, p: f32| -> ([f32; 3], [f32; 3]) {
                    let x = t.sin() * p.cos();
                    let y = t.cos();
                    let z = t.sin() * p.sin();
                    ([center.x + x*radius, center.y + y*radius, center.z + z*radius], [x, y, z])
                };

                let (p00, n00) = p(theta0, phi0);
                let (p10, n10) = p(theta1, phi0);
                let (p11, n11) = p(theta1, phi1);
                let (p01, n01) = p(theta0, phi1);

                self.vertices.push(Vertex3D { position: p00, normal: n00, color });
                self.vertices.push(Vertex3D { position: p10, normal: n10, color });
                self.vertices.push(Vertex3D { position: p11, normal: n11, color });
                self.vertices.push(Vertex3D { position: p00, normal: n00, color });
                self.vertices.push(Vertex3D { position: p11, normal: n11, color });
                self.vertices.push(Vertex3D { position: p01, normal: n01, color });
            }
        }
    }

    /// Draw a wireframe box (12 edges as thin quads).
    pub fn draw_wire_box(&mut self, center: glam::Vec3, half: glam::Vec3, color: [f32; 4]) {
        // Simplified: draw as solid cube with low alpha
        let mut wire_color = color;
        wire_color[3] = 0.3;
        self.draw_cube(center, half * 2.0, wire_color);
    }

    /// Capture the current frame to a PNG file.
    #[cfg(feature = "capture")]
    pub fn capture_frame(&mut self, path: &str) {
        use crate::renderer::capture::{CaptureResources, save_png};
        let device = self.device;
        let w = self.width as u32;
        let h = self.height as u32;
        let fmt = self.renderer.surface_format;

        // Pre-compute uniform and upload before mutably borrowing capture_resources
        let uniform = self.build_uniform();
        device.queue().write_buffer(&self.renderer.uniform_buffer, 0, bytemuck::bytes_of(&uniform));
        if !self.vertices.is_empty() {
            device.queue().write_buffer(
                &self.renderer.vertex_buffer, 0,
                bytemuck::cast_slice(&self.vertices),
            );
        }
        let bg = self.create_bind_group();
        let clear = to_wgpu_color(self.clear_color);

        let cr = self.renderer.capture_resources.get_or_insert_with(|| {
            CaptureResources::new(device.device(), w, h, fmt)
        });
        cr.resize(device.device(), w, h);

        let mut enc = device.device().create_command_encoder(&Default::default());
        {
            let mut rp = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Canvas3D Capture"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &cr.capture_view,
                    resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(clear), store: wgpu::StoreOp::Store },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.renderer.depth_texture,
                    depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Store }),
                    stencil_ops: None,
                }),
                timestamp_writes: None, occlusion_query_set: None,
            });
            if !self.vertices.is_empty() {
                rp.set_pipeline(&self.renderer.pipeline);
                rp.set_bind_group(0, &bg, &[]);
                rp.set_vertex_buffer(0, self.renderer.vertex_buffer.slice(..));
                rp.draw(0..self.vertices.len() as u32, 0..1);
            }
        }
        cr.encode_copy(&mut enc);
        device.queue().submit(std::iter::once(enc.finish()));

        if let Ok(pixels) = cr.read_pixels(device.device()) {
            save_png(&pixels, w, h, std::path::Path::new(path));
        }
    }

    /// Submit all draw commands and present.
    pub fn finish(self) {
        let device = self.device;
        let uniform = self.build_uniform();
        device.queue().write_buffer(&self.renderer.uniform_buffer, 0, bytemuck::bytes_of(&uniform));

        if !self.vertices.is_empty() {
            device.queue().write_buffer(
                &self.renderer.vertex_buffer, 0,
                bytemuck::cast_slice(&self.vertices),
            );
        }

        let bg = self.create_bind_group();
        let clear = to_wgpu_color(self.clear_color);

        let mut enc = device.device().create_command_encoder(&Default::default());
        {
            let mut rp = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Canvas3D Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.swapchain_view,
                    resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(clear), store: wgpu::StoreOp::Store },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.renderer.depth_texture,
                    depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Store }),
                    stencil_ops: None,
                }),
                timestamp_writes: None, occlusion_query_set: None,
            });
            if !self.vertices.is_empty() {
                rp.set_pipeline(&self.renderer.pipeline);
                rp.set_bind_group(0, &bg, &[]);
                rp.set_vertex_buffer(0, self.renderer.vertex_buffer.slice(..));
                rp.draw(0..self.vertices.len() as u32, 0..1);
            }
        }
        device.queue().submit(std::iter::once(enc.finish()));
        self.frame.present();
    }

    // --- Internal helpers ---

    fn build_uniform(&self) -> Canvas3DUniform {
        let aspect = self.width / self.height;
        let view = glam::Mat4::look_at_rh(self.camera_pos, self.camera_target, glam::Vec3::Y);
        let proj = glam::Mat4::perspective_rh(self.camera_fov.to_radians(), aspect, 0.1, 1000.0);
        Canvas3DUniform {
            view_proj: (proj * view).to_cols_array_2d(),
            light_dir: [self.light_dir.x, self.light_dir.y, self.light_dir.z, 0.0],
            light_color: [self.light_color[0], self.light_color[1], self.light_color[2], 1.0],
            ambient: [self.ambient[0], self.ambient[1], self.ambient[2], 1.0],
        }
    }

    fn create_bind_group(&self) -> wgpu::BindGroup {
        self.device.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Canvas3D BG"),
            layout: &self.renderer.bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: self.renderer.uniform_buffer.as_entire_binding(),
            }],
        })
    }

    fn push_quad(&mut self, normal: [f32; 3], verts: &[[f32; 3]; 4], color: [f32; 4]) {
        // Two triangles: 0-1-2, 0-2-3
        self.vertices.push(Vertex3D { position: verts[0], normal, color });
        self.vertices.push(Vertex3D { position: verts[1], normal, color });
        self.vertices.push(Vertex3D { position: verts[2], normal, color });
        self.vertices.push(Vertex3D { position: verts[0], normal, color });
        self.vertices.push(Vertex3D { position: verts[2], normal, color });
        self.vertices.push(Vertex3D { position: verts[3], normal, color });
    }
}

fn to_wgpu_color(c: [f32; 4]) -> wgpu::Color {
    wgpu::Color { r: c[0] as f64, g: c[1] as f64, b: c[2] as f64, a: c[3] as f64 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertex3d_size() {
        assert_eq!(std::mem::size_of::<Vertex3D>(), 40); // 3+3+4 floats = 10 * 4
    }

    #[test]
    fn test_uniform_size() {
        // mat4(64) + vec4(16) + vec4(16) + vec4(16) = 112 bytes
        assert_eq!(std::mem::size_of::<Canvas3DUniform>(), 112);
    }
}
