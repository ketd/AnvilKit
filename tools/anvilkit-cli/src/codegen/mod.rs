pub mod component;
pub mod system;
pub mod resource;

use crate::error::{CliError, Result};

/// Rust keyword list (subset of commonly misused ones).
const RUST_KEYWORDS: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "crate", "dyn",
    "else", "enum", "extern", "false", "fn", "for", "if", "impl", "in",
    "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return",
    "self", "Self", "static", "struct", "super", "trait", "true", "type",
    "unsafe", "use", "where", "while", "yield",
];

/// Validate that `name` is a valid Rust identifier (XID_Start + XID_Continue*, not a keyword).
pub fn validate_identifier(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(CliError::InvalidInput("标识符不能为空".into()));
    }
    let mut chars = name.chars();
    let first = chars.next().unwrap();
    if !first.is_ascii_alphabetic() && first != '_' {
        return Err(CliError::InvalidInput(format!(
            "'{}' 不是有效的 Rust 标识符 — 必须以字母或下划线开头", name
        )));
    }
    for ch in chars {
        if !ch.is_ascii_alphanumeric() && ch != '_' {
            return Err(CliError::InvalidInput(format!(
                "'{}' 不是有效的 Rust 标识符 — 包含非法字符 '{}'", name, ch
            )));
        }
    }
    if RUST_KEYWORDS.contains(&name) {
        return Err(CliError::InvalidInput(format!(
            "'{}' 是 Rust 保留关键字", name
        )));
    }
    Ok(())
}
