//! # 组件系统
//! 
//! 提供 AnvilKit 的核心组件类型和组件管理功能。
//! 
//! ## 设计理念
//! 
//! - **数据导向**: 组件只存储数据，不包含行为逻辑
//! - **组合优于继承**: 通过组合不同组件来创建复杂实体
//! - **缓存友好**: 组件按类型连续存储，提高访问效率
//! - **类型安全**: 编译时检查组件类型，避免运行时错误
//! 
//! ## 核心组件
//! 
//! AnvilKit 提供以下核心组件：
//! 
//! - **Name**: 实体名称标识
//! - **Tag**: 通用标签组件
//! - **Visibility**: 可见性控制
//! - **Layer**: 渲染层级
//! 
//! ## 使用示例
//! 
//! ```rust
//! use anvilkit_ecs::prelude::*;
//! 
//! // 创建带有多个组件的实体
//! let mut world = World::new();
//! let entity = world.spawn((
//!     Name::new("玩家"),
//!     Tag::new("player"),
//!     Visibility::Visible,
//!     Layer(1),
//! )).id();
//! 
//! // 查询特定组件
//! let mut query = world.query::<(&Name, &Tag)>();
//! for (name, tag) in query.iter(&world) {
//!     println!("实体: {}, 标签: {}", name.as_str(), tag.as_str());
//! }
//! ```

use bevy_ecs::prelude::*;
use std::fmt;

/// 实体名称组件
/// 
/// 为实体提供人类可读的名称标识，主要用于调试和编辑器显示。
/// 
/// # 特性
/// 
/// - **调试友好**: 在日志和调试器中显示有意义的名称
/// - **编辑器支持**: 在可视化编辑器中显示实体名称
/// - **查询支持**: 可以通过名称查找实体
/// - **序列化**: 支持保存和加载实体名称
/// 
/// # 示例
/// 
/// ```rust
/// use anvilkit_ecs::prelude::*;
/// 
/// let mut world = World::new();
/// 
/// // 创建带名称的实体
/// let player = world.spawn(Name::new("主角")).id();
/// let enemy = world.spawn(Name::new("敌人_01")).id();
/// 
/// // 查询所有带名称的实体
/// let mut query = world.query::<(Entity, &Name)>();
/// for (entity, name) in query.iter(&world) {
///     println!("实体 {:?}: {}", entity, name.as_str());
/// }
/// ```
#[derive(Component, Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Name {
    name: String,
}

impl Name {
    /// 创建新的名称组件
    /// 
    /// # 参数
    /// 
    /// - `name`: 实体名称
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_ecs::component::Name;
    /// 
    /// let name = Name::new("我的实体");
    /// assert_eq!(name.as_str(), "我的实体");
    /// ```
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
        }
    }

    /// 获取名称字符串引用
    pub fn as_str(&self) -> &str {
        &self.name
    }

    /// 设置新的名称
    /// 
    /// # 参数
    /// 
    /// - `name`: 新的名称
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_ecs::component::Name;
    /// 
    /// let mut name = Name::new("旧名称");
    /// name.set("新名称");
    /// assert_eq!(name.as_str(), "新名称");
    /// ```
    pub fn set(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    /// 检查名称是否为空
    pub fn is_empty(&self) -> bool {
        self.name.is_empty()
    }

    /// 获取名称长度
    pub fn len(&self) -> usize {
        self.name.len()
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl From<String> for Name {
    fn from(name: String) -> Self {
        Self::new(name)
    }
}

impl From<&str> for Name {
    fn from(name: &str) -> Self {
        Self::new(name)
    }
}

/// 通用标签组件
/// 
/// 用于给实体添加分类标签，便于查询和过滤。
/// 
/// # 使用场景
/// 
/// - **分类**: 将实体按功能或类型分组
/// - **过滤**: 在查询中过滤特定类型的实体
/// - **状态**: 标记实体的临时状态
/// - **系统**: 控制哪些系统处理哪些实体
/// 
/// # 示例
/// 
/// ```rust
/// use anvilkit_ecs::prelude::*;
/// 
/// let mut world = World::new();
/// 
/// // 创建不同标签的实体
/// world.spawn((Name::new("玩家"), Tag::new("player")));
/// world.spawn((Name::new("敌人"), Tag::new("enemy")));
/// world.spawn((Name::new("道具"), Tag::new("item")));
/// 
/// // 查询特定标签的实体
/// let mut player_query = world.query::<&Name>().with::<Tag>();
/// // 注意：这里需要更复杂的查询来过滤特定标签值
/// ```
#[derive(Component, Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Tag {
    tag: String,
}

impl Tag {
    /// 创建新的标签组件
    /// 
    /// # 参数
    /// 
    /// - `tag`: 标签字符串
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_ecs::component::Tag;
    /// 
    /// let tag = Tag::new("player");
    /// assert_eq!(tag.as_str(), "player");
    /// ```
    pub fn new(tag: impl Into<String>) -> Self {
        Self {
            tag: tag.into(),
        }
    }

    /// 获取标签字符串引用
    pub fn as_str(&self) -> &str {
        &self.tag
    }

    /// 设置新的标签
    pub fn set(&mut self, tag: impl Into<String>) {
        self.tag = tag.into();
    }

    /// 检查是否匹配指定标签
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_ecs::component::Tag;
    /// 
    /// let tag = Tag::new("player");
    /// assert!(tag.matches("player"));
    /// assert!(!tag.matches("enemy"));
    /// ```
    pub fn matches(&self, other: &str) -> bool {
        self.tag == other
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.tag)
    }
}

impl From<String> for Tag {
    fn from(tag: String) -> Self {
        Self::new(tag)
    }
}

impl From<&str> for Tag {
    fn from(tag: &str) -> Self {
        Self::new(tag)
    }
}

/// 可见性组件
/// 
/// 控制实体的可见性状态，影响渲染和某些系统的处理。
/// 
/// # 可见性状态
/// 
/// - **Visible**: 实体可见，正常渲染
/// - **Hidden**: 实体隐藏，不进行渲染
/// - **Inherited**: 继承父实体的可见性（用于层次结构）
/// 
/// # 示例
/// 
/// ```rust
/// use anvilkit_ecs::prelude::*;
/// 
/// let mut world = World::new();
/// 
/// // 创建不同可见性的实体
/// world.spawn((Name::new("可见实体"), Visibility::Visible));
/// world.spawn((Name::new("隐藏实体"), Visibility::Hidden));
/// 
/// // 查询可见实体
/// let mut visible_query = world.query::<&Name>()
///     .with::<Visibility>();
/// ```
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Visibility {
    /// 实体可见
    Visible,
    /// 实体隐藏
    Hidden,
    /// 继承父实体的可见性
    Inherited,
}

impl Default for Visibility {
    fn default() -> Self {
        Self::Visible
    }
}

impl Visibility {
    /// 检查是否可见
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_ecs::component::Visibility;
    /// 
    /// assert!(Visibility::Visible.is_visible());
    /// assert!(!Visibility::Hidden.is_visible());
    /// ```
    pub fn is_visible(&self) -> bool {
        matches!(self, Self::Visible)
    }

    /// 检查是否隐藏
    pub fn is_hidden(&self) -> bool {
        matches!(self, Self::Hidden)
    }

    /// 检查是否继承
    pub fn is_inherited(&self) -> bool {
        matches!(self, Self::Inherited)
    }

    /// 切换可见性
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_ecs::component::Visibility;
    /// 
    /// let mut visibility = Visibility::Visible;
    /// visibility.toggle();
    /// assert_eq!(visibility, Visibility::Hidden);
    /// 
    /// visibility.toggle();
    /// assert_eq!(visibility, Visibility::Visible);
    /// ```
    pub fn toggle(&mut self) {
        *self = match *self {
            Self::Visible => Self::Hidden,
            Self::Hidden => Self::Visible,
            Self::Inherited => Self::Inherited, // 继承状态不变
        };
    }
}

/// 渲染层级组件
/// 
/// 控制实体的渲染顺序，数值越大越靠前渲染。
/// 
/// # 使用场景
/// 
/// - **UI 层级**: 控制 UI 元素的显示顺序
/// - **精灵排序**: 2D 游戏中精灵的前后关系
/// - **透明度排序**: 透明物体的渲染顺序
/// - **调试显示**: 调试信息的显示层级
/// 
/// # 示例
/// 
/// ```rust
/// use anvilkit_ecs::prelude::*;
/// 
/// let mut world = World::new();
/// 
/// // 创建不同层级的实体
/// world.spawn((Name::new("背景"), Layer(0)));
/// world.spawn((Name::new("游戏对象"), Layer(1)));
/// world.spawn((Name::new("UI"), Layer(2)));
/// world.spawn((Name::new("调试信息"), Layer(999)));
/// ```
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Layer(pub i32);

impl Layer {
    /// 创建新的层级组件
    /// 
    /// # 参数
    /// 
    /// - `layer`: 层级数值
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_ecs::component::Layer;
    /// 
    /// let layer = Layer::new(5);
    /// assert_eq!(layer.value(), 5);
    /// ```
    pub fn new(layer: i32) -> Self {
        Self(layer)
    }

    /// 获取层级数值
    pub fn value(&self) -> i32 {
        self.0
    }

    /// 设置层级数值
    pub fn set(&mut self, layer: i32) {
        self.0 = layer;
    }

    /// 增加层级
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_ecs::component::Layer;
    /// 
    /// let mut layer = Layer::new(1);
    /// layer.increase(2);
    /// assert_eq!(layer.value(), 3);
    /// ```
    pub fn increase(&mut self, delta: i32) {
        self.0 += delta;
    }

    /// 减少层级
    pub fn decrease(&mut self, delta: i32) {
        self.0 -= delta;
    }
}

impl Default for Layer {
    fn default() -> Self {
        Self(0)
    }
}

impl From<i32> for Layer {
    fn from(layer: i32) -> Self {
        Self::new(layer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name_component() {
        let name = Name::new("测试实体");
        assert_eq!(name.as_str(), "测试实体");
        assert!(!name.is_empty());
        assert_eq!(name.len(), "测试实体".len());

        let mut name = Name::new("旧名称");
        name.set("新名称");
        assert_eq!(name.as_str(), "新名称");
    }

    #[test]
    fn test_tag_component() {
        let tag = Tag::new("player");
        assert_eq!(tag.as_str(), "player");
        assert!(tag.matches("player"));
        assert!(!tag.matches("enemy"));

        let mut tag = Tag::new("old_tag");
        tag.set("new_tag");
        assert_eq!(tag.as_str(), "new_tag");
    }

    #[test]
    fn test_visibility_component() {
        let mut visibility = Visibility::Visible;
        assert!(visibility.is_visible());
        assert!(!visibility.is_hidden());

        visibility.toggle();
        assert!(visibility.is_hidden());
        assert!(!visibility.is_visible());

        visibility.toggle();
        assert!(visibility.is_visible());

        let inherited = Visibility::Inherited;
        assert!(inherited.is_inherited());
    }

    #[test]
    fn test_layer_component() {
        let mut layer = Layer::new(5);
        assert_eq!(layer.value(), 5);

        layer.increase(3);
        assert_eq!(layer.value(), 8);

        layer.decrease(2);
        assert_eq!(layer.value(), 6);

        layer.set(10);
        assert_eq!(layer.value(), 10);
    }

    #[test]
    fn test_layer_ordering() {
        let layer1 = Layer::new(1);
        let layer2 = Layer::new(2);
        let layer3 = Layer::new(1);

        assert!(layer1 < layer2);
        assert!(layer2 > layer1);
        assert_eq!(layer1, layer3);
    }

    #[test]
    fn test_component_conversions() {
        let name: Name = "测试".into();
        assert_eq!(name.as_str(), "测试");

        let tag: Tag = "player".into();
        assert_eq!(tag.as_str(), "player");

        let layer: Layer = 5.into();
        assert_eq!(layer.value(), 5);
    }

    #[test]
    fn test_component_display() {
        let name = Name::new("显示测试");
        assert_eq!(format!("{}", name), "显示测试");

        let tag = Tag::new("test_tag");
        assert_eq!(format!("{}", tag), "test_tag");
    }
}
