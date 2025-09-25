//! # Bundle 系统
//! 
//! 提供组件的预定义组合，简化实体创建和管理。
//! 
//! ## 设计理念
//! 
//! - **组合优于继承**: 通过组合不同组件创建复杂实体
//! - **预定义模板**: 提供常用的组件组合模板
//! - **类型安全**: 编译时检查 Bundle 的完整性
//! - **性能优化**: 批量插入组件，提高创建效率
//! 
//! ## 核心 Bundle
//! 
//! AnvilKit 提供以下预定义 Bundle：
//! 
//! - **EntityBundle**: 基础实体 Bundle，包含名称和标签
//! - **SpatialBundle**: 空间实体 Bundle，包含变换和可见性
//! - **RenderBundle**: 渲染实体 Bundle，包含渲染相关组件
//! 
//! ## 使用示例
//! 
//! ```rust
//! use anvilkit_ecs::prelude::*;
//! 
//! let mut world = World::new();
//! 
//! // 使用预定义 Bundle 创建实体
//! let entity = world.spawn(SpatialBundle {
//!     name: Name::new("空间实体"),
//!     transform: Transform::default(),
//!     global_transform: GlobalTransform::default(),
//!     visibility: Visibility::Visible,
//!     layer: Layer::new(1),
//! }).id();
//! 
//! // 使用 Bundle 构建器
//! let entity2 = world.spawn(
//!     SpatialBundle::new("另一个实体")
//!         .with_layer(2)
//!         .with_visibility(Visibility::Hidden)
//! ).id();
//! ```

use bevy_ecs::prelude::*;
use crate::component::{Name, Tag, Visibility, Layer};
use crate::transform::{Transform, GlobalTransform};

/// 基础实体 Bundle
/// 
/// 包含实体的基本标识信息，适用于大多数实体。
/// 
/// # 包含组件
/// 
/// - `Name`: 实体名称
/// - `Tag`: 实体标签
/// 
/// # 示例
/// 
/// ```rust
/// use anvilkit_ecs::prelude::*;
/// 
/// let mut world = World::new();
/// 
/// // 直接创建
/// let entity1 = world.spawn(EntityBundle {
///     name: Name::new("基础实体"),
///     tag: Tag::new("basic"),
/// }).id();
/// 
/// // 使用构建器
/// let entity2 = world.spawn(
///     EntityBundle::new("另一个实体", "another")
/// ).id();
/// ```
#[derive(Bundle, Debug, Clone)]
pub struct EntityBundle {
    /// 实体名称
    pub name: Name,
    /// 实体标签
    pub tag: Tag,
}

impl EntityBundle {
    /// 创建新的基础实体 Bundle
    /// 
    /// # 参数
    /// 
    /// - `name`: 实体名称
    /// - `tag`: 实体标签
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_ecs::bundle::EntityBundle;
    /// 
    /// let bundle = EntityBundle::new("我的实体", "my_tag");
    /// ```
    pub fn new(name: impl Into<String>, tag: impl Into<String>) -> Self {
        Self {
            name: Name::new(name),
            tag: Tag::new(tag),
        }
    }

    /// 设置名称
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Name::new(name);
        self
    }

    /// 设置标签
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tag = Tag::new(tag);
        self
    }
}

impl Default for EntityBundle {
    fn default() -> Self {
        Self {
            name: Name::new("未命名实体"),
            tag: Tag::new("default"),
        }
    }
}

/// 空间实体 Bundle
/// 
/// 包含空间变换和可见性信息，适用于需要位置和渲染的实体。
/// 
/// # 包含组件
/// 
/// - `Name`: 实体名称
/// - `Transform`: 本地变换
/// - `GlobalTransform`: 全局变换
/// - `Visibility`: 可见性
/// - `Layer`: 渲染层级
/// 
/// # 示例
/// 
/// ```rust
/// use anvilkit_ecs::prelude::*;
/// 
/// let mut world = World::new();
/// 
/// // 直接创建
/// let entity = world.spawn(SpatialBundle {
///     name: Name::new("空间实体"),
///     transform: Transform::from_translation(Vec3::new(1.0, 2.0, 3.0)),
///     global_transform: GlobalTransform::default(),
///     visibility: Visibility::Visible,
///     layer: Layer::new(1),
/// }).id();
/// 
/// // 使用构建器
/// let entity2 = world.spawn(
///     SpatialBundle::new("移动实体")
///         .with_position(Vec3::new(5.0, 0.0, 0.0))
///         .with_layer(2)
/// ).id();
/// ```
#[derive(Bundle, Debug, Clone)]
pub struct SpatialBundle {
    /// 实体名称
    pub name: Name,
    /// 本地变换
    pub transform: Transform,
    /// 全局变换
    pub global_transform: GlobalTransform,
    /// 可见性
    pub visibility: Visibility,
    /// 渲染层级
    pub layer: Layer,
}

impl SpatialBundle {
    /// 创建新的空间实体 Bundle
    /// 
    /// # 参数
    /// 
    /// - `name`: 实体名称
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_ecs::bundle::SpatialBundle;
    /// 
    /// let bundle = SpatialBundle::new("空间实体");
    /// ```
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: Name::new(name),
            transform: Transform::default(),
            global_transform: GlobalTransform::default(),
            visibility: Visibility::default(),
            layer: Layer::default(),
        }
    }

    /// 设置位置
    /// 
    /// # 参数
    /// 
    /// - `position`: 世界坐标位置
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_ecs::prelude::*;
    /// 
    /// let bundle = SpatialBundle::new("实体")
    ///     .with_position(Vec3::new(1.0, 2.0, 3.0));
    /// ```
    pub fn with_position(mut self, position: glam::Vec3) -> Self {
        self.transform.translation = position;
        self
    }

    /// 设置旋转
    /// 
    /// # 参数
    /// 
    /// - `rotation`: 四元数旋转
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_ecs::prelude::*;
    /// 
    /// let bundle = SpatialBundle::new("实体")
    ///     .with_rotation(Quat::from_rotation_y(std::f32::consts::PI));
    /// ```
    pub fn with_rotation(mut self, rotation: glam::Quat) -> Self {
        self.transform.rotation = rotation;
        self
    }

    /// 设置缩放
    /// 
    /// # 参数
    /// 
    /// - `scale`: 缩放向量
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_ecs::prelude::*;
    /// 
    /// let bundle = SpatialBundle::new("实体")
    ///     .with_scale(Vec3::splat(2.0));
    /// ```
    pub fn with_scale(mut self, scale: glam::Vec3) -> Self {
        self.transform.scale = scale;
        self
    }

    /// 设置变换
    /// 
    /// # 参数
    /// 
    /// - `transform`: 完整的变换
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_ecs::prelude::*;
    /// 
    /// let transform = Transform::from_translation(Vec3::new(1.0, 0.0, 0.0));
    /// let bundle = SpatialBundle::new("实体")
    ///     .with_transform(transform);
    /// ```
    pub fn with_transform(mut self, transform: Transform) -> Self {
        self.transform = transform;
        self
    }

    /// 设置可见性
    /// 
    /// # 参数
    /// 
    /// - `visibility`: 可见性状态
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_ecs::prelude::*;
    /// 
    /// let bundle = SpatialBundle::new("实体")
    ///     .with_visibility(Visibility::Hidden);
    /// ```
    pub fn with_visibility(mut self, visibility: Visibility) -> Self {
        self.visibility = visibility;
        self
    }

    /// 设置渲染层级
    /// 
    /// # 参数
    /// 
    /// - `layer`: 层级数值
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_ecs::prelude::*;
    /// 
    /// let bundle = SpatialBundle::new("实体")
    ///     .with_layer(5);
    /// ```
    pub fn with_layer(mut self, layer: i32) -> Self {
        self.layer = Layer::new(layer);
        self
    }
}

impl Default for SpatialBundle {
    fn default() -> Self {
        Self::new("空间实体")
    }
}

/// 渲染实体 Bundle
/// 
/// 扩展空间 Bundle，添加渲染相关的组件。
/// 
/// # 包含组件
/// 
/// - 继承 `SpatialBundle` 的所有组件
/// - `Tag`: 渲染标签（用于渲染系统过滤）
/// 
/// # 示例
/// 
/// ```rust
/// use anvilkit_ecs::prelude::*;
/// 
/// let mut world = World::new();
/// 
/// let entity = world.spawn(
///     RenderBundle::new("渲染实体")
///         .with_render_tag("sprite")
///         .with_position(Vec3::new(0.0, 0.0, 0.0))
///         .with_layer(1)
/// ).id();
/// ```
#[derive(Bundle, Debug, Clone)]
pub struct RenderBundle {
    /// 空间组件
    pub spatial: SpatialBundle,
    /// 渲染标签
    pub render_tag: Tag,
}

impl RenderBundle {
    /// 创建新的渲染实体 Bundle
    /// 
    /// # 参数
    /// 
    /// - `name`: 实体名称
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_ecs::bundle::RenderBundle;
    /// 
    /// let bundle = RenderBundle::new("渲染实体");
    /// ```
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            spatial: SpatialBundle::new(name),
            render_tag: Tag::new("renderable"),
        }
    }

    /// 设置渲染标签
    /// 
    /// # 参数
    /// 
    /// - `tag`: 渲染标签
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_ecs::bundle::RenderBundle;
    /// 
    /// let bundle = RenderBundle::new("实体")
    ///     .with_render_tag("sprite");
    /// ```
    pub fn with_render_tag(mut self, tag: impl Into<String>) -> Self {
        self.render_tag = Tag::new(tag);
        self
    }

    /// 设置位置（委托给空间 Bundle）
    pub fn with_position(mut self, position: glam::Vec3) -> Self {
        self.spatial = self.spatial.with_position(position);
        self
    }

    /// 设置旋转（委托给空间 Bundle）
    pub fn with_rotation(mut self, rotation: glam::Quat) -> Self {
        self.spatial = self.spatial.with_rotation(rotation);
        self
    }

    /// 设置缩放（委托给空间 Bundle）
    pub fn with_scale(mut self, scale: glam::Vec3) -> Self {
        self.spatial = self.spatial.with_scale(scale);
        self
    }

    /// 设置变换（委托给空间 Bundle）
    pub fn with_transform(mut self, transform: Transform) -> Self {
        self.spatial = self.spatial.with_transform(transform);
        self
    }

    /// 设置可见性（委托给空间 Bundle）
    pub fn with_visibility(mut self, visibility: Visibility) -> Self {
        self.spatial = self.spatial.with_visibility(visibility);
        self
    }

    /// 设置渲染层级（委托给空间 Bundle）
    pub fn with_layer(mut self, layer: i32) -> Self {
        self.spatial = self.spatial.with_layer(layer);
        self
    }
}

impl Default for RenderBundle {
    fn default() -> Self {
        Self::new("渲染实体")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;

    #[test]
    fn test_entity_bundle() {
        let bundle = EntityBundle::new("测试实体", "test");
        assert_eq!(bundle.name.as_str(), "测试实体");
        assert_eq!(bundle.tag.as_str(), "test");

        let bundle = EntityBundle::default()
            .with_name("新名称")
            .with_tag("new_tag");
        assert_eq!(bundle.name.as_str(), "新名称");
        assert_eq!(bundle.tag.as_str(), "new_tag");
    }

    #[test]
    fn test_spatial_bundle() {
        let bundle = SpatialBundle::new("空间实体");
        assert_eq!(bundle.name.as_str(), "空间实体");
        assert_eq!(bundle.transform.translation, glam::Vec3::ZERO);
        assert_eq!(bundle.visibility, Visibility::Visible);
        assert_eq!(bundle.layer.value(), 0);

        let position = glam::Vec3::new(1.0, 2.0, 3.0);
        let bundle = SpatialBundle::new("移动实体")
            .with_position(position)
            .with_layer(5)
            .with_visibility(Visibility::Hidden);
        
        assert_eq!(bundle.transform.translation, position);
        assert_eq!(bundle.layer.value(), 5);
        assert_eq!(bundle.visibility, Visibility::Hidden);
    }

    #[test]
    fn test_render_bundle() {
        let bundle = RenderBundle::new("渲染实体");
        assert_eq!(bundle.spatial.name.as_str(), "渲染实体");
        assert_eq!(bundle.render_tag.as_str(), "renderable");

        let bundle = RenderBundle::new("精灵")
            .with_render_tag("sprite")
            .with_position(glam::Vec3::new(10.0, 20.0, 0.0))
            .with_layer(2);
        
        assert_eq!(bundle.render_tag.as_str(), "sprite");
        assert_eq!(bundle.spatial.transform.translation, glam::Vec3::new(10.0, 20.0, 0.0));
        assert_eq!(bundle.spatial.layer.value(), 2);
    }

    #[test]
    fn test_bundle_in_world() {
        let mut world = World::new();

        // 测试 EntityBundle
        let entity1 = world.spawn(EntityBundle::new("实体1", "tag1")).id();
        
        // 测试 SpatialBundle
        let entity2 = world.spawn(
            SpatialBundle::new("实体2")
                .with_position(glam::Vec3::new(5.0, 0.0, 0.0))
        ).id();

        // 测试 RenderBundle
        let entity3 = world.spawn(
            RenderBundle::new("实体3")
                .with_render_tag("mesh")
        ).id();

        // 验证实体存在
        assert!(world.get_entity(entity1).is_some());
        assert!(world.get_entity(entity2).is_some());
        assert!(world.get_entity(entity3).is_some());

        // 验证组件
        let name1 = world.get::<Name>(entity1).unwrap();
        assert_eq!(name1.as_str(), "实体1");

        let transform2 = world.get::<Transform>(entity2).unwrap();
        assert_eq!(transform2.translation, glam::Vec3::new(5.0, 0.0, 0.0));

        let render_tag3 = world.get::<Tag>(entity3).unwrap();
        assert_eq!(render_tag3.as_str(), "mesh");
    }
}
