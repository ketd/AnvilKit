//! # Gamepad 输入支持
//!
//! 提供手柄/控制器的按钮和摇杆输入支持。

use std::collections::{HashMap, HashSet};
use bevy_ecs::prelude::*;

/// Gamepad 按钮
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GamepadButton {
    /// A (Xbox) / Cross (PS)
    South,
    /// B / Circle
    East,
    /// X / Square
    West,
    /// Y / Triangle
    North,
    /// D-Pad 上
    DPadUp,
    /// D-Pad 下
    DPadDown,
    /// D-Pad 左
    DPadLeft,
    /// D-Pad 右
    DPadRight,
    /// 左肩键
    LeftShoulder,
    /// 右肩键
    RightShoulder,
    /// 左扳机键
    LeftTrigger,
    /// 右扳机键
    RightTrigger,
    /// 左摇杆按下
    LeftThumb,
    /// 右摇杆按下
    RightThumb,
    /// Start / Menu
    Start,
    /// Select / View
    Select,
}

/// Gamepad 模拟轴
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GamepadAxis {
    /// 左摇杆 X 轴
    LeftStickX,
    /// 左摇杆 Y 轴
    LeftStickY,
    /// 右摇杆 X 轴
    RightStickX,
    /// 右摇杆 Y 轴
    RightStickY,
    /// 左扳机模拟轴
    LeftTriggerAxis,
    /// 右扳机模拟轴
    RightTriggerAxis,
}

/// 单个 gamepad 的状态
#[derive(Debug, Clone, Default)]
pub struct SingleGamepadState {
    pressed: HashSet<GamepadButton>,
    just_pressed: HashSet<GamepadButton>,
    just_released: HashSet<GamepadButton>,
    axes: HashMap<GamepadAxis, f32>,
}

/// Gamepad 输入状态资源
#[derive(Resource, Debug, Clone, Default)]
pub struct GamepadState {
    gamepads: HashMap<u32, SingleGamepadState>,
}

impl GamepadState {
    /// 创建空的 gamepad 状态
    pub fn new() -> Self { Self::default() }

    /// 注册一个连接的 gamepad
    pub fn connect(&mut self, id: u32) {
        self.gamepads.entry(id).or_default();
    }

    /// 移除断开的 gamepad
    pub fn disconnect(&mut self, id: u32) {
        self.gamepads.remove(&id);
    }

    /// 获取所有连接的 gamepad ID
    pub fn connected_gamepads(&self) -> Vec<u32> {
        self.gamepads.keys().copied().collect()
    }

    /// 按下按钮
    pub fn press_button(&mut self, id: u32, button: GamepadButton) {
        if let Some(gp) = self.gamepads.get_mut(&id) {
            if gp.pressed.insert(button) {
                gp.just_pressed.insert(button);
            }
        }
    }

    /// 释放按钮
    pub fn release_button(&mut self, id: u32, button: GamepadButton) {
        if let Some(gp) = self.gamepads.get_mut(&id) {
            if gp.pressed.remove(&button) {
                gp.just_released.insert(button);
            }
        }
    }

    /// 设置轴值
    pub fn set_axis(&mut self, id: u32, axis: GamepadAxis, value: f32) {
        if let Some(gp) = self.gamepads.get_mut(&id) {
            gp.axes.insert(axis, value);
        }
    }

    /// 查询按钮是否按下
    pub fn is_button_pressed(&self, id: u32, button: GamepadButton) -> bool {
        self.gamepads.get(&id).map_or(false, |gp| gp.pressed.contains(&button))
    }

    /// 查询按钮是否刚按下
    pub fn is_button_just_pressed(&self, id: u32, button: GamepadButton) -> bool {
        self.gamepads.get(&id).map_or(false, |gp| gp.just_pressed.contains(&button))
    }

    /// 查询轴值
    pub fn axis_value(&self, id: u32, axis: GamepadAxis) -> f32 {
        self.gamepads.get(&id).and_then(|gp| gp.axes.get(&axis)).copied().unwrap_or(0.0)
    }

    /// 帧结束清除 per-frame 状态
    pub fn end_frame(&mut self) {
        for gp in self.gamepads.values_mut() {
            gp.just_pressed.clear();
            gp.just_released.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gamepad_connect_disconnect() {
        let mut state = GamepadState::new();
        state.connect(0);
        assert_eq!(state.connected_gamepads().len(), 1);
        state.disconnect(0);
        assert_eq!(state.connected_gamepads().len(), 0);
    }

    #[test]
    fn test_button_press() {
        let mut state = GamepadState::new();
        state.connect(0);
        state.press_button(0, GamepadButton::South);
        assert!(state.is_button_pressed(0, GamepadButton::South));
        assert!(state.is_button_just_pressed(0, GamepadButton::South));
        state.end_frame();
        assert!(state.is_button_pressed(0, GamepadButton::South));
        assert!(!state.is_button_just_pressed(0, GamepadButton::South));
    }

    #[test]
    fn test_axis_value() {
        let mut state = GamepadState::new();
        state.connect(0);
        state.set_axis(0, GamepadAxis::LeftStickX, 0.75);
        assert!((state.axis_value(0, GamepadAxis::LeftStickX) - 0.75).abs() < 0.001);
        assert_eq!(state.axis_value(0, GamepadAxis::LeftStickY), 0.0); // default
    }
}
