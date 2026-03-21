//! # 骨骼动画数据
//!
//! 定义骨骼、蒙皮和动画剪辑的 CPU 侧数据结构。
//! 从 glTF 文件提取，用于驱动 GPU 端的蒙皮网格渲染。
//!
//! ## 核心类型
//!
//! - [`Skeleton`]: 骨骼层次结构（关节树）
//! - [`SkinData`]: 蒙皮数据（骨骼权重和索引）
//! - [`AnimationClip`]: 动画剪辑（关键帧序列）

use glam::Mat4;

/// 单个关节（骨骼节点）
///
/// # 示例
///
/// ```rust
/// use anvilkit_assets::animation::Joint;
/// use glam::Mat4;
///
/// let joint = Joint {
///     name: "Hips".to_string(),
///     parent: None,
///     inverse_bind_matrix: Mat4::IDENTITY,
/// };
/// assert!(joint.parent.is_none());
/// ```
#[derive(Debug, Clone)]
pub struct Joint {
    /// 关节名称
    pub name: String,
    /// 父关节索引（None = 根关节）
    pub parent: Option<usize>,
    /// 逆绑定矩阵（将顶点从模型空间变换到关节空间）
    pub inverse_bind_matrix: Mat4,
}

/// 骨骼层次结构
///
/// # 示例
///
/// ```rust
/// use anvilkit_assets::animation::{Skeleton, Joint};
/// use glam::Mat4;
///
/// let skeleton = Skeleton {
///     joints: vec![
///         Joint { name: "Root".into(), parent: None, inverse_bind_matrix: Mat4::IDENTITY },
///         Joint { name: "Spine".into(), parent: Some(0), inverse_bind_matrix: Mat4::IDENTITY },
///     ],
/// };
/// assert_eq!(skeleton.joint_count(), 2);
/// ```
#[derive(Debug, Clone)]
pub struct Skeleton {
    pub joints: Vec<Joint>,
}

impl Skeleton {
    pub fn joint_count(&self) -> usize {
        self.joints.len()
    }

    /// 查找关节索引
    pub fn find_joint(&self, name: &str) -> Option<usize> {
        self.joints.iter().position(|j| j.name == name)
    }
}

/// 蒙皮数据
///
/// 每个顶点最多受 4 个骨骼影响（标准 LBS 限制）。
///
/// # 示例
///
/// ```rust
/// use anvilkit_assets::animation::SkinData;
///
/// let skin = SkinData {
///     joint_indices: vec![[0, 0, 0, 0]],
///     joint_weights: vec![[1.0, 0.0, 0.0, 0.0]],
/// };
/// assert_eq!(skin.vertex_count(), 1);
/// ```
#[derive(Debug, Clone)]
pub struct SkinData {
    /// 每个顶点的骨骼索引 [4 个]
    pub joint_indices: Vec<[u16; 4]>,
    /// 每个顶点的骨骼权重 [4 个，总和 = 1.0]
    pub joint_weights: Vec<[f32; 4]>,
}

impl SkinData {
    pub fn vertex_count(&self) -> usize {
        self.joint_indices.len()
    }
}

/// 动画插值方式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interpolation {
    /// 阶梯（跳变）
    Step,
    /// 线性
    Linear,
    /// 三次样条
    CubicSpline,
}

/// 动画通道目标属性
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationProperty {
    Translation,
    Rotation,
    Scale,
}

/// 单个动画通道（一个关节的一个属性的关键帧序列）
///
/// # 示例
///
/// ```rust
/// use anvilkit_assets::animation::{AnimationChannel, AnimationProperty, Interpolation, Keyframe};
/// use glam::Vec3;
///
/// let channel = AnimationChannel {
///     joint_index: 0,
///     property: AnimationProperty::Translation,
///     interpolation: Interpolation::Linear,
///     keyframes: vec![
///         Keyframe { time: 0.0, value: [0.0, 0.0, 0.0, 0.0] },
///         Keyframe { time: 1.0, value: [1.0, 0.0, 0.0, 0.0] },
///     ],
/// };
/// assert_eq!(channel.duration(), 1.0);
/// ```
#[derive(Debug, Clone)]
pub struct AnimationChannel {
    /// 目标关节索引
    pub joint_index: usize,
    /// 目标属性
    pub property: AnimationProperty,
    /// 插值方式
    pub interpolation: Interpolation,
    /// 关键帧列表（按时间排序）
    pub keyframes: Vec<Keyframe>,
}

impl AnimationChannel {
    /// 通道时长
    pub fn duration(&self) -> f32 {
        self.keyframes.last().map(|k| k.time).unwrap_or(0.0)
    }

    /// 在指定时间采样（线性插值）
    pub fn sample(&self, time: f32) -> [f32; 4] {
        if self.keyframes.is_empty() {
            return [0.0; 4];
        }
        if time <= self.keyframes[0].time {
            return self.keyframes[0].value;
        }
        if time >= self.keyframes.last().unwrap().time {
            return self.keyframes.last().unwrap().value;
        }

        // 查找包含 time 的两个关键帧
        for i in 0..self.keyframes.len() - 1 {
            let a = &self.keyframes[i];
            let b = &self.keyframes[i + 1];
            if time >= a.time && time <= b.time {
                match self.interpolation {
                    Interpolation::Step => return a.value,
                    Interpolation::Linear => {
                        let t = (time - a.time) / (b.time - a.time);
                        return [
                            a.value[0] + (b.value[0] - a.value[0]) * t,
                            a.value[1] + (b.value[1] - a.value[1]) * t,
                            a.value[2] + (b.value[2] - a.value[2]) * t,
                            a.value[3] + (b.value[3] - a.value[3]) * t,
                        ];
                    }
                    Interpolation::CubicSpline => {
                        // glTF cubic spline: each keyframe stores [in_tangent, value, out_tangent]
                        // In our simplified model, tangents default to zero (flat tangent),
                        // producing a Hermite spline: p(t) = (2t³-3t²+1)v₀ + (t³-2t²+t)·dt·b₀
                        //                                   + (-2t³+3t²)v₁ + (t³-t²)·dt·a₁
                        // Without explicit tangent storage, use Catmull-Rom estimate:
                        let dt_seg = b.time - a.time;
                        let t = (time - a.time) / dt_seg;
                        let t2 = t * t;
                        let t3 = t2 * t;
                        // Hermite basis with zero tangents (smooth step)
                        let h00 = 2.0 * t3 - 3.0 * t2 + 1.0;
                        let h01 = -2.0 * t3 + 3.0 * t2;
                        return [
                            h00 * a.value[0] + h01 * b.value[0],
                            h00 * a.value[1] + h01 * b.value[1],
                            h00 * a.value[2] + h01 * b.value[2],
                            h00 * a.value[3] + h01 * b.value[3],
                        ];
                    }
                }
            }
        }
        self.keyframes.last().unwrap().value
    }
}

/// 单个关键帧
#[derive(Debug, Clone)]
pub struct Keyframe {
    /// 时间戳（秒）
    pub time: f32,
    /// 值（Translation: xyz+0, Rotation: xyzw, Scale: xyz+0）
    pub value: [f32; 4],
}

/// 动画剪辑
///
/// # 示例
///
/// ```rust
/// use anvilkit_assets::animation::AnimationClip;
///
/// let clip = AnimationClip {
///     name: "Walk".to_string(),
///     channels: vec![],
/// };
/// assert_eq!(clip.duration(), 0.0);
/// ```
#[derive(Debug, Clone)]
pub struct AnimationClip {
    /// 动画名称
    pub name: String,
    /// 动画通道列表
    pub channels: Vec<AnimationChannel>,
}

impl AnimationClip {
    /// 动画总时长
    pub fn duration(&self) -> f32 {
        self.channels.iter().map(|c| c.duration()).fold(0.0f32, f32::max)
    }
}

/// 动画播放器（运行时状态）
///
/// # 示例
///
/// ```rust
/// use anvilkit_assets::animation::{AnimationPlayer, AnimationClip};
///
/// let clip = AnimationClip { name: "Idle".into(), channels: vec![] };
/// let mut player = AnimationPlayer::new(clip);
/// player.playing = true;
/// player.advance(0.5);
/// assert_eq!(player.current_time, 0.5);
/// ```
#[derive(Debug, Clone)]
pub struct AnimationPlayer {
    /// 当前播放的剪辑
    pub clip: AnimationClip,
    /// 当前时间
    pub current_time: f32,
    /// 是否播放中
    pub playing: bool,
    /// 是否循环
    pub looping: bool,
    /// 播放速度（1.0 = 正常）
    pub speed: f32,
}

impl AnimationPlayer {
    pub fn new(clip: AnimationClip) -> Self {
        Self {
            clip,
            current_time: 0.0,
            playing: false,
            looping: true,
            speed: 1.0,
        }
    }

    /// 推进时间
    pub fn advance(&mut self, dt: f32) {
        if !self.playing { return; }
        self.current_time += dt * self.speed;
        let duration = self.clip.duration();
        if duration > 0.0 && self.current_time > duration {
            if self.looping {
                self.current_time %= duration;
            } else {
                self.current_time = duration;
                self.playing = false;
            }
        }
    }
}

// ---------------------------------------------------------------------------
//  Bone matrix computation for GPU skinning
// ---------------------------------------------------------------------------

/// 从骨骼层次和动画播放器计算每个关节的最终骨骼矩阵
///
/// 返回 joint_count 个矩阵，每个 = global_transform[j] * inverse_bind_matrix[j]
/// 顶点着色器中: skinned_pos = sum(weight[i] * bone_matrices[joint[i]] * pos)
pub fn compute_bone_matrices(skeleton: &Skeleton, player: &AnimationPlayer) -> Vec<Mat4> {
    let n = skeleton.joint_count();
    let mut local_transforms: Vec<Mat4> = vec![Mat4::IDENTITY; n];

    // Accumulate T/R/S per joint separately, then compose as T × R × S (glTF standard)
    let mut translations: Vec<Option<glam::Vec3>> = vec![None; n];
    let mut rotations: Vec<Option<glam::Quat>> = vec![None; n];
    let mut scales: Vec<Option<glam::Vec3>> = vec![None; n];

    for channel in &player.clip.channels {
        let idx = channel.joint_index;
        if idx >= n { continue; }
        let value = channel.sample(player.current_time);

        match channel.property {
            AnimationProperty::Translation => {
                translations[idx] = Some(glam::Vec3::new(value[0], value[1], value[2]));
            }
            AnimationProperty::Rotation => {
                rotations[idx] = Some(
                    glam::Quat::from_xyzw(value[0], value[1], value[2], value[3]).normalize()
                );
            }
            AnimationProperty::Scale => {
                scales[idx] = Some(glam::Vec3::new(value[0], value[1], value[2]));
            }
        }
    }

    // Compose per-joint: T × R × S
    for idx in 0..n {
        let t = translations[idx].unwrap_or(glam::Vec3::ZERO);
        let r = rotations[idx].unwrap_or(glam::Quat::IDENTITY);
        let s = scales[idx].unwrap_or(glam::Vec3::ONE);
        local_transforms[idx] = Mat4::from_scale_rotation_translation(s, r, t);
    }

    // Propagate through hierarchy to get global transforms
    let mut global_transforms: Vec<Mat4> = vec![Mat4::IDENTITY; n];
    for j in 0..n {
        global_transforms[j] = if let Some(parent) = skeleton.joints[j].parent {
            global_transforms[parent] * local_transforms[j]
        } else {
            local_transforms[j]
        };
    }

    // Final bone matrices = global * inverse_bind
    global_transforms
        .iter()
        .enumerate()
        .map(|(j, g)| *g * skeleton.joints[j].inverse_bind_matrix)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skeleton() {
        let skeleton = Skeleton {
            joints: vec![
                Joint { name: "Root".into(), parent: None, inverse_bind_matrix: Mat4::IDENTITY },
                Joint { name: "Spine".into(), parent: Some(0), inverse_bind_matrix: Mat4::IDENTITY },
            ],
        };
        assert_eq!(skeleton.joint_count(), 2);
        assert_eq!(skeleton.find_joint("Spine"), Some(1));
        assert_eq!(skeleton.find_joint("Unknown"), None);
    }

    #[test]
    fn test_channel_sample() {
        let channel = AnimationChannel {
            joint_index: 0,
            property: AnimationProperty::Translation,
            interpolation: Interpolation::Linear,
            keyframes: vec![
                Keyframe { time: 0.0, value: [0.0, 0.0, 0.0, 0.0] },
                Keyframe { time: 1.0, value: [2.0, 0.0, 0.0, 0.0] },
            ],
        };
        let v = channel.sample(0.5);
        assert!((v[0] - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_animation_player_loop() {
        let clip = AnimationClip {
            name: "Test".into(),
            channels: vec![AnimationChannel {
                joint_index: 0,
                property: AnimationProperty::Translation,
                interpolation: Interpolation::Linear,
                keyframes: vec![
                    Keyframe { time: 0.0, value: [0.0; 4] },
                    Keyframe { time: 2.0, value: [1.0; 4] },
                ],
            }],
        };
        let mut player = AnimationPlayer::new(clip);
        player.playing = true;
        player.looping = true;
        player.advance(2.5);
        assert!((player.current_time - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_compute_bone_matrices_identity() {
        let skeleton = Skeleton {
            joints: vec![
                Joint { name: "Root".into(), parent: None, inverse_bind_matrix: Mat4::IDENTITY },
            ],
        };
        let clip = AnimationClip { name: "Empty".into(), channels: vec![] };
        let player = AnimationPlayer::new(clip);

        let matrices = compute_bone_matrices(&skeleton, &player);
        assert_eq!(matrices.len(), 1);
        // Identity local * Identity inverse_bind = Identity
        let diff = (matrices[0] - Mat4::IDENTITY).abs_diff_eq(Mat4::ZERO, 0.001);
        assert!(diff);
    }

    #[test]
    fn test_compute_bone_matrices_translation() {
        let skeleton = Skeleton {
            joints: vec![
                Joint { name: "Root".into(), parent: None, inverse_bind_matrix: Mat4::IDENTITY },
            ],
        };
        let clip = AnimationClip {
            name: "Move".into(),
            channels: vec![AnimationChannel {
                joint_index: 0,
                property: AnimationProperty::Translation,
                interpolation: Interpolation::Linear,
                keyframes: vec![
                    Keyframe { time: 0.0, value: [0.0, 0.0, 0.0, 0.0] },
                    Keyframe { time: 1.0, value: [2.0, 0.0, 0.0, 0.0] },
                ],
            }],
        };
        let mut player = AnimationPlayer::new(clip);
        player.current_time = 0.5;

        let matrices = compute_bone_matrices(&skeleton, &player);
        // At t=0.5, translation should be (1, 0, 0)
        let col3 = matrices[0].col(3);
        assert!((col3.x - 1.0).abs() < 0.01);
    }
}
