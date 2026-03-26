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
    /// 实体名称（来自 Name 组件）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// 实体标签（来自 Tag 组件）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    /// 父实体索引（在 entities 数组中的位置，用于重建层级）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_index: Option<usize>,
    /// 自定义键值数据（用户扩展）
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub custom_data: std::collections::HashMap<String, String>,
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

/// 可序列化组件注册表
///
/// 允许用户注册自定义组件类型用于场景序列化。
/// 注册后，`SceneSerializer` 可以通过 type name 查找组件的序列化/反序列化函数。
///
/// # 示例
///
/// ```rust,ignore
/// use anvilkit_ecs::scene::SerializableRegistry;
///
/// let mut registry = SerializableRegistry::new();
/// registry.register::<MyComponent>("MyComponent");
/// ```
#[derive(Resource, Default)]
pub struct SerializableRegistry {
    /// type_name → type_id 映射
    entries: std::collections::HashMap<String, std::any::TypeId>,
}

impl SerializableRegistry {
    /// 创建空的注册表
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册组件类型
    ///
    /// # 参数
    ///
    /// - `name`: 序列化使用的类型名称（应稳定、跨版本一致）
    pub fn register<T: 'static>(&mut self, name: &str) {
        self.entries.insert(name.to_string(), std::any::TypeId::of::<T>());
    }

    /// 检查类型是否已注册
    pub fn is_registered(&self, name: &str) -> bool {
        self.entries.contains_key(name)
    }

    /// 已注册类型数量
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 注册表是否为空
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// 获取已注册的所有类型名称
    pub fn registered_names(&self) -> Vec<&str> {
        self.entries.keys().map(|s| s.as_str()).collect()
    }
}

#[cfg(feature = "serde")]
impl SceneSerializer {
    /// 保存场景到 RON 文件。
    ///
    /// 只包含具有 `Serializable` 组件的实体。
    ///
    /// Note: Takes `&mut World` because bevy_ecs query initialization requires it,
    /// but this function is semantically read-only.
    pub fn save(world: &mut World, path: &str) -> Result<usize, String> {
        use crate::component::{Name, Tag};
        use crate::transform::Parent;

        // 先收集所有 Serializable 实体的 Entity ID，建立 entity → index 映射
        let mut entity_ids: Vec<Entity> = Vec::new();
        {
            let mut query = world.query_filtered::<Entity, With<Serializable>>();
            for entity in query.iter(world) {
                entity_ids.push(entity);
            }
        }

        let entity_to_index: std::collections::HashMap<Entity, usize> = entity_ids.iter()
            .enumerate()
            .map(|(i, &e)| (e, i))
            .collect();

        let mut entities = Vec::new();
        for &entity in &entity_ids {
            let transform = world.get::<Transform>(entity).map(TransformData::from);
            let name = world.get::<Name>(entity).map(|n| n.as_str().to_string());
            let tag = world.get::<Tag>(entity).map(|t| t.as_str().to_string());
            let parent_index = world.get::<Parent>(entity)
                .and_then(|p| entity_to_index.get(&p.get()).copied());

            entities.push(SerializedEntity {
                transform,
                name,
                tag,
                parent_index,
                custom_data: std::collections::HashMap::new(),
            });
        }

        let scene = SerializedScene {
            version: 2,
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
    /// 为每个序列化实体创建新实体，恢复 Transform、Name、Tag、层级关系。
    pub fn load(world: &mut World, path: &str) -> Result<usize, String> {
        use crate::component::{Name, Tag};
        use crate::transform::{Parent, Children};

        let data = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path, e))?;

        let scene: SerializedScene = ron::from_str(&data)
            .map_err(|e| format!("RON deserialization failed: {}", e))?;

        let count = scene.entities.len();
        let mut spawned_entities: Vec<Entity> = Vec::with_capacity(count);

        // 第一遍：创建所有实体
        for entity_data in &scene.entities {
            let transform = entity_data.transform
                .as_ref()
                .map(|t| t.to_transform())
                .unwrap_or_default();

            let mut entity_cmd = world.spawn((
                transform,
                GlobalTransform::from(transform),
                Serializable,
            ));

            if let Some(ref name) = entity_data.name {
                entity_cmd.insert(Name::new(name.as_str()));
            }
            if let Some(ref tag) = entity_data.tag {
                entity_cmd.insert(Tag::new(tag.as_str()));
            }

            spawned_entities.push(entity_cmd.id());
        }

        // 第二遍：恢复层级关系
        for (i, entity_data) in scene.entities.iter().enumerate() {
            if let Some(parent_idx) = entity_data.parent_index {
                if parent_idx < spawned_entities.len() {
                    let child = spawned_entities[i];
                    let parent = spawned_entities[parent_idx];
                    world.entity_mut(child).insert(Parent::new(parent));
                    if let Some(mut children) = world.get_mut::<Children>(parent) {
                        children.push(child);
                    } else {
                        let mut c = Children::empty();
                        c.push(child);
                        world.entity_mut(parent).insert(c);
                    }
                }
            }
        }

        Ok(count)
    }
}

#[cfg(all(test, feature = "serde"))]
mod tests {
    use super::*;
    use crate::prelude::*;
    use crate::component::{Name, Tag};

    #[test]
    fn test_scene_round_trip() {
        let mut world = World::new();

        // Create entities with various components
        let parent = world.spawn((
            Transform::from_xyz(1.0, 2.0, 3.0),
            GlobalTransform::default(),
            Name::new("Parent"),
            Serializable,
        )).id();

        let child = world.spawn((
            Transform::from_xyz(4.0, 5.0, 6.0),
            GlobalTransform::default(),
            Name::new("Child"),
            Tag::new("enemy"),
            crate::transform::Parent::new(parent),
            Serializable,
        )).id();

        world.entity_mut(parent).insert(crate::transform::Children::new(vec![child]));

        // Save
        let path = "/tmp/anvilkit_test_scene.ron";
        let count = SceneSerializer::save(&mut world, path).unwrap();
        assert_eq!(count, 2);

        // Load into new world
        let mut new_world = World::new();
        let loaded = SceneSerializer::load(&mut new_world, path).unwrap();
        assert_eq!(loaded, 2);

        // Cleanup
        let _ = std::fs::remove_file(path);
    }
}

#[cfg(test)]
mod registry_tests {
    use super::*;

    #[test]
    fn test_serializable_registry() {
        let mut registry = SerializableRegistry::new();
        assert!(registry.is_empty());

        #[derive(Debug)]
        struct FakeComponent;

        registry.register::<FakeComponent>("FakeComponent");
        assert_eq!(registry.len(), 1);
        assert!(registry.is_registered("FakeComponent"));
        assert!(!registry.is_registered("Unknown"));
    }

    #[test]
    fn test_serializable_registry_names() {
        let mut registry = SerializableRegistry::new();
        registry.register::<u32>("u32");
        registry.register::<String>("String");

        let names = registry.registered_names();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"u32"));
        assert!(names.contains(&"String"));
    }
}
