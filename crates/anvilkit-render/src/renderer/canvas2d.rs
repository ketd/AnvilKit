//! # Canvas2D — Agent-Friendly 2D Rendering
//!
//! Zero-wgpu-knowledge 2D drawing API. Agents call `draw_rect`, `draw_circle`,
//! `draw_text` — the Canvas handles swapchain, encoder, render pass internally.
//!
//! ## Usage
//!
//! ```rust,ignore
//! impl GameCallbacks for MyGame {
//!     fn render(&mut self, ctx: &mut GameContext) {
//!         let Some(mut canvas) = Canvas2D::begin(ctx) else { return };
//!         canvas.clear([0.1, 0.1, 0.2, 1.0]);
//!         canvas.draw_rect(100.0, 300.0, 30.0, 30.0, [1.0, 1.0, 0.0, 1.0]); // yellow bird
//!         canvas.draw_rect(400.0, 0.0, 60.0, 200.0, [0.2, 0.8, 0.2, 1.0]); // green pipe
//!         canvas.draw_text(10.0, 10.0, "Score: 5", 32.0, [1.0, 1.0, 1.0, 1.0]);
//!         canvas.finish(); // submits all draw calls
//!     }
//! }
//! ```

use crate::renderer::RenderDevice;
use crate::renderer::text::TextRenderer;

const MAX_CANVAS_VERTICES: usize = 65536;

const CANVAS_SHADER: &str = r#"
struct CanvasUniform {
    projection: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> canvas: CanvasUniform;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = canvas.projection * vec4<f32>(in.position, 0.0, 1.0);
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
"#;

/// A single 2D vertex: position + color.
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Canvas2DVertex {
    position: [f32; 2],
    color: [f32; 4],
}

impl Canvas2DVertex {
    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 8,
                    shader_location: 1,
                },
            ],
        }
    }
}

/// GPU resources for Canvas2D. Created once, reused every frame.
pub struct Canvas2DRenderer {
    vertex_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    pipeline: wgpu::RenderPipeline,
    bgl: wgpu::BindGroupLayout,
    text_renderer: TextRenderer,
    surface_format: wgpu::TextureFormat,
    #[cfg(feature = "capture")]
    capture_resources: Option<crate::renderer::capture::CaptureResources>,
}

impl Canvas2DRenderer {
    /// Create the Canvas2D GPU resources. Call once in `GameCallbacks::init()`.
    pub fn new(device: &RenderDevice, format: wgpu::TextureFormat) -> Self {
        let vertex_buffer = device.device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("Canvas2D VB"),
            size: (MAX_CANVAS_VERTICES * std::mem::size_of::<Canvas2DVertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_buffer = device.device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("Canvas2D Uniform"),
            size: 64, // mat4x4<f32>
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bgl = device.device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Canvas2D BGL"),
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
        });

        let shader = device.device().create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Canvas2D Shader"),
            source: wgpu::ShaderSource::Wgsl(CANVAS_SHADER.into()),
        });

        let pl = device.device().create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Canvas2D PL"),
            bind_group_layouts: &[&bgl],
            push_constant_ranges: &[],
        });

        let pipeline = device.device().create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Canvas2D Pipeline"),
            layout: Some(&pl),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Canvas2DVertex::layout()],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });

        let text_renderer = TextRenderer::new(device, format);

        Self {
            vertex_buffer,
            uniform_buffer,
            pipeline,
            bgl,
            text_renderer,
            surface_format: format,
            #[cfg(feature = "capture")]
            capture_resources: None,
        }
    }
}

/// A 2D drawing canvas for one frame. Agent-facing API — no wgpu knowledge required.
///
/// Created via `Canvas2D::begin()`, draws are batched, then `finish()` submits everything.
pub struct Canvas2D<'a> {
    device: &'a RenderDevice,
    renderer: &'a mut Canvas2DRenderer,
    swapchain_view: wgpu::TextureView,
    frame: wgpu::SurfaceTexture,
    vertices: Vec<Canvas2DVertex>,
    text_commands: Vec<TextCommand>,
    clear_color: [f32; 4],
    width: f32,
    height: f32,
}

struct TextCommand {
    x: f32,
    y: f32,
    text: String,
    size: f32,
    color: [f32; 4],
}

impl<'a> Canvas2D<'a> {
    /// Begin a 2D drawing frame. Returns `None` if GPU resources aren't ready.
    ///
    /// # Usage
    ///
    /// ```rust,ignore
    /// let Some(mut canvas) = Canvas2D::begin(ctx.render_app, &self.canvas_renderer) else { return };
    /// canvas.draw_rect(10.0, 10.0, 100.0, 50.0, [1.0, 0.0, 0.0, 1.0]);
    /// canvas.finish();
    /// ```
    pub fn begin(
        render_app: &'a mut crate::window::events::RenderApp,
        renderer: &'a mut Canvas2DRenderer,
    ) -> Option<Self> {
        let device = render_app.render_device()?;
        let (w, h) = render_app.window_state().size();
        let frame = render_app.get_current_frame()?;
        let swapchain_view = frame.texture.create_view(&Default::default());

        Some(Canvas2D {
            device,
            renderer,
            swapchain_view,
            frame,
            vertices: Vec::new(),
            text_commands: Vec::new(),
            clear_color: [0.0, 0.0, 0.0, 1.0],
            width: w as f32,
            height: h as f32,
        })
    }

    /// Canvas width in pixels.
    pub fn width(&self) -> f32 { self.width }

    /// Canvas height in pixels.
    pub fn height(&self) -> f32 { self.height }

    /// Set the background clear color. Call before any draw commands.
    pub fn clear(&mut self, color: [f32; 4]) {
        self.clear_color = color;
    }

    /// Draw a filled rectangle.
    ///
    /// - `x, y`: top-left corner in screen pixels (origin = top-left)
    /// - `w, h`: width and height in pixels
    /// - `color`: RGBA, each component 0.0..1.0
    pub fn draw_rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: [f32; 4]) {
        let (x0, y0) = (x, y);
        let (x1, y1) = (x + w, y + h);

        // Two triangles forming a quad
        self.vertices.push(Canvas2DVertex { position: [x0, y0], color });
        self.vertices.push(Canvas2DVertex { position: [x1, y0], color });
        self.vertices.push(Canvas2DVertex { position: [x0, y1], color });

        self.vertices.push(Canvas2DVertex { position: [x1, y0], color });
        self.vertices.push(Canvas2DVertex { position: [x1, y1], color });
        self.vertices.push(Canvas2DVertex { position: [x0, y1], color });
    }

    /// Draw a filled circle approximated by a triangle fan.
    ///
    /// - `cx, cy`: center in screen pixels
    /// - `radius`: radius in pixels
    /// - `color`: RGBA
    pub fn draw_circle(&mut self, cx: f32, cy: f32, radius: f32, color: [f32; 4]) {
        let segments = 24u32;
        for i in 0..segments {
            let a0 = (i as f32) / (segments as f32) * std::f32::consts::TAU;
            let a1 = ((i + 1) as f32) / (segments as f32) * std::f32::consts::TAU;
            self.vertices.push(Canvas2DVertex { position: [cx, cy], color });
            self.vertices.push(Canvas2DVertex { position: [cx + radius * a0.cos(), cy + radius * a0.sin()], color });
            self.vertices.push(Canvas2DVertex { position: [cx + radius * a1.cos(), cy + radius * a1.sin()], color });
        }
    }

    /// Draw text at the given screen position.
    ///
    /// - `x, y`: position in screen pixels (top-left of first character)
    /// - `text`: string to render
    /// - `size`: font size in pixels
    /// - `color`: RGBA
    pub fn draw_text(&mut self, x: f32, y: f32, text: &str, size: f32, color: [f32; 4]) {
        self.text_commands.push(TextCommand {
            x, y,
            text: text.to_string(),
            size,
            color,
        });
    }

    /// Submit all draw commands, capture the frame as a PNG file, then present.
    ///
    /// Returns `true` if a screenshot was saved. The capture texture is lazily
    /// initialized on first call. Only available with the `capture` feature.
    ///
    /// # Usage
    ///
    /// ```rust,ignore
    /// canvas.draw_rect(100.0, 100.0, 50.0, 50.0, [1.0, 0.0, 0.0, 1.0]);
    /// canvas.capture_frame("screenshots/frame.png");
    /// canvas.finish();
    /// ```
    #[cfg(feature = "capture")]
    pub fn capture_frame(&mut self, path: &str) {
        use crate::renderer::capture::{CaptureResources, save_png};

        let device = self.device;
        let w = self.width as u32;
        let h = self.height as u32;

        // Lazy init capture resources
        let cr = self.renderer.capture_resources.get_or_insert_with(|| {
            CaptureResources::new(device.device(), w, h, self.renderer.surface_format)
        });
        cr.resize(device.device(), w, h);

        // Render geometry to capture texture
        let proj = ortho_matrix(self.width, self.height);
        device.queue().write_buffer(&self.renderer.uniform_buffer, 0, bytemuck::cast_slice(&proj));

        if !self.vertices.is_empty() {
            let byte_len = self.vertices.len() * std::mem::size_of::<Canvas2DVertex>();
            device.queue().write_buffer(
                &self.renderer.vertex_buffer, 0,
                &bytemuck::cast_slice(&self.vertices)[..byte_len],
            );
        }

        let bg = device.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Canvas2D Capture BG"),
            layout: &self.renderer.bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: self.renderer.uniform_buffer.as_entire_binding(),
            }],
        });

        let clear = wgpu::Color {
            r: self.clear_color[0] as f64,
            g: self.clear_color[1] as f64,
            b: self.clear_color[2] as f64,
            a: self.clear_color[3] as f64,
        };

        let mut enc = device.device().create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Canvas2D Capture Enc"),
        });

        // Render to capture texture
        {
            let mut rp = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Canvas2D Capture Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &cr.capture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(clear),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            if !self.vertices.is_empty() {
                rp.set_pipeline(&self.renderer.pipeline);
                rp.set_bind_group(0, &bg, &[]);
                rp.set_vertex_buffer(0, self.renderer.vertex_buffer.slice(..));
                rp.draw(0..self.vertices.len() as u32, 0..1);
            }
        }

        // Copy capture texture to staging buffer
        cr.encode_copy(&mut enc);
        device.queue().submit(std::iter::once(enc.finish()));

        // Read pixels and save PNG
        if let Ok(pixels) = cr.read_pixels(device.device()) {
            save_png(&pixels, w, h, std::path::Path::new(path));
        }
    }

    /// Submit all draw commands to the GPU and present the frame.
    ///
    /// This is the only method that touches wgpu — and it's internal.
    /// After calling `finish()`, the Canvas2D is consumed and the frame is presented.
    pub fn finish(self) {
        let device = self.device;

        // Build orthographic projection: screen pixels, origin top-left
        let proj = ortho_matrix(self.width, self.height);
        device.queue().write_buffer(&self.renderer.uniform_buffer, 0, bytemuck::cast_slice(&proj));

        // Upload vertices
        if !self.vertices.is_empty() {
            let byte_len = self.vertices.len() * std::mem::size_of::<Canvas2DVertex>();
            device.queue().write_buffer(
                &self.renderer.vertex_buffer,
                0,
                &bytemuck::cast_slice(&self.vertices)[..byte_len],
            );
        }

        let bg = device.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Canvas2D BG"),
            layout: &self.renderer.bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: self.renderer.uniform_buffer.as_entire_binding(),
            }],
        });

        let clear = wgpu::Color {
            r: self.clear_color[0] as f64,
            g: self.clear_color[1] as f64,
            b: self.clear_color[2] as f64,
            a: self.clear_color[3] as f64,
        };

        // Geometry pass
        {
            let mut enc = device.device().create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Canvas2D Enc"),
            });
            {
                let mut rp = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Canvas2D Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &self.swapchain_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(clear),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

                if !self.vertices.is_empty() {
                    rp.set_pipeline(&self.renderer.pipeline);
                    rp.set_bind_group(0, &bg, &[]);
                    rp.set_vertex_buffer(0, self.renderer.vertex_buffer.slice(..));
                    rp.draw(0..self.vertices.len() as u32, 0..1);
                }
            }
            device.queue().submit(std::iter::once(enc.finish()));
        }

        // Text pass (on top of geometry, separate encoder)
        if !self.text_commands.is_empty() {
            let mut enc = device.device().create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Canvas2D Text Enc"),
            });
            for cmd in &self.text_commands {
                let color = glam::Vec3::new(cmd.color[0], cmd.color[1], cmd.color[2]);
                self.renderer.text_renderer.draw_text(
                    device,
                    &mut enc,
                    &self.swapchain_view,
                    &cmd.text,
                    cmd.x,
                    cmd.y,
                    cmd.size,
                    color,
                    self.width,
                    self.height,
                );
            }
            device.queue().submit(std::iter::once(enc.finish()));
        }

        // Present
        self.frame.present();
    }
}

/// Orthographic projection matrix: screen pixels, origin top-left, Y down.
/// Returns a flat [f32; 16] for uniform upload.
fn ortho_matrix(width: f32, height: f32) -> [f32; 16] {
    let l = 0.0;
    let r = width;
    let t = 0.0;
    let b = height;
    let n = -1.0;
    let f = 1.0;
    [
        2.0 / (r - l),     0.0,                0.0,             0.0,
        0.0,                2.0 / (t - b),      0.0,             0.0,
        0.0,                0.0,                1.0 / (f - n),   0.0,
        -(r + l) / (r - l), -(t + b) / (t - b), -n / (f - n),   1.0,
    ]
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ortho_matrix_dimensions() {
        let m = ortho_matrix(800.0, 600.0);
        // m[0] = 2/800 = 0.0025
        assert!((m[0] - 2.0 / 800.0).abs() < 1e-6);
        // m[5] = 2/(0-600) = -0.00333...
        assert!((m[5] - 2.0 / -600.0).abs() < 1e-6);
    }

    #[test]
    fn test_canvas2d_vertex_size() {
        assert_eq!(std::mem::size_of::<Canvas2DVertex>(), 24); // 2 floats + 4 floats
    }
}
