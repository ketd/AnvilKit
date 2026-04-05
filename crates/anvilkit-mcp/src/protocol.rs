//! JSON-RPC 2.0 protocol types for MCP communication.
//!
//! MCP is a subset of JSON-RPC 2.0. Each message is a single line of JSON
//! sent over stdio. Requests have an `id`, notifications do not.

use serde::{Serialize, Deserialize};
use serde_json::Value;

/// JSON-RPC 2.0 request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    /// Must be "2.0".
    pub jsonrpc: String,
    /// Request identifier (numeric or string). Absent for notifications.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    /// Method name to invoke (e.g., "list_resources").
    pub method: String,
    /// Method parameters as a JSON object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

/// JSON-RPC 2.0 response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    /// Must be "2.0".
    pub jsonrpc: String,
    /// Matches the request id.
    pub id: Value,
    /// Result value on success.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// Error object on failure.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

impl JsonRpcResponse {
    /// Create a successful response.
    pub fn success(id: Value, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    /// Create an error response.
    pub fn error(id: Value, error: JsonRpcError) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(error),
        }
    }
}

/// JSON-RPC 2.0 error object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    /// Error code (e.g., -32601 for method not found).
    pub code: i32,
    /// Short human-readable message.
    pub message: String,
    /// Additional error data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl JsonRpcError {
    /// Method not found (-32601).
    pub fn method_not_found(method: &str) -> Self {
        Self {
            code: -32601,
            message: format!("Method not found: {}", method),
            data: None,
        }
    }

    /// Invalid parameters (-32602).
    pub fn invalid_params(msg: impl Into<String>) -> Self {
        Self {
            code: -32602,
            message: msg.into(),
            data: None,
        }
    }

    /// Internal error (-32603).
    pub fn internal(msg: impl Into<String>) -> Self {
        Self {
            code: -32603,
            message: msg.into(),
            data: None,
        }
    }

    /// Parse error (-32700).
    pub fn parse_error() -> Self {
        Self {
            code: -32700,
            message: "Parse error".to_string(),
            data: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_request_deserialize() {
        let raw = r#"{"jsonrpc":"2.0","id":1,"method":"list_resources","params":{}}"#;
        let req: JsonRpcRequest = serde_json::from_str(raw).unwrap();
        assert_eq!(req.method, "list_resources");
        assert_eq!(req.id, Some(json!(1)));
    }

    #[test]
    fn test_response_success() {
        let resp = JsonRpcResponse::success(json!(1), json!({"count": 5}));
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"result\""));
        assert!(!json.contains("\"error\""));
    }

    #[test]
    fn test_response_error() {
        let resp = JsonRpcResponse::error(json!(1), JsonRpcError::method_not_found("foo"));
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"error\""));
        assert!(json.contains("-32601"));
        assert!(!json.contains("\"result\""));
    }

    #[test]
    fn test_error_codes() {
        assert_eq!(JsonRpcError::method_not_found("x").code, -32601);
        assert_eq!(JsonRpcError::invalid_params("x").code, -32602);
        assert_eq!(JsonRpcError::internal("x").code, -32603);
        assert_eq!(JsonRpcError::parse_error().code, -32700);
    }
}
