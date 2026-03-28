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

/// 可序列化组件的注册表条目，存储类型信息和序列化/反序列化函数指针。
#[cfg(feature = "serde")]
pub struct SerializableEntry {
    /// 组件的 TypeId
    pub type_id: std::any::TypeId,
    /// 序列化函数：从 World 读取实体的组件并返回 RON 字符串
    pub serialize_fn: fn(&World, Entity) -> Option<String>,
    /// 反序列化函数：从 RON 字符串恢复组件并插入到实体
    pub deserialize_fn: fn(&mut World, Entity, &str),
}

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
    /// type_name → SerializableEntry (with serde) or TypeId (without serde)
    #[cfg(feature = "serde")]
    entries: std::collections::HashMap<String, SerializableEntry>,
    #[cfg(not(feature = "serde"))]
    entries: std::collections::HashMap<String, std::any::TypeId>,
}

impl SerializableRegistry {
    /// 创建空的注册表
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册组件类型（需要 `serde` feature）
    ///
    /// 注册组件的序列化和反序列化函数，使 `SceneSerializer` 能自动保存/加载该组件。
    /// 组件必须实现 `Component + Serialize + DeserializeOwned`。
    ///
    /// # 参数
    ///
    /// - `name`: 序列化使用的类型名称（应稳定、跨版本一致）
    #[cfg(feature = "serde")]
    pub fn register<T: Component + serde::Serialize + serde::de::DeserializeOwned>(
        &mut self,
        name: &str,
    ) {
        let entry = SerializableEntry {
            type_id: std::any::TypeId::of::<T>(),
            serialize_fn: |world, entity| {
                world.get::<T>(entity)
                    .and_then(|comp| ron::to_string(comp).ok())
            },
            deserialize_fn: |world, entity, data| {
                if let Ok(comp) = ron::from_str::<T>(data) {
                    world.entity_mut(entity).insert(comp);
                }
            },
        };
        self.entries.insert(name.to_string(), entry);
    }

    /// 注册组件类型（无 serde feature 时的简化版本，仅记录 TypeId）
    #[cfg(not(feature = "serde"))]
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

    /// 获取指定名称的注册条目
    #[cfg(feature = "serde")]
    pub fn get(&self, name: &str) -> Option<&SerializableEntry> {
        self.entries.get(name)
    }

    /// 迭代所有注册条目
    #[cfg(feature = "serde")]
    pub fn iter(&self) -> impl Iterator<Item = (&str, &SerializableEntry)> {
        self.entries.iter().map(|(k, v)| (k.as_str(), v))
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

        // Populate custom_data from SerializableRegistry
        // Collect registry entries first to avoid borrow conflicts with world
        let registry_entries: Vec<(String, fn(&World, Entity) -> Option<String>)> = world
            .get_resource::<SerializableRegistry>()
            .map(|registry| {
                registry.iter()
                    .map(|(name, entry)| (name.to_string(), entry.serialize_fn))
                    .collect()
            })
            .unwrap_or_default();

        if !registry_entries.is_empty() {
            for (idx, &entity) in entity_ids.iter().enumerate() {
                for (name, serialize_fn) in &registry_entries {
                    if let Some(data) = serialize_fn(world, entity) {
                        entities[idx].custom_data.insert(name.clone(), data);
                    }
                }
            }
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

        // 第三遍：恢复自定义组件（通过 SerializableRegistry）
        // Collect deserialize fns first to avoid borrow conflicts with world
        let registry_deserialize: std::collections::HashMap<String, fn(&mut World, Entity, &str)> =
            world.get_resource::<SerializableRegistry>()
                .map(|registry| {
                    registry.iter()
                        .map(|(name, entry)| (name.to_string(), entry.deserialize_fn))
                        .collect()
                })
                .unwrap_or_default();

        if !registry_deserialize.is_empty() {
            for (entity, entity_data) in spawned_entities.iter().zip(scene.entities.iter()) {
                for (name, data) in &entity_data.custom_data {
                    if let Some(deserialize_fn) = registry_deserialize.get(name.as_str()) {
                        deserialize_fn(world, *entity, data);
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

    #[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct Health {
        hp: f32,
    }

    #[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct Mana {
        mp: f32,
        max_mp: f32,
    }

    #[test]
    fn test_custom_component_round_trip() {
        let mut world = World::new();

        // Register custom components
        let mut registry = SerializableRegistry::new();
        registry.register::<Health>("Health");
        registry.register::<Mana>("Mana");
        world.insert_resource(registry);

        // Spawn entity with Serializable + custom components
        world.spawn((
            Transform::from_xyz(1.0, 2.0, 3.0),
            GlobalTransform::default(),
            Name::new("Hero"),
            Serializable,
            Health { hp: 100.0 },
            Mana { mp: 50.0, max_mp: 80.0 },
        ));

        // Spawn another entity with only Health (no Mana)
        world.spawn((
            Transform::from_xyz(4.0, 5.0, 6.0),
            GlobalTransform::default(),
            Name::new("Warrior"),
            Serializable,
            Health { hp: 200.0 },
        ));

        // Save
        let path = "/tmp/anvilkit_test_custom_round_trip.ron";
        let count = SceneSerializer::save(&mut world, path).unwrap();
        assert_eq!(count, 2);

        // Load into new world (with same registry)
        let mut new_world = World::new();
        let mut new_registry = SerializableRegistry::new();
        new_registry.register::<Health>("Health");
        new_registry.register::<Mana>("Mana");
        new_world.insert_resource(new_registry);

        let loaded = SceneSerializer::load(&mut new_world, path).unwrap();
        assert_eq!(loaded, 2);

        // Verify Health components are restored
        let mut health_query = new_world.query::<(&Name, &Health)>();
        let mut found_hero = false;
        let mut found_warrior = false;
        for (name, health) in health_query.iter(&new_world) {
            match name.as_str() {
                "Hero" => {
                    assert_eq!(health.hp, 100.0);
                    found_hero = true;
                }
                "Warrior" => {
                    assert_eq!(health.hp, 200.0);
                    found_warrior = true;
                }
                _ => panic!("Unexpected entity name: {}", name.as_str()),
            }
        }
        assert!(found_hero, "Hero entity not found after load");
        assert!(found_warrior, "Warrior entity not found after load");

        // Verify Mana component only on Hero
        let mut mana_query = new_world.query::<(&Name, &Mana)>();
        let mana_results: Vec<_> = mana_query.iter(&new_world).collect();
        assert_eq!(mana_results.len(), 1);
        assert_eq!(mana_results[0].0.as_str(), "Hero");
        assert_eq!(mana_results[0].1.mp, 50.0);
        assert_eq!(mana_results[0].1.max_mp, 80.0);

        // Cleanup
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_load_without_registry_ignores_custom_data() {
        // Save with registry
        let mut world = World::new();
        let mut registry = SerializableRegistry::new();
        registry.register::<Health>("Health");
        world.insert_resource(registry);

        world.spawn((
            Transform::default(),
            GlobalTransform::default(),
            Serializable,
            Health { hp: 42.0 },
        ));

        let path = "/tmp/anvilkit_test_no_registry_load.ron";
        SceneSerializer::save(&mut world, path).unwrap();

        // Load into world WITHOUT registry — should still work, just no Health
        let mut new_world = World::new();
        let loaded = SceneSerializer::load(&mut new_world, path).unwrap();
        assert_eq!(loaded, 1);

        // Verify entity exists but has no Health
        let mut query = new_world.query::<(Entity, Option<&Health>)>();
        let results: Vec<_> = query.iter(&new_world).collect();
        assert_eq!(results.len(), 1);
        assert!(results[0].1.is_none(), "Health should not be restored without registry");

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_load_legacy_file_without_custom_data() {
        // Create a RON file that has no custom_data (backward compatibility)
        let scene = SerializedScene {
            version: 2,
            entities: vec![
                SerializedEntity {
                    transform: Some(TransformData {
                        translation: [1.0, 2.0, 3.0],
                        rotation: [0.0, 0.0, 0.0, 1.0],
                        scale: [1.0, 1.0, 1.0],
                    }),
                    name: Some("Legacy".to_string()),
                    tag: None,
                    parent_index: None,
                    custom_data: std::collections::HashMap::new(),
                },
            ],
        };

        let path = "/tmp/anvilkit_test_legacy_load.ron";
        let ron_str = ron::ser::to_string_pretty(&scene, ron::ser::PrettyConfig::default()).unwrap();
        std::fs::write(path, ron_str).unwrap();

        // Load with registry — should work fine
        let mut world = World::new();
        let mut registry = SerializableRegistry::new();
        registry.register::<Health>("Health");
        world.insert_resource(registry);

        let loaded = SceneSerializer::load(&mut world, path).unwrap();
        assert_eq!(loaded, 1);

        // Verify entity loaded but no Health (since file had no custom_data)
        let mut query = world.query::<(&Name, Option<&Health>)>();
        let results: Vec<_> = query.iter(&world).collect();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0.as_str(), "Legacy");
        assert!(results[0].1.is_none());

        let _ = std::fs::remove_file(path);
    }
}

#[cfg(all(test, not(feature = "serde")))]
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

#[cfg(all(test, feature = "serde"))]
mod registry_serde_tests {
    use super::*;

    #[derive(Component, Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct FakeComponent {
        value: i32,
    }

    #[test]
    fn test_serializable_registry_with_serde() {
        let mut registry = SerializableRegistry::new();
        assert!(registry.is_empty());

        registry.register::<FakeComponent>("FakeComponent");
        assert_eq!(registry.len(), 1);
        assert!(registry.is_registered("FakeComponent"));
        assert!(!registry.is_registered("Unknown"));
    }

    #[test]
    fn test_serializable_registry_names_with_serde() {
        let mut registry = SerializableRegistry::new();
        registry.register::<FakeComponent>("FakeComponent");

        let names = registry.registered_names();
        assert_eq!(names.len(), 1);
        assert!(names.contains(&"FakeComponent"));
    }

    #[test]
    fn test_registry_get_and_iter() {
        let mut registry = SerializableRegistry::new();
        registry.register::<FakeComponent>("FakeComponent");

        assert!(registry.get("FakeComponent").is_some());
        assert!(registry.get("Unknown").is_none());

        let entries: Vec<_> = registry.iter().collect();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, "FakeComponent");
    }

    #[test]
    fn test_serialize_fn_round_trip() {
        let mut world = World::new();
        let entity = world.spawn(FakeComponent { value: 42 }).id();

        let mut registry = SerializableRegistry::new();
        registry.register::<FakeComponent>("FakeComponent");

        let entry = registry.get("FakeComponent").unwrap();

        // Serialize
        let data = (entry.serialize_fn)(&world, entity);
        assert!(data.is_some());

        // Deserialize into a new entity
        let new_entity = world.spawn_empty().id();
        (entry.deserialize_fn)(&mut world, new_entity, &data.unwrap());

        let restored = world.get::<FakeComponent>(new_entity).unwrap();
        assert_eq!(restored, &FakeComponent { value: 42 });
    }
}
