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
                    Interpolation::Linear | Interpolation::CubicSpline => {
                        let t = (time - a.time) / (b.time - a.time);
                        return [
                            a.value[0] + (b.value[0] - a.value[0]) * t,
                            a.value[1] + (b.value[1] - a.value[1]) * t,
                            a.value[2] + (b.value[2] - a.value[2]) * t,
                            a.value[3] + (b.value[3] - a.value[3]) * t,
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
}
