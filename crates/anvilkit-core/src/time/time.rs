//! # 核心时间资源
//! 
//! 提供游戏应用的核心时间跟踪功能，包括帧时间、总运行时间和 FPS 计算。
//! 
//! ## 核心概念
//! 
//! - **Delta Time**: 上一帧到当前帧的时间间隔，用于帧率无关的游戏逻辑
//! - **Elapsed Time**: 应用启动以来的总时间
//! - **Frame Count**: 总帧数，用于 FPS 计算和调试
//! 
//! ## 使用模式
//! 
//! `Time` 通常作为全局资源在 ECS 系统中使用，每帧调用 `update()` 方法更新时间信息。

use std::time::{Duration, Instant};

/// 核心时间资源，跟踪应用的时间信息
/// 
/// `Time` 提供了游戏开发中必需的时间信息，包括帧间隔时间（delta time）、
/// 总运行时间和帧计数。它是帧率无关游戏逻辑的基础。
/// 
/// ## 线程安全
/// 
/// `Time` 实现了 `Send` 和 `Sync`，可以安全地在多线程环境中使用。
/// 
/// ## 示例
/// 
/// ```rust
/// use anvilkit_core::time::Time;
/// use std::time::Duration;
/// 
/// let mut time = Time::new();
/// 
/// // 模拟游戏循环
/// loop {
///     time.update();
///     
///     // 使用 delta time 进行帧率无关的移动
///     let movement_speed = 100.0; // 单位/秒
///     let distance = movement_speed * time.delta_seconds();
///     
///     println!("FPS: {:.1}", time.fps());
///     
///     // 游戏逻辑...
///     
///     std::thread::sleep(Duration::from_millis(16)); // ~60 FPS
///     
///     if time.frame_count() > 100 {
///         break;
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Time {
    /// 应用启动时的时间点
    startup_time: Instant,
    /// 上一帧的时间点
    last_update: Instant,
    /// 当前帧的时间点
    current_time: Instant,
    /// 上一帧到当前帧的时间间隔
    delta_time: Duration,
    /// 应用启动以来的总时间
    elapsed_time: Duration,
    /// 总帧数
    frame_count: u64,
    /// 是否是第一次更新
    first_update: bool,
}

impl Default for Time {
    fn default() -> Self {
        Self::new()
    }
}

impl Time {
    /// 创建新的时间资源
    /// 
    /// 初始化时间资源，记录创建时的时间点作为应用启动时间。
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_core::time::Time;
    /// 
    /// let time = Time::new();
    /// assert_eq!(time.frame_count(), 0);
    /// assert_eq!(time.delta_seconds(), 0.0);
    /// ```
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            startup_time: now,
            last_update: now,
            current_time: now,
            delta_time: Duration::ZERO,
            elapsed_time: Duration::ZERO,
            frame_count: 0,
            first_update: true,
        }
    }

    /// 更新时间信息
    /// 
    /// 应该在每帧开始时调用此方法来更新时间信息。
    /// 这会更新 delta time、elapsed time 和 frame count。
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_core::time::Time;
    /// use std::time::Duration;
    ///
    /// let mut time = Time::new();
    ///
    /// // 第一次更新初始化时间
    /// time.update();
    /// assert_eq!(time.frame_count(), 1);
    ///
    /// // 模拟时间流逝
    /// std::thread::sleep(Duration::from_millis(16));
    /// time.update();
    ///
    /// assert!(time.delta_seconds() > 0.0);
    /// assert_eq!(time.frame_count(), 2);
    /// ```
    pub fn update(&mut self) {
        let now = Instant::now();
        
        if self.first_update {
            // 第一次更新时，delta time 为 0
            self.first_update = false;
            self.delta_time = Duration::ZERO;
        } else {
            self.delta_time = now.duration_since(self.current_time);
        }
        
        self.last_update = self.current_time;
        self.current_time = now;
        self.elapsed_time = now.duration_since(self.startup_time);
        self.frame_count += 1;
    }

    /// 获取上一帧到当前帧的时间间隔
    /// 
    /// Delta time 是实现帧率无关游戏逻辑的关键。
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_core::time::Time;
    /// 
    /// let mut time = Time::new();
    /// time.update();
    /// 
    /// let delta = time.delta();
    /// println!("Frame time: {:?}", delta);
    /// ```
    pub fn delta(&self) -> Duration {
        self.delta_time
    }

    /// 获取 delta time 的秒数表示（f32）
    /// 
    /// 这是最常用的 delta time 获取方法，适用于大多数游戏逻辑计算。
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_core::time::Time;
    /// 
    /// let mut time = Time::new();
    /// time.update();
    /// 
    /// let speed = 100.0; // 单位/秒
    /// let distance = speed * time.delta_seconds();
    /// ```
    pub fn delta_seconds(&self) -> f32 {
        self.delta_time.as_secs_f32()
    }

    /// 获取 delta time 的秒数表示（f64）
    /// 
    /// 提供更高精度的 delta time，适用于需要高精度计算的场景。
    pub fn delta_seconds_f64(&self) -> f64 {
        self.delta_time.as_secs_f64()
    }

    /// 获取 delta time 的毫秒数表示
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_core::time::Time;
    /// 
    /// let mut time = Time::new();
    /// time.update();
    /// 
    /// println!("Frame time: {}ms", time.delta_millis());
    /// ```
    pub fn delta_millis(&self) -> u128 {
        self.delta_time.as_millis()
    }

    /// 获取应用启动以来的总时间
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_core::time::Time;
    /// use std::time::Duration;
    /// 
    /// let mut time = Time::new();
    /// std::thread::sleep(Duration::from_millis(100));
    /// time.update();
    /// 
    /// assert!(time.elapsed().as_millis() >= 100);
    /// ```
    pub fn elapsed(&self) -> Duration {
        self.elapsed_time
    }

    /// 获取总运行时间的秒数表示（f32）
    pub fn elapsed_seconds(&self) -> f32 {
        self.elapsed_time.as_secs_f32()
    }

    /// 获取总运行时间的秒数表示（f64）
    pub fn elapsed_seconds_f64(&self) -> f64 {
        self.elapsed_time.as_secs_f64()
    }

    /// 获取总帧数
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_core::time::Time;
    /// 
    /// let mut time = Time::new();
    /// assert_eq!(time.frame_count(), 0);
    /// 
    /// time.update();
    /// assert_eq!(time.frame_count(), 1);
    /// ```
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// 获取平均帧率（基于总运行时间）
    /// 
    /// 计算从应用启动到现在的平均 FPS。
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_core::time::Time;
    /// use std::time::Duration;
    /// 
    /// let mut time = Time::new();
    /// 
    /// // 模拟多帧
    /// for _ in 0..10 {
    ///     std::thread::sleep(Duration::from_millis(16));
    ///     time.update();
    /// }
    /// 
    /// let fps = time.fps();
    /// println!("Average FPS: {:.1}", fps);
    /// ```
    pub fn fps(&self) -> f64 {
        if self.elapsed_time.is_zero() || self.frame_count == 0 {
            0.0
        } else {
            self.frame_count as f64 / self.elapsed_seconds_f64()
        }
    }

    /// 获取瞬时帧率（基于当前 delta time）
    /// 
    /// 计算基于当前帧时间的瞬时 FPS，可能会有较大波动。
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_core::time::Time;
    /// 
    /// let mut time = Time::new();
    /// time.update();
    /// 
    /// let instant_fps = time.instant_fps();
    /// println!("Instant FPS: {:.1}", instant_fps);
    /// ```
    pub fn instant_fps(&self) -> f64 {
        if self.delta_time.is_zero() {
            0.0
        } else {
            1.0 / self.delta_seconds_f64()
        }
    }

    /// 获取应用启动时间点
    /// 
    /// 返回应用启动时的 `Instant`，可用于计算绝对时间间隔。
    pub fn startup_time(&self) -> Instant {
        self.startup_time
    }

    /// 获取当前时间点
    /// 
    /// 返回最后一次调用 `update()` 时的时间点。
    pub fn current_time(&self) -> Instant {
        self.current_time
    }

    /// 检查是否是第一帧
    /// 
    /// 在某些初始化逻辑中可能需要知道是否是第一帧。
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_core::time::Time;
    /// 
    /// let mut time = Time::new();
    /// assert!(time.is_first_frame());
    /// 
    /// time.update();
    /// assert!(!time.is_first_frame());
    /// ```
    pub fn is_first_frame(&self) -> bool {
        self.frame_count == 0
    }

    /// 重置时间资源
    /// 
    /// 将时间资源重置到初始状态，就像刚创建一样。
    /// 这在场景切换或游戏重启时可能有用。
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_core::time::Time;
    /// 
    /// let mut time = Time::new();
    /// time.update();
    /// 
    /// assert_eq!(time.frame_count(), 1);
    /// 
    /// time.reset();
    /// assert_eq!(time.frame_count(), 0);
    /// ```
    pub fn reset(&mut self) {
        let now = Instant::now();
        self.startup_time = now;
        self.last_update = now;
        self.current_time = now;
        self.delta_time = Duration::ZERO;
        self.elapsed_time = Duration::ZERO;
        self.frame_count = 0;
        self.first_update = true;
    }

    /// 设置时间缩放因子
    ///
    /// 注意：这个方法返回一个新的 `ScaledTime` 包装器，而不是修改当前实例。
    ///
    /// # 参数
    ///
    /// - `scale`: 时间缩放因子，1.0 为正常速度，0.5 为半速，2.0 为双速
    ///
    /// # 示例
    ///
    /// ```rust
    /// use anvilkit_core::time::Time;
    ///
    /// let time = Time::new();
    /// let slow_time = time.with_scale(0.5); // 半速
    ///
    /// assert_eq!(slow_time.scale(), 0.5);
    /// ```
    pub fn with_scale(&self, scale: f32) -> ScaledTime {
        ScaledTime::new(self.clone(), scale)
    }
}

/// 带时间缩放的时间包装器
/// 
/// `ScaledTime` 允许对时间进行缩放，实现慢动作、快进等效果。
/// 它包装了一个 `Time` 实例，并对其时间值应用缩放因子。
/// 
/// ## 使用场景
/// 
/// - 慢动作效果（scale < 1.0）
/// - 快进效果（scale > 1.0）
/// - 暂停效果（scale = 0.0）
/// - 时间倒流效果（scale < 0.0）
#[derive(Debug, Clone)]
pub struct ScaledTime {
    /// 原始时间资源
    inner: Time,
    /// 时间缩放因子
    scale: f32,
}

impl ScaledTime {
    /// 创建新的缩放时间包装器
    /// 
    /// # 参数
    /// 
    /// - `time`: 原始时间资源
    /// - `scale`: 缩放因子
    pub fn new(time: Time, scale: f32) -> Self {
        Self {
            inner: time,
            scale,
        }
    }

    /// 获取缩放因子
    pub fn scale(&self) -> f32 {
        self.scale
    }

    /// 设置缩放因子
    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale;
    }

    /// 获取缩放后的 delta time
    pub fn delta(&self) -> Duration {
        if self.scale >= 0.0 {
            Duration::from_secs_f32(self.inner.delta_seconds() * self.scale)
        } else {
            // 负缩放因子表示时间倒流
            Duration::ZERO
        }
    }

    /// 获取缩放后的 delta time（秒）
    pub fn delta_seconds(&self) -> f32 {
        self.inner.delta_seconds() * self.scale
    }

    /// 获取原始（未缩放）的时间资源
    pub fn inner(&self) -> &Time {
        &self.inner
    }

    /// 获取原始（未缩放）的时间资源（可变引用）
    pub fn inner_mut(&mut self) -> &mut Time {
        &mut self.inner
    }

    /// 更新内部时间资源
    pub fn update(&mut self) {
        self.inner.update();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use approx::assert_relative_eq;

    #[test]
    fn test_time_creation() {
        let time = Time::new();
        assert_eq!(time.frame_count(), 0);
        assert_eq!(time.delta_seconds(), 0.0);
        assert!(time.is_first_frame());
    }

    #[test]
    fn test_time_update() {
        let mut time = Time::new();
        
        // 第一次更新
        time.update();
        assert_eq!(time.frame_count(), 1);
        assert_eq!(time.delta_seconds(), 0.0); // 第一帧 delta 为 0
        assert!(!time.is_first_frame());
        
        // 模拟时间流逝
        std::thread::sleep(Duration::from_millis(10));
        time.update();
        
        assert_eq!(time.frame_count(), 2);
        assert!(time.delta_seconds() > 0.0);
        assert!(time.elapsed_seconds() > 0.0);
    }

    #[test]
    fn test_fps_calculation() {
        let mut time = Time::new();
        
        // 模拟稳定的帧率
        for _ in 0..10 {
            std::thread::sleep(Duration::from_millis(16)); // ~60 FPS
            time.update();
        }
        
        let fps = time.fps();
        assert!(fps > 50.0 && fps < 70.0); // 应该接近 60 FPS
        
        let instant_fps = time.instant_fps();
        assert!(instant_fps > 0.0);
    }

    #[test]
    fn test_time_reset() {
        let mut time = Time::new();
        time.update();
        time.update();
        
        assert_eq!(time.frame_count(), 2);
        
        time.reset();
        assert_eq!(time.frame_count(), 0);
        assert!(time.is_first_frame());
    }

    #[test]
    fn test_scaled_time() {
        let mut time = Time::new();
        std::thread::sleep(Duration::from_millis(10));
        time.update();
        
        let original_delta = time.delta_seconds();
        let scaled_time = time.with_scale(0.5);
        
        assert_eq!(scaled_time.scale(), 0.5);
        assert_relative_eq!(scaled_time.delta_seconds(), original_delta * 0.5, epsilon = 1e-6);
    }

    #[test]
    fn test_time_precision() {
        let mut time = Time::new();

        // 先进行一次更新以初始化时间
        time.update();

        // 等待一段时间
        std::thread::sleep(Duration::from_millis(50));

        // 再次更新以计算时间差
        time.update();

        let delta_f32 = time.delta_seconds();
        let delta_f64 = time.delta_seconds_f64();
        let delta_millis = time.delta_millis();

        assert!(delta_f32 > 0.0, "delta_f32 should be positive, got: {}", delta_f32);
        assert!(delta_f64 > 0.0, "delta_f64 should be positive, got: {}", delta_f64);
        assert!(delta_millis > 0, "delta_millis should be positive, got: {}", delta_millis);

        // 验证时间值在合理范围内（应该接近50ms，但允许一些误差）
        assert!(delta_f32 >= 0.01 && delta_f32 <= 0.2, "delta_f32 out of expected range: {}", delta_f32);
        assert!(delta_f64 >= 0.01 && delta_f64 <= 0.2, "delta_f64 out of expected range: {}", delta_f64);
    }

    #[test]
    fn test_time_consistency() {
        let mut time = Time::new();
        let start_time = time.startup_time();
        
        std::thread::sleep(Duration::from_millis(50));
        time.update();
        
        // 验证时间一致性
        assert_eq!(time.startup_time(), start_time);
        assert!(time.current_time() > start_time);
        assert!(time.elapsed() > Duration::ZERO);
        
        let manual_elapsed = time.current_time().duration_since(start_time);
        let reported_elapsed = time.elapsed();
        
        // 应该非常接近
        let diff = if manual_elapsed > reported_elapsed {
            manual_elapsed - reported_elapsed
        } else {
            reported_elapsed - manual_elapsed
        };
        assert!(diff < Duration::from_millis(1));
    }
}
