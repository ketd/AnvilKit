//! # 场景序列化
//!
//! 基于 serde + RON 的 ECS 场景保存和加载。
//! 只序列化标记了 `Serializable` 组件的实体。

#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};

use bevy_ecs::prelude::*;

#[cfg(feature = "serde")]
use anvilkit_core::math::{Transform, GlobalTransform};

/// 标记组件：标记实体应被场景序列化器包含。
///
/// 只有带此组件的实体会被 `SceneSerializer::save` 写入。
///
/// # 示例
///
/// ```rust
/// use anvilkit_ecs::prelude::*;
/// use anvilkit_ecs::scene::Serializable;
///
/// let mut world = World::new();
/// world.spawn((Transform::default(), GlobalTransform::default(), Serializable));
/// ```
#[derive(Component, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Serializable;

/// 序列化后的单个实体数据。
#[cfg(feature = "serde")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedEntity {
    /// Transform (local).
    pub transform: Option<TransformData>,
}

/// Transform 的序列化友好表示。
#[cfg(feature = "serde")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformData {
    /// Translation [x, y, z].
    pub translation: [f32; 3],
    /// Rotation quaternion [x, y, z, w].
    pub rotation: [f32; 4],
    /// Scale [x, y, z].
    pub scale: [f32; 3],
}

#[cfg(feature = "serde")]
impl From<&Transform> for TransformData {
    fn from(t: &Transform) -> Self {
        Self {
            translation: t.translation.to_array(),
            rotation: [t.rotation.x, t.rotation.y, t.rotation.z, t.rotation.w],
            scale: t.scale.to_array(),
        }
    }
}

#[cfg(feature = "serde")]
impl TransformData {
    /// 转换回 Transform。
    pub fn to_transform(&self) -> Transform {
        Transform {
            translation: glam::Vec3::from_array(self.translation),
            rotation: glam::Quat::from_xyzw(
                self.rotation[0], self.rotation[1],
                self.rotation[2], self.rotation[3],
            ),
            scale: glam::Vec3::from_array(self.scale),
        }
    }
}

/// 序列化后的场景。
#[cfg(feature = "serde")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedScene {
    /// Scene format version.
    pub version: u32,
    /// All serialized entities.
    pub entities: Vec<SerializedEntity>,
}

/// 场景序列化器。
///
/// 将 ECS World 中标记了 `Serializable` 的实体保存为 RON 文件，
/// 或从 RON 文件加载实体到 World。
///
/// # 示例
///
/// ```rust,ignore
/// use anvilkit_ecs::prelude::*;
/// use anvilkit_ecs::scene::{SceneSerializer, Serializable};
///
/// let mut world = World::new();
/// world.spawn((Transform::default(), GlobalTransform::default(), Serializable));
///
/// // Save (requires "serde" feature)
/// SceneSerializer::save(&mut world, "scene.ron").unwrap();
///
/// // Load into a new world
/// let mut new_world = World::new();
/// SceneSerializer::load(&mut new_world, "scene.ron").unwrap();
/// ```
pub struct SceneSerializer;

#[cfg(feature = "serde")]
impl SceneSerializer {
    /// 保存场景到 RON 文件。
    ///
    /// 只包含具有 `Serializable` 组件的实体。
    ///
    /// Note: Takes `&mut World` because bevy_ecs query initialization requires it,
    /// but this function is semantically read-only.
    pub fn save(world: &mut World, path: &str) -> Result<usize, String> {
        let mut query = world.query_filtered::<Option<&Transform>, With<Serializable>>();
        let mut entities = Vec::new();

        for transform in query.iter(world) {
            entities.push(SerializedEntity {
                transform: transform.map(TransformData::from),
            });
        }

        let scene = SerializedScene {
            version: 1,
            entities,
        };

        let count = scene.entities.len();
        let ron_str = ron::ser::to_string_pretty(&scene, ron::ser::PrettyConfig::default())
            .map_err(|e| format!("RON serialization failed: {}", e))?;

        std::fs::write(path, ron_str)
            .map_err(|e| format!("Failed to write {}: {}", path, e))?;

        Ok(count)
    }

    /// 从 RON 文件加载场景到 World。
    ///
    /// 为每个序列化实体创建新实体，带 `Transform`、`GlobalTransform` 和 `Serializable`。
    pub fn load(world: &mut World, path: &str) -> Result<usize, String> {
        let data = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path, e))?;

        let scene: SerializedScene = ron::from_str(&data)
            .map_err(|e| format!("RON deserialization failed: {}", e))?;

        let count = scene.entities.len();
        for entity_data in &scene.entities {
            let transform = entity_data.transform
                .as_ref()
                .map(|t| t.to_transform())
                .unwrap_or_default();

            world.spawn((
                transform,
                GlobalTransform::from(transform),
                Serializable,
            ));
        }

        Ok(count)
    }
}
