//! # 粒子系统
//!
//! 提供 CPU 端粒子发射器、粒子生命周期管理和力场支持。
//!
//! ## 核心类型
//!
//! - [`ParticleEmitter`]: 粒子发射器组件
//! - [`Particle`]: 单个粒子运行时状态
//! - [`ParticleSystem`]: 粒子池管理和更新逻辑

use bevy_ecs::prelude::*;
use anvilkit_describe::Describe;
use glam::Vec3;
use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

/// 单个粒子的运行时状态
///
/// # 示例
///
/// ```rust
/// use anvilkit_render::renderer::particle::Particle;
/// use glam::Vec3;
///
/// let p = Particle::new(Vec3::ZERO, Vec3::Y, 2.0);
/// assert!(p.is_alive());
/// assert_eq!(p.age, 0.0);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Particle {
    /// World-space position of the particle.
    pub position: Vec3,
    /// Current velocity vector.
    pub velocity: Vec3,
    /// Particle color [R, G, B, A].
    pub color: [f32; 4],
    /// Visual size of the particle in world units.
    pub size: f32,
    /// Elapsed time since the particle was spawned (seconds).
    pub age: f32,
    /// Total lifespan of the particle (seconds).
    pub lifetime: f32,
}

impl Particle {
    /// Creates a new particle with the given position, velocity, and lifetime.
    pub fn new(position: Vec3, velocity: Vec3, lifetime: f32) -> Self {
        Self {
            position,
            velocity,
            color: [1.0, 1.0, 1.0, 1.0],
            size: 0.1,
            age: 0.0,
            lifetime,
        }
    }

    /// 粒子是否存活
    pub fn is_alive(&self) -> bool {
        self.age < self.lifetime
    }

    /// 归一化年龄 [0, 1]
    pub fn normalized_age(&self) -> f32 {
        (self.age / self.lifetime).clamp(0.0, 1.0)
    }

    /// 更新粒子状态
    pub fn update(&mut self, dt: f32, gravity: Vec3) {
        self.velocity += gravity * dt;
        self.position += self.velocity * dt;
        self.age += dt;
        // 淡出：alpha 随年龄线性衰减
        self.color[3] = 1.0 - self.normalized_age();
    }
}

/// 发射形状
///
/// # 示例
///
/// ```rust
/// use anvilkit_render::renderer::particle::EmitShape;
/// let shape = EmitShape::Sphere { radius: 1.0 };
/// ```
#[derive(Debug, Clone)]
pub enum EmitShape {
    /// 从一个点发射
    Point,
    /// 从球体表面发射
    Sphere {
        /// Sphere radius in world units.
        radius: f32,
    },
    /// 从圆锥体发射（角度弧度）
    Cone {
        /// Half-angle of the cone in radians.
        angle: f32,
        /// Base radius of the cone.
        radius: f32,
    },
    /// 从长方体区域发射
    Box {
        /// Half-size of the box along each axis.
        half_extents: Vec3,
    },
}

impl Default for EmitShape {
    fn default() -> Self { EmitShape::Point }
}

/// 粒子发射器组件
///
/// # 示例
///
/// ```rust
/// use anvilkit_render::renderer::particle::{ParticleEmitter, EmitShape};
/// use glam::Vec3;
///
/// let emitter = ParticleEmitter {
///     emit_rate: 50.0,
///     lifetime: 2.0,
///     initial_speed: 3.0,
///     gravity: Vec3::new(0.0, -9.8, 0.0),
///     shape: EmitShape::Cone { angle: 0.3, radius: 0.1 },
///     max_particles: 500,
///     ..Default::default()
/// };
/// assert!(emitter.enabled);
/// ```
#[derive(Debug, Clone, Component, Describe)]
/// CPU-side particle emitter component.
pub struct ParticleEmitter {
    /// 每秒发射粒子数
    #[describe(hint = "Particles spawned per second", range = "0.0..1000.0", default = "20.0")]
    pub emit_rate: f32,
    /// 粒子生命周期（秒）
    #[describe(hint = "How long each particle lives in seconds", range = "0.01..30.0", default = "1.5")]
    pub lifetime: f32,
    /// 初始速度大小
    #[describe(hint = "Initial speed magnitude", range = "0.0..100.0", default = "2.0")]
    pub initial_speed: f32,
    /// 速度随机偏差
    #[describe(hint = "Random speed variation (+/-)", range = "0.0..50.0", default = "0.5")]
    pub speed_variance: f32,
    /// 初始粒子大小
    #[describe(hint = "Initial visual size in world units", range = "0.001..10.0", default = "0.05")]
    pub initial_size: f32,
    /// 大小随机偏差
    #[describe(hint = "Random size variation (+/-)", range = "0.0..5.0", default = "0.02")]
    pub size_variance: f32,
    /// 起始颜色
    #[describe(hint = "Particle color at spawn [R,G,B,A]", default = "[1.0, 1.0, 1.0, 1.0]")]
    pub start_color: [f32; 4],
    /// 结束颜色（生命周期末端）
    #[describe(hint = "Particle color at end of life [R,G,B,A]", default = "[1.0, 1.0, 1.0, 0.0]")]
    pub end_color: [f32; 4],
    /// 重力
    #[describe(hint = "Gravity acceleration vector (m/s^2)", default = "(0.0, -9.8, 0.0)")]
    pub gravity: Vec3,
    /// 发射形状
    #[describe(hint = "EmitShape: Point, Sphere, Cone, or Box", default = "Point")]
    pub shape: EmitShape,
    /// 最大粒子数
    #[describe(hint = "Pool capacity; older particles recycled", range = "1..100000", default = "200")]
    pub max_particles: usize,
    /// 是否启用
    #[describe(hint = "Toggle emission on/off", default = "true")]
    pub enabled: bool,
    /// 发射累积器（内部使用）
    #[describe(hint = "Internal fractional emit counter; do not set manually", default = "0.0")]
    pub emit_accumulator: f32,
}

impl Default for ParticleEmitter {
    fn default() -> Self {
        Self {
            emit_rate: 20.0,
            lifetime: 1.5,
            initial_speed: 2.0,
            speed_variance: 0.5,
            initial_size: 0.05,
            size_variance: 0.02,
            start_color: [1.0, 1.0, 1.0, 1.0],
            end_color: [1.0, 1.0, 1.0, 0.0],
            gravity: Vec3::new(0.0, -9.8, 0.0),
            shape: EmitShape::Point,
            max_particles: 200,
            enabled: true,
            emit_accumulator: 0.0,
        }
    }
}

/// 粒子系统（粒子池 + 更新逻辑）
///
/// # 示例
///
/// ```rust
/// use anvilkit_render::renderer::particle::ParticleSystem;
///
/// let mut sys = ParticleSystem::new(100);
/// assert_eq!(sys.alive_count(), 0);
/// assert_eq!(sys.capacity(), 100);
/// ```
pub struct ParticleSystem {
    particles: Vec<Particle>,
    capacity: usize,
}

impl ParticleSystem {
    /// Creates a new particle system with the given maximum capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            particles: Vec::with_capacity(capacity),
            capacity,
        }
    }

    /// 存活粒子数
    pub fn alive_count(&self) -> usize {
        self.particles.iter().filter(|p| p.is_alive()).count()
    }

    /// 最大容量
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// 发射一个粒子
    pub fn emit(&mut self, particle: Particle) {
        if self.particles.len() < self.capacity {
            self.particles.push(particle);
        } else {
            // 复用已死亡粒子的槽位
            if let Some(dead) = self.particles.iter_mut().find(|p| !p.is_alive()) {
                *dead = particle;
            }
        }
    }

    /// 更新所有粒子
    pub fn update(&mut self, dt: f32, gravity: Vec3) {
        for p in &mut self.particles {
            if p.is_alive() {
                p.update(dt, gravity);
            }
        }
    }

    /// 获取存活粒子的迭代器
    pub fn alive_particles(&self) -> impl Iterator<Item = &Particle> {
        self.particles.iter().filter(|p| p.is_alive())
    }

    /// 清除所有粒子
    pub fn clear(&mut self) {
        self.particles.clear();
    }
}

// ---------------------------------------------------------------------------
//  ParticleRenderer — GPU pipeline for particle point-sprite rendering
// ---------------------------------------------------------------------------

const PARTICLE_SHADER: &str = include_str!("../shaders/particle.wgsl");

/// 粒子 GPU 顶点 (32 bytes)
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct ParticleVertex {
    /// World-space position (x, y, z).
    pub position: [f32; 3],
    /// Vertex color [R, G, B, A].
    pub color: [f32; 4],
    /// Billboard size in world units.
    pub size: f32,
}

impl ParticleVertex {
    /// Returns the GPU vertex buffer layout for particle instances.
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: &[wgpu::VertexAttribute] = &[
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x3,
            },
            wgpu::VertexAttribute {
                offset: 12,
                shader_location: 1,
                format: wgpu::VertexFormat::Float32x4,
            },
            wgpu::VertexAttribute {
                offset: 28,
                shader_location: 2,
                format: wgpu::VertexFormat::Float32,
            },
        ];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<ParticleVertex>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: ATTRIBUTES,
        }
    }
}

/// GPU 粒子渲染器
pub struct ParticleRenderer {
    /// The wgpu render pipeline for particle point-sprites.
    pub pipeline: wgpu::RenderPipeline,
    /// Uniform buffer holding the scene view-projection matrix.
    pub scene_buffer: wgpu::Buffer,
    /// Bind group for the scene uniform buffer.
    pub scene_bind_group: wgpu::BindGroup,
    /// Cached instance buffer for per-frame reuse.
    cached_instance_buf: super::shared::CachedBuffer,
}

impl ParticleRenderer {
    /// Creates the particle render pipeline, uniform buffer, and bind group.
    pub fn new(device: &super::RenderDevice, format: wgpu::TextureFormat) -> Self {
        let shader = device.device().create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Particle Shader"),
            source: wgpu::ShaderSource::Wgsl(PARTICLE_SHADER.into()),
        });

        let scene_bgl = device.device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Particle Scene BGL"),
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

        let pipeline_layout = device.device().create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Particle Pipeline Layout"),
            bind_group_layouts: &[&scene_bgl],
            push_constant_ranges: &[],
        });

        let pipeline = device.device().create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Particle Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[ParticleVertex::layout()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: false, // read-only: particles don't write depth
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let initial = super::shared::MatrixUniform::identity();
        let scene_buffer = device.device().create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Particle Scene UB"),
            contents: bytemuck::bytes_of(&initial),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let scene_bg = device.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Particle Scene BG"),
            layout: &scene_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: scene_buffer.as_entire_binding(),
            }],
        });

        Self {
            pipeline,
            scene_buffer,
            scene_bind_group: scene_bg,
            cached_instance_buf: super::shared::CachedBuffer::vertex("Particle Instance (cached)"),
        }
    }

    /// 从 ParticleSystem 收集存活粒子并渲染。
    ///
    /// - `depth_view`: 如果提供，启用深度测试（read-only）。
    /// - `camera_pos`: 如果提供，按相机距离排序（远→近，正确 alpha 混合）。
    pub fn render(
        &mut self,
        device: &super::RenderDevice,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        depth_view: Option<&wgpu::TextureView>,
        particle_system: &ParticleSystem,
        view_proj: &glam::Mat4,
        camera_pos: Option<Vec3>,
    ) {
        let mut particles: Vec<&Particle> = particle_system.alive_particles().collect();

        if particles.is_empty() {
            return;
        }

        // Sort back-to-front for correct alpha blending
        if let Some(cam) = camera_pos {
            particles.sort_by(|a, b| {
                let da = (b.position - cam).length_squared();
                let db = (a.position - cam).length_squared();
                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
            });
        }

        let vertices: Vec<ParticleVertex> = particles.iter()
            .map(|p| ParticleVertex {
                position: p.position.into(),
                color: p.color,
                size: p.size,
            })
            .collect();

        // Update view-projection
        let uniform = super::shared::MatrixUniform::from_mat4(view_proj);
        device.queue().write_buffer(&self.scene_buffer, 0, bytemuck::bytes_of(&uniform));

        // Reuse cached instance buffer if large enough
        let data: &[u8] = bytemuck::cast_slice(&vertices);
        let instance_buffer = self.cached_instance_buf.ensure_and_write(
            device.device(),
            device.queue(),
            data,
        );

        {
            let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Particle Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: depth_view.map(|dv| wgpu::RenderPassDepthStencilAttachment {
                    view: dv,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store, // read-only but StoreOp required
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            rp.set_pipeline(&self.pipeline);
            rp.set_bind_group(0, &self.scene_bind_group, &[]);
            rp.set_vertex_buffer(0, instance_buffer.slice(..));
            rp.draw(0..6, 0..vertices.len() as u32);
        }
    }
}

// ---------------------------------------------------------------------------
//  ECS Systems — automatic emit and update for ParticleEmitter components
// ---------------------------------------------------------------------------

/// ECS 资源：全局粒子系统池，按实体管理。
#[derive(Resource, Default)]
pub struct ParticleSystems {
    /// 每个拥有 ParticleEmitter 的实体对应一个 ParticleSystem。
    pub systems: std::collections::HashMap<bevy_ecs::entity::Entity, ParticleSystem>,
}

/// 发射系统：遍历所有 ParticleEmitter，按 emit_rate 发射粒子。
///
/// 需要 `DeltaTime` 资源（来自 `anvilkit_core::time::DeltaTime`）和
/// `Transform`（来自 `anvilkit_core::math::Transform`）。
pub fn particle_emit_system(
    dt: Res<anvilkit_core::time::DeltaTime>,
    mut emitters: Query<(Entity, &mut ParticleEmitter, &anvilkit_core::math::Transform)>,
    mut pool: ResMut<ParticleSystems>,
) {
    for (entity, mut emitter, transform) in &mut emitters {
        if !emitter.enabled {
            continue;
        }

        let sys = pool.systems
            .entry(entity)
            .or_insert_with(|| ParticleSystem::new(emitter.max_particles));

        emitter.emit_accumulator += emitter.emit_rate * dt.0;
        let emit_count = emitter.emit_accumulator as usize;
        emitter.emit_accumulator -= emit_count as f32;

        for _ in 0..emit_count {
            let velocity = Vec3::Y * emitter.initial_speed;
            let mut p = Particle::new(transform.translation, velocity, emitter.lifetime);
            p.size = emitter.initial_size;
            p.color = emitter.start_color;
            sys.emit(p);
        }
    }
}

/// 更新系统：推进所有粒子生命周期。
pub fn particle_update_system(
    dt: Res<anvilkit_core::time::DeltaTime>,
    emitters: Query<(Entity, &ParticleEmitter)>,
    mut pool: ResMut<ParticleSystems>,
) {
    for (entity, emitter) in &emitters {
        if let Some(sys) = pool.systems.get_mut(&entity) {
            sys.update(dt.0, emitter.gravity);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_particle_vertex_size() {
        assert_eq!(std::mem::size_of::<ParticleVertex>(), 32);
    }

    #[test]
    fn test_particle_lifecycle() {
        let mut p = Particle::new(Vec3::ZERO, Vec3::Y, 1.0);
        assert!(p.is_alive());

        p.update(0.5, Vec3::ZERO);
        assert!(p.is_alive());
        assert!((p.position.y - 0.5).abs() < 0.001);

        p.update(0.6, Vec3::ZERO);
        assert!(!p.is_alive());
    }

    #[test]
    fn test_particle_system() {
        let mut sys = ParticleSystem::new(10);
        sys.emit(Particle::new(Vec3::ZERO, Vec3::Y, 1.0));
        sys.emit(Particle::new(Vec3::ZERO, Vec3::X, 0.5));

        assert_eq!(sys.alive_count(), 2);

        sys.update(0.6, Vec3::ZERO);
        assert_eq!(sys.alive_count(), 1); // second particle died (0.5s lifetime)

        sys.update(0.5, Vec3::ZERO);
        assert_eq!(sys.alive_count(), 0);
    }

    #[test]
    fn test_particle_emit_system() {
        use bevy_ecs::world::World;
        use bevy_ecs::schedule::Schedule;

        let mut world = World::new();
        world.insert_resource(anvilkit_core::time::DeltaTime(1.0 / 60.0));
        world.init_resource::<ParticleSystems>();

        // Spawn emitter entity with Transform
        let emitter = ParticleEmitter {
            emit_rate: 60.0, // 60 per second → 1 per frame at 60fps
            lifetime: 2.0,
            ..Default::default()
        };
        let transform = anvilkit_core::math::Transform::from_translation(Vec3::ZERO);
        world.spawn((emitter, transform));

        let mut schedule = Schedule::default();
        schedule.add_systems((particle_emit_system, particle_update_system.after(particle_emit_system)));

        // Run 60 frames
        for _ in 0..60 {
            schedule.run(&mut world);
        }

        let pool = world.resource::<ParticleSystems>();
        let total_alive: usize = pool.systems.values().map(|s| s.alive_count()).sum();
        assert!(total_alive > 0, "Expected alive particles after 60 frames of emission");
    }

    #[test]
    fn test_particle_system_recycle() {
        let mut sys = ParticleSystem::new(2);
        sys.emit(Particle::new(Vec3::ZERO, Vec3::ZERO, 0.1));
        sys.emit(Particle::new(Vec3::ZERO, Vec3::ZERO, 0.1));

        // Both die
        sys.update(0.2, Vec3::ZERO);
        assert_eq!(sys.alive_count(), 0);

        // Recycle slot
        sys.emit(Particle::new(Vec3::ONE, Vec3::ZERO, 1.0));
        assert_eq!(sys.alive_count(), 1);
    }
}
