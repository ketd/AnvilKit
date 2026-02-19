//! # 调试和性能分析工具
//!
//! 提供渲染调试可视化和帧统计信息。
//!
//! ## 核心类型
//!
//! - [`DebugMode`]: 调试渲染模式（线框、法线、光照等）
//! - [`RenderStats`]: 每帧渲染统计
//! - [`DebugOverlay`]: 调试信息叠加层

use bevy_ecs::prelude::*;

/// 调试渲染模式
///
/// # 示例
///
/// ```rust
/// use anvilkit_render::renderer::debug::DebugMode;
///
/// let mode = DebugMode::Wireframe;
/// assert!(!mode.is_normal());
/// assert!(DebugMode::None.is_normal());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugMode {
    /// 正常渲染
    None,
    /// 线框叠加
    Wireframe,
    /// 法线可视化
    Normals,
    /// 仅漫反射
    DiffuseOnly,
    /// 仅镜面反射
    SpecularOnly,
    /// 金属度可视化
    Metallic,
    /// 粗糙度可视化
    Roughness,
    /// AO 可视化
    AmbientOcclusion,
    /// UV 坐标可视化
    UVs,
    /// 深度缓冲可视化
    Depth,
}

impl DebugMode {
    /// 是否为正常渲染模式
    pub fn is_normal(&self) -> bool {
        matches!(self, DebugMode::None)
    }
}

impl Default for DebugMode {
    fn default() -> Self {
        DebugMode::None
    }
}

/// 每帧渲染统计
///
/// # 示例
///
/// ```rust
/// use anvilkit_render::renderer::debug::RenderStats;
///
/// let mut stats = RenderStats::new();
/// stats.record_draw_call(100);
/// stats.record_draw_call(200);
/// assert_eq!(stats.draw_calls, 2);
/// assert_eq!(stats.triangles, 300);
/// ```
#[derive(Debug, Clone, Resource)]
pub struct RenderStats {
    /// 绘制调用次数
    pub draw_calls: u32,
    /// 渲染的三角形总数
    pub triangles: u32,
    /// 渲染的顶点总数
    pub vertices: u32,
    /// 活跃的光源数
    pub active_lights: u32,
    /// 视锥体剔除掉的物体数
    pub culled_objects: u32,
    /// 可见物体数
    pub visible_objects: u32,
    /// 帧时间（毫秒）
    pub frame_time_ms: f32,
    /// FPS（基于帧时间计算）
    pub fps: f32,
    /// GPU 内存使用估计（字节）
    pub gpu_memory_bytes: u64,
}

impl RenderStats {
    pub fn new() -> Self {
        Self {
            draw_calls: 0,
            triangles: 0,
            vertices: 0,
            active_lights: 0,
            culled_objects: 0,
            visible_objects: 0,
            frame_time_ms: 0.0,
            fps: 0.0,
            gpu_memory_bytes: 0,
        }
    }

    /// 记录一次绘制调用
    pub fn record_draw_call(&mut self, triangle_count: u32) {
        self.draw_calls += 1;
        self.triangles += triangle_count;
    }

    /// 更新帧时间
    pub fn update_frame_time(&mut self, dt_seconds: f32) {
        self.frame_time_ms = dt_seconds * 1000.0;
        self.fps = if dt_seconds > 0.0 { 1.0 / dt_seconds } else { 0.0 };
    }

    /// 帧开始时重置计数器
    pub fn reset_frame(&mut self) {
        self.draw_calls = 0;
        self.triangles = 0;
        self.vertices = 0;
        self.culled_objects = 0;
        self.visible_objects = 0;
    }

    /// 格式化为摘要字符串
    pub fn summary(&self) -> String {
        format!(
            "FPS: {:.0} | {:.1}ms | Draw: {} | Tri: {} | Vis: {}/{}",
            self.fps, self.frame_time_ms,
            self.draw_calls, self.triangles,
            self.visible_objects, self.visible_objects + self.culled_objects,
        )
    }
}

impl Default for RenderStats {
    fn default() -> Self {
        Self::new()
    }
}

/// 调试叠加层配置
///
/// # 示例
///
/// ```rust
/// use anvilkit_render::renderer::debug::DebugOverlay;
///
/// let overlay = DebugOverlay::default();
/// assert!(!overlay.show_stats);
/// assert!(!overlay.show_wireframe);
/// ```
#[derive(Debug, Clone, Resource)]
pub struct DebugOverlay {
    /// 是否显示统计信息
    pub show_stats: bool,
    /// 是否显示线框
    pub show_wireframe: bool,
    /// 是否显示包围盒
    pub show_bounds: bool,
    /// 是否显示灯光图标
    pub show_lights: bool,
    /// 是否显示骨骼
    pub show_skeleton: bool,
    /// 当前调试模式
    pub debug_mode: DebugMode,
}

impl Default for DebugOverlay {
    fn default() -> Self {
        Self {
            show_stats: false,
            show_wireframe: false,
            show_bounds: false,
            show_lights: false,
            show_skeleton: false,
            debug_mode: DebugMode::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_mode() {
        assert!(DebugMode::None.is_normal());
        assert!(!DebugMode::Wireframe.is_normal());
        assert!(!DebugMode::Normals.is_normal());
    }

    #[test]
    fn test_render_stats() {
        let mut stats = RenderStats::new();
        stats.record_draw_call(100);
        stats.record_draw_call(50);
        assert_eq!(stats.draw_calls, 2);
        assert_eq!(stats.triangles, 150);

        stats.update_frame_time(1.0 / 60.0);
        assert!((stats.fps - 60.0).abs() < 1.0);

        let summary = stats.summary();
        assert!(summary.contains("FPS:"));
        assert!(summary.contains("Draw: 2"));
    }

    #[test]
    fn test_render_stats_reset() {
        let mut stats = RenderStats::new();
        stats.record_draw_call(100);
        stats.visible_objects = 5;
        stats.culled_objects = 3;

        stats.reset_frame();
        assert_eq!(stats.draw_calls, 0);
        assert_eq!(stats.triangles, 0);
        // fps and frame_time are NOT reset (they're per-frame measurements)
        assert_eq!(stats.visible_objects, 0);
    }

    #[test]
    fn test_debug_overlay_default() {
        let overlay = DebugOverlay::default();
        assert!(!overlay.show_stats);
        assert!(overlay.debug_mode.is_normal());
    }
}
