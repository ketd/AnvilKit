//! # 输入状态
//!
//! 追踪键盘按键和鼠标按钮的当前帧状态和上一帧状态，
//! 支持 pressed / just_pressed / just_released 查询。

use std::collections::HashSet;
use bevy_ecs::prelude::*;
use glam::Vec2;
use anvilkit_describe::Describe;

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Describe)]
/// Keyboard key codes.
pub enum KeyCode {
    // Letters
    /// The A key.
    A,
    /// The B key.
    B,
    /// The C key.
    C,
    /// The D key.
    D,
    /// The E key.
    E,
    /// The F key.
    F,
    /// The G key.
    G,
    /// The H key.
    H,
    /// The I key.
    I,
    /// The J key.
    J,
    /// The K key.
    K,
    /// The L key.
    L,
    /// The M key.
    M,
    /// The N key.
    N,
    /// The O key.
    O,
    /// The P key.
    P,
    /// The Q key.
    Q,
    /// The R key.
    R,
    /// The S key.
    S,
    /// The T key.
    T,
    /// The U key.
    U,
    /// The V key.
    V,
    /// The W key.
    W,
    /// The X key.
    X,
    /// The Y key.
    Y,
    /// The Z key.
    Z,
    // Numbers
    /// The 0 key.
    Key0,
    /// The 1 key.
    Key1,
    /// The 2 key.
    Key2,
    /// The 3 key.
    Key3,
    /// The 4 key.
    Key4,
    /// The 5 key.
    Key5,
    /// The 6 key.
    Key6,
    /// The 7 key.
    Key7,
    /// The 8 key.
    Key8,
    /// The 9 key.
    Key9,
    // Function keys
    /// The F1 key.
    F1,
    /// The F2 key.
    F2,
    /// The F3 key.
    F3,
    /// The F4 key.
    F4,
    /// The F5 key.
    F5,
    /// The F6 key.
    F6,
    /// The F7 key.
    F7,
    /// The F8 key.
    F8,
    /// The F9 key.
    F9,
    /// The F10 key.
    F10,
    /// The F11 key.
    F11,
    /// The F12 key.
    F12,
    // Special keys
    /// The Space key.
    Space,
    /// The Enter key.
    Enter,
    /// The Escape key.
    Escape,
    /// The Tab key.
    Tab,
    /// The Backspace key.
    Backspace,
    /// The Delete key.
    Delete,
    /// The Left arrow key.
    Left,
    /// The Right arrow key.
    Right,
    /// The Up arrow key.
    Up,
    /// The Down arrow key.
    Down,
    /// The left Shift key.
    LShift,
    /// The right Shift key.
    RShift,
    /// The left Control key.
    LControl,
    /// The right Control key.
    RControl,
    /// The left Alt key.
    LAlt,
    /// The right Alt key.
    RAlt,
}

impl KeyCode {
    /// Parse a human-readable key name string into a `KeyCode`.
    ///
    /// Recognises single-letter keys (`"A"` .. `"Z"`), digit keys (`"Key0"` .. `"Key9"`),
    /// function keys (`"F1"` .. `"F12"`), arrow keys, and common modifier / special keys.
    /// Returns `None` for unrecognised names.
    pub fn from_name(name: &str) -> Option<KeyCode> {
        match name {
            // Letters
            "A" => Some(KeyCode::A), "B" => Some(KeyCode::B),
            "C" => Some(KeyCode::C), "D" => Some(KeyCode::D),
            "E" => Some(KeyCode::E), "F" => Some(KeyCode::F),
            "G" => Some(KeyCode::G), "H" => Some(KeyCode::H),
            "I" => Some(KeyCode::I), "J" => Some(KeyCode::J),
            "K" => Some(KeyCode::K), "L" => Some(KeyCode::L),
            "M" => Some(KeyCode::M), "N" => Some(KeyCode::N),
            "O" => Some(KeyCode::O), "P" => Some(KeyCode::P),
            "Q" => Some(KeyCode::Q), "R" => Some(KeyCode::R),
            "S" => Some(KeyCode::S), "T" => Some(KeyCode::T),
            "U" => Some(KeyCode::U), "V" => Some(KeyCode::V),
            "W" => Some(KeyCode::W), "X" => Some(KeyCode::X),
            "Y" => Some(KeyCode::Y), "Z" => Some(KeyCode::Z),
            // Digit keys
            "Key0" => Some(KeyCode::Key0), "Key1" => Some(KeyCode::Key1),
            "Key2" => Some(KeyCode::Key2), "Key3" => Some(KeyCode::Key3),
            "Key4" => Some(KeyCode::Key4), "Key5" => Some(KeyCode::Key5),
            "Key6" => Some(KeyCode::Key6), "Key7" => Some(KeyCode::Key7),
            "Key8" => Some(KeyCode::Key8), "Key9" => Some(KeyCode::Key9),
            // Function keys
            "F1" => Some(KeyCode::F1), "F2" => Some(KeyCode::F2),
            "F3" => Some(KeyCode::F3), "F4" => Some(KeyCode::F4),
            "F5" => Some(KeyCode::F5), "F6" => Some(KeyCode::F6),
            "F7" => Some(KeyCode::F7), "F8" => Some(KeyCode::F8),
            "F9" => Some(KeyCode::F9), "F10" => Some(KeyCode::F10),
            "F11" => Some(KeyCode::F11), "F12" => Some(KeyCode::F12),
            // Special keys
            "Space" => Some(KeyCode::Space), "Enter" => Some(KeyCode::Enter),
            "Escape" => Some(KeyCode::Escape), "Tab" => Some(KeyCode::Tab),
            "Backspace" => Some(KeyCode::Backspace), "Delete" => Some(KeyCode::Delete),
            // Arrow keys
            "Left" => Some(KeyCode::Left), "Right" => Some(KeyCode::Right),
            "Up" => Some(KeyCode::Up), "Down" => Some(KeyCode::Down),
            // Modifiers
            "LShift" => Some(KeyCode::LShift), "RShift" => Some(KeyCode::RShift),
            "LControl" => Some(KeyCode::LControl), "RControl" => Some(KeyCode::RControl),
            "LAlt" => Some(KeyCode::LAlt), "RAlt" => Some(KeyCode::RAlt),
            _ => None,
        }
    }

    /// 将 winit KeyCode 映射到 AnvilKit KeyCode
    pub fn from_winit(key: winit::keyboard::KeyCode) -> Option<KeyCode> {
        use winit::keyboard::KeyCode as WK;
        match key {
            WK::KeyA => Some(KeyCode::A), WK::KeyB => Some(KeyCode::B),
            WK::KeyC => Some(KeyCode::C), WK::KeyD => Some(KeyCode::D),
            WK::KeyE => Some(KeyCode::E), WK::KeyF => Some(KeyCode::F),
            WK::KeyG => Some(KeyCode::G), WK::KeyH => Some(KeyCode::H),
            WK::KeyI => Some(KeyCode::I), WK::KeyJ => Some(KeyCode::J),
            WK::KeyK => Some(KeyCode::K), WK::KeyL => Some(KeyCode::L),
            WK::KeyM => Some(KeyCode::M), WK::KeyN => Some(KeyCode::N),
            WK::KeyO => Some(KeyCode::O), WK::KeyP => Some(KeyCode::P),
            WK::KeyQ => Some(KeyCode::Q), WK::KeyR => Some(KeyCode::R),
            WK::KeyS => Some(KeyCode::S), WK::KeyT => Some(KeyCode::T),
            WK::KeyU => Some(KeyCode::U), WK::KeyV => Some(KeyCode::V),
            WK::KeyW => Some(KeyCode::W), WK::KeyX => Some(KeyCode::X),
            WK::KeyY => Some(KeyCode::Y), WK::KeyZ => Some(KeyCode::Z),
            WK::Digit0 => Some(KeyCode::Key0), WK::Digit1 => Some(KeyCode::Key1),
            WK::Digit2 => Some(KeyCode::Key2), WK::Digit3 => Some(KeyCode::Key3),
            WK::Digit4 => Some(KeyCode::Key4), WK::Digit5 => Some(KeyCode::Key5),
            WK::Digit6 => Some(KeyCode::Key6), WK::Digit7 => Some(KeyCode::Key7),
            WK::Digit8 => Some(KeyCode::Key8), WK::Digit9 => Some(KeyCode::Key9),
            WK::F1 => Some(KeyCode::F1), WK::F2 => Some(KeyCode::F2),
            WK::F3 => Some(KeyCode::F3), WK::F4 => Some(KeyCode::F4),
            WK::F5 => Some(KeyCode::F5), WK::F6 => Some(KeyCode::F6),
            WK::F7 => Some(KeyCode::F7), WK::F8 => Some(KeyCode::F8),
            WK::F9 => Some(KeyCode::F9), WK::F10 => Some(KeyCode::F10),
            WK::F11 => Some(KeyCode::F11), WK::F12 => Some(KeyCode::F12),
            WK::Space => Some(KeyCode::Space), WK::Enter => Some(KeyCode::Enter),
            WK::Escape => Some(KeyCode::Escape), WK::Tab => Some(KeyCode::Tab),
            WK::Backspace => Some(KeyCode::Backspace), WK::Delete => Some(KeyCode::Delete),
            WK::ArrowLeft => Some(KeyCode::Left), WK::ArrowRight => Some(KeyCode::Right),
            WK::ArrowUp => Some(KeyCode::Up), WK::ArrowDown => Some(KeyCode::Down),
            WK::ShiftLeft => Some(KeyCode::LShift), WK::ShiftRight => Some(KeyCode::RShift),
            WK::ControlLeft => Some(KeyCode::LControl), WK::ControlRight => Some(KeyCode::RControl),
            WK::AltLeft => Some(KeyCode::LAlt), WK::AltRight => Some(KeyCode::RAlt),
            _ => None,
        }
    }
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Describe)]
/// Mouse button identifier.
pub enum MouseButton {
    /// Left mouse button.
    Left,
    /// Right mouse button.
    Right,
    /// Middle mouse button.
    Middle,
}

impl MouseButton {
    /// 将 winit MouseButton 映射到 AnvilKit MouseButton
    pub fn from_winit(button: winit::event::MouseButton) -> Option<MouseButton> {
        match button {
            winit::event::MouseButton::Left => Some(MouseButton::Left),
            winit::event::MouseButton::Right => Some(MouseButton::Right),
            winit::event::MouseButton::Middle => Some(MouseButton::Middle),
            _ => None,
        }
    }
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
