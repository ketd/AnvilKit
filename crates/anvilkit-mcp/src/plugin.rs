//! Bevy plugin that integrates the MCP server into the game loop.
//!
//! Adds an `McpServer` as a non-Send resource and processes pending requests
//! each frame in the `Last` schedule.

use bevy_app::Plugin;
use bevy_ecs::prelude::*;
use bevy_ecs::system::NonSend;

use crate::server::McpServer;

/// Plugin that starts an MCP server on stdin/stdout.
///
/// # Usage
///
/// ```rust,ignore
/// use anvilkit_mcp::McpPlugin;
///
/// app.add_plugins(McpPlugin);
/// ```
///
/// Once added, the game will accept JSON-RPC requests on stdin and respond
/// on stdout. The server processes requests once per frame in `Last`.
pub struct McpPlugin;

impl Plugin for McpPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        let server = McpServer::start();
        app.insert_non_send_resource(server);
        app.add_systems(bevy_app::Last, mcp_process_system);
        log::info!("MCP server started on stdin/stdout");
    }
}

/// Exclusive system that processes pending MCP requests each frame.
///
/// Uses exclusive system access to get both `&McpServer` (non-Send) and `&mut World`.
fn mcp_process_system(world: &mut World) {
    // Take the server out temporarily to avoid aliased borrows.
    let server = world.remove_non_send_resource::<McpServer>();
    if let Some(server) = server {
        server.process_requests(world);
        world.insert_non_send_resource(server);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_plugin_name() {
        let plugin = McpPlugin;
        assert!(bevy_app::Plugin::name(&plugin).contains("McpPlugin"));
    }
}
