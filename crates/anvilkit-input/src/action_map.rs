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
}
