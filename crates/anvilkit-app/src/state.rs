//! # 游戏状态机
//!
//! 提供简单的类型化状态管理，支持状态转换、条件系统执行。
//!
//! ## 使用示例
//!
//! ```rust
//! use anvilkit_app::prelude::*;
//! use anvilkit_app::state::{GameState, NextGameState, in_state};
//! use anvilkit_app::schedule::AnvilKitSchedule;
//!
//! #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
//! enum AppState { #[default] Menu, Playing, Paused }
//!
//! let mut app = App::new();
//! app.add_plugins(AnvilKitEcsPlugin);
//! app.insert_resource(GameState(AppState::Menu));
//! app.insert_resource(NextGameState::<AppState>::default());
//! ```

use bevy_ecs::prelude::*;
use std::fmt::Debug;
use std::hash::Hash;

/// 当前游戏状态资源
///
/// 包含当前活跃的状态值。通过 `NextGameState<S>` 请求转换。
///
/// # 示例
///
/// ```rust
/// use anvilkit_app::state::GameState;
///
/// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
/// enum MyState { #[default] Menu, Playing }
///
/// let state = GameState(MyState::Menu);
/// assert_eq!(state.0, MyState::Menu);
/// ```
#[derive(Resource, Debug, Clone)]
pub struct GameState<S: StateValue>(pub S);

impl<S: StateValue + Default> Default for GameState<S> {
    fn default() -> Self {
        Self(S::default())
    }
}

/// 下一帧的目标状态资源
///
/// 设置此资源将在下一次 `state_transition_system` 运行时触发状态转换。
///
/// # 示例
///
/// ```rust
/// use anvilkit_app::state::NextGameState;
///
/// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
/// enum MyState { #[default] Menu, Playing }
///
/// let mut next = NextGameState::<MyState>::default();
/// next.set(MyState::Playing);
/// assert_eq!(next.0, Some(MyState::Playing));
/// ```
#[derive(Resource, Debug, Clone)]
pub struct NextGameState<S: StateValue>(pub Option<S>);

impl<S: StateValue> Default for NextGameState<S> {
    fn default() -> Self {
        Self(None)
    }
}

impl<S: StateValue> NextGameState<S> {
    /// 请求状态转换
    pub fn set(&mut self, state: S) {
        self.0 = Some(state);
    }

    /// 清除待处理的转换请求
    pub fn clear(&mut self) {
        self.0 = None;
    }
}

/// 状态值 trait bound
pub trait StateValue: Debug + Clone + Copy + PartialEq + Eq + Hash + Send + Sync + 'static {}

/// 自动为满足条件的类型实现 StateValue
impl<T> StateValue for T where T: Debug + Clone + Copy + PartialEq + Eq + Hash + Send + Sync + 'static {}

/// 状态转换事件
///
/// 当状态从 `from` 变为 `to` 时触发。
#[derive(Debug, Clone, Event)]
pub struct StateTransitionEvent<S: StateValue> {
    /// 转换前的状态
    pub from: S,
    /// 转换后的状态
    pub to: S,
}

/// 创建"当前状态为 S"的运行条件
///
/// 用于 `.run_if(in_state(MyState::Playing))` 限制系统仅在特定状态下执行。
///
/// # 示例
///
/// ```rust
/// use anvilkit_app::state::{GameState, in_state};
///
/// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
/// enum MyState { #[default] Menu, Playing }
///
/// // 系统仅在 Playing 状态下运行:
/// // app.add_systems(Update, my_system.run_if(in_state(MyState::Playing)));
/// ```
pub fn in_state<S: StateValue>(state: S) -> impl Fn(Option<Res<GameState<S>>>) -> bool + Clone {
    move |current: Option<Res<GameState<S>>>| {
        current.map_or(false, |s| s.0 == state)
    }
}

/// 状态转换系统
///
/// 检查 `NextGameState<S>`，如果有待处理的转换请求，
/// 更新 `GameState<S>` 并清除请求。
///
/// 应注册在 `PreUpdate` 阶段，确保状态在 Update 系统之前已更新。
pub fn state_transition_system<S: StateValue>(
    mut current: ResMut<GameState<S>>,
    mut next: ResMut<NextGameState<S>>,
    mut events: EventWriter<StateTransitionEvent<S>>,
) {
    if let Some(new_state) = next.0.take() {
        if current.0 != new_state {
            let from = current.0;
            log::debug!("状态转换: {:?} → {:?}", from, new_state);
            current.0 = new_state;
            events.send(StateTransitionEvent { from, to: new_state });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    enum TestState {
        #[default]
        Menu,
        Playing,
        Paused,
    }

    #[test]
    fn test_game_state_default() {
        let state = GameState::<TestState>::default();
        assert_eq!(state.0, TestState::Menu);
    }

    #[test]
    fn test_next_state_set_clear() {
        let mut next = NextGameState::<TestState>::default();
        assert_eq!(next.0, None);

        next.set(TestState::Playing);
        assert_eq!(next.0, Some(TestState::Playing));

        next.clear();
        assert_eq!(next.0, None);
    }

    #[test]
    fn test_in_state_condition() {
        // in_state returns a closure — verify it type-checks
        let _check = in_state(TestState::Playing);
    }

    #[test]
    fn test_state_transition_system() {
        use crate::ecs_app::App;
        use crate::ecs_plugin::AnvilKitEcsPlugin;
        use crate::schedule::AnvilKitSchedule;
        use bevy_app::Plugin;

        let mut app = App::new();
        app.add_plugins(AnvilKitEcsPlugin);
        app.insert_resource(GameState(TestState::Menu));
        app.insert_resource(NextGameState::<TestState>::default());
        app.add_event::<StateTransitionEvent<TestState>>();
        app.add_systems(AnvilKitSchedule::PreUpdate, state_transition_system::<TestState>);

        // No transition requested — state stays
        app.update();
        assert_eq!(app.world().resource::<GameState<TestState>>().0, TestState::Menu);

        // Request transition
        app.world_mut().resource_mut::<NextGameState<TestState>>().set(TestState::Playing);
        app.update();
        assert_eq!(app.world().resource::<GameState<TestState>>().0, TestState::Playing);

        // Next should be cleared
        assert_eq!(app.world().resource::<NextGameState<TestState>>().0, None);

        // Verify event was emitted
        let events = app.world().resource::<Events<StateTransitionEvent<TestState>>>();
        let mut reader = events.get_cursor();
        let transition_events: Vec<_> = reader.read(events).collect();
        assert_eq!(transition_events.len(), 1);
        assert_eq!(transition_events[0].from, TestState::Menu);
        assert_eq!(transition_events[0].to, TestState::Playing);
    }
}
