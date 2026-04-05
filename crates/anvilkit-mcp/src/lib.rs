//! # AnvilKit MCP — Model Context Protocol Server
//!
//! Exposes AnvilKit engine state to AI agents via MCP (JSON-RPC over stdio).
//!
//! ## Architecture
//!
//! - [`McpServer`] runs a stdio-based JSON-RPC loop
//! - Registered [`Tool`]s are dispatched to by name
//! - Built-in tools: `list_resources`, `list_components`, `spawn_entity`,
//!   `query_world`, `inspect_component`
//!
//! ## Usage
//!
//! ```rust,ignore
//! use anvilkit_mcp::McpPlugin;
//! use bevy_app::App;
//!
//! let mut app = App::new();
//! app.add_plugins(McpPlugin);
//! // The MCP server now accepts JSON-RPC on stdin, responds on stdout
//! ```
//!
//! Or use the tool registry directly without the stdio server:
//!
//! ```rust
//! use anvilkit_mcp::{ToolRegistry, tools::register_builtin_tools};
//!
//! let mut registry = ToolRegistry::new();
//! register_builtin_tools(&mut registry);
//! assert!(registry.get("list_entities").is_some());
//! ```

use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::collections::HashMap;

pub mod protocol;
pub mod registry;
pub mod tools;
pub mod server;
pub mod plugin;
pub mod scene;

pub use protocol::{JsonRpcRequest, JsonRpcResponse, JsonRpcError};
pub use registry::{Tool, ToolRegistry};
pub use server::McpServer;
pub use plugin::McpPlugin;

/// MCP tool result — either a successful response or an error.
pub type ToolResult = Result<Value, ToolError>;

/// Error returned by a tool invocation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolError {
    /// Stable error code (e.g., "ENTITY_NOT_FOUND").
    pub code: String,
    /// Human-readable message.
    pub message: String,
    /// Agent-readable hint for resolution.
    pub hint: String,
}

impl ToolError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            hint: String::new(),
        }
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = hint.into();
        self
    }
}

/// Engine capability metadata returned by the `initialize` MCP handshake.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineCapabilities {
    /// Engine name.
    pub name: String,
    /// Engine version.
    pub version: String,
    /// Available tools and their descriptions.
    pub tools: HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_error_construction() {
        let err = ToolError::new("TEST_ERROR", "something failed")
            .with_hint("try restarting");
        assert_eq!(err.code, "TEST_ERROR");
        assert_eq!(err.hint, "try restarting");
    }

    #[test]
    fn test_tool_error_serialize() {
        let err = ToolError::new("X", "y");
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("\"code\":\"X\""));
    }
}
