//! Screen management — automatic cursor control tied to game state.
//!
//! [`ScreenPlugin`] registers a [`GameState<S>`](crate::state::GameState)
//! state machine and keeps the window cursor mode in sync: locked (grabbed + invisible)
//! for gameplay states, free for menu states.

pub mod cursor;
pub mod plugin;

pub use cursor::CursorMode;
pub use plugin::{ScreenPlugin, ScreenPluginConfig};
