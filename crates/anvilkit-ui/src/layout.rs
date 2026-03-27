//! Flexbox layout engine backed by taffy.

use bevy_ecs::prelude::*;
use taffy::prelude as tf;

use crate::style::*;

/// Flexbox layout engine.
#[derive(Resource)]
pub struct UiLayoutEngine {
    #[allow(dead_code)]
    taffy: tf::TaffyTree<()>,
}

impl UiLayoutEngine {
    pub fn new() -> Self {
        Self { taffy: tf::TaffyTree::<()>::new() }
    }

    fn convert_val_dimension(val: &Val) -> tf::Dimension {
        match val {
            Val::Auto => tf::Dimension::Auto,
            Val::Px(v) => tf::Dimension::Length(*v),
            Val::Percent(v) => tf::Dimension::Percent(*v / 100.0),
        }
    }

    fn convert_val_lpa(val: &Val) -> tf::LengthPercentageAuto {
        match val {
            Val::Auto => tf::LengthPercentageAuto::Auto,
            Val::Px(v) => tf::LengthPercentageAuto::Length(*v),
            Val::Percent(v) => tf::LengthPercentageAuto::Percent(*v / 100.0),
        }
    }

    fn convert_style(node: &UiNode) -> tf::Style {
        let s = &node.style;
        let flex_dir = match s.flex_direction {
            FlexDirection::Row => tf::FlexDirection::Row,
            FlexDirection::RowReverse => tf::FlexDirection::RowReverse,
            FlexDirection::Column => tf::FlexDirection::Column,
            FlexDirection::ColumnReverse => tf::FlexDirection::ColumnReverse,
        };
        let justify = match s.justify_content {
            Align::Start => tf::JustifyContent::Start,
            Align::Center => tf::JustifyContent::Center,
            Align::End => tf::JustifyContent::End,
            Align::SpaceBetween => tf::JustifyContent::SpaceBetween,
            Align::SpaceAround => tf::JustifyContent::SpaceAround,
            _ => tf::JustifyContent::Start,
        };
        let align = match s.align_items {
            Align::Start => tf::AlignItems::Start,
            Align::Center => tf::AlignItems::Center,
            Align::End => tf::AlignItems::End,
            Align::Stretch => tf::AlignItems::Stretch,
            _ => tf::AlignItems::Stretch,
        };

        let p = &s.padding;
        let m = &s.margin;

        // Estimate text size for leaf nodes
        let mut min_w = Self::convert_val_dimension(&s.min_width);
        let mut min_h = Self::convert_val_dimension(&s.min_height);
        if let Some(ref text) = node.text {
            let char_w = text.font_size * 0.6;
            let text_w = char_w * text.content.len() as f32 + p[1] + p[3];
            let text_h = text.font_size + p[0] + p[2];
            if matches!(min_w, tf::Dimension::Auto) {
                min_w = tf::Dimension::Length(text_w);
            }
            if matches!(min_h, tf::Dimension::Auto) {
                min_h = tf::Dimension::Length(text_h);
            }
        }

        tf::Style {
            display: tf::Display::Flex,
            flex_direction: flex_dir,
            justify_content: Some(justify),
            align_items: Some(align),
            size: tf::Size {
                width: Self::convert_val_dimension(&s.width),
                height: Self::convert_val_dimension(&s.height),
            },
            min_size: tf::Size { width: min_w, height: min_h },
            max_size: tf::Size {
                width: Self::convert_val_dimension(&s.max_width),
                height: Self::convert_val_dimension(&s.max_height),
            },
            padding: tf::Rect {
                top: tf::LengthPercentage::Length(p[0]),
                right: tf::LengthPercentage::Length(p[1]),
                bottom: tf::LengthPercentage::Length(p[2]),
                left: tf::LengthPercentage::Length(p[3]),
            },
            margin: tf::Rect {
                top: Self::convert_val_lpa(&Val::Px(m[0])),
                right: Self::convert_val_lpa(&Val::Px(m[1])),
                bottom: Self::convert_val_lpa(&Val::Px(m[2])),
                left: Self::convert_val_lpa(&Val::Px(m[3])),
            },
            gap: tf::Size {
                width: tf::LengthPercentage::Length(s.gap),
                height: tf::LengthPercentage::Length(s.gap),
            },
            flex_grow: s.flex_grow,
            ..Default::default()
        }
    }

    /// Compute layout for a flat list of root-level UI nodes.
    /// Returns `(entity, [x, y, w, h])` for each node.
    pub fn compute_layout(
        &mut self,
        nodes: &[(Entity, &UiNode)],
        screen_width: f32,
        screen_height: f32,
    ) -> Vec<(Entity, [f32; 4])> {
        let mut tree: tf::TaffyTree<()> = tf::TaffyTree::<()>::new();
        let mut results = Vec::with_capacity(nodes.len());

        // Build taffy nodes — each root node's children are laid out inside it
        let mut child_ids = Vec::new();
        let mut entity_map = Vec::new();

        for (entity, node) in nodes {
            let style = Self::convert_style(node);
            let id = tree.new_leaf(style).unwrap();
            child_ids.push(id);
            entity_map.push(*entity);
        }

        // Create a container for all root nodes (column layout, full screen)
        let container_style = tf::Style {
            display: tf::Display::Flex,
            flex_direction: tf::FlexDirection::Column,
            size: tf::Size {
                width: tf::Dimension::Length(screen_width),
                height: tf::Dimension::Length(screen_height),
            },
            ..Default::default()
        };
        let container = tree.new_with_children(container_style, &child_ids).unwrap();
        tree.compute_layout(container, tf::Size {
            width: tf::AvailableSpace::Definite(screen_width),
            height: tf::AvailableSpace::Definite(screen_height),
        }).unwrap();

        for (i, entity) in entity_map.iter().enumerate() {
            let layout = tree.layout(child_ids[i]).unwrap();
            results.push((*entity, [
                layout.location.x,
                layout.location.y,
                layout.size.width,
                layout.size.height,
            ]));
        }

        results
    }

    /// Compute layout for a tree of UI nodes with parent-child relationships.
    /// `children_map` maps parent Entity → list of child Entities.
    pub fn compute_tree_layout(
        &mut self,
        nodes: &[(Entity, &UiNode)],
        roots: &[Entity],
        children_map: &std::collections::HashMap<Entity, Vec<Entity>>,
        screen_width: f32,
        screen_height: f32,
    ) -> Vec<(Entity, [f32; 4])> {
        let mut tree = tf::TaffyTree::<()>::new();
        let mut results = Vec::new();

        let node_map: std::collections::HashMap<Entity, &UiNode> =
            nodes.iter().map(|(e, n)| (*e, *n)).collect();

        // Recursively build taffy tree
        fn build_node(
            tree: &mut tf::TaffyTree<()>,
            entity: Entity,
            node_map: &std::collections::HashMap<Entity, &UiNode>,
            children_map: &std::collections::HashMap<Entity, Vec<Entity>>,
            taffy_map: &mut std::collections::HashMap<Entity, tf::NodeId>,
        ) {
            let Some(node) = node_map.get(&entity) else { return };
            let style = UiLayoutEngine::convert_style(node);

            let children = children_map.get(&entity);
            if let Some(child_entities) = children {
                // Build children first
                for &child in child_entities {
                    if !taffy_map.contains_key(&child) {
                        build_node(tree, child, node_map, children_map, taffy_map);
                    }
                }
                let child_ids: Vec<tf::NodeId> = child_entities.iter()
                    .filter_map(|e| taffy_map.get(e).copied())
                    .collect();
                let id = tree.new_with_children(style, &child_ids).unwrap();
                taffy_map.insert(entity, id);
            } else {
                let id = tree.new_leaf(style).unwrap();
                taffy_map.insert(entity, id);
            }
        }

        let mut taffy_map = std::collections::HashMap::new();
        for &root in roots {
            build_node(&mut tree, root, &node_map, children_map, &mut taffy_map);
        }

        // Create screen container
        let root_ids: Vec<tf::NodeId> = roots.iter()
            .filter_map(|e| taffy_map.get(e).copied())
            .collect();

        let container_style = tf::Style {
            display: tf::Display::Flex,
            flex_direction: tf::FlexDirection::Column,
            size: tf::Size {
                width: tf::Dimension::Length(screen_width),
                height: tf::Dimension::Length(screen_height),
            },
            ..Default::default()
        };
        let container = tree.new_with_children(container_style, &root_ids).unwrap();
        tree.compute_layout(container, tf::Size {
            width: tf::AvailableSpace::Definite(screen_width),
            height: tf::AvailableSpace::Definite(screen_height),
        }).unwrap();

        // Collect results recursively with absolute positions
        fn collect_results(
            tree: &tf::TaffyTree<()>,
            entity: Entity,
            taffy_map: &std::collections::HashMap<Entity, tf::NodeId>,
            children_map: &std::collections::HashMap<Entity, Vec<Entity>>,
            results: &mut Vec<(Entity, [f32; 4])>,
            parent_x: f32,
            parent_y: f32,
        ) {
            let Some(&node_id) = taffy_map.get(&entity) else { return };
            let layout = tree.layout(node_id).unwrap();
            let abs_x = parent_x + layout.location.x;
            let abs_y = parent_y + layout.location.y;
            results.push((entity, [abs_x, abs_y, layout.size.width, layout.size.height]));

            if let Some(children) = children_map.get(&entity) {
                for &child in children {
                    collect_results(tree, child, taffy_map, children_map, results, abs_x, abs_y);
                }
            }
        }

        for &root in roots {
            let layout = taffy_map.get(&root)
                .and_then(|id| tree.layout(*id).ok());
            if let Some(l) = layout {
                results.push((root, [l.location.x, l.location.y, l.size.width, l.size.height]));
                if let Some(children) = children_map.get(&root) {
                    for &child in children {
                        collect_results(&tree, child, &taffy_map, children_map, &mut results, l.location.x, l.location.y);
                    }
                }
            }
        }

        results
    }
}

impl Default for UiLayoutEngine {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_engine_flat() {
        let mut engine = UiLayoutEngine::new();
        let mut world = bevy_ecs::world::World::new();

        let e1 = world.spawn_empty().id();
        let e2 = world.spawn_empty().id();

        let n1 = UiNode {
            style: UiStyle { height: Val::Px(50.0), width: Val::Percent(100.0), ..Default::default() },
            ..Default::default()
        };
        let n2 = UiNode {
            style: UiStyle { height: Val::Px(30.0), width: Val::Percent(100.0), ..Default::default() },
            ..Default::default()
        };

        let results = engine.compute_layout(&[(e1, &n1), (e2, &n2)], 800.0, 600.0);
        assert_eq!(results.len(), 2);
        // First node at y=0, second below it
        assert_eq!(results[0].1[1], 0.0);
        assert!(results[1].1[1] >= 50.0);
    }

    #[test]
    fn test_layout_engine_tree() {
        let mut engine = UiLayoutEngine::new();
        let mut world = bevy_ecs::world::World::new();

        let parent = world.spawn_empty().id();
        let child1 = world.spawn_empty().id();
        let child2 = world.spawn_empty().id();

        let parent_node = UiNode {
            style: UiStyle {
                flex_direction: FlexDirection::Column,
                width: Val::Px(200.0),
                height: Val::Px(100.0),
                ..Default::default()
            },
            ..Default::default()
        };
        let child_node = UiNode {
            style: UiStyle { height: Val::Px(40.0), ..Default::default() },
            ..Default::default()
        };

        let mut children_map = std::collections::HashMap::new();
        children_map.insert(parent, vec![child1, child2]);

        let results = engine.compute_tree_layout(
            &[(parent, &parent_node), (child1, &child_node), (child2, &child_node)],
            &[parent],
            &children_map,
            800.0, 600.0,
        );
        assert!(results.len() >= 3);
    }
}
