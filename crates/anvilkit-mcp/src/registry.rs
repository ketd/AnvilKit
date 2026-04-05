//! Tool registry for MCP server.
//!
//! Tools are registered by name and dispatched at runtime when the MCP server
//! receives a JSON-RPC request.

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use crate::ToolResult;

/// Metadata describing an MCP tool's interface.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDescription {
    /// Human-readable tool name (e.g., "list_resources").
    pub name: String,
    /// What this tool does.
    pub description: String,
    /// JSON Schema for the parameters object.
    pub input_schema: Value,
}

/// Trait for implementing an MCP tool.
///
/// Each tool processes a JSON `params` object and returns a JSON result.
pub trait Tool: Send + Sync {
    /// Tool name (used as the JSON-RPC method name).
    fn name(&self) -> &str;

    /// Human-readable description of what this tool does.
    fn description(&self) -> &str;

    /// JSON Schema describing the expected parameters.
    fn input_schema(&self) -> Value;

    /// Execute the tool with the given parameters and ECS world.
    fn execute(&self, params: Value, world: &mut bevy_ecs::world::World) -> ToolResult;
}

/// Registry of available MCP tools.
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl ToolRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self { tools: HashMap::new() }
    }

    /// Register a tool. Replaces any existing tool with the same name.
    pub fn register(&mut self, tool: impl Tool + 'static) {
        self.tools.insert(tool.name().to_string(), Box::new(tool));
    }

    /// Look up a tool by name.
    pub fn get(&self, name: &str) -> Option<&dyn Tool> {
        self.tools.get(name).map(|t| t.as_ref())
    }

    /// List all registered tool descriptions.
    pub fn list_tools(&self) -> Vec<ToolDescription> {
        self.tools.values().map(|t| ToolDescription {
            name: t.name().to_string(),
            description: t.description().to_string(),
            input_schema: t.input_schema(),
        }).collect()
    }

    /// Dispatch a method call to the appropriate tool.
    pub fn dispatch(&self, method: &str, params: Value, world: &mut bevy_ecs::world::World) -> Option<ToolResult> {
        self.tools.get(method).map(|tool| tool.execute(params, world))
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    struct DummyTool;

    impl Tool for DummyTool {
        fn name(&self) -> &str { "ping" }
        fn description(&self) -> &str { "Returns pong" }
        fn input_schema(&self) -> Value { json!({}) }
        fn execute(&self, _params: Value, _world: &mut bevy_ecs::world::World) -> ToolResult {
            Ok(json!("pong"))
        }
    }

    #[test]
    fn test_registry_register_and_get() {
        let mut reg = ToolRegistry::new();
        reg.register(DummyTool);
        assert!(reg.get("ping").is_some());
        assert!(reg.get("unknown").is_none());
    }

    #[test]
    fn test_registry_list_tools() {
        let mut reg = ToolRegistry::new();
        reg.register(DummyTool);
        let tools = reg.list_tools();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "ping");
    }

    #[test]
    fn test_registry_dispatch() {
        let mut reg = ToolRegistry::new();
        reg.register(DummyTool);
        let mut world = bevy_ecs::world::World::new();
        let result = reg.dispatch("ping", json!({}), &mut world);
        assert!(result.is_some());
        assert_eq!(result.unwrap().unwrap(), json!("pong"));
    }

    #[test]
    fn test_registry_dispatch_unknown() {
        let reg = ToolRegistry::new();
        let mut world = bevy_ecs::world::World::new();
        assert!(reg.dispatch("missing", json!({}), &mut world).is_none());
    }
}
