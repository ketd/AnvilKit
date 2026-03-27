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
use bytemuck::{Pod, Zeroable};
use super::shared::MatrixUniform;
use wgpu::util::DeviceExt;
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
    /// Items laid out horizontally from left to right.
    Row,
    /// Items laid out horizontally from right to left.
    RowReverse,
    /// Items laid out vertically from top to bottom.
    Column,
    /// Items laid out vertically from bottom to top.
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
    /// Align items to the start of the axis.
    Start,
    /// Align items to the center of the axis.
    Center,
    /// Align items to the end of the axis.
    End,
    /// Stretch items to fill the cross axis.
    Stretch,
    /// Distribute items with equal space between them.
    SpaceBetween,
    /// Distribute items with equal space around them.
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
    /// Automatically determined by the layout engine.
    Auto,
    /// Absolute size in pixels.
    Px(f32),
    /// Relative size as a percentage of the parent.
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
    /// Main axis direction for child layout.
    pub flex_direction: FlexDirection,
    /// Alignment of children along the main axis.
    pub justify_content: Align,
    /// Alignment of children along the cross axis.
    pub align_items: Align,
    /// Desired width of the element.
    pub width: Val,
    /// Desired height of the element.
    pub height: Val,
    /// Minimum width constraint.
    pub min_width: Val,
    /// Minimum height constraint.
    pub min_height: Val,
    /// Maximum width constraint.
    pub max_width: Val,
    /// Maximum height constraint.
    pub max_height: Val,
    /// Inner padding in pixels [top, right, bottom, left].
    pub padding: [f32; 4],
    /// Outer margin in pixels [top, right, bottom, left].
    pub margin: [f32; 4],
    /// Spacing between child elements in pixels.
    pub gap: f32,
    /// How much this element grows to fill available space.
    pub flex_grow: f32,
    /// How much this element shrinks when space is insufficient.
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
    /// The text string to display.
    pub content: String,
    /// Font size in pixels.
    pub font_size: f32,
    /// Text color [R, G, B, A].
    pub color: [f32; 4],
    /// Font family name.
    pub font_family: String,
}

impl UiText {
    /// Creates a new `UiText` with default font size and white color.
    pub fn new(content: &str) -> Self {
        Self {
            content: content.to_string(),
            font_size: 16.0,
            color: [1.0, 1.0, 1.0, 1.0],
            font_family: "default".to_string(),
        }
    }

    /// Sets the font size and returns self for chaining.
    pub fn with_font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    /// Sets the text color and returns self for chaining.
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

// ---------------------------------------------------------------------------
//  UiLayoutEngine — taffy-based layout computation
// ---------------------------------------------------------------------------

use taffy::prelude as tf;

/// UI 布局引擎
///
/// 将 UiNode 树转换为 taffy 布局树，计算 computed_rect。
pub struct UiLayoutEngine {
    /// The underlying taffy layout tree.
    taffy: tf::TaffyTree,
}

impl UiLayoutEngine {
    /// Creates a new layout engine with an empty taffy tree.
    pub fn new() -> Self {
        Self {
            taffy: tf::TaffyTree::new(),
        }
    }

    /// 将 UiStyle 转换为 taffy Style
    fn convert_style(style: &UiStyle, node: &UiNode) -> tf::Style {
        let to_dim = |v: &Val| match v {
            Val::Auto => tf::Dimension::Auto,
            Val::Px(px) => tf::Dimension::Length(*px),
            Val::Percent(pct) => tf::Dimension::Percent(*pct / 100.0),
        };

        let to_len_pct_auto = |v: &Val| match v {
            Val::Auto => tf::LengthPercentageAuto::Auto,
            Val::Px(px) => tf::LengthPercentageAuto::Length(*px),
            Val::Percent(pct) => tf::LengthPercentageAuto::Percent(*pct / 100.0),
        };

        let flex_dir = match style.flex_direction {
            FlexDirection::Row => tf::FlexDirection::Row,
            FlexDirection::RowReverse => tf::FlexDirection::RowReverse,
            FlexDirection::Column => tf::FlexDirection::Column,
            FlexDirection::ColumnReverse => tf::FlexDirection::ColumnReverse,
        };

        let justify = match style.justify_content {
            Align::Start => Some(tf::JustifyContent::Start),
            Align::Center => Some(tf::JustifyContent::Center),
            Align::End => Some(tf::JustifyContent::End),
            Align::SpaceBetween => Some(tf::JustifyContent::SpaceBetween),
            Align::SpaceAround => Some(tf::JustifyContent::SpaceAround),
            _ => Some(tf::JustifyContent::Start),
        };

        let align = match style.align_items {
            Align::Start => Some(tf::AlignItems::Start),
            Align::Center => Some(tf::AlignItems::Center),
            Align::End => Some(tf::AlignItems::End),
            Align::Stretch => Some(tf::AlignItems::Stretch),
            _ => Some(tf::AlignItems::Start),
        };

        let _ = node; // node is reserved for text sizing hints in the future

        tf::Style {
            display: tf::Display::Flex,
            flex_direction: flex_dir,
            justify_content: justify,
            align_items: align,
            size: tf::Size {
                width: to_dim(&style.width),
                height: to_dim(&style.height),
            },
            min_size: tf::Size {
                width: to_dim(&style.min_width),
                height: to_dim(&style.min_height),
            },
            max_size: tf::Size {
                width: to_dim(&style.max_width),
                height: to_dim(&style.max_height),
            },
            padding: tf::Rect {
                top: tf::LengthPercentage::Length(style.padding[0]),
                right: tf::LengthPercentage::Length(style.padding[1]),
                bottom: tf::LengthPercentage::Length(style.padding[2]),
                left: tf::LengthPercentage::Length(style.padding[3]),
            },
            margin: tf::Rect {
                top: to_len_pct_auto(&Val::Px(style.margin[0])),
                right: to_len_pct_auto(&Val::Px(style.margin[1])),
                bottom: to_len_pct_auto(&Val::Px(style.margin[2])),
                left: to_len_pct_auto(&Val::Px(style.margin[3])),
            },
            gap: tf::Size {
                width: tf::LengthPercentage::Length(style.gap),
                height: tf::LengthPercentage::Length(style.gap),
            },
            flex_grow: style.flex_grow,
            flex_shrink: style.flex_shrink,
            ..Default::default()
        }
    }

    /// 计算一组根级 UiNode 的布局
    ///
    /// 返回 `(entity, [x, y, width, height])` 列表
    pub fn compute_layout(
        &mut self,
        nodes: &[(Entity, &UiNode)],
        container_width: f32,
        container_height: f32,
    ) -> Vec<(Entity, [f32; 4])> {
        self.taffy = tf::TaffyTree::new();
        let mut results = Vec::new();
        let mut children = Vec::new();

        for (entity, node) in nodes {
            let style = Self::convert_style(&node.style, node);
            let taffy_node = match self.taffy.new_leaf(style) {
                Ok(node) => node,
                Err(e) => {
                    log::error!("Taffy layout error: {:?}", e);
                    continue;
                }
            };
            children.push((*entity, taffy_node));
        }

        // Root container
        let child_ids: Vec<_> = children.iter().map(|(_, n)| *n).collect();
        let root = self.taffy.new_with_children(
            tf::Style {
                display: tf::Display::Flex,
                flex_direction: tf::FlexDirection::Column,
                size: tf::Size {
                    width: tf::Dimension::Length(container_width),
                    height: tf::Dimension::Length(container_height),
                },
                ..Default::default()
            },
            &child_ids,
        ).unwrap_or_else(|e| {
            log::error!("Taffy root layout error: {:?}", e);
            // Return a fallback node — create an empty leaf as a placeholder root
            self.taffy.new_leaf(tf::Style::default()).unwrap_or_else(|_| {
                panic!("Taffy failed to create even a fallback leaf node");
            })
        });

        let available = tf::Size {
            width: tf::AvailableSpace::Definite(container_width),
            height: tf::AvailableSpace::Definite(container_height),
        };
        self.taffy.compute_layout(root, available).ok();

        for (entity, taffy_node) in &children {
            if let Ok(layout) = self.taffy.layout(*taffy_node) {
                results.push((*entity, [
                    layout.location.x,
                    layout.location.y,
                    layout.size.width,
                    layout.size.height,
                ]));
            }
        }

        results
    }
}

impl Default for UiLayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
//  UiRenderer — GPU pipeline for UI rectangles
// ---------------------------------------------------------------------------

const UI_SHADER: &str = include_str!("../shaders/ui.wgsl");

/// UI 矩形 GPU 顶点
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct UiVertex {
    /// Normalized corner position within the quad (0..1).
    pub position: [f32; 2],
    /// Top-left corner of the rectangle in screen pixels.
    pub rect_min: [f32; 2],
    /// Width and height of the rectangle in screen pixels.
    pub rect_size: [f32; 2],
    /// Background fill color [R, G, B, A].
    pub color: [f32; 4],
    /// Border stroke color [R, G, B, A].
    pub border_color: [f32; 4],
    /// Packed parameters: [border_radius, border_width, 0, 0].
    pub params: [f32; 4],
}

impl UiVertex {
    fn layout() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: &[wgpu::VertexAttribute] = &[
            wgpu::VertexAttribute { offset: 0, shader_location: 0, format: wgpu::VertexFormat::Float32x2 },
            wgpu::VertexAttribute { offset: 8, shader_location: 1, format: wgpu::VertexFormat::Float32x2 },
            wgpu::VertexAttribute { offset: 16, shader_location: 2, format: wgpu::VertexFormat::Float32x2 },
            wgpu::VertexAttribute { offset: 24, shader_location: 3, format: wgpu::VertexFormat::Float32x4 },
            wgpu::VertexAttribute { offset: 40, shader_location: 4, format: wgpu::VertexFormat::Float32x4 },
            wgpu::VertexAttribute { offset: 56, shader_location: 5, format: wgpu::VertexFormat::Float32x4 },
        ];
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<UiVertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: ATTRIBUTES,
        }
    }
}

/// GPU UI 渲染器
pub struct UiRenderer {
    /// The wgpu render pipeline for UI rectangles.
    pub pipeline: wgpu::RenderPipeline,
    /// Uniform buffer holding the orthographic projection matrix.
    pub ortho_buffer: wgpu::Buffer,
    /// Bind group for the orthographic projection uniform.
    pub ortho_bind_group: wgpu::BindGroup,
    /// Cached vertex buffer for per-frame reuse.
    cached_vb: super::shared::CachedBuffer,
}

impl UiRenderer {
    /// Creates the UI render pipeline, uniform buffer, and bind group.
    pub fn new(device: &super::RenderDevice, format: wgpu::TextureFormat) -> Self {
        let shader = device.device().create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("UI Shader"),
            source: wgpu::ShaderSource::Wgsl(UI_SHADER.into()),
        });

        let ortho_bgl = device.device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("UI Ortho BGL"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let pipeline_layout = device.device().create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("UI Pipeline Layout"),
            bind_group_layouts: &[&ortho_bgl],
            push_constant_ranges: &[],
        });

        let pipeline = device.device().create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("UI Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[UiVertex::layout()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let initial = MatrixUniform::identity();
        let ortho_buffer = device.device().create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("UI Ortho UB"),
            contents: bytemuck::bytes_of(&initial),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let ortho_bg = device.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("UI Ortho BG"),
            layout: &ortho_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: ortho_buffer.as_entire_binding(),
            }],
        });

        Self {
            pipeline,
            ortho_buffer,
            ortho_bind_group: ortho_bg,
            cached_vb: super::shared::CachedBuffer::vertex("UI VB (cached)"),
        }
    }

    /// 从 computed_rect 列表渲染 UI 矩形
    pub fn render(
        &mut self,
        device: &super::RenderDevice,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        nodes: &[&UiNode],
        screen_width: f32,
        screen_height: f32,
    ) {
        if nodes.is_empty() {
            return;
        }

        // Update ortho
        let ortho = glam::Mat4::orthographic_lh(0.0, screen_width, screen_height, 0.0, -1.0, 1.0);
        let uniform = MatrixUniform::from_mat4(&ortho);
        device.queue().write_buffer(&self.ortho_buffer, 0, bytemuck::bytes_of(&uniform));

        // Build vertices
        let mut vertices = Vec::new();
        for node in nodes {
            if !node.visible || node.computed_rect[2] <= 0.0 || node.computed_rect[3] <= 0.0 {
                continue;
            }
            let [x, y, w, h] = node.computed_rect;
            let params = [node.border_radius, node.border_width, 0.0, 0.0];

            // 6 vertices (2 triangles)
            let corners = [
                [0.0f32, 0.0], [1.0, 0.0], [1.0, 1.0],
                [0.0, 0.0], [1.0, 1.0], [0.0, 1.0],
            ];
            for corner in &corners {
                vertices.push(UiVertex {
                    position: *corner,
                    rect_min: [x, y],
                    rect_size: [w, h],
                    color: node.background_color,
                    border_color: node.border_color,
                    params,
                });
            }
        }

        if vertices.is_empty() {
            return;
        }

        // Reuse cached buffer if large enough
        let data = bytemuck::cast_slice(&vertices);
        let vb = self.cached_vb.ensure_and_write(device.device(), device.queue(), data);

        {
            let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("UI Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            rp.set_pipeline(&self.pipeline);
            rp.set_bind_group(0, &self.ortho_bind_group, &[]);
            rp.set_vertex_buffer(0, vb.slice(..));
            rp.draw(0..vertices.len() as u32, 0..1);
        }
    }
}

// ---------------------------------------------------------------------------
//  UI Event System — hit testing, click, hover
// ---------------------------------------------------------------------------

/// UI 交互事件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiEventKind {
    /// Mouse entered the node's bounds.
    HoverEnter,
    /// Mouse left the node's bounds.
    HoverLeave,
    /// Mouse button pressed within the node.
    Click,
}

/// 单个 UI 事件
#[derive(Debug, Clone, Copy)]
pub struct UiEvent {
    /// The entity that received the event.
    pub entity: Entity,
    /// The type of interaction.
    pub kind: UiEventKind,
}

/// UI 事件收集器（ECS Resource）
///
/// 每帧由 UI 系统填充，游戏代码读取。
#[derive(Resource, Default)]
pub struct UiEvents {
    /// Events collected this frame.
    pub events: Vec<UiEvent>,
    /// Currently hovered entity (if any).
    pub hovered: Option<Entity>,
}

impl UiEvents {
    /// Clear all events (called at frame start).
    pub fn clear(&mut self) { self.events.clear(); }
    /// Iterate over events this frame.
    pub fn iter(&self) -> impl Iterator<Item = &UiEvent> { self.events.iter() }
    /// Check if a specific entity was clicked this frame.
    pub fn was_clicked(&self, entity: Entity) -> bool {
        self.events.iter().any(|e| e.entity == entity && e.kind == UiEventKind::Click)
    }
    /// Check if a specific entity is currently hovered.
    pub fn is_hovered(&self, entity: Entity) -> bool {
        self.hovered == Some(entity)
    }
}

/// 对一组已计算布局的 UI 节点执行 hit test
///
/// `mouse_pos` 是屏幕像素坐标。
/// `nodes` 按 z-order 排列（后面的覆盖前面的）。
/// 返回最顶层被命中的实体。
pub fn ui_hit_test(
    mouse_pos: glam::Vec2,
    nodes: &[(Entity, [f32; 4])], // (entity, [x, y, w, h])
) -> Option<Entity> {
    // Iterate in reverse to find the topmost node
    for &(entity, [x, y, w, h]) in nodes.iter().rev() {
        if mouse_pos.x >= x && mouse_pos.x <= x + w
            && mouse_pos.y >= y && mouse_pos.y <= y + h
        {
            return Some(entity);
        }
    }
    None
}

/// 处理 UI 交互：根据鼠标位置和按键状态生成事件。
///
/// 每帧调用一次。
pub fn process_ui_interactions(
    mouse_pos: glam::Vec2,
    mouse_just_pressed: bool,
    nodes: &[(Entity, [f32; 4])],
    ui_events: &mut UiEvents,
) {
    ui_events.clear();

    let hit = ui_hit_test(mouse_pos, nodes);
    let prev_hovered = ui_events.hovered;

    // Hover leave
    if let Some(prev) = prev_hovered {
        if hit != Some(prev) {
            ui_events.events.push(UiEvent { entity: prev, kind: UiEventKind::HoverLeave });
        }
    }

    // Hover enter
    if let Some(current) = hit {
        if prev_hovered != Some(current) {
            ui_events.events.push(UiEvent { entity: current, kind: UiEventKind::HoverEnter });
        }
        // Click
        if mouse_just_pressed {
            ui_events.events.push(UiEvent { entity: current, kind: UiEventKind::Click });
        }
    }

    ui_events.hovered = hit;
}

// ---------------------------------------------------------------------------
//  Widget Constructors — convenience builders for common UI elements
// ---------------------------------------------------------------------------

/// 快速构建常用 UI 控件的工厂方法。
pub struct Widget;

impl Widget {
    /// 创建一个按钮节点。
    pub fn button(label: &str) -> UiNode {
        UiNode {
            background_color: [0.25, 0.25, 0.30, 1.0],
            border_radius: 6.0,
            border_width: 1.0,
            border_color: [0.4, 0.4, 0.5, 1.0],
            text: Some(UiText::new(label).with_font_size(16.0)),
            style: UiStyle {
                padding: [8.0, 16.0, 8.0, 16.0],
                justify_content: Align::Center,
                align_items: Align::Center,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// 创建一个文本标签节点。
    pub fn label(text: &str) -> UiNode {
        UiNode {
            text: Some(UiText::new(text).with_font_size(14.0)),
            ..Default::default()
        }
    }

    /// 创建一个面板容器节点。
    pub fn panel() -> UiNode {
        UiNode {
            background_color: [0.1, 0.1, 0.12, 0.9],
            border_radius: 8.0,
            style: UiStyle {
                flex_direction: FlexDirection::Column,
                padding: [12.0; 4],
                gap: 8.0,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// 创建一个水平行布局容器。
    pub fn row() -> UiNode {
        UiNode {
            style: UiStyle {
                flex_direction: FlexDirection::Row,
                gap: 8.0,
                align_items: Align::Center,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// 创建一个垂直列布局容器。
    pub fn column() -> UiNode {
        UiNode {
            style: UiStyle {
                flex_direction: FlexDirection::Column,
                gap: 8.0,
                ..Default::default()
            },
            ..Default::default()
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

    #[test]
    fn test_ui_hit_test() {
        let e1 = Entity::from_raw(1);
        let e2 = Entity::from_raw(2);
        let nodes = vec![
            (e1, [10.0, 10.0, 100.0, 50.0]),
            (e2, [50.0, 20.0, 80.0, 40.0]),
        ];
        // Hit e2 (topmost at overlap)
        assert_eq!(ui_hit_test(glam::Vec2::new(60.0, 30.0), &nodes), Some(e2));
        // Hit e1 only
        assert_eq!(ui_hit_test(glam::Vec2::new(15.0, 15.0), &nodes), Some(e1));
        // Miss all
        assert_eq!(ui_hit_test(glam::Vec2::new(0.0, 0.0), &nodes), None);
    }

    #[test]
    fn test_widget_button() {
        let btn = Widget::button("OK");
        assert!(btn.text.is_some());
        assert_eq!(btn.text.unwrap().content, "OK");
        assert!(btn.border_radius > 0.0);
    }

    #[test]
    fn test_widget_panel() {
        let panel = Widget::panel();
        assert_eq!(panel.style.flex_direction, FlexDirection::Column);
        assert!(panel.background_color[3] > 0.0);
    }
}
