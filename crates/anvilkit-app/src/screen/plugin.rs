use bevy_ecs::prelude::*;

use crate::ecs_app::App;
use crate::schedule::AnvilKitSchedule;
use crate::state::{
    GameState, NextGameState, StateTransitionEvent, StateValue, state_transition_system,
};

use super::cursor::CursorMode;

/// Configuration resource storing which states require a locked cursor.
#[derive(Resource)]
pub struct ScreenPluginConfig<S: StateValue> {
    pub locked_states: Vec<S>,
}

/// Plugin that wires up game-state management with automatic cursor control.
///
/// Registers [`GameState<S>`], [`NextGameState<S>`], [`StateTransitionEvent<S>`],
/// [`state_transition_system`], and a cursor-sync system that keeps [`CursorMode`]
/// in sync with the current state.
///
/// # Example
///
/// ```ignore
/// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
/// enum Screen { #[default] Menu, Playing, Paused }
///
/// ScreenPlugin::new(Screen::Menu)
///     .lock_cursor_in(Screen::Playing)
///     .build(&mut app);
///
/// // Now systems can use `in_state(Screen::Playing)` as a run condition,
/// // and the cursor is automatically grabbed when Playing.
/// ```
pub struct ScreenPlugin<S: StateValue> {
    initial: S,
    locked_states: Vec<S>,
}

impl<S: StateValue> ScreenPlugin<S> {
    /// Create a new plugin with the given initial screen state.
    pub fn new(initial: S) -> Self {
        Self {
            initial,
            locked_states: Vec::new(),
        }
    }

    /// Mark a state as requiring a locked (grabbed, invisible) cursor.
    /// Call multiple times for multiple locked states.
    pub fn lock_cursor_in(mut self, state: S) -> Self {
        self.locked_states.push(state);
        self
    }

    /// Register all resources, events, and systems on the App.
    pub fn build(self, app: &mut App) {
        app.insert_resource(GameState(self.initial));
        app.insert_resource(NextGameState::<S>::default());
        app.insert_resource(CursorMode::default());
        app.insert_resource(ScreenPluginConfig::<S> {
            locked_states: self.locked_states,
        });
        app.add_event::<StateTransitionEvent<S>>();
        app.add_systems(AnvilKitSchedule::PreUpdate, state_transition_system::<S>);
        app.add_systems(AnvilKitSchedule::PostUpdate, cursor_sync_system::<S>);
    }
}

/// System that reads the current game state and updates [`CursorMode`] accordingly.
fn cursor_sync_system<S: StateValue>(
    state: Res<GameState<S>>,
    config: Res<ScreenPluginConfig<S>>,
    mut cursor: ResMut<CursorMode>,
) {
    let should_lock = config.locked_states.contains(&state.0);
    let new_mode = if should_lock {
        CursorMode::Locked
    } else {
        CursorMode::Free
    };
    if *cursor != new_mode {
        *cursor = new_mode;
    }
}
