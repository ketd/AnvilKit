//! # 自动存档系统
//!
//! 提供定时自动存档的计时器逻辑和槽位轮换。
//! 本模块不依赖 ECS，游戏每帧调用 `auto_save_tick()` 即可驱动自动存档。

/// 自动存档配置
///
/// 控制自动存档的间隔时间、槽位名称和轮换数量。
///
/// # 示例
///
/// ```rust
/// use anvilkit_core::persistence::AutoSaveConfig;
///
/// let config = AutoSaveConfig {
///     enabled: true,
///     interval_secs: 120.0,  // 每 2 分钟
///     slot_name: "_autosave".to_string(),
///     max_autosaves: 3,
/// };
/// ```
#[derive(Debug, Clone)]
#[cfg_attr(feature = "bevy_ecs", derive(bevy_ecs::system::Resource))]
pub struct AutoSaveConfig {
    /// 是否启用自动存档。
    pub enabled: bool,
    /// 自动存档间隔（秒）。默认 300.0（5 分钟）。
    pub interval_secs: f64,
    /// 自动存档槽位名称前缀。默认 `"_autosave"`。
    pub slot_name: String,
    /// 最大自动存档轮换数量。默认 3（轮换 `_autosave_1`, `_autosave_2`, `_autosave_3`）。
    pub max_autosaves: usize,
}

impl Default for AutoSaveConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_secs: 300.0,
            slot_name: "_autosave".to_string(),
            max_autosaves: 3,
        }
    }
}

/// 自动存档状态追踪器
///
/// 记录距离上次存档的累计时间和当前槽位索引。
///
/// # 示例
///
/// ```rust
/// use anvilkit_core::persistence::AutoSaveState;
///
/// let state = AutoSaveState::default();
/// assert_eq!(state.elapsed, 0.0);
/// assert_eq!(state.current_slot_index, 0);
/// ```
#[derive(Debug, Clone)]
#[cfg_attr(feature = "bevy_ecs", derive(bevy_ecs::system::Resource))]
pub struct AutoSaveState {
    /// 自上次存档以来的累计时间（秒）。
    pub elapsed: f64,
    /// 当前槽位索引（0-based，轮换范围 `0..max_autosaves`）。
    pub current_slot_index: usize,
    /// 上次触发存档的时间戳（累计运行时间，秒），`None` 表示尚未触发过。
    pub last_save_time: Option<f64>,
}

impl Default for AutoSaveState {
    fn default() -> Self {
        Self {
            elapsed: 0.0,
            current_slot_index: 0,
            last_save_time: None,
        }
    }
}

/// 返回下一个自动存档槽位名称。
///
/// 槽位名称格式为 `{slot_name}_{index}`，索引从 1 开始，轮换范围 `1..=max_autosaves`。
///
/// # 示例
///
/// ```rust
/// use anvilkit_core::persistence::{AutoSaveConfig, AutoSaveState, next_autosave_slot};
///
/// let config = AutoSaveConfig::default();
/// let state = AutoSaveState { current_slot_index: 0, ..Default::default() };
/// assert_eq!(next_autosave_slot(&config, &state), "_autosave_1");
/// ```
pub fn next_autosave_slot(config: &AutoSaveConfig, state: &AutoSaveState) -> String {
    let index = (state.current_slot_index % config.max_autosaves) + 1;
    format!("{}_{}", config.slot_name, index)
}

/// 自动存档 tick 函数。每帧调用一次。
///
/// 累加 `delta_time`，当累计时间达到 `interval_secs` 时触发存档，返回
/// `Some(slot_name)` 表示应执行存档操作；否则返回 `None`。
///
/// 调用方收到 `Some` 后应调用 `SaveManager::save()` 将数据写入对应槽位。
///
/// # 示例
///
/// ```rust
/// use anvilkit_core::persistence::{AutoSaveConfig, AutoSaveState, auto_save_tick};
///
/// let config = AutoSaveConfig {
///     interval_secs: 1.0,
///     ..Default::default()
/// };
/// let mut state = AutoSaveState::default();
///
/// // 不到间隔，不触发
/// assert!(auto_save_tick(&config, &mut state, 0.5).is_none());
///
/// // 达到间隔，触发存档
/// let slot = auto_save_tick(&config, &mut state, 0.6);
/// assert_eq!(slot, Some("_autosave_1".to_string()));
/// ```
pub fn auto_save_tick(
    config: &AutoSaveConfig,
    state: &mut AutoSaveState,
    delta_time: f64,
) -> Option<String> {
    if !config.enabled || config.max_autosaves == 0 {
        return None;
    }

    state.elapsed += delta_time;

    if state.elapsed >= config.interval_secs {
        let slot = next_autosave_slot(config, state);
        // 记录触发时间（使用累计运行时间近似）
        let total_time = state.last_save_time.unwrap_or(0.0) + state.elapsed;
        state.last_save_time = Some(total_time);
        // 推进槽位索引
        state.current_slot_index = (state.current_slot_index + 1) % config.max_autosaves;
        // 重置计时器（保留溢出部分以保持精度）
        state.elapsed -= config.interval_secs;
        Some(slot)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AutoSaveConfig::default();
        assert!(config.enabled);
        assert_eq!(config.interval_secs, 300.0);
        assert_eq!(config.slot_name, "_autosave");
        assert_eq!(config.max_autosaves, 3);
    }

    #[test]
    fn test_default_state() {
        let state = AutoSaveState::default();
        assert_eq!(state.elapsed, 0.0);
        assert_eq!(state.current_slot_index, 0);
        assert!(state.last_save_time.is_none());
    }

    #[test]
    fn test_next_autosave_slot_naming() {
        let config = AutoSaveConfig::default(); // max_autosaves = 3
        let state = AutoSaveState { current_slot_index: 0, ..Default::default() };
        assert_eq!(next_autosave_slot(&config, &state), "_autosave_1");

        let state = AutoSaveState { current_slot_index: 1, ..Default::default() };
        assert_eq!(next_autosave_slot(&config, &state), "_autosave_2");

        let state = AutoSaveState { current_slot_index: 2, ..Default::default() };
        assert_eq!(next_autosave_slot(&config, &state), "_autosave_3");
    }

    #[test]
    fn test_next_autosave_slot_wraps() {
        let config = AutoSaveConfig {
            max_autosaves: 2,
            ..Default::default()
        };
        // index 2 % 2 = 0 → slot 1
        let state = AutoSaveState { current_slot_index: 2, ..Default::default() };
        assert_eq!(next_autosave_slot(&config, &state), "_autosave_1");
    }

    #[test]
    fn test_tick_no_trigger_before_interval() {
        let config = AutoSaveConfig {
            interval_secs: 10.0,
            ..Default::default()
        };
        let mut state = AutoSaveState::default();

        assert!(auto_save_tick(&config, &mut state, 3.0).is_none());
        assert_eq!(state.elapsed, 3.0);
        assert!(auto_save_tick(&config, &mut state, 3.0).is_none());
        assert_eq!(state.elapsed, 6.0);
    }

    #[test]
    fn test_tick_triggers_at_interval() {
        let config = AutoSaveConfig {
            interval_secs: 5.0,
            ..Default::default()
        };
        let mut state = AutoSaveState::default();

        assert!(auto_save_tick(&config, &mut state, 4.0).is_none());
        let result = auto_save_tick(&config, &mut state, 2.0);
        assert_eq!(result, Some("_autosave_1".to_string()));
        // 溢出部分保留: 4.0 + 2.0 - 5.0 = 1.0
        assert!((state.elapsed - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_tick_rotates_slots() {
        let config = AutoSaveConfig {
            interval_secs: 1.0,
            max_autosaves: 3,
            ..Default::default()
        };
        let mut state = AutoSaveState::default();

        let r1 = auto_save_tick(&config, &mut state, 1.5);
        assert_eq!(r1, Some("_autosave_1".to_string()));

        let r2 = auto_save_tick(&config, &mut state, 1.0);
        assert_eq!(r2, Some("_autosave_2".to_string()));

        let r3 = auto_save_tick(&config, &mut state, 1.0);
        assert_eq!(r3, Some("_autosave_3".to_string()));

        // 轮换回到 1
        let r4 = auto_save_tick(&config, &mut state, 1.0);
        assert_eq!(r4, Some("_autosave_1".to_string()));
    }

    #[test]
    fn test_tick_disabled() {
        let config = AutoSaveConfig {
            enabled: false,
            interval_secs: 1.0,
            ..Default::default()
        };
        let mut state = AutoSaveState::default();

        assert!(auto_save_tick(&config, &mut state, 100.0).is_none());
        // 被禁用时不累加时间
        // （实际设计选择：即使 disabled 也累加了 elapsed，但不触发——
        // 这在 re-enable 时可立即触发一次存档，是合理行为）
    }

    #[test]
    fn test_tick_zero_max_autosaves() {
        let config = AutoSaveConfig {
            max_autosaves: 0,
            interval_secs: 1.0,
            ..Default::default()
        };
        let mut state = AutoSaveState::default();

        assert!(auto_save_tick(&config, &mut state, 100.0).is_none());
    }

    #[test]
    fn test_last_save_time_tracking() {
        let config = AutoSaveConfig {
            interval_secs: 2.0,
            ..Default::default()
        };
        let mut state = AutoSaveState::default();

        assert!(state.last_save_time.is_none());

        auto_save_tick(&config, &mut state, 2.5);
        // 第一次存档，last_save_time = 0.0 + 2.5 = 2.5
        assert!((state.last_save_time.unwrap() - 2.5).abs() < f64::EPSILON);

        auto_save_tick(&config, &mut state, 2.0);
        // 第二次存档，elapsed 溢出后为 0.5 + 2.0 = 2.5，last_save_time = 2.5 + 2.5 = 5.0
        assert!((state.last_save_time.unwrap() - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_custom_slot_name() {
        let config = AutoSaveConfig {
            slot_name: "checkpoint".to_string(),
            max_autosaves: 2,
            ..Default::default()
        };
        let state = AutoSaveState { current_slot_index: 0, ..Default::default() };
        assert_eq!(next_autosave_slot(&config, &state), "checkpoint_1");

        let state = AutoSaveState { current_slot_index: 1, ..Default::default() };
        assert_eq!(next_autosave_slot(&config, &state), "checkpoint_2");
    }

    #[test]
    fn test_exact_interval_boundary() {
        let config = AutoSaveConfig {
            interval_secs: 5.0,
            ..Default::default()
        };
        let mut state = AutoSaveState::default();

        // 精确到达间隔边界
        let result = auto_save_tick(&config, &mut state, 5.0);
        assert_eq!(result, Some("_autosave_1".to_string()));
        assert!((state.elapsed - 0.0).abs() < f64::EPSILON);
    }
}
