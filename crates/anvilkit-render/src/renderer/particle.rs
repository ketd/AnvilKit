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
use glam::Vec3;

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
    pub position: Vec3,
    pub velocity: Vec3,
    pub color: [f32; 4],
    pub size: f32,
    pub age: f32,
    pub lifetime: f32,
}

impl Particle {
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
    Sphere { radius: f32 },
    /// 从圆锥体发射（角度弧度）
    Cone { angle: f32, radius: f32 },
    /// 从长方体区域发射
    Box { half_extents: Vec3 },
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
#[derive(Debug, Clone, Component)]
pub struct ParticleEmitter {
    /// 每秒发射粒子数
    pub emit_rate: f32,
    /// 粒子生命周期（秒）
    pub lifetime: f32,
    /// 初始速度大小
    pub initial_speed: f32,
    /// 速度随机偏差
    pub speed_variance: f32,
    /// 初始粒子大小
    pub initial_size: f32,
    /// 大小随机偏差
    pub size_variance: f32,
    /// 起始颜色
    pub start_color: [f32; 4],
    /// 结束颜色（生命周期末端）
    pub end_color: [f32; 4],
    /// 重力
    pub gravity: Vec3,
    /// 发射形状
    pub shape: EmitShape,
    /// 最大粒子数
    pub max_particles: usize,
    /// 是否启用
    pub enabled: bool,
    /// 发射累积器（内部使用）
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

#[cfg(test)]
mod tests {
    use super::*;

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
