//! # UI 系统
//!
//! 提供保留模式 UI 节点树、Flexbox 布局和文本渲染数据结构。
//!
//! ## 核心类型
//!
//! - [`UiNode`]: UI 元素组件（矩形、文本、图像）
//! - [`UiStyle`]: 布局样式（Flexbox 属性）
//! - [`UiText`]: 文本内容和字体配置

use bevy_ecs::prelude::*;
/// Flexbox 排列方向
///
/// # 示例
///
/// ```rust
/// use anvilkit_render::renderer::ui::FlexDirection;
/// assert_ne!(FlexDirection::Row, FlexDirection::Column);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexDirection {
    Row,
    RowReverse,
    Column,
    ColumnReverse,
}

/// Flexbox 对齐
///
/// # 示例
///
/// ```rust
/// use anvilkit_render::renderer::ui::Align;
/// let center = Align::Center;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Align {
    Start,
    Center,
    End,
    Stretch,
    SpaceBetween,
    SpaceAround,
}

/// 尺寸值（像素或百分比）
///
/// # 示例
///
/// ```rust
/// use anvilkit_render::renderer::ui::Val;
/// let px = Val::Px(100.0);
/// let pct = Val::Percent(50.0);
/// let auto = Val::Auto;
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Val {
    Auto,
    Px(f32),
    Percent(f32),
}

impl Default for Val {
    fn default() -> Self { Val::Auto }
}

/// UI 布局样式
///
/// Flexbox 属性集合，控制 UI 元素的布局行为。
///
/// # 示例
///
/// ```rust
/// use anvilkit_render::renderer::ui::{UiStyle, FlexDirection, Align, Val};
///
/// let style = UiStyle {
///     flex_direction: FlexDirection::Column,
///     justify_content: Align::Center,
///     align_items: Align::Center,
///     width: Val::Percent(100.0),
///     height: Val::Px(50.0),
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone)]
pub struct UiStyle {
    pub flex_direction: FlexDirection,
    pub justify_content: Align,
    pub align_items: Align,
    pub width: Val,
    pub height: Val,
    pub min_width: Val,
    pub min_height: Val,
    pub max_width: Val,
    pub max_height: Val,
    pub padding: [f32; 4],  // top, right, bottom, left
    pub margin: [f32; 4],
    pub gap: f32,
    pub flex_grow: f32,
    pub flex_shrink: f32,
}

impl Default for UiStyle {
    fn default() -> Self {
        Self {
            flex_direction: FlexDirection::Row,
            justify_content: Align::Start,
            align_items: Align::Stretch,
            width: Val::Auto,
            height: Val::Auto,
            min_width: Val::Auto,
            min_height: Val::Auto,
            max_width: Val::Auto,
            max_height: Val::Auto,
            padding: [0.0; 4],
            margin: [0.0; 4],
            gap: 0.0,
            flex_grow: 0.0,
            flex_shrink: 1.0,
        }
    }
}

/// UI 文本内容
///
/// # 示例
///
/// ```rust
/// use anvilkit_render::renderer::ui::UiText;
///
/// let text = UiText::new("Hello, AnvilKit!").with_font_size(24.0);
/// assert_eq!(text.content, "Hello, AnvilKit!");
/// assert_eq!(text.font_size, 24.0);
/// ```
#[derive(Debug, Clone)]
pub struct UiText {
    pub content: String,
    pub font_size: f32,
    pub color: [f32; 4],
    pub font_family: String,
}

impl UiText {
    pub fn new(content: &str) -> Self {
        Self {
            content: content.to_string(),
            font_size: 16.0,
            color: [1.0, 1.0, 1.0, 1.0],
            font_family: "default".to_string(),
        }
    }

    pub fn with_font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    pub fn with_color(mut self, color: [f32; 4]) -> Self {
        self.color = color;
        self
    }
}

/// UI 节点组件
///
/// 表示 UI 树中的一个元素。可包含背景色、文本或图像。
///
/// # 示例
///
/// ```rust
/// use anvilkit_render::renderer::ui::{UiNode, UiText};
///
/// let button = UiNode {
///     background_color: [0.2, 0.4, 0.8, 1.0],
///     border_radius: 8.0,
///     text: Some(UiText::new("Click Me")),
///     visible: true,
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, Component)]
pub struct UiNode {
    /// 背景色 [R, G, B, A]
    pub background_color: [f32; 4],
    /// 边框圆角半径
    pub border_radius: f32,
    /// 边框宽度
    pub border_width: f32,
    /// 边框颜色
    pub border_color: [f32; 4],
    /// 文本内容
    pub text: Option<UiText>,
    /// 布局样式
    pub style: UiStyle,
    /// 是否可见
    pub visible: bool,
    /// 计算后的布局矩形（由布局系统填充）
    pub computed_rect: [f32; 4], // x, y, width, height
}

impl Default for UiNode {
    fn default() -> Self {
        Self {
            background_color: [0.0, 0.0, 0.0, 0.0],
            border_radius: 0.0,
            border_width: 0.0,
            border_color: [0.0; 4],
            text: None,
            style: UiStyle::default(),
            visible: true,
            computed_rect: [0.0; 4],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_text() {
        let text = UiText::new("Hello").with_font_size(32.0).with_color([1.0, 0.0, 0.0, 1.0]);
        assert_eq!(text.content, "Hello");
        assert_eq!(text.font_size, 32.0);
        assert_eq!(text.color[0], 1.0);
    }

    #[test]
    fn test_ui_node_default() {
        let node = UiNode::default();
        assert!(node.visible);
        assert!(node.text.is_none());
        assert_eq!(node.background_color, [0.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_val() {
        let auto = Val::Auto;
        let px = Val::Px(100.0);
        let pct = Val::Percent(50.0);
        assert_ne!(auto, px);
        assert_ne!(px, pct);
    }
}
