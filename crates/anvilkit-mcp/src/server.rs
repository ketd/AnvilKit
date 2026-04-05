//! MCP stdio server — reads JSON-RPC from stdin, dispatches to tools, writes to stdout.
//!
//! The server runs a background thread for IO. Communication with the bevy main thread
//! is via crossbeam-style channels: `(request_tx, request_rx)` and `(response_tx, response_rx)`.
//!
//! ## Threading model
//!
//! ```text
//! [IO Thread]                    [Main Thread (bevy ECS)]
//!   stdin → parse JSON-RPC
//!   → request_tx ───────────────→ request_rx
//!                                  dispatch(method, params, &mut World)
//!   response_rx ←─────────────── response_tx ←
//!   → write JSON to stdout
//! ```

use std::io::{BufRead, Write};
use std::sync::mpsc;
use std::thread;

use crate::protocol::{JsonRpcRequest, JsonRpcResponse, JsonRpcError};
use crate::registry::ToolRegistry;
use serde_json::Value;

/// A pending request from the IO thread, waiting for dispatch.
pub struct PendingRequest {
    pub id: Value,
    pub method: String,
    pub params: Value,
}

/// MCP Server state — holds channels and the tool registry.
pub struct McpServer {
    /// Receives requests from the IO thread.
    request_rx: mpsc::Receiver<PendingRequest>,
    /// Sends responses back to the IO thread.
    response_tx: mpsc::Sender<JsonRpcResponse>,
    /// Tool registry for dispatching.
    pub registry: ToolRegistry,
    /// Handle to the IO thread (kept alive).
    _io_thread: Option<thread::JoinHandle<()>>,
}

impl McpServer {
    /// Start the MCP server. Spawns a background IO thread reading from stdin.
    pub fn start() -> Self {
        let (req_tx, req_rx) = mpsc::channel::<PendingRequest>();
        let (resp_tx, resp_rx) = mpsc::channel::<JsonRpcResponse>();

        let io_thread = thread::Builder::new()
            .name("anvilkit-mcp-io".into())
            .spawn(move || {
                Self::io_loop(req_tx, resp_rx);
            })
            .expect("Failed to spawn MCP IO thread");

        let mut registry = ToolRegistry::new();
        crate::tools::register_builtin_tools(&mut registry);

        McpServer {
            request_rx: req_rx,
            response_tx: resp_tx,
            registry,
            _io_thread: Some(io_thread),
        }
    }

    /// Process all pending requests using the given World.
    ///
    /// Call this once per frame from the main thread (e.g., in a bevy system).
    /// Non-blocking: processes all queued requests, then returns.
    pub fn process_requests(&self, world: &mut bevy_ecs::world::World) {
        while let Ok(req) = self.request_rx.try_recv() {
            let response = if req.method == "initialize" {
                let caps = self.capabilities();
                JsonRpcResponse::success(req.id, serde_json::to_value(caps).unwrap())
            } else if req.method == "tools/list" {
                let tools = self.registry.list_tools();
                JsonRpcResponse::success(req.id, serde_json::to_value(tools).unwrap())
            } else {
                match self.registry.dispatch(&req.method, req.params, world) {
                    Some(Ok(result)) => JsonRpcResponse::success(req.id, result),
                    Some(Err(tool_err)) => {
                        JsonRpcResponse::error(req.id, JsonRpcError {
                            code: -32000,
                            message: tool_err.message,
                            data: Some(serde_json::json!({
                                "code": tool_err.code,
                                "hint": tool_err.hint,
                            })),
                        })
                    }
                    None => JsonRpcResponse::error(
                        req.id,
                        JsonRpcError::method_not_found(&req.method),
                    ),
                }
            };

            let _ = self.response_tx.send(response);
        }
    }

    /// Engine capabilities for MCP initialize handshake.
    fn capabilities(&self) -> crate::EngineCapabilities {
        let tools = self.registry.list_tools()
            .into_iter()
            .map(|t| (t.name, t.description))
            .collect();
        crate::EngineCapabilities {
            name: "AnvilKit".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            tools,
        }
    }

    /// IO thread main loop: reads lines from stdin, parses JSON-RPC, forwards to main thread.
    fn io_loop(req_tx: mpsc::Sender<PendingRequest>, resp_rx: mpsc::Receiver<JsonRpcResponse>) {
        let stdin = std::io::stdin();
        let mut stdout = std::io::stdout();

        for line in stdin.lock().lines() {
            let line = match line {
                Ok(l) => l,
                Err(_) => break,
            };

            if line.trim().is_empty() {
                continue;
            }

            // Parse JSON-RPC request
            let request: JsonRpcRequest = match serde_json::from_str(&line) {
                Ok(r) => r,
                Err(_) => {
                    let err_resp = JsonRpcResponse::error(
                        Value::Null,
                        JsonRpcError::parse_error(),
                    );
                    let _ = writeln!(stdout, "{}", serde_json::to_string(&err_resp).unwrap());
                    let _ = stdout.flush();
                    continue;
                }
            };

            let id = request.id.clone().unwrap_or(Value::Null);
            let method = request.method.clone();
            let params = request.params.unwrap_or(Value::Object(Default::default()));

            // Send to main thread for dispatch
            if req_tx.send(PendingRequest { id: id.clone(), method, params }).is_err() {
                break; // Main thread dropped — exit
            }

            // Wait for response from main thread
            match resp_rx.recv() {
                Ok(response) => {
                    let _ = writeln!(stdout, "{}", serde_json::to_string(&response).unwrap());
                    let _ = stdout.flush();
                }
                Err(_) => break, // Main thread dropped
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pending_request_creation() {
        let req = PendingRequest {
            id: serde_json::json!(1),
            method: "list_entities".to_string(),
            params: serde_json::json!({}),
        };
        assert_eq!(req.method, "list_entities");
    }
}
