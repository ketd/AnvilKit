//! Built-in MCP tools for AnvilKit engine introspection.

use serde_json::{Value, json};
use crate::{Tool, ToolResult, ToolError};

/// Tool: list all entities in the world.
///
/// Returns entity IDs as a JSON array.
pub struct ListEntitiesTool;

impl Tool for ListEntitiesTool {
    fn name(&self) -> &str { "list_entities" }

    fn description(&self) -> &str {
        "List all entity IDs currently in the ECS world"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {},
            "required": []
        })
    }

    fn execute(&self, _params: Value, world: &mut bevy_ecs::world::World) -> ToolResult {
        let entities: Vec<u64> = world.iter_entities()
            .map(|e| e.id().to_bits())
            .collect();
        Ok(json!({ "entities": entities, "count": entities.len() }))
    }
}

/// Tool: count entities matching a simple filter.
///
/// Parameters: `{}` (future: component filter spec)
pub struct EntityCountTool;

impl Tool for EntityCountTool {
    fn name(&self) -> &str { "entity_count" }

    fn description(&self) -> &str {
        "Count the total number of entities in the world"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {},
            "required": []
        })
    }

    fn execute(&self, _params: Value, world: &mut bevy_ecs::world::World) -> ToolResult {
        let count = world.iter_entities().count();
        Ok(json!({ "count": count }))
    }
}

/// Tool: engine info (name, version).
pub struct EngineInfoTool;

impl Tool for EngineInfoTool {
    fn name(&self) -> &str { "engine_info" }

    fn description(&self) -> &str {
        "Get engine name, version, and build information"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {},
            "required": []
        })
    }

    fn execute(&self, _params: Value, _world: &mut bevy_ecs::world::World) -> ToolResult {
        Ok(json!({
            "name": "AnvilKit",
            "version": env!("CARGO_PKG_VERSION"),
            "description": "AI-first Rust game engine"
        }))
    }
}

/// Tool: spawn an empty entity and return its ID.
///
/// Parameters: `{}` (future: component spec)
pub struct SpawnEmptyEntityTool;

impl Tool for SpawnEmptyEntityTool {
    fn name(&self) -> &str { "spawn_empty_entity" }

    fn description(&self) -> &str {
        "Spawn a new empty entity with no components and return its ID"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {},
            "required": []
        })
    }

    fn execute(&self, _params: Value, world: &mut bevy_ecs::world::World) -> ToolResult {
        let entity = world.spawn_empty().id();
        Ok(json!({ "entity_id": entity.to_bits() }))
    }
}

/// Tool: despawn an entity by ID.
///
/// Parameters: `{"entity_id": u64}`
pub struct DespawnEntityTool;

impl Tool for DespawnEntityTool {
    fn name(&self) -> &str { "despawn_entity" }

    fn description(&self) -> &str {
        "Remove an entity from the world by its ID"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "entity_id": {
                    "type": "integer",
                    "description": "The entity ID to despawn"
                }
            },
            "required": ["entity_id"]
        })
    }

    fn execute(&self, params: Value, world: &mut bevy_ecs::world::World) -> ToolResult {
        let entity_id = params.get("entity_id")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| ToolError::new("MISSING_PARAM", "entity_id is required")
                .with_hint("Pass { \"entity_id\": <u64> } in params"))?;

        let entity = bevy_ecs::entity::Entity::from_bits(entity_id);
        if world.despawn(entity) {
            Ok(json!({ "despawned": true, "entity_id": entity_id }))
        } else {
            Err(ToolError::new("ENTITY_NOT_FOUND", format!("no entity with id {}", entity_id))
                .with_hint("Call list_entities to see valid IDs"))
        }
    }
}

/// Tool: high-level world summary.
///
/// Returns entity count, resource count, and archetype count.
pub struct WorldSummaryTool;

impl Tool for WorldSummaryTool {
    fn name(&self) -> &str { "get_world_summary" }

    fn description(&self) -> &str {
        "Get a high-level summary of the ECS world: entity count, resource count, and archetype count"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {},
            "required": []
        })
    }

    fn execute(&self, _params: Value, world: &mut bevy_ecs::world::World) -> ToolResult {
        let entity_count = world.iter_entities().count();
        let resource_count = world.iter_resources().count();
        let archetype_count = world.archetypes().len();

        Ok(json!({
            "entity_count": entity_count,
            "resource_count": resource_count,
            "archetype_count": archetype_count
        }))
    }
}

/// Tool: list all registered schedule names.
///
/// Reads the `Schedules` resource and returns the debug-formatted label of each schedule.
pub struct ListSchedulesTool;

impl Tool for ListSchedulesTool {
    fn name(&self) -> &str { "list_schedules" }

    fn description(&self) -> &str {
        "List the names of all registered schedules in the ECS world"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {},
            "required": []
        })
    }

    fn execute(&self, _params: Value, world: &mut bevy_ecs::world::World) -> ToolResult {
        use bevy_ecs::schedule::Schedules;

        let schedules: Vec<String> = match world.get_resource::<Schedules>() {
            Some(s) => s.iter().map(|(label, _)| format!("{label:?}")).collect(),
            None => Vec::new(),
        };

        Ok(json!({
            "schedules": schedules,
            "count": schedules.len()
        }))
    }
}

/// Tool: list component type names attached to an entity.
///
/// Parameters: `{"entity_id": u64}`
pub struct GetComponentIdsTool;

impl Tool for GetComponentIdsTool {
    fn name(&self) -> &str { "get_component_ids" }

    fn description(&self) -> &str {
        "List the component type names attached to a given entity"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "entity_id": {
                    "type": "integer",
                    "description": "The entity ID to inspect"
                }
            },
            "required": ["entity_id"]
        })
    }

    fn execute(&self, params: Value, world: &mut bevy_ecs::world::World) -> ToolResult {
        let entity_id = params.get("entity_id")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| ToolError::new("MISSING_PARAM", "entity_id is required")
                .with_hint("Pass { \"entity_id\": <u64> } in params"))?;

        let entity = bevy_ecs::entity::Entity::from_bits(entity_id);

        // Check that the entity exists before calling inspect_entity (which panics).
        if world.entities().get(entity).is_none() {
            return Err(ToolError::new(
                "ENTITY_NOT_FOUND",
                format!("no entity with id {entity_id}"),
            ).with_hint("Call list_entities to see valid IDs"));
        }

        let components: Vec<String> = world
            .inspect_entity(entity)
            .map(|info| info.name().to_string())
            .collect();

        Ok(json!({
            "entity_id": entity_id,
            "components": components,
            "count": components.len()
        }))
    }
}

// ==================== Input Control Tools ====================

/// Tool: simulate a key press.
pub struct PressKeyTool;

impl Tool for PressKeyTool {
    fn name(&self) -> &str { "press_key" }
    fn description(&self) -> &str { "Simulate pressing a keyboard key (key stays held until release_key)" }
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": { "key": { "type": "string", "description": "Key name: Space, A-Z, 0-9, Left, Right, Up, Down, Escape, Enter, etc." } },
            "required": ["key"]
        })
    }
    fn execute(&self, params: Value, world: &mut bevy_ecs::world::World) -> ToolResult {
        let key_name = params.get("key").and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::new("MISSING_PARAM", "key is required"))?;
        let key = anvilkit_input::prelude::KeyCode::from_name(key_name)
            .ok_or_else(|| ToolError::new("INVALID_KEY", format!("Unknown key: {}", key_name))
                .with_hint("Valid keys: Space, A-Z, 0-9, Left, Right, Up, Down, Escape, Enter, Tab, LShift, RShift"))?;
        if let Some(mut input) = world.get_resource_mut::<anvilkit_input::prelude::InputState>() {
            input.press_key(key);
        }
        Ok(json!({ "pressed": key_name }))
    }
}

/// Tool: simulate releasing a key.
pub struct ReleaseKeyTool;

impl Tool for ReleaseKeyTool {
    fn name(&self) -> &str { "release_key" }
    fn description(&self) -> &str { "Release a previously pressed key" }
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": { "key": { "type": "string", "description": "Key name to release" } },
            "required": ["key"]
        })
    }
    fn execute(&self, params: Value, world: &mut bevy_ecs::world::World) -> ToolResult {
        let key_name = params.get("key").and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::new("MISSING_PARAM", "key is required"))?;
        let key = anvilkit_input::prelude::KeyCode::from_name(key_name)
            .ok_or_else(|| ToolError::new("INVALID_KEY", format!("Unknown key: {}", key_name)))?;
        if let Some(mut input) = world.get_resource_mut::<anvilkit_input::prelude::InputState>() {
            input.release_key(key);
        }
        Ok(json!({ "released": key_name }))
    }
}

/// Tool: simulate a mouse click at screen coordinates.
pub struct ClickTool;

impl Tool for ClickTool {
    fn name(&self) -> &str { "click" }
    fn description(&self) -> &str { "Simulate a mouse click at screen coordinates (x, y)" }
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "x": { "type": "number", "description": "X coordinate in pixels" },
                "y": { "type": "number", "description": "Y coordinate in pixels" },
                "button": { "type": "string", "description": "Mouse button: Left, Right, Middle (default: Left)" }
            },
            "required": ["x", "y"]
        })
    }
    fn execute(&self, params: Value, world: &mut bevy_ecs::world::World) -> ToolResult {
        let x = params.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
        let y = params.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
        let btn_name = params.get("button").and_then(|v| v.as_str()).unwrap_or("Left");
        let button = match btn_name {
            "Right" => anvilkit_input::prelude::MouseButton::Right,
            "Middle" => anvilkit_input::prelude::MouseButton::Middle,
            _ => anvilkit_input::prelude::MouseButton::Left,
        };
        if let Some(mut input) = world.get_resource_mut::<anvilkit_input::prelude::InputState>() {
            input.set_mouse_position(glam::Vec2::new(x, y));
            input.press_mouse(button);
        }
        Ok(json!({ "clicked": { "x": x, "y": y, "button": btn_name } }))
    }
}

// ==================== State Query Tools ====================

/// Tool: get frame timing info (fps, delta_time, frame_count).
pub struct GetFrameInfoTool;

impl Tool for GetFrameInfoTool {
    fn name(&self) -> &str { "get_frame_info" }
    fn description(&self) -> &str { "Get current frame timing: FPS, delta time, frame count, elapsed time" }
    fn input_schema(&self) -> Value { json!({ "type": "object", "properties": {}, "required": [] }) }
    fn execute(&self, _params: Value, world: &mut bevy_ecs::world::World) -> ToolResult {
        if let Some(time) = world.get_resource::<anvilkit_core::time::Time>() {
            Ok(json!({
                "fps": time.fps(),
                "delta_seconds": time.delta_seconds(),
                "frame_count": time.frame_count(),
                "elapsed_seconds": time.elapsed_seconds(),
            }))
        } else {
            Ok(json!({ "fps": 0, "delta_seconds": 0.0, "frame_count": 0, "elapsed_seconds": 0.0 }))
        }
    }
}

/// Tool: get current keyboard and mouse input state.
pub struct GetInputStateTool;

impl Tool for GetInputStateTool {
    fn name(&self) -> &str { "get_input_state" }
    fn description(&self) -> &str { "Get currently pressed keys, mouse position, and mouse buttons" }
    fn input_schema(&self) -> Value { json!({ "type": "object", "properties": {}, "required": [] }) }
    fn execute(&self, _params: Value, world: &mut bevy_ecs::world::World) -> ToolResult {
        if let Some(input) = world.get_resource::<anvilkit_input::prelude::InputState>() {
            let pressed: Vec<String> = input.pressed_keys().iter()
                .map(|k| format!("{:?}", k))
                .collect();
            let mouse_pos = input.mouse_position();
            Ok(json!({
                "pressed_keys": pressed,
                "mouse_position": { "x": mouse_pos.x, "y": mouse_pos.y },
                "mouse_delta": { "x": input.mouse_delta().x, "y": input.mouse_delta().y },
                "scroll_delta": input.scroll_delta(),
            }))
        } else {
            Ok(json!({ "pressed_keys": [], "mouse_position": { "x": 0, "y": 0 } }))
        }
    }
}

// ==================== Scene Editing Tools ====================

/// Tool: spawn a visible object (cube, sphere, ground) with position and color.
pub struct SpawnObjectTool;

impl Tool for SpawnObjectTool {
    fn name(&self) -> &str { "spawn_object" }
    fn description(&self) -> &str { "Spawn a visible 3D object (cube, sphere, ground) at a position with a color. Returns entity ID." }
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "type": { "type": "string", "description": "Shape: cube, sphere, ground" },
                "pos": { "type": "array", "items": { "type": "number" }, "description": "[x, y, z] position" },
                "size": { "type": "array", "items": { "type": "number" }, "description": "[w, h, d] for cube, or [radius] for sphere, or [size] for ground" },
                "color": { "type": "array", "items": { "type": "number" }, "description": "[r, g, b, a] each 0.0-1.0" },
                "name": { "type": "string", "description": "Optional name for this object" }
            },
            "required": ["type", "pos", "color"]
        })
    }
    fn execute(&self, params: Value, world: &mut bevy_ecs::world::World) -> ToolResult {
        use crate::scene::*;

        let shape_type = params.get("type").and_then(|v| v.as_str()).unwrap_or("cube");
        let pos = parse_vec3(&params, "pos").unwrap_or([0.0; 3]);
        let color = parse_color(&params);
        let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();

        let shape = match shape_type {
            "sphere" => {
                let r = params.get("size").and_then(|v| v.as_array())
                    .and_then(|a| a.first()).and_then(|v| v.as_f64()).unwrap_or(1.0) as f32;
                Shape::Sphere { radius: r }
            }
            "ground" => {
                let s = params.get("size").and_then(|v| v.as_array())
                    .and_then(|a| a.first()).and_then(|v| v.as_f64()).unwrap_or(20.0) as f32;
                Shape::Ground { size: s }
            }
            _ => { // cube
                let size = parse_vec3(&params, "size").unwrap_or([1.0, 1.0, 1.0]);
                Shape::Cube { size }
            }
        };

        let entity = world.spawn((
            SceneObject { shape, color, name: name.clone() },
            SceneTransform { translation: pos, ..Default::default() },
        )).id();

        Ok(json!({
            "entity_id": entity.to_bits(),
            "type": shape_type,
            "pos": pos,
            "name": name
        }))
    }
}

/// Tool: modify a component field on an existing entity.
pub struct SetComponentTool;

impl Tool for SetComponentTool {
    fn name(&self) -> &str { "set_component" }
    fn description(&self) -> &str { "Modify a SceneTransform field (translation, rotation, scale) or SceneObject field (color, name) on an entity" }
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "entity_id": { "type": "integer", "description": "Entity ID to modify" },
                "field": { "type": "string", "description": "Field: translation, rotation, scale, color, name" },
                "value": { "description": "New value (array for vectors, string for name)" }
            },
            "required": ["entity_id", "field", "value"]
        })
    }
    fn execute(&self, params: Value, world: &mut bevy_ecs::world::World) -> ToolResult {
        use crate::scene::*;

        let entity_id = params.get("entity_id").and_then(|v| v.as_u64())
            .ok_or_else(|| ToolError::new("MISSING_PARAM", "entity_id required"))?;
        let field = params.get("field").and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::new("MISSING_PARAM", "field required"))?;
        let value = params.get("value")
            .ok_or_else(|| ToolError::new("MISSING_PARAM", "value required"))?;

        let entity = bevy_ecs::entity::Entity::from_bits(entity_id);

        match field {
            "translation" | "rotation" | "scale" => {
                let vec = value.as_array()
                    .and_then(|a| if a.len() >= 3 {
                        Some([
                            a[0].as_f64().unwrap_or(0.0) as f32,
                            a[1].as_f64().unwrap_or(0.0) as f32,
                            a[2].as_f64().unwrap_or(0.0) as f32,
                        ])
                    } else { None })
                    .ok_or_else(|| ToolError::new("INVALID_VALUE", "Expected [x, y, z] array"))?;

                let mut transform = world.get_mut::<SceneTransform>(entity)
                    .ok_or_else(|| ToolError::new("COMPONENT_NOT_FOUND", "Entity has no SceneTransform"))?;

                match field {
                    "translation" => transform.translation = vec,
                    "rotation" => transform.rotation = vec,
                    "scale" => transform.scale = vec,
                    _ => {}
                }
            }
            "color" => {
                let color = value.as_array()
                    .and_then(|a| if a.len() >= 4 {
                        Some([
                            a[0].as_f64().unwrap_or(0.0) as f32,
                            a[1].as_f64().unwrap_or(0.0) as f32,
                            a[2].as_f64().unwrap_or(0.0) as f32,
                            a[3].as_f64().unwrap_or(1.0) as f32,
                        ])
                    } else { None })
                    .ok_or_else(|| ToolError::new("INVALID_VALUE", "Expected [r, g, b, a] array"))?;

                let mut obj = world.get_mut::<SceneObject>(entity)
                    .ok_or_else(|| ToolError::new("COMPONENT_NOT_FOUND", "Entity has no SceneObject"))?;
                obj.color = color;
            }
            "name" => {
                let name = value.as_str()
                    .ok_or_else(|| ToolError::new("INVALID_VALUE", "Expected string"))?;
                let mut obj = world.get_mut::<SceneObject>(entity)
                    .ok_or_else(|| ToolError::new("COMPONENT_NOT_FOUND", "Entity has no SceneObject"))?;
                obj.name = name.to_string();
            }
            _ => {
                return Err(ToolError::new("UNKNOWN_FIELD", format!("Unknown field: {}", field))
                    .with_hint("Valid fields: translation, rotation, scale, color, name"));
            }
        }

        Ok(json!({ "modified": { "entity_id": entity_id, "field": field } }))
    }
}

/// Tool: save current scene objects to a RON file.
pub struct SaveSceneTool;

impl Tool for SaveSceneTool {
    fn name(&self) -> &str { "save_scene" }
    fn description(&self) -> &str { "Save all scene objects to a RON file. Only objects spawned via spawn_object are included." }
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "File path (e.g., levels/level1.ron)" }
            },
            "required": ["path"]
        })
    }
    fn execute(&self, params: Value, world: &mut bevy_ecs::world::World) -> ToolResult {
        use crate::scene::SerializedScene;

        let path = params.get("path").and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::new("MISSING_PARAM", "path required"))?;

        let scene = SerializedScene::from_world(world);
        scene.save(path).map_err(|e| ToolError::new("SAVE_ERROR", e))?;

        Ok(json!({
            "saved": path,
            "object_count": scene.objects.len()
        }))
    }
}

/// Tool: load scene objects from a RON file, spawning them into the world.
pub struct LoadSceneTool;

impl Tool for LoadSceneTool {
    fn name(&self) -> &str { "load_scene" }
    fn description(&self) -> &str { "Load scene objects from a RON file and spawn them into the world" }
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "File path to load (e.g., levels/level1.ron)" },
                "clear": { "type": "boolean", "description": "Clear existing scene objects first (default: true)" }
            },
            "required": ["path"]
        })
    }
    fn execute(&self, params: Value, world: &mut bevy_ecs::world::World) -> ToolResult {
        use crate::scene::{SerializedScene, SceneObject};

        let path = params.get("path").and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::new("MISSING_PARAM", "path required"))?;
        let clear = params.get("clear").and_then(|v| v.as_bool()).unwrap_or(true);

        // Optionally clear existing scene objects
        if clear {
            let to_despawn: Vec<_> = world.query_filtered::<bevy_ecs::entity::Entity, bevy_ecs::query::With<SceneObject>>()
                .iter(world).collect();
            for entity in to_despawn {
                world.despawn(entity);
            }
        }

        let scene = SerializedScene::load(path)
            .map_err(|e| ToolError::new("LOAD_ERROR", e))?;
        let ids = scene.spawn_into(world);

        Ok(json!({
            "loaded": path,
            "spawned_count": ids.len(),
            "entity_ids": ids.iter().map(|e| e.to_bits()).collect::<Vec<_>>()
        }))
    }
}

// --- Helpers ---

fn parse_vec3(params: &Value, key: &str) -> Option<[f32; 3]> {
    params.get(key)?.as_array().and_then(|a| {
        if a.len() >= 3 {
            Some([
                a[0].as_f64()? as f32,
                a[1].as_f64()? as f32,
                a[2].as_f64()? as f32,
            ])
        } else { None }
    })
}

fn parse_color(params: &Value) -> [f32; 4] {
    params.get("color").and_then(|v| v.as_array()).map(|a| {
        [
            a.get(0).and_then(|v| v.as_f64()).unwrap_or(1.0) as f32,
            a.get(1).and_then(|v| v.as_f64()).unwrap_or(1.0) as f32,
            a.get(2).and_then(|v| v.as_f64()).unwrap_or(1.0) as f32,
            a.get(3).and_then(|v| v.as_f64()).unwrap_or(1.0) as f32,
        ]
    }).unwrap_or([1.0, 1.0, 1.0, 1.0])
}

/// Register all built-in tools into a registry.
pub fn register_builtin_tools(registry: &mut crate::ToolRegistry) {
    // Entity tools
    registry.register(ListEntitiesTool);
    registry.register(EntityCountTool);
    registry.register(EngineInfoTool);
    registry.register(SpawnEmptyEntityTool);
    registry.register(DespawnEntityTool);
    registry.register(WorldSummaryTool);
    registry.register(ListSchedulesTool);
    registry.register(GetComponentIdsTool);
    // Input control tools
    registry.register(PressKeyTool);
    registry.register(ReleaseKeyTool);
    registry.register(ClickTool);
    // State query tools
    registry.register(GetFrameInfoTool);
    registry.register(GetInputStateTool);
    // Scene editing tools
    registry.register(SpawnObjectTool);
    registry.register(SetComponentTool);
    registry.register(SaveSceneTool);
    registry.register(LoadSceneTool);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ToolRegistry;

    #[test]
    fn test_builtin_tools_registered() {
        let mut registry = ToolRegistry::new();
        register_builtin_tools(&mut registry);
        assert!(registry.get("list_entities").is_some());
        assert!(registry.get("entity_count").is_some());
        assert!(registry.get("engine_info").is_some());
        assert!(registry.get("spawn_empty_entity").is_some());
        assert!(registry.get("despawn_entity").is_some());
    }

    #[test]
    fn test_spawn_then_list() {
        let mut registry = ToolRegistry::new();
        register_builtin_tools(&mut registry);
        let mut world = bevy_ecs::world::World::new();

        // Count entities (should be 0)
        let result = registry.dispatch("entity_count", json!({}), &mut world).unwrap().unwrap();
        assert_eq!(result["count"], 0);

        // Spawn an entity
        let spawn_result = registry.dispatch("spawn_empty_entity", json!({}), &mut world).unwrap().unwrap();
        let entity_id = spawn_result["entity_id"].as_u64().unwrap();

        // Count entities (should be 1)
        let result = registry.dispatch("entity_count", json!({}), &mut world).unwrap().unwrap();
        assert_eq!(result["count"], 1);

        // Despawn
        let despawn_result = registry.dispatch("despawn_entity", json!({"entity_id": entity_id}), &mut world).unwrap().unwrap();
        assert_eq!(despawn_result["despawned"], true);

        // Count again (should be 0)
        let result = registry.dispatch("entity_count", json!({}), &mut world).unwrap().unwrap();
        assert_eq!(result["count"], 0);
    }

    #[test]
    fn test_despawn_missing_param() {
        let mut registry = ToolRegistry::new();
        register_builtin_tools(&mut registry);
        let mut world = bevy_ecs::world::World::new();

        let result = registry.dispatch("despawn_entity", json!({}), &mut world).unwrap();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, "MISSING_PARAM");
    }

    #[test]
    fn test_despawn_nonexistent() {
        let mut registry = ToolRegistry::new();
        register_builtin_tools(&mut registry);
        let mut world = bevy_ecs::world::World::new();

        // Spawn and despawn an entity — then try to despawn the same ID again
        let spawn = registry.dispatch("spawn_empty_entity", json!({}), &mut world).unwrap().unwrap();
        let entity_id = spawn["entity_id"].as_u64().unwrap();
        registry.dispatch("despawn_entity", json!({"entity_id": entity_id}), &mut world).unwrap().unwrap();

        // Second despawn of same ID should return ENTITY_NOT_FOUND
        let result = registry.dispatch("despawn_entity", json!({"entity_id": entity_id}), &mut world).unwrap();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, "ENTITY_NOT_FOUND");
    }

    #[test]
    fn test_engine_info() {
        let mut registry = ToolRegistry::new();
        register_builtin_tools(&mut registry);
        let mut world = bevy_ecs::world::World::new();

        let result = registry.dispatch("engine_info", json!({}), &mut world).unwrap().unwrap();
        assert_eq!(result["name"], "AnvilKit");
    }

    #[test]
    fn test_world_summary_empty() {
        let mut registry = ToolRegistry::new();
        register_builtin_tools(&mut registry);
        let mut world = bevy_ecs::world::World::new();

        let result = registry.dispatch("get_world_summary", json!({}), &mut world).unwrap().unwrap();
        assert_eq!(result["entity_count"], 0);
        assert!(result["archetype_count"].as_u64().unwrap() >= 1); // at least the empty archetype
    }

    #[test]
    fn test_world_summary_with_entities() {
        let mut registry = ToolRegistry::new();
        register_builtin_tools(&mut registry);
        let mut world = bevy_ecs::world::World::new();

        world.spawn_empty();
        world.spawn_empty();

        let result = registry.dispatch("get_world_summary", json!({}), &mut world).unwrap().unwrap();
        assert_eq!(result["entity_count"], 2);
    }

    #[test]
    fn test_list_schedules_empty() {
        let mut registry = ToolRegistry::new();
        register_builtin_tools(&mut registry);
        let mut world = bevy_ecs::world::World::new();

        // No Schedules resource inserted — should return empty list.
        let result = registry.dispatch("list_schedules", json!({}), &mut world).unwrap().unwrap();
        assert_eq!(result["count"], 0);
        assert_eq!(result["schedules"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn test_list_schedules_with_schedules() {
        use bevy_ecs::schedule::{Schedules, Schedule, ScheduleLabel};

        #[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
        struct MySchedule;

        let mut registry = ToolRegistry::new();
        register_builtin_tools(&mut registry);
        let mut world = bevy_ecs::world::World::new();

        let mut schedules = Schedules::new();
        schedules.insert(Schedule::new(MySchedule));
        world.insert_resource(schedules);

        let result = registry.dispatch("list_schedules", json!({}), &mut world).unwrap().unwrap();
        assert_eq!(result["count"], 1);
        let names = result["schedules"].as_array().unwrap();
        assert_eq!(names.len(), 1);
        assert!(names[0].as_str().unwrap().contains("MySchedule"));
    }

    #[test]
    fn test_get_component_ids_empty_entity() {
        let mut registry = ToolRegistry::new();
        register_builtin_tools(&mut registry);
        let mut world = bevy_ecs::world::World::new();

        let entity = world.spawn_empty().id();
        let entity_bits = entity.to_bits();

        let result = registry.dispatch(
            "get_component_ids",
            json!({"entity_id": entity_bits}),
            &mut world,
        ).unwrap().unwrap();

        assert_eq!(result["entity_id"], entity_bits);
        assert_eq!(result["count"], 0);
        assert_eq!(result["components"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn test_get_component_ids_with_components() {
        use bevy_ecs::prelude::Component;

        #[derive(Component)]
        struct Health(f32);

        #[derive(Component)]
        struct Name(String);

        let mut registry = ToolRegistry::new();
        register_builtin_tools(&mut registry);
        let mut world = bevy_ecs::world::World::new();

        let entity = world.spawn((Health(100.0), Name("test".into()))).id();
        let entity_bits = entity.to_bits();

        let result = registry.dispatch(
            "get_component_ids",
            json!({"entity_id": entity_bits}),
            &mut world,
        ).unwrap().unwrap();

        assert_eq!(result["entity_id"], entity_bits);
        assert_eq!(result["count"], 2);

        let components = result["components"].as_array().unwrap();
        let names: Vec<&str> = components.iter().map(|v| v.as_str().unwrap()).collect();
        assert!(names.iter().any(|n| n.contains("Health")));
        assert!(names.iter().any(|n| n.contains("Name")));
    }

    #[test]
    fn test_get_component_ids_missing_param() {
        let mut registry = ToolRegistry::new();
        register_builtin_tools(&mut registry);
        let mut world = bevy_ecs::world::World::new();

        let result = registry.dispatch("get_component_ids", json!({}), &mut world).unwrap();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, "MISSING_PARAM");
    }

    #[test]
    fn test_get_component_ids_nonexistent_entity() {
        let mut registry = ToolRegistry::new();
        register_builtin_tools(&mut registry);
        let mut world = bevy_ecs::world::World::new();

        // Spawn and despawn to get a dead entity ID
        let entity = world.spawn_empty().id();
        let bits = entity.to_bits();
        world.despawn(entity);

        let result = registry.dispatch(
            "get_component_ids",
            json!({"entity_id": bits}),
            &mut world,
        ).unwrap();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, "ENTITY_NOT_FOUND");
    }

    #[test]
    fn test_new_tools_registered() {
        let mut registry = ToolRegistry::new();
        register_builtin_tools(&mut registry);
        assert!(registry.get("get_world_summary").is_some());
        assert!(registry.get("list_schedules").is_some());
        assert!(registry.get("get_component_ids").is_some());
    }
}
