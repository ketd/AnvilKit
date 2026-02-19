//! # 输入状态
//!
//! 追踪键盘按键和鼠标按钮的当前帧状态和上一帧状态，
//! 支持 pressed / just_pressed / just_released 查询。

use std::collections::HashSet;
use bevy_ecs::prelude::*;
use glam::Vec2;

/// 键盘键码
///
/// 常用键的枚举，与 winit VirtualKeyCode 对应。
///
/// # 示例
///
/// ```rust
/// use anvilkit_input::input_state::KeyCode;
/// let key = KeyCode::W;
/// assert_ne!(key, KeyCode::S);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyCode {
    // Letters
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    // Numbers
    Key0, Key1, Key2, Key3, Key4,
    Key5, Key6, Key7, Key8, Key9,
    // Function keys
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    // Special keys
    Space, Enter, Escape, Tab, Backspace, Delete,
    Left, Right, Up, Down,
    LShift, RShift, LControl, RControl, LAlt, RAlt,
}

/// 鼠标按钮
///
/// # 示例
///
/// ```rust
/// use anvilkit_input::input_state::MouseButton;
/// let btn = MouseButton::Left;
/// assert_ne!(btn, MouseButton::Right);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

/// 输入状态资源
///
/// 每帧追踪键盘和鼠标的完整输入状态。
/// 支持 pressed（持续按下）、just_pressed（本帧按下）、just_released（本帧松开）查询。
///
/// # 示例
///
/// ```rust
/// use anvilkit_input::input_state::{InputState, KeyCode, MouseButton};
///
/// let mut input = InputState::new();
///
/// // 模拟按键
/// input.press_key(KeyCode::W);
/// assert!(input.is_key_pressed(KeyCode::W));
/// assert!(input.is_key_just_pressed(KeyCode::W));
///
/// // 帧结束
/// input.end_frame();
/// assert!(input.is_key_pressed(KeyCode::W));
/// assert!(!input.is_key_just_pressed(KeyCode::W));
///
/// // 松开
/// input.release_key(KeyCode::W);
/// assert!(!input.is_key_pressed(KeyCode::W));
/// assert!(input.is_key_just_released(KeyCode::W));
/// ```
#[derive(Resource)]
pub struct InputState {
    /// 当前帧按下的键
    keys_pressed: HashSet<KeyCode>,
    /// 本帧新按下的键
    keys_just_pressed: HashSet<KeyCode>,
    /// 本帧刚松开的键
    keys_just_released: HashSet<KeyCode>,

    /// 当前帧按下的鼠标按钮
    mouse_pressed: HashSet<MouseButton>,
    /// 本帧新按下的鼠标按钮
    mouse_just_pressed: HashSet<MouseButton>,
    /// 本帧刚松开的鼠标按钮
    mouse_just_released: HashSet<MouseButton>,

    /// 鼠标位置（像素坐标）
    mouse_position: Vec2,
    /// 鼠标本帧移动量（像素）
    mouse_delta: Vec2,
    /// 滚轮本帧滚动量（行数）
    scroll_delta: f32,
}

impl InputState {
    /// 创建空的输入状态
    pub fn new() -> Self {
        Self {
            keys_pressed: HashSet::new(),
            keys_just_pressed: HashSet::new(),
            keys_just_released: HashSet::new(),
            mouse_pressed: HashSet::new(),
            mouse_just_pressed: HashSet::new(),
            mouse_just_released: HashSet::new(),
            mouse_position: Vec2::ZERO,
            mouse_delta: Vec2::ZERO,
            scroll_delta: 0.0,
        }
    }

    // --- Keyboard ---

    /// 记录按键按下
    pub fn press_key(&mut self, key: KeyCode) {
        if self.keys_pressed.insert(key) {
            self.keys_just_pressed.insert(key);
        }
    }

    /// 记录按键松开
    pub fn release_key(&mut self, key: KeyCode) {
        if self.keys_pressed.remove(&key) {
            self.keys_just_released.insert(key);
        }
    }

    /// 键是否正在按下
    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.keys_pressed.contains(&key)
    }

    /// 键是否本帧刚按下
    pub fn is_key_just_pressed(&self, key: KeyCode) -> bool {
        self.keys_just_pressed.contains(&key)
    }

    /// 键是否本帧刚松开
    pub fn is_key_just_released(&self, key: KeyCode) -> bool {
        self.keys_just_released.contains(&key)
    }

    // --- Mouse buttons ---

    /// 记录鼠标按钮按下
    pub fn press_mouse(&mut self, button: MouseButton) {
        if self.mouse_pressed.insert(button) {
            self.mouse_just_pressed.insert(button);
        }
    }

    /// 记录鼠标按钮松开
    pub fn release_mouse(&mut self, button: MouseButton) {
        if self.mouse_pressed.remove(&button) {
            self.mouse_just_released.insert(button);
        }
    }

    /// 鼠标按钮是否正在按下
    pub fn is_mouse_pressed(&self, button: MouseButton) -> bool {
        self.mouse_pressed.contains(&button)
    }

    /// 鼠标按钮是否本帧刚按下
    pub fn is_mouse_just_pressed(&self, button: MouseButton) -> bool {
        self.mouse_just_pressed.contains(&button)
    }

    /// 鼠标按钮是否本帧刚松开
    pub fn is_mouse_just_released(&self, button: MouseButton) -> bool {
        self.mouse_just_released.contains(&button)
    }

    // --- Mouse position/motion ---

    /// 设置鼠标位置
    pub fn set_mouse_position(&mut self, position: Vec2) {
        self.mouse_position = position;
    }

    /// 累加鼠标移动量
    pub fn add_mouse_delta(&mut self, delta: Vec2) {
        self.mouse_delta += delta;
    }

    /// 获取鼠标位置
    pub fn mouse_position(&self) -> Vec2 {
        self.mouse_position
    }

    /// 获取本帧鼠标移动量
    pub fn mouse_delta(&self) -> Vec2 {
        self.mouse_delta
    }

    /// 累加滚轮滚动量
    pub fn add_scroll_delta(&mut self, delta: f32) {
        self.scroll_delta += delta;
    }

    /// 获取本帧滚轮滚动量
    pub fn scroll_delta(&self) -> f32 {
        self.scroll_delta
    }

    // --- Frame lifecycle ---

    /// 帧结束，清除 just_pressed / just_released / delta 状态
    pub fn end_frame(&mut self) {
        self.keys_just_pressed.clear();
        self.keys_just_released.clear();
        self.mouse_just_pressed.clear();
        self.mouse_just_released.clear();
        self.mouse_delta = Vec2::ZERO;
        self.scroll_delta = 0.0;
    }

    /// 获取当前按下的所有键
    pub fn pressed_keys(&self) -> &HashSet<KeyCode> {
        &self.keys_pressed
    }
}

impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_press_release() {
        let mut input = InputState::new();

        input.press_key(KeyCode::W);
        assert!(input.is_key_pressed(KeyCode::W));
        assert!(input.is_key_just_pressed(KeyCode::W));
        assert!(!input.is_key_just_released(KeyCode::W));

        input.end_frame();
        assert!(input.is_key_pressed(KeyCode::W));
        assert!(!input.is_key_just_pressed(KeyCode::W));

        input.release_key(KeyCode::W);
        assert!(!input.is_key_pressed(KeyCode::W));
        assert!(input.is_key_just_released(KeyCode::W));

        input.end_frame();
        assert!(!input.is_key_just_released(KeyCode::W));
    }

    #[test]
    fn test_mouse_buttons() {
        let mut input = InputState::new();

        input.press_mouse(MouseButton::Left);
        assert!(input.is_mouse_pressed(MouseButton::Left));
        assert!(input.is_mouse_just_pressed(MouseButton::Left));

        input.end_frame();
        assert!(input.is_mouse_pressed(MouseButton::Left));
        assert!(!input.is_mouse_just_pressed(MouseButton::Left));
    }

    #[test]
    fn test_mouse_delta() {
        let mut input = InputState::new();
        input.add_mouse_delta(Vec2::new(5.0, 3.0));
        input.add_mouse_delta(Vec2::new(2.0, -1.0));
        assert_eq!(input.mouse_delta(), Vec2::new(7.0, 2.0));

        input.end_frame();
        assert_eq!(input.mouse_delta(), Vec2::ZERO);
    }

    #[test]
    fn test_scroll_delta() {
        let mut input = InputState::new();
        input.add_scroll_delta(1.5);
        input.add_scroll_delta(-0.5);
        assert_eq!(input.scroll_delta(), 1.0);

        input.end_frame();
        assert_eq!(input.scroll_delta(), 0.0);
    }

    #[test]
    fn test_duplicate_press() {
        let mut input = InputState::new();
        input.press_key(KeyCode::A);
        input.press_key(KeyCode::A); // duplicate
        assert_eq!(input.pressed_keys().len(), 1);
    }
}
