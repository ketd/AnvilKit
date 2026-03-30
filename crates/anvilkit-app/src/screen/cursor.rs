use bevy_ecs::system::Resource;

/// Desired cursor mode, synced to the window each frame by [`AnvilKitApp`].
///
/// Set this resource from ECS systems (typically via [`ScreenPlugin`]) and the
/// app runner will apply the corresponding grab/visibility state to the window.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CursorMode {
    /// Cursor visible, not grabbed — for menus and UI screens.
    #[default]
    Free,
    /// Cursor invisible and grabbed — for FPS-style gameplay.
    Locked,
}
