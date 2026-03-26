//! # AnvilKit 输入系统
//!
//! 提供键盘、鼠标和手柄的抽象输入层，支持 action mapping 和状态查询。
//!
//! ## 使用示例
//!
//! ```rust
//! use anvilkit_input::prelude::*;
//!
//! let mut input = InputState::new();
//! input.press_key(KeyCode::Space);
//! assert!(input.is_key_pressed(KeyCode::Space));
//! assert!(input.is_key_just_pressed(KeyCode::Space));
//!
//! input.end_frame();
//! assert!(input.is_key_pressed(KeyCode::Space));
//! assert!(!input.is_key_just_pressed(KeyCode::Space));
//! ```

#![warn(missing_docs)]

pub mod input_state;
pub mod action_map;
pub mod gamepad;

/// Convenient re-exports for common input types.
pub mod prelude {
    pub use crate::input_state::{InputState, KeyCode, MouseButton};
    pub use crate::action_map::{ActionId, ActionMap, ActionState};
    pub use crate::gamepad::{GamepadAxis, GamepadButton, GamepadState};
}
