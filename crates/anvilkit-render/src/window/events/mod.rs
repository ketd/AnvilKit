//! # 事件处理和应用生命周期
//!
//! 基于 winit 0.30 的 ApplicationHandler 实现应用生命周期管理和事件处理。

mod lighting;
mod render_app;
mod gpu_init;
mod render_loop;
mod input;

pub use render_app::RenderApp;
pub use lighting::{pack_lights, compute_cascade_matrices, compute_light_space_matrix};
