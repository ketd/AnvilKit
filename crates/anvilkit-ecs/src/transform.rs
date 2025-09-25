//! # 变换系统
//! 
//! 提供基于 ECS 的变换层次系统，支持父子关系和全局变换传播。
//! 
//! ## 设计理念
//! 
//! - **层次结构**: 支持父子实体的变换层次关系
//! - **自动传播**: 父实体变换自动传播到子实体
//! - **缓存友好**: 使用 SoA (Structure of Arrays) 布局优化性能
//! - **变更检测**: 只在变换发生变化时进行传播计算
//! 
//! ## 核心组件
//! 
//! - **Transform**: 本地变换，相对于父实体的变换
//! - **GlobalTransform**: 全局变换，世界空间中的绝对变换
//! - **Parent**: 父实体引用
//! - **Children**: 子实体列表
//! 
//! ## 使用示例
//! 
//! ```rust
//! use anvilkit_ecs::prelude::*;
//! 
//! let mut world = World::new();
//! 
//! // 创建父实体
//! let parent = world.spawn((
//!     Transform::from_translation(Vec3::new(10.0, 0.0, 0.0)),
//!     GlobalTransform::default(),
//! )).id();
//! 
//! // 创建子实体
//! let child = world.spawn((
//!     Transform::from_translation(Vec3::new(5.0, 0.0, 0.0)),
//!     GlobalTransform::default(),
//!     Parent(parent),
//! )).id();
//! 
//! // 运行变换传播系统
//! // 子实体的全局位置将是 (15.0, 0.0, 0.0)
//! ```

use bevy_ecs::prelude::*;
use glam::Vec3;

// 重新导出 anvilkit-core 的变换类型
pub use anvilkit_core::math::{Transform, GlobalTransform};

/// 父实体组件
/// 
/// 标识实体的父实体，用于构建变换层次结构。
/// 
/// # 示例
/// 
/// ```rust
/// use anvilkit_ecs::prelude::*;
/// 
/// let mut world = World::new();
/// 
/// let parent_entity = world.spawn((
///     Transform::default(),
///     GlobalTransform::default(),
/// )).id();
/// 
/// let child_entity = world.spawn((
///     Transform::default(),
///     GlobalTransform::default(),
///     Parent(parent_entity),
/// )).id();
/// ```
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Parent(pub Entity);

impl Parent {
    /// 创建新的父实体组件
    pub fn new(entity: Entity) -> Self {
        Self(entity)
    }

    /// 获取父实体
    pub fn get(&self) -> Entity {
        self.0
    }

    /// 设置父实体
    pub fn set(&mut self, entity: Entity) {
        self.0 = entity;
    }
}

/// 子实体列表组件
/// 
/// 存储实体的所有子实体，用于变换传播和层次管理。
/// 
/// # 示例
/// 
/// ```rust
/// use anvilkit_ecs::prelude::*;
/// 
/// let mut world = World::new();
/// 
/// let child1 = world.spawn_empty().id();
/// let child2 = world.spawn_empty().id();
/// 
/// let parent = world.spawn((
///     Transform::default(),
///     GlobalTransform::default(),
///     Children::new(vec![child1, child2]),
/// )).id();
/// ```
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct Children {
    children: Vec<Entity>,
}

impl Children {
    /// 创建新的子实体列表
    pub fn new(children: Vec<Entity>) -> Self {
        Self { children }
    }

    /// 创建空的子实体列表
    pub fn empty() -> Self {
        Self {
            children: Vec::new(),
        }
    }

    /// 获取子实体列表
    pub fn iter(&self) -> impl Iterator<Item = &Entity> {
        self.children.iter()
    }

    /// 获取子实体数量
    pub fn len(&self) -> usize {
        self.children.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }

    /// 添加子实体
    pub fn push(&mut self, entity: Entity) {
        if !self.children.contains(&entity) {
            self.children.push(entity);
        }
    }

    /// 移除子实体
    pub fn remove(&mut self, entity: Entity) {
        self.children.retain(|&e| e != entity);
    }

    /// 检查是否包含指定子实体
    pub fn contains(&self, entity: Entity) -> bool {
        self.children.contains(&entity)
    }

    /// 清空所有子实体
    pub fn clear(&mut self) {
        self.children.clear();
    }

    /// 获取第一个子实体
    pub fn first(&self) -> Option<Entity> {
        self.children.first().copied()
    }

    /// 获取最后一个子实体
    pub fn last(&self) -> Option<Entity> {
        self.children.last().copied()
    }
}

impl Default for Children {
    fn default() -> Self {
        Self::empty()
    }
}

impl From<Vec<Entity>> for Children {
    fn from(children: Vec<Entity>) -> Self {
        Self::new(children)
    }
}

/// 变换插件
/// 
/// 提供变换系统的完整功能，包括层次传播和变更检测。
/// 
/// # 功能
/// 
/// - 变换层次传播
/// - 父子关系管理
/// - 变更检测优化
/// - 全局变换计算
/// 
/// # 示例
/// 
/// ```rust
/// use anvilkit_ecs::prelude::*;
/// 
/// let mut app = App::new();
/// app.add_plugins(TransformPlugin);
/// ```
pub struct TransformPlugin;

impl crate::plugin::Plugin for TransformPlugin {
    fn build(&self, app: &mut crate::app::App) {
        use crate::schedule::{AnvilKitSchedule, AnvilKitSystemSet};
        
        // 添加变换传播系统到 PostUpdate 阶段
        app.add_systems(
            AnvilKitSchedule::PostUpdate,
            (
                sync_simple_transforms,
                propagate_transforms,
            )
                .chain()
                .in_set(AnvilKitSystemSet::Transform),
        );
    }

    fn name(&self) -> &str {
        "TransformPlugin"
    }
}

/// 同步简单变换系统
/// 
/// 对于没有父实体的实体，直接将本地变换复制到全局变换。
/// 
/// 这个系统处理根实体的变换更新，为层次传播做准备。
pub fn sync_simple_transforms(
    mut query: Query<
        (&Transform, &mut GlobalTransform),
        (Changed<Transform>, Without<Parent>),
    >,
) {
    for (transform, mut global_transform) in &mut query {
        *global_transform = GlobalTransform::from(*transform);
    }
}

/// 传播变换系统
/// 
/// 将父实体的全局变换传播到所有子实体。
/// 
/// 这个系统实现了变换层次的核心逻辑，确保子实体的全局变换
/// 正确反映其在世界空间中的位置。
pub fn propagate_transforms(
    mut root_query: Query<
        (Entity, &Children, Ref<GlobalTransform>),
        (Changed<GlobalTransform>, Without<Parent>),
    >,
    mut transform_query: Query<(&Transform, &mut GlobalTransform, Option<&Children>), With<Parent>>,
    children_query: Query<&Children, (With<Parent>, Without<GlobalTransform>)>,
) {
    // 处理根实体的变换传播
    for (_entity, children, global_transform) in &mut root_query {
        if global_transform.is_changed() {
            propagate_recursive(
                &global_transform,
                children,
                &mut transform_query,
                &children_query,
            );
        }
    }
}

/// 递归传播变换
/// 
/// 递归地将父变换传播到所有子实体及其后代。
/// 
/// # 参数
/// 
/// - `parent_global`: 父实体的全局变换
/// - `children`: 子实体列表
/// - `transform_query`: 变换查询
/// - `children_query`: 子实体查询
fn propagate_recursive(
    parent_global: &GlobalTransform,
    children: &Children,
    transform_query: &mut Query<(&Transform, &mut GlobalTransform, Option<&Children>), With<Parent>>,
    children_query: &Query<&Children, (With<Parent>, Without<GlobalTransform>)>,
) {
    // 收集需要递归处理的子实体
    let mut to_recurse = Vec::new();

    for &child_entity in children.iter() {
        // 尝试获取子实体的变换组件
        if let Ok((transform, mut global_transform, child_children)) =
            transform_query.get_mut(child_entity) {

            // 计算子实体的全局变换
            let new_global = parent_global.mul_transform(&GlobalTransform::from(*transform));
            *global_transform = new_global;

            // 如果子实体还有自己的子实体，记录下来稍后处理
            if let Some(grandchildren) = child_children {
                to_recurse.push((new_global, grandchildren.clone()));
            }
        }
    }

    // 递归处理子实体
    for (global_transform, grandchildren) in to_recurse {
        propagate_recursive(
            &global_transform,
            &grandchildren,
            transform_query,
            children_query,
        );
    }
}

/// 变换层次工具
/// 
/// 提供管理变换层次关系的便捷方法。
pub struct TransformHierarchy;

impl TransformHierarchy {
    /// 设置父子关系
    /// 
    /// 建立两个实体之间的父子关系，并更新相关组件。
    /// 
    /// # 参数
    /// 
    /// - `commands`: 命令缓冲区
    /// - `child`: 子实体
    /// - `parent`: 父实体
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_ecs::prelude::*;
    /// 
    /// fn setup_hierarchy(mut commands: Commands) {
    ///     let parent = commands.spawn((
    ///         Transform::default(),
    ///         GlobalTransform::default(),
    ///     )).id();
    ///     
    ///     let child = commands.spawn((
    ///         Transform::default(),
    ///         GlobalTransform::default(),
    ///     )).id();
    ///     
    ///     TransformHierarchy::set_parent(&mut commands, child, parent);
    /// }
    /// ```
    pub fn set_parent(commands: &mut Commands, child: Entity, parent: Entity) {
        // 为子实体添加 Parent 组件
        commands.entity(child).insert(Parent::new(parent));
        
        // 为父实体添加或更新 Children 组件
        // 使用 try_insert 来避免重复插入
        commands.entity(parent).try_insert(Children::empty());
        
        // 这里需要一个系统来实际更新 Children 列表
        // 在实际实现中，这通常通过专门的系统来处理
    }

    /// 移除父子关系
    /// 
    /// 断开子实体与其父实体的关系。
    /// 
    /// # 参数
    /// 
    /// - `commands`: 命令缓冲区
    /// - `child`: 子实体
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_ecs::prelude::*;
    /// 
    /// fn remove_from_parent(mut commands: Commands, child_entity: Entity) {
    ///     TransformHierarchy::remove_parent(&mut commands, child_entity);
    /// }
    /// ```
    pub fn remove_parent(commands: &mut Commands, child: Entity) {
        commands.entity(child).remove::<Parent>();
    }

    /// 获取实体的所有祖先
    /// 
    /// 返回从实体到根实体的所有祖先实体列表。
    /// 
    /// # 参数
    /// 
    /// - `world`: 世界引用
    /// - `entity`: 起始实体
    /// 
    /// # 返回
    /// 
    /// 祖先实体列表，从直接父实体到根实体
    pub fn get_ancestors(world: &World, entity: Entity) -> Vec<Entity> {
        let mut ancestors = Vec::new();
        let mut current = entity;
        
        while let Some(parent) = world.get::<Parent>(current) {
            ancestors.push(parent.get());
            current = parent.get();
        }
        
        ancestors
    }

    /// 获取实体的所有后代
    /// 
    /// 递归获取实体的所有子实体和后代实体。
    /// 
    /// # 参数
    /// 
    /// - `world`: 世界引用
    /// - `entity`: 起始实体
    /// 
    /// # 返回
    /// 
    /// 所有后代实体列表
    pub fn get_descendants(world: &World, entity: Entity) -> Vec<Entity> {
        let mut descendants = Vec::new();
        
        if let Some(children) = world.get::<Children>(entity) {
            for &child in children.iter() {
                descendants.push(child);
                descendants.extend(Self::get_descendants(world, child));
            }
        }
        
        descendants
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_parent_component() {
        let mut world = World::new();
        let parent_entity = world.spawn_empty().id();
        
        let parent = Parent::new(parent_entity);
        assert_eq!(parent.get(), parent_entity);
        
        let mut parent = Parent::new(parent_entity);
        let new_parent = world.spawn_empty().id();
        parent.set(new_parent);
        assert_eq!(parent.get(), new_parent);
    }

    #[test]
    fn test_children_component() {
        let mut world = World::new();
        let child1 = world.spawn_empty().id();
        let child2 = world.spawn_empty().id();
        
        let mut children = Children::empty();
        assert!(children.is_empty());
        assert_eq!(children.len(), 0);
        
        children.push(child1);
        children.push(child2);
        assert_eq!(children.len(), 2);
        assert!(children.contains(child1));
        assert!(children.contains(child2));
        
        children.remove(child1);
        assert_eq!(children.len(), 1);
        assert!(!children.contains(child1));
        assert!(children.contains(child2));
        
        children.clear();
        assert!(children.is_empty());
    }

    #[test]
    fn test_transform_hierarchy() {
        let mut world = World::new();
        
        // 创建父实体
        let parent = world.spawn((
            Transform::from_translation(Vec3::new(10.0, 0.0, 0.0)),
            GlobalTransform::default(),
        )).id();
        
        // 创建子实体
        let child = world.spawn((
            Transform::from_translation(Vec3::new(5.0, 0.0, 0.0)),
            GlobalTransform::default(),
            Parent::new(parent),
        )).id();
        
        // 测试祖先查询
        let ancestors = TransformHierarchy::get_ancestors(&world, child);
        assert_eq!(ancestors.len(), 1);
        assert_eq!(ancestors[0], parent);
        
        // 测试根实体的祖先
        let root_ancestors = TransformHierarchy::get_ancestors(&world, parent);
        assert!(root_ancestors.is_empty());
    }

    #[test]
    fn test_sync_simple_transforms() {
        let mut world = World::new();
        
        // 创建没有父实体的实体
        let entity = world.spawn((
            Transform::from_translation(Vec3::new(1.0, 2.0, 3.0)),
            GlobalTransform::default(),
        )).id();
        
        // 运行同步系统
        let mut system = IntoSystem::into_system(sync_simple_transforms);
        system.initialize(&mut world);
        system.run((), &mut world);
        
        // 验证全局变换已更新
        let global_transform = world.get::<GlobalTransform>(entity).unwrap();
        assert_eq!(global_transform.translation(), Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_children_from_vec() {
        let mut world = World::new();
        let child1 = world.spawn_empty().id();
        let child2 = world.spawn_empty().id();
        
        let children: Children = vec![child1, child2].into();
        assert_eq!(children.len(), 2);
        assert!(children.contains(child1));
        assert!(children.contains(child2));
    }

    #[test]
    fn test_children_first_last() {
        let mut world = World::new();
        let child1 = world.spawn_empty().id();
        let child2 = world.spawn_empty().id();
        let child3 = world.spawn_empty().id();
        
        let children = Children::new(vec![child1, child2, child3]);
        assert_eq!(children.first(), Some(child1));
        assert_eq!(children.last(), Some(child3));
        
        let empty_children = Children::empty();
        assert_eq!(empty_children.first(), None);
        assert_eq!(empty_children.last(), None);
    }
}
