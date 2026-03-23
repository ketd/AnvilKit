//! # 持久化系统
//!
//! 提供游戏存档管理、玩家设置持久化和大规模数据 KV 存储。
//!
//! 需要启用 `persistence` feature flag。

#[cfg(feature = "persistence")]
mod settings;
#[cfg(feature = "persistence")]
mod storage;
#[cfg(feature = "persistence")]
mod save_manager;

#[cfg(feature = "persistence")]
pub use settings::*;
#[cfg(feature = "persistence")]
pub use storage::*;
#[cfg(feature = "persistence")]
pub use save_manager::*;
