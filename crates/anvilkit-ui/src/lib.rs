//! # AnvilKit UI Framework
//!
//! Retained-mode UI system with flexbox layout, event handling, and widgets.
//! GPU-independent — rendering is handled by `anvilkit-render`'s `UiRenderer`.
//!
//! ## Quick Start
//!
//! ```rust
//! use anvilkit_ui::prelude::*;
//!
//! // Create widgets
//! let button = Widget::button("Click me");
//! let label = Widget::label("Hello, world!");
//! let panel = Widget::panel();
//!
//! // Layout engine computes positions
//! let mut engine = UiLayoutEngine::new();
//! ```

pub mod style;
pub mod layout;
pub mod events;
pub mod widgets;
pub mod theme;
pub mod focus;
pub mod plugin;
pub mod controls;

pub use style::*;
pub use layout::UiLayoutEngine;
pub use events::{UiEventKind, UiEvent, UiEvents, ui_hit_test, process_ui_interactions};
pub use widgets::Widget;
pub use theme::UiTheme;
pub use focus::{UiFocus, focus_interaction_system};
pub use plugin::UiPlugin;

/// Prelude for convenient imports.
pub mod prelude {
    pub use crate::style::*;
    pub use crate::layout::UiLayoutEngine;
    pub use crate::events::{UiEventKind, UiEvent, UiEvents, ui_hit_test, process_ui_interactions};
    pub use crate::widgets::Widget;
    pub use crate::theme::UiTheme;
    pub use crate::focus::{UiFocus, focus_interaction_system};
    pub use crate::plugin::UiPlugin;
    pub use crate::controls::*;
}
