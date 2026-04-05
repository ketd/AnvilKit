//! Scene editing components and serialization for MCP-driven map editing.
//!
//! These ECS components represent editable scene objects that agents
//! can spawn, modify, and persist via MCP tools.

use bevy_ecs::prelude::*;
use serde::{Serialize, Deserialize};

/// A scene object placed by the agent via MCP.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct SceneObject {
    /// Object shape.
    pub shape: Shape,
    /// RGBA color.
    pub color: [f32; 4],
    /// Object name (optional, for querying).
    pub name: String,
}

/// Shape of a scene object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Shape {
    Cube { size: [f32; 3] },
    Sphere { radius: f32 },
    Ground { size: f32 },
}

/// Position/rotation/scale of a scene object.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct SceneTransform {
    pub translation: [f32; 3],
    pub rotation: [f32; 3],  // euler angles in degrees
    pub scale: [f32; 3],
}

impl Default for SceneTransform {
    fn default() -> Self {
        Self {
            translation: [0.0; 3],
            rotation: [0.0; 3],
            scale: [1.0, 1.0, 1.0],
        }
    }
}

/// A serializable scene (list of objects with transforms).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedScene {
    pub objects: Vec<SerializedObject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedObject {
    pub object: SceneObject,
    pub transform: SceneTransform,
}

impl SerializedScene {
    /// Save scene to RON file.
    pub fn save(&self, path: &str) -> Result<(), String> {
        let ron = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
            .map_err(|e| format!("RON serialize error: {}", e))?;
        if let Some(parent) = std::path::Path::new(path).parent() {
            std::fs::create_dir_all(parent).ok();
        }
        std::fs::write(path, &ron).map_err(|e| format!("Write error: {}", e))?;
        Ok(())
    }

    /// Load scene from RON file.
    pub fn load(path: &str) -> Result<Self, String> {
        let data = std::fs::read_to_string(path)
            .map_err(|e| format!("Read error: {}", e))?;
        ron::from_str(&data)
            .map_err(|e| format!("RON parse error: {}", e))
    }

    /// Collect all SceneObject entities from the world into a SerializedScene.
    pub fn from_world(world: &mut World) -> Self {
        let mut objects = Vec::new();
        let mut query = world.query::<(&SceneObject, &SceneTransform)>();
        for (obj, transform) in query.iter(world) {
            objects.push(SerializedObject {
                object: obj.clone(),
                transform: transform.clone(),
            });
        }
        SerializedScene { objects }
    }

    /// Spawn all objects into the world, returning entity IDs.
    pub fn spawn_into(&self, world: &mut World) -> Vec<Entity> {
        self.objects.iter().map(|so| {
            world.spawn((so.object.clone(), so.transform.clone())).id()
        }).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scene_roundtrip() {
        let scene = SerializedScene {
            objects: vec![
                SerializedObject {
                    object: SceneObject {
                        shape: Shape::Cube { size: [1.0, 1.0, 1.0] },
                        color: [1.0, 0.0, 0.0, 1.0],
                        name: "red_cube".into(),
                    },
                    transform: SceneTransform {
                        translation: [5.0, 0.0, 3.0],
                        ..Default::default()
                    },
                },
            ],
        };

        let ron = ron::ser::to_string(&scene).unwrap();
        let loaded: SerializedScene = ron::from_str(&ron).unwrap();
        assert_eq!(loaded.objects.len(), 1);
        assert_eq!(loaded.objects[0].object.name, "red_cube");
    }

    #[test]
    fn test_spawn_and_collect() {
        let mut world = World::new();
        let scene = SerializedScene {
            objects: vec![
                SerializedObject {
                    object: SceneObject {
                        shape: Shape::Sphere { radius: 2.0 },
                        color: [0.0, 0.0, 1.0, 1.0],
                        name: "blue_ball".into(),
                    },
                    transform: SceneTransform::default(),
                },
            ],
        };

        let ids = scene.spawn_into(&mut world);
        assert_eq!(ids.len(), 1);

        let collected = SerializedScene::from_world(&mut world);
        assert_eq!(collected.objects.len(), 1);
        assert_eq!(collected.objects[0].object.name, "blue_ball");
    }
}
