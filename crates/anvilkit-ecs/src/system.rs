//! # 系统工具
//! 
//! 提供系统开发的工具函数和常用系统实现。
//! 
//! ## 设计理念
//! 
//! - **功能专一**: 每个系统专注于单一职责
//! - **数据驱动**: 系统通过查询组件来处理数据
//! - **无状态**: 系统本身不存储状态，状态存储在组件和资源中
//! - **可组合**: 系统可以通过调度器组合和排序
//! 
//! ## 系统类型
//! 
//! - **更新系统**: 每帧执行的常规系统
//! - **启动系统**: 应用启动时执行一次的系统
//! - **条件系统**: 满足特定条件时才执行的系统
//! - **独占系统**: 需要独占访问 World 的系统
//! 
//! ## 使用示例
//! 
//! ```rust
//! use anvilkit_ecs::prelude::*;
//! use anvilkit_ecs::schedule::AnvilKitSchedule;
//!
//! // 定义组件
//! #[derive(Component)]
//! struct Velocity {
//!     x: f32,
//!     y: f32,
//! }
//!
//! // 定义系统
//! fn movement_system(mut query: Query<(&mut Transform, &Velocity)>) {
//!     for (mut transform, velocity) in &mut query {
//!         transform.translation.x += velocity.x;
//!         transform.translation.y += velocity.y;
//!     }
//! }
//!
//! // 添加到应用
//! let mut app = App::new();
//! app.add_systems(AnvilKitSchedule::Update, movement_system);
//! ```

use bevy_ecs::prelude::*;
use anvilkit_core::time::Time;
use crate::component::{Name, Visibility, Layer};
use crate::transform::{Transform, Parent};

/// 系统工具集合
/// 
/// 提供常用的系统开发工具和辅助函数。
pub struct SystemUtils;

impl SystemUtils {
    /// 创建条件系统
    /// 
    /// 根据提供的条件函数创建一个条件系统，只有当条件为真时才执行。
    /// 
    /// # 参数
    /// 
    /// - `condition`: 条件函数
    /// - `system`: 要执行的系统
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_ecs::prelude::*;
    /// use anvilkit_ecs::schedule::AnvilKitSchedule;
    ///
    /// fn debug_condition() -> bool {
    ///     cfg!(debug_assertions)
    /// }
    ///
    /// fn debug_system() {
    ///     println!("调试模式下执行");
    /// }
    ///
    /// let mut app = App::new();
    /// app.add_systems(AnvilKitSchedule::Update, debug_system.run_if(debug_condition));
    /// ```
    pub fn conditional_system<S, C>(
        system: S,
        condition: C,
    ) -> impl IntoSystemConfigs<()>
    where
        S: IntoSystemConfigs<()>,
        C: Condition<()>,
    {
        system.run_if(condition)
    }

}

/// 调试系统
/// 
/// 提供调试和开发时有用的系统。
pub struct DebugSystems;

impl DebugSystems {
    /// 实体计数系统
    /// 
    /// 定期打印当前世界中的实体数量。
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_ecs::prelude::*;
    /// use anvilkit_ecs::schedule::AnvilKitSchedule;
    ///
    /// let mut app = App::new();
    /// app.add_systems(AnvilKitSchedule::Update, DebugSystems::entity_count_system);
    /// ```
    pub fn entity_count_system(query: Query<Entity>) {
        let count = query.iter().count();
        if count > 0 {
            log::debug!("当前实体数量: {}", count);
        }
    }

    /// 名称实体列表系统
    /// 
    /// 打印所有带名称的实体。
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_ecs::prelude::*;
    /// use anvilkit_ecs::schedule::AnvilKitSchedule;
    ///
    /// let mut app = App::new();
    /// app.add_systems(AnvilKitSchedule::Update, DebugSystems::named_entities_system);
    /// ```
    pub fn named_entities_system(query: Query<(Entity, &Name)>) {
        for (entity, name) in &query {
            log::debug!("实体 {:?}: {}", entity, name.as_str());
        }
    }

    /// 变换调试系统
    /// 
    /// 打印所有实体的变换信息。
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_ecs::prelude::*;
    /// use anvilkit_ecs::schedule::AnvilKitSchedule;
    ///
    /// let mut app = App::new();
    /// app.add_systems(AnvilKitSchedule::Update, DebugSystems::transform_debug_system);
    /// ```
    pub fn transform_debug_system(query: Query<(Entity, &Transform), With<Name>>) {
        for (entity, transform) in &query {
            log::debug!(
                "实体 {:?} 位置: ({:.2}, {:.2}, {:.2})",
                entity,
                transform.translation.x,
                transform.translation.y,
                transform.translation.z
            );
        }
    }

    /// 性能监控系统
    /// 
    /// 监控和报告系统性能信息。
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_ecs::prelude::*;
    /// use anvilkit_ecs::schedule::AnvilKitSchedule;
    ///
    /// let mut app = App::new();
    /// app.add_systems(AnvilKitSchedule::Update, DebugSystems::performance_monitor_system);
    /// ```
    pub fn performance_monitor_system(time: Res<Time>) {
        let dt = time.delta_seconds();
        if dt <= 0.0 {
            return; // 首帧 delta 为零，跳过避免除零
        }
        // 每秒报告一次性能信息
        let elapsed = time.elapsed_seconds();
        if elapsed > 0.0 && (elapsed % 1.0) < dt {
            log::info!(
                "FPS: {:.1}, 帧时间: {:.3}ms",
                1.0 / dt,
                dt * 1000.0
            );
        }
    }
}

/// 实用系统
/// 
/// 提供常用的实用系统实现。
pub struct UtilitySystems;

impl UtilitySystems {
    /// 时间更新系统
    /// 
    /// 更新全局时间资源。
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_ecs::prelude::*;
    /// use anvilkit_ecs::schedule::AnvilKitSchedule;
    ///
    /// let mut app = App::new();
    /// app.add_systems(AnvilKitSchedule::PreUpdate, UtilitySystems::time_update_system);
    /// ```
    pub fn time_update_system(mut time: ResMut<Time>) {
        time.update();
    }

    /// 可见性过滤系统
    /// 
    /// 根据可见性组件过滤实体的处理。
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_ecs::prelude::*;
    /// 
    /// fn render_system(query: Query<&Transform, With<Visibility>>) {
    ///     // 只处理可见的实体
    ///     for transform in &query {
    ///         // 渲染逻辑
    ///     }
    /// }
    /// ```
    pub fn visibility_filter_system(
        mut query: Query<(Entity, &mut Visibility, Option<&Parent>), With<Transform>>,
    ) {
        // Collect inherited entities to resolve
        let to_resolve: Vec<(Entity, Option<Entity>)> = query.iter()
            .filter(|(_, vis, _)| vis.is_inherited())
            .map(|(e, _, parent)| (e, parent.map(|p| p.get())))
            .collect();

        for (entity, parent_entity) in to_resolve {
            let parent_visible = parent_entity
                .and_then(|pe| query.get(pe).ok())
                .map(|(_, vis, _)| vis.is_visible())
                .unwrap_or(true);

            if let Ok((_, mut vis, _)) = query.get_mut(entity) {
                if parent_visible {
                    *vis = Visibility::Visible;
                } else {
                    *vis = Visibility::Hidden;
                }
            }
        }
    }

    /// 层级排序系统
    /// 
    /// 根据层级组件对实体进行排序处理。
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_ecs::prelude::*;
    /// use anvilkit_ecs::schedule::AnvilKitSchedule;
    ///
    /// let mut app = App::new();
    /// app.add_systems(AnvilKitSchedule::PostUpdate, UtilitySystems::layer_sorting_system);
    /// ```
    pub fn layer_sorting_system(query: Query<(Entity, &Layer), Changed<Layer>>) {
        let mut entities: Vec<_> = query.iter().collect();
        entities.sort_by_key(|(_, layer)| layer.value());

        // 这里可以添加基于排序结果的处理逻辑
        for (entity, layer) in entities {
            // 处理排序后的实体
            log::debug!("实体 {:?} 在层级 {}", entity, layer.value());
        }
    }

    /// 清理系统
    /// 
    /// 清理标记为删除的实体和组件。
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_ecs::prelude::*;
    /// use anvilkit_ecs::schedule::AnvilKitSchedule;
    ///
    /// #[derive(Component)]
    /// struct ToDelete;
    ///
    /// let mut app = App::new();
    /// app.add_systems(AnvilKitSchedule::Cleanup, UtilitySystems::cleanup_system::<ToDelete>);
    /// ```
    pub fn cleanup_system<T: Component>(
        mut commands: Commands,
        query: Query<Entity, With<T>>,
    ) {
        for entity in &query {
            commands.entity(entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;
    use crate::schedule::AnvilKitSchedule;

    #[derive(Component)]
    struct TestComponent {
        value: i32,
    }

    #[derive(Component)]
    struct ToDelete;

    fn test_system(mut query: Query<&mut TestComponent>) {
        for mut component in &mut query {
            component.value += 1;
        }
    }

    fn condition_true() -> bool {
        true
    }

    fn condition_false() -> bool {
        false
    }

    #[test]
    fn test_debug_systems() {
        let mut app = App::new();
        app.init_resource::<Time>();
        
        // 创建一些测试实体
        app.world.spawn((Name::new("测试实体1"), TestComponent { value: 0 }));
        app.world.spawn((Name::new("测试实体2"), TestComponent { value: 0 }));
        
        // 添加调试系统
        app.add_systems(AnvilKitSchedule::Update, (
            DebugSystems::entity_count_system,
            DebugSystems::named_entities_system,
        ));
        
        // 执行一次更新
        app.update();
    }

    #[test]
    fn test_utility_systems() {
        let mut app = App::new();
        app.init_resource::<Time>();
        
        // 创建带可见性的实体
        app.world.spawn((
            Transform::default(),
            Visibility::Inherited,
        ));
        
        // 添加实用系统
        app.add_systems(AnvilKitSchedule::Update, UtilitySystems::visibility_filter_system);
        
        // 执行更新
        app.update();
        
        // 验证可见性已更新
        let mut query = app.world.query::<&Visibility>();
        for visibility in query.iter(&app.world) {
            assert_eq!(*visibility, Visibility::Visible);
        }
    }

    #[test]
    fn test_cleanup_system() {
        let mut app = App::new();
        
        // 创建要删除的实体
        let entity = app.world.spawn((
            Name::new("待删除实体"),
            ToDelete,
        )).id();
        
        // 添加清理系统
        app.add_systems(AnvilKitSchedule::Update, UtilitySystems::cleanup_system::<ToDelete>);
        
        // 验证实体存在
        assert!(app.world.get_entity(entity).is_some());
        
        // 执行更新
        app.update();
        
        // 验证实体已被删除
        assert!(app.world.get_entity(entity).is_none());
    }

    #[test]
    fn test_conditional_system() {
        let mut app = App::new();
        
        // 创建测试实体
        app.world.spawn(TestComponent { value: 0 });
        
        // 添加条件系统
        app.add_systems(AnvilKitSchedule::Update, (
            test_system.run_if(condition_true),
            test_system.run_if(condition_false),
        ));
        
        // 执行更新
        app.update();
        
        // 验证只有条件为真的系统执行了
        let component = app.world.query::<&TestComponent>().single(&app.world);
        assert_eq!(component.value, 1); // 只执行了一次
    }

    #[test]
    fn test_layer_sorting_system() {
        let mut app = App::new();

        // 创建不同层级的实体
        app.world.spawn(Layer::new(3));
        app.world.spawn(Layer::new(1));
        app.world.spawn(Layer::new(2));

        // 添加层级排序系统
        app.add_systems(AnvilKitSchedule::Update, UtilitySystems::layer_sorting_system);

        // 执行更新
        app.update();

        // 验证系统执行（通过日志输出验证，这里只是确保不崩溃）
    }

    #[test]
    fn test_system_with_multiple_components() {
        use crate::component::Tag;

        let mut world = World::new();

        world.spawn((Name::new("entity1"), Tag::new("player"), Layer::new(1)));
        world.spawn((Name::new("entity2"), Tag::new("enemy"), Layer::new(2)));
        world.spawn((Name::new("entity3"), Tag::new("enemy"), Layer::new(3)));

        let mut count = 0;
        let mut query = world.query::<(&Name, &Tag)>();
        for (_name, _tag) in query.iter(&world) {
            count += 1;
        }
        assert_eq!(count, 3);
    }

    #[test]
    fn test_visibility_filter() {
        let mut world = World::new();

        world.spawn((Name::new("visible"), Visibility::Visible));
        world.spawn((Name::new("hidden"), Visibility::Hidden));
        world.spawn((Name::new("also_visible"), Visibility::Visible));

        let mut visible_count = 0;
        let mut query = world.query::<(&Name, &Visibility)>();
        for (_name, vis) in query.iter(&world) {
            if vis.is_visible() {
                visible_count += 1;
            }
        }
        assert_eq!(visible_count, 2);
    }
}
