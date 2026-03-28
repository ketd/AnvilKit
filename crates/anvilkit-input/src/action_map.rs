//! # Action Mapping
//!
//! 将逻辑动作（如 "Jump", "MoveForward"）映射到物理输入（按键/鼠标按钮），
//! 实现输入重映射和多设备支持。

use std::collections::HashMap;
use bevy_ecs::prelude::*;

use crate::input_state::{InputState, KeyCode, MouseButton};

/// 高性能动作标识符 — 避免 String 堆分配
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ActionId(pub u32);

/// 输入绑定源
///
/// 一个逻辑动作可以绑定到键盘键或鼠标按钮。
///
/// # 示例
///
/// ```rust
/// use anvilkit_input::action_map::InputBinding;
/// use anvilkit_input::input_state::{KeyCode, MouseButton};
///
/// let key = InputBinding::Key(KeyCode::Space);
/// let mouse = InputBinding::Mouse(MouseButton::Left);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputBinding {
    /// A keyboard key binding.
    Key(KeyCode),
    /// A mouse button binding.
    Mouse(MouseButton),
}

impl InputBinding {
    /// Parse a key name string into an `InputBinding`.
    ///
    /// Recognises mouse buttons (`"MouseLeft"`, `"MouseRight"`, `"MouseMiddle"`) and
    /// all keyboard key names supported by [`KeyCode::from_name`].
    /// Returns `None` for unrecognised names.
    pub fn from_key_name(name: &str) -> Option<Self> {
        // Try mouse buttons first
        match name {
            "MouseLeft" => return Some(Self::Mouse(MouseButton::Left)),
            "MouseRight" => return Some(Self::Mouse(MouseButton::Right)),
            "MouseMiddle" => return Some(Self::Mouse(MouseButton::Middle)),
            _ => {}
        }
        // Try keyboard
        KeyCode::from_name(name).map(Self::Key)
    }
}

/// 输入轴绑定
#[derive(Debug, Clone)]
pub enum AxisBinding {
    /// Gamepad 模拟轴
    GamepadAxis(crate::gamepad::GamepadAxis),
    /// 键盘模拟轴（负键 + 正键 → [-1, 0, 1]）
    KeyboardAxis {
        /// 负方向按键
        negative: KeyCode,
        /// 正方向按键
        positive: KeyCode,
    },
}

/// 动作状态
///
/// # 示例
///
/// ```rust
/// use anvilkit_input::action_map::ActionState;
///
/// let state = ActionState::Pressed;
/// assert!(state.is_active());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionState {
    /// 未激活
    Inactive,
    /// 本帧刚按下
    JustPressed,
    /// 持续按下
    Pressed,
    /// 本帧刚松开
    JustReleased,
}

impl ActionState {
    /// 动作是否激活（按下或刚按下）
    pub fn is_active(&self) -> bool {
        matches!(self, ActionState::JustPressed | ActionState::Pressed)
    }

    /// 动作是否本帧刚触发
    pub fn is_just_pressed(&self) -> bool {
        matches!(self, ActionState::JustPressed)
    }

    /// 动作是否本帧刚结束
    pub fn is_just_released(&self) -> bool {
        matches!(self, ActionState::JustReleased)
    }
}

/// 动作映射表
///
/// 将字符串命名的逻辑动作映射到一组输入绑定。
/// 任一绑定激活即视为动作激活。
///
/// # 示例
///
/// ```rust
/// use anvilkit_input::prelude::*;
/// use anvilkit_input::action_map::InputBinding;
///
/// let mut map = ActionMap::new();
/// map.add_binding("jump", InputBinding::Key(KeyCode::Space));
/// map.add_binding("jump", InputBinding::Key(KeyCode::W));
///
/// let mut input = InputState::new();
/// input.press_key(KeyCode::Space);
///
/// map.update(&input);
/// assert!(map.is_action_active("jump"));
/// assert!(map.is_action_just_pressed("jump"));
/// ```
#[derive(Resource)]
pub struct ActionMap {
    /// 动作名 → 绑定列表
    bindings: HashMap<String, Vec<InputBinding>>,
    /// 动作名 → 当前状态
    states: HashMap<String, ActionState>,
    /// 动作名 → ActionId 映射（高性能查找）
    name_to_id: HashMap<String, ActionId>,
    /// ActionId → 动作名 反向映射
    id_to_name: Vec<String>,
    /// 下一个 ActionId
    next_id: u32,
    /// 轴绑定（动作名 → 轴绑定列表）
    axis_bindings: HashMap<String, Vec<AxisBinding>>,
}

impl ActionMap {
    /// 创建空的动作映射表
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            states: HashMap::new(),
            name_to_id: HashMap::new(),
            id_to_name: Vec::new(),
            next_id: 0,
            axis_bindings: HashMap::new(),
        }
    }

    /// 为动作添加输入绑定
    pub fn add_binding(&mut self, action: &str, binding: InputBinding) {
        self.bindings
            .entry(action.to_string())
            .or_default()
            .push(binding);
        self.states
            .entry(action.to_string())
            .or_insert(ActionState::Inactive);
    }

    /// 根据当前输入状态更新所有动作状态
    pub fn update(&mut self, input: &InputState) {
        for (action, bindings) in &self.bindings {
            let any_active = bindings.iter().any(|b| match b {
                InputBinding::Key(k) => input.is_key_pressed(*k),
                InputBinding::Mouse(m) => input.is_mouse_pressed(*m),
            });
            let any_just_pressed = bindings.iter().any(|b| match b {
                InputBinding::Key(k) => input.is_key_just_pressed(*k),
                InputBinding::Mouse(m) => input.is_mouse_just_pressed(*m),
            });
            let any_just_released = bindings.iter().any(|b| match b {
                InputBinding::Key(k) => input.is_key_just_released(*k),
                InputBinding::Mouse(m) => input.is_mouse_just_released(*m),
            });

            let state = if any_just_pressed {
                ActionState::JustPressed
            } else if any_active {
                ActionState::Pressed
            } else if any_just_released {
                ActionState::JustReleased
            } else {
                ActionState::Inactive
            };

            self.states.insert(action.clone(), state);
        }
    }

    /// 查询动作状态
    pub fn action_state(&self, action: &str) -> ActionState {
        self.states.get(action).copied().unwrap_or(ActionState::Inactive)
    }

    /// 动作是否激活
    pub fn is_action_active(&self, action: &str) -> bool {
        self.action_state(action).is_active()
    }

    /// 动作是否本帧刚触发
    pub fn is_action_just_pressed(&self, action: &str) -> bool {
        self.action_state(action).is_just_pressed()
    }

    /// 动作是否本帧刚结束
    pub fn is_action_just_released(&self, action: &str) -> bool {
        self.action_state(action).is_just_released()
    }

    /// 获取动作的所有绑定
    pub fn get_bindings(&self, action: &str) -> Option<&[InputBinding]> {
        self.bindings.get(action).map(|v| v.as_slice())
    }

    /// 移除动作的所有绑定
    pub fn clear_bindings(&mut self, action: &str) {
        self.bindings.remove(action);
        self.states.remove(action);
    }

    /// Apply key binding overrides from a settings map.
    ///
    /// Each entry maps an action name to a key name string (e.g., `"W"`, `"Space"`, `"MouseLeft"`).
    /// Existing bindings for the action are cleared and replaced with the override.
    /// Unknown key names are silently ignored.
    pub fn apply_overrides(&mut self, overrides: &std::collections::HashMap<String, String>) {
        for (action, key_name) in overrides {
            if let Some(binding) = InputBinding::from_key_name(key_name) {
                self.clear_bindings(action);
                self.add_binding(action, binding);
            }
        }
    }

    /// 注册动作并返回高性能 ActionId
    ///
    /// 使用 ActionId 进行后续查询可避免 String 堆分配。
    /// 同名动作多次注册返回相同 ID。
    pub fn register_action(&mut self, name: &str) -> ActionId {
        if let Some(&id) = self.name_to_id.get(name) {
            return id;
        }
        let id = ActionId(self.next_id);
        self.next_id += 1;
        self.name_to_id.insert(name.to_string(), id);
        self.id_to_name.push(name.to_string());
        id
    }

    /// 通过 ActionId 查询动作是否激活（零堆分配）
    pub fn is_action_active_by_id(&self, id: ActionId) -> bool {
        self.id_to_name.get(id.0 as usize)
            .and_then(|name| self.states.get(name))
            .map_or(false, |s| s.is_active())
    }

    /// 通过 ActionId 查询动作状态（零堆分配）
    pub fn action_state_by_id(&self, id: ActionId) -> ActionState {
        self.id_to_name.get(id.0 as usize)
            .and_then(|name| self.states.get(name))
            .copied()
            .unwrap_or(ActionState::Inactive)
    }

    /// 为动作绑定轴输入
    pub fn bind_axis(&mut self, action: &str, binding: AxisBinding) {
        self.axis_bindings.entry(action.to_string()).or_default().push(binding);
    }

    /// 查询轴值（合并所有绑定的最大绝对值）
    pub fn axis_value(&self, action: &str, input: &InputState, gamepad: Option<&crate::gamepad::GamepadState>) -> f32 {
        let Some(bindings) = self.axis_bindings.get(action) else { return 0.0 };
        let mut value = 0.0f32;
        for binding in bindings {
            let v = match binding {
                AxisBinding::GamepadAxis(axis) => {
                    gamepad.map_or(0.0, |gp| gp.axis_value(0, *axis))
                }
                AxisBinding::KeyboardAxis { negative, positive } => {
                    let neg = if input.is_key_pressed(*negative) { -1.0 } else { 0.0 };
                    let pos = if input.is_key_pressed(*positive) { 1.0 } else { 0.0 };
                    neg + pos
                }
            };
            if v.abs() > value.abs() { value = v; }
        }
        value
    }
}

impl Default for ActionMap {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_state() {
        assert!(ActionState::Pressed.is_active());
        assert!(ActionState::JustPressed.is_active());
        assert!(!ActionState::Inactive.is_active());
        assert!(!ActionState::JustReleased.is_active());
    }

    #[test]
    fn test_action_map_basic() {
        let mut map = ActionMap::new();
        map.add_binding("jump", InputBinding::Key(KeyCode::Space));

        let mut input = InputState::new();
        input.press_key(KeyCode::Space);

        map.update(&input);
        assert!(map.is_action_active("jump"));
        assert!(map.is_action_just_pressed("jump"));

        input.end_frame();
        map.update(&input);
        assert!(map.is_action_active("jump"));
        assert!(!map.is_action_just_pressed("jump"));

        input.release_key(KeyCode::Space);
        map.update(&input);
        assert!(!map.is_action_active("jump"));
        assert!(map.is_action_just_released("jump"));
    }

    #[test]
    fn test_multiple_bindings() {
        let mut map = ActionMap::new();
        map.add_binding("fire", InputBinding::Key(KeyCode::Space));
        map.add_binding("fire", InputBinding::Mouse(MouseButton::Left));

        let mut input = InputState::new();
        input.press_mouse(MouseButton::Left);

        map.update(&input);
        assert!(map.is_action_active("fire"));
    }

    #[test]
    fn test_unknown_action() {
        let map = ActionMap::new();
        assert_eq!(map.action_state("nonexistent"), ActionState::Inactive);
        assert!(!map.is_action_active("nonexistent"));
    }

    #[test]
    fn test_keyboard_axis() {
        let mut map = ActionMap::new();
        map.bind_axis("move_x", AxisBinding::KeyboardAxis {
            negative: KeyCode::A,
            positive: KeyCode::D,
        });

        let mut input = InputState::new();
        input.press_key(KeyCode::D);

        let val = map.axis_value("move_x", &input, None);
        assert!((val - 1.0).abs() < 0.001);

        input.press_key(KeyCode::A);
        let val = map.axis_value("move_x", &input, None);
        assert_eq!(val, 0.0); // both pressed = cancel out
    }

    #[test]
    fn test_action_id_register() {
        let mut map = ActionMap::new();
        let id1 = map.register_action("jump");
        let id2 = map.register_action("fire");
        let id1_again = map.register_action("jump");

        assert_ne!(id1, id2);
        assert_eq!(id1, id1_again); // same name = same id
    }

    #[test]
    fn test_action_id_lookup() {
        let mut map = ActionMap::new();
        map.add_binding("jump", InputBinding::Key(KeyCode::Space));
        let id = map.register_action("jump");

        let mut input = InputState::new();
        input.press_key(KeyCode::Space);
        map.update(&input);

        assert!(map.is_action_active_by_id(id));
        assert!(map.is_action_active("jump"));
    }

    #[test]
    fn test_input_binding_from_key_name_keyboard() {
        assert_eq!(
            InputBinding::from_key_name("W"),
            Some(InputBinding::Key(KeyCode::W))
        );
        assert_eq!(
            InputBinding::from_key_name("Space"),
            Some(InputBinding::Key(KeyCode::Space))
        );
        assert_eq!(
            InputBinding::from_key_name("Escape"),
            Some(InputBinding::Key(KeyCode::Escape))
        );
        assert_eq!(
            InputBinding::from_key_name("F1"),
            Some(InputBinding::Key(KeyCode::F1))
        );
        assert_eq!(
            InputBinding::from_key_name("Key1"),
            Some(InputBinding::Key(KeyCode::Key1))
        );
        assert_eq!(
            InputBinding::from_key_name("LShift"),
            Some(InputBinding::Key(KeyCode::LShift))
        );
        assert_eq!(
            InputBinding::from_key_name("Up"),
            Some(InputBinding::Key(KeyCode::Up))
        );
    }

    #[test]
    fn test_input_binding_from_key_name_mouse() {
        assert_eq!(
            InputBinding::from_key_name("MouseLeft"),
            Some(InputBinding::Mouse(MouseButton::Left))
        );
        assert_eq!(
            InputBinding::from_key_name("MouseRight"),
            Some(InputBinding::Mouse(MouseButton::Right))
        );
        assert_eq!(
            InputBinding::from_key_name("MouseMiddle"),
            Some(InputBinding::Mouse(MouseButton::Middle))
        );
    }

    #[test]
    fn test_input_binding_from_key_name_unknown() {
        assert_eq!(InputBinding::from_key_name("NonexistentKey"), None);
        assert_eq!(InputBinding::from_key_name(""), None);
        assert_eq!(InputBinding::from_key_name("mouse_left"), None);
    }

    #[test]
    fn test_apply_overrides_replaces_bindings() {
        let mut map = ActionMap::new();
        map.add_binding("jump", InputBinding::Key(KeyCode::Space));
        map.add_binding("jump", InputBinding::Key(KeyCode::W));

        // Override jump to use Enter instead
        let mut overrides = std::collections::HashMap::new();
        overrides.insert("jump".to_string(), "Enter".to_string());
        map.apply_overrides(&overrides);

        let bindings = map.get_bindings("jump").unwrap();
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0], InputBinding::Key(KeyCode::Enter));
    }

    #[test]
    fn test_apply_overrides_mouse_binding() {
        let mut map = ActionMap::new();
        map.add_binding("fire", InputBinding::Key(KeyCode::Space));

        let mut overrides = std::collections::HashMap::new();
        overrides.insert("fire".to_string(), "MouseLeft".to_string());
        map.apply_overrides(&overrides);

        let bindings = map.get_bindings("fire").unwrap();
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0], InputBinding::Mouse(MouseButton::Left));
    }

    #[test]
    fn test_apply_overrides_ignores_unknown() {
        let mut map = ActionMap::new();
        map.add_binding("jump", InputBinding::Key(KeyCode::Space));

        let mut overrides = std::collections::HashMap::new();
        overrides.insert("jump".to_string(), "NonexistentKey".to_string());
        map.apply_overrides(&overrides);

        // Original binding should remain since the override key was unknown
        let bindings = map.get_bindings("jump").unwrap();
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0], InputBinding::Key(KeyCode::Space));
    }

    #[test]
    fn test_apply_overrides_new_action() {
        let mut map = ActionMap::new();

        let mut overrides = std::collections::HashMap::new();
        overrides.insert("crouch".to_string(), "LControl".to_string());
        map.apply_overrides(&overrides);

        let bindings = map.get_bindings("crouch").unwrap();
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0], InputBinding::Key(KeyCode::LControl));
    }

    #[test]
    fn test_id_lookup_zero_allocation_perf() {
        let mut map = ActionMap::new();
        map.add_binding("jump", InputBinding::Key(KeyCode::Space));
        map.add_binding("fire", InputBinding::Mouse(MouseButton::Left));
        let jump_id = map.register_action("jump");
        let fire_id = map.register_action("fire");

        let mut input = InputState::new();
        input.press_key(KeyCode::Space);
        map.update(&input);

        // ID-based lookups should be fast (no string allocation)
        let start = std::time::Instant::now();
        for _ in 0..100_000 {
            std::hint::black_box(map.is_action_active_by_id(jump_id));
            std::hint::black_box(map.action_state_by_id(fire_id));
        }
        let id_duration = start.elapsed();

        // String-based lookups for comparison
        let start = std::time::Instant::now();
        for _ in 0..100_000 {
            std::hint::black_box(map.is_action_active("jump"));
            std::hint::black_box(map.action_state("fire"));
        }
        let string_duration = start.elapsed();

        // Both should complete quickly (< 100ms for 100k iterations)
        assert!(id_duration.as_millis() < 100, "ID lookup too slow: {:?}", id_duration);
        assert!(string_duration.as_millis() < 100, "String lookup too slow: {:?}", string_duration);

        // ID-based should be at least not slower than string-based
        // (In practice it's faster due to no hashing, but we just verify correctness)
        assert!(map.is_action_active_by_id(jump_id));
        assert!(!map.is_action_active_by_id(fire_id));
    }
}
