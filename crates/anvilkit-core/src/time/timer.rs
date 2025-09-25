//! # 计时器工具
//! 
//! 提供灵活的计时器功能，用于实现延时、周期性事件和时间相关的游戏逻辑。
//! 
//! ## 核心概念
//! 
//! - **一次性计时器**: 计时结束后停止，适用于延时操作
//! - **重复计时器**: 计时结束后自动重置，适用于周期性事件
//! - **暂停/恢复**: 支持计时器的暂停和恢复操作
//! 
//! ## 使用场景
//! 
//! - 武器冷却时间
//! - 技能释放间隔
//! - UI 动画延时
//! - 周期性事件触发
//! - 游戏状态切换延时

use std::time::Duration;

/// 计时器状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerState {
    /// 运行中
    Running,
    /// 已暂停
    Paused,
    /// 已完成（仅对一次性计时器有意义）
    Finished,
}

/// 灵活的计时器工具
/// 
/// `Timer` 提供了丰富的计时功能，支持一次性和重复计时、暂停恢复等操作。
/// 它是实现时间相关游戏逻辑的核心工具。
/// 
/// ## 设计特点
/// 
/// - **零分配**: 所有操作都不涉及内存分配
/// - **高精度**: 使用 `Duration` 提供微秒级精度
/// - **状态管理**: 清晰的状态转换和查询接口
/// - **灵活配置**: 支持多种创建和配置方式
/// 
/// ## 示例
/// 
/// ```rust
/// use anvilkit_core::time::Timer;
/// use std::time::Duration;
/// 
/// // 创建 3 秒一次性计时器
/// let mut timer = Timer::from_seconds(3.0);
/// 
/// // 创建 1 秒重复计时器
/// let mut repeat_timer = Timer::repeating_from_seconds(1.0);
/// 
/// // 在游戏循环中更新
/// let delta = Duration::from_millis(16); // ~60 FPS
/// 
/// timer.tick(delta);
/// repeat_timer.tick(delta);
/// 
/// if timer.just_finished() {
///     println!("Timer finished!");
/// }
/// 
/// if repeat_timer.just_finished() {
///     println!("Repeat timer triggered!");
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Timer {
    /// 计时器总时长
    duration: Duration,
    /// 已经过的时间
    elapsed: Duration,
    /// 是否为重复计时器
    repeating: bool,
    /// 计时器状态
    state: TimerState,
    /// 本帧是否刚完成（用于 just_finished 检测）
    just_finished: bool,
}

impl Timer {
    /// 创建新的一次性计时器
    /// 
    /// # 参数
    /// 
    /// - `duration`: 计时时长
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_core::time::Timer;
    /// use std::time::Duration;
    /// 
    /// let timer = Timer::new(Duration::from_secs(5));
    /// assert_eq!(timer.duration(), Duration::from_secs(5));
    /// assert!(!timer.is_repeating());
    /// ```
    pub fn new(duration: Duration) -> Self {
        Self {
            duration,
            elapsed: Duration::ZERO,
            repeating: false,
            state: TimerState::Running,
            just_finished: false,
        }
    }

    /// 创建新的重复计时器
    /// 
    /// # 参数
    /// 
    /// - `duration`: 每次计时的时长
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_core::time::Timer;
    /// use std::time::Duration;
    /// 
    /// let timer = Timer::repeating(Duration::from_secs(2));
    /// assert!(timer.is_repeating());
    /// ```
    pub fn repeating(duration: Duration) -> Self {
        Self {
            duration,
            elapsed: Duration::ZERO,
            repeating: true,
            state: TimerState::Running,
            just_finished: false,
        }
    }

    /// 从秒数创建一次性计时器
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_core::time::Timer;
    /// 
    /// let timer = Timer::from_seconds(3.5);
    /// assert_eq!(timer.duration_seconds(), 3.5);
    /// ```
    pub fn from_seconds(seconds: f32) -> Self {
        Self::new(Duration::from_secs_f32(seconds))
    }

    /// 从秒数创建重复计时器
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_core::time::Timer;
    /// 
    /// let timer = Timer::repeating_from_seconds(1.0);
    /// assert!(timer.is_repeating());
    /// assert_eq!(timer.duration_seconds(), 1.0);
    /// ```
    pub fn repeating_from_seconds(seconds: f32) -> Self {
        Self::repeating(Duration::from_secs_f32(seconds))
    }

    /// 从毫秒数创建一次性计时器
    pub fn from_millis(millis: u64) -> Self {
        Self::new(Duration::from_millis(millis))
    }

    /// 从毫秒数创建重复计时器
    pub fn repeating_from_millis(millis: u64) -> Self {
        Self::repeating(Duration::from_millis(millis))
    }

    /// 更新计时器
    /// 
    /// 应该在每帧调用此方法来推进计时器。
    /// 
    /// # 参数
    /// 
    /// - `delta`: 自上次更新以来的时间间隔
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_core::time::Timer;
    /// use std::time::Duration;
    /// 
    /// let mut timer = Timer::from_seconds(1.0);
    /// 
    /// // 模拟 0.5 秒
    /// timer.tick(Duration::from_millis(500));
    /// assert_eq!(timer.percent(), 0.5);
    /// assert!(!timer.finished());
    /// 
    /// // 再模拟 0.5 秒
    /// timer.tick(Duration::from_millis(500));
    /// assert!(timer.finished());
    /// assert!(timer.just_finished());
    /// ```
    pub fn tick(&mut self, delta: Duration) {
        self.just_finished = false;

        if self.state != TimerState::Running {
            return;
        }

        let _old_elapsed = self.elapsed;
        self.elapsed += delta;

        // 检查是否完成
        if self.elapsed >= self.duration {
            self.just_finished = true;

            if self.repeating {
                // 重复计时器：重置并保留超出的时间
                let overflow = self.elapsed - self.duration;
                self.elapsed = overflow;
                
                // 如果超出时间仍然大于等于持续时间，继续处理
                // 这处理了 delta 时间非常大的情况
                while self.elapsed >= self.duration {
                    self.elapsed -= self.duration;
                }
            } else {
                // 一次性计时器：标记为完成
                self.elapsed = self.duration;
                self.state = TimerState::Finished;
            }
        }
    }

    /// 检查计时器是否已完成
    /// 
    /// 对于一次性计时器，完成后会一直返回 `true`。
    /// 对于重复计时器，只有在刚完成的那一帧返回 `true`。
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_core::time::Timer;
    /// use std::time::Duration;
    /// 
    /// let mut timer = Timer::from_seconds(1.0);
    /// assert!(!timer.finished());
    /// 
    /// timer.tick(Duration::from_secs(1));
    /// assert!(timer.finished());
    /// ```
    pub fn finished(&self) -> bool {
        match self.state {
            TimerState::Finished => true,
            _ => self.repeating && self.just_finished,
        }
    }

    /// 检查计时器是否在本帧刚完成
    /// 
    /// 这对于触发一次性事件非常有用，避免重复触发。
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_core::time::Timer;
    /// use std::time::Duration;
    /// 
    /// let mut timer = Timer::from_seconds(1.0);
    /// 
    /// timer.tick(Duration::from_millis(999));
    /// assert!(!timer.just_finished());
    /// 
    /// timer.tick(Duration::from_millis(1));
    /// assert!(timer.just_finished());
    /// 
    /// timer.tick(Duration::from_millis(1));
    /// assert!(!timer.just_finished()); // 下一帧不再是 "刚完成"
    /// ```
    pub fn just_finished(&self) -> bool {
        self.just_finished
    }

    /// 获取已经过的时间
    pub fn elapsed(&self) -> Duration {
        self.elapsed
    }

    /// 获取已经过的时间（秒）
    pub fn elapsed_seconds(&self) -> f32 {
        self.elapsed.as_secs_f32()
    }

    /// 获取计时器总时长
    pub fn duration(&self) -> Duration {
        self.duration
    }

    /// 获取计时器总时长（秒）
    pub fn duration_seconds(&self) -> f32 {
        self.duration.as_secs_f32()
    }

    /// 获取完成百分比 (0.0 到 1.0)
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_core::time::Timer;
    /// use std::time::Duration;
    /// 
    /// let mut timer = Timer::from_seconds(2.0);
    /// timer.tick(Duration::from_secs(1));
    /// 
    /// assert_eq!(timer.percent(), 0.5);
    /// ```
    pub fn percent(&self) -> f32 {
        if self.duration.is_zero() {
            1.0
        } else {
            (self.elapsed.as_secs_f32() / self.duration.as_secs_f32()).min(1.0)
        }
    }

    /// 获取剩余时间
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_core::time::Timer;
    /// use std::time::Duration;
    /// 
    /// let mut timer = Timer::from_seconds(3.0);
    /// timer.tick(Duration::from_secs(1));
    /// 
    /// assert_eq!(timer.remaining(), Duration::from_secs(2));
    /// ```
    pub fn remaining(&self) -> Duration {
        self.duration.saturating_sub(self.elapsed)
    }

    /// 获取剩余时间（秒）
    pub fn remaining_seconds(&self) -> f32 {
        self.remaining().as_secs_f32()
    }

    /// 重置计时器到初始状态
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_core::time::Timer;
    /// use std::time::Duration;
    /// 
    /// let mut timer = Timer::from_seconds(1.0);
    /// timer.tick(Duration::from_millis(500));
    /// 
    /// assert_eq!(timer.percent(), 0.5);
    /// 
    /// timer.reset();
    /// assert_eq!(timer.percent(), 0.0);
    /// assert!(!timer.finished());
    /// ```
    pub fn reset(&mut self) {
        self.elapsed = Duration::ZERO;
        self.state = TimerState::Running;
        self.just_finished = false;
    }

    /// 暂停计时器
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_core::time::Timer;
    /// use std::time::Duration;
    /// 
    /// let mut timer = Timer::from_seconds(1.0);
    /// timer.pause();
    /// 
    /// assert!(timer.is_paused());
    /// 
    /// // 暂停状态下 tick 不会推进时间
    /// timer.tick(Duration::from_secs(1));
    /// assert_eq!(timer.elapsed_seconds(), 0.0);
    /// ```
    pub fn pause(&mut self) {
        if self.state == TimerState::Running {
            self.state = TimerState::Paused;
        }
    }

    /// 恢复计时器
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_core::time::Timer;
    /// use std::time::Duration;
    /// 
    /// let mut timer = Timer::from_seconds(1.0);
    /// timer.pause();
    /// timer.resume();
    /// 
    /// assert!(timer.is_running());
    /// ```
    pub fn resume(&mut self) {
        if self.state == TimerState::Paused {
            self.state = TimerState::Running;
        }
    }

    /// 设置计时器时长
    /// 
    /// # 注意
    /// 
    /// 如果新时长小于已经过的时间，计时器会立即完成。
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_core::time::Timer;
    /// use std::time::Duration;
    /// 
    /// let mut timer = Timer::from_seconds(1.0);
    /// timer.set_duration(Duration::from_secs(2));
    /// 
    /// assert_eq!(timer.duration_seconds(), 2.0);
    /// ```
    pub fn set_duration(&mut self, duration: Duration) {
        self.duration = duration;
        
        // 如果新时长小于已经过的时间，立即完成
        if self.elapsed >= self.duration {
            if self.repeating {
                self.elapsed = Duration::ZERO;
                self.just_finished = true;
            } else {
                self.elapsed = self.duration;
                self.state = TimerState::Finished;
                self.just_finished = true;
            }
        }
    }

    /// 设置是否为重复计时器
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_core::time::Timer;
    /// 
    /// let mut timer = Timer::from_seconds(1.0);
    /// assert!(!timer.is_repeating());
    /// 
    /// timer.set_repeating(true);
    /// assert!(timer.is_repeating());
    /// ```
    pub fn set_repeating(&mut self, repeating: bool) {
        self.repeating = repeating;

        if repeating {
            // 如果设置为重复模式且当前已完成，则重置定时器
            if self.state == TimerState::Finished {
                self.elapsed = Duration::ZERO;
                self.state = TimerState::Running;
            }
        } else {
            // 如果从重复改为非重复，且已完成，则标记为完成状态
            if self.elapsed >= self.duration {
                self.state = TimerState::Finished;
            }
        }
    }

    /// 检查是否为重复计时器
    pub fn is_repeating(&self) -> bool {
        self.repeating
    }

    /// 检查计时器是否正在运行
    pub fn is_running(&self) -> bool {
        self.state == TimerState::Running
    }

    /// 检查计时器是否已暂停
    pub fn is_paused(&self) -> bool {
        self.state == TimerState::Paused
    }

    /// 获取计时器状态
    pub fn state(&self) -> TimerState {
        self.state
    }

    /// 强制完成计时器
    /// 
    /// 将已经过时间设置为总时长，并触发完成状态。
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_core::time::Timer;
    /// 
    /// let mut timer = Timer::from_seconds(10.0);
    /// timer.finish();
    /// 
    /// assert!(timer.finished());
    /// assert!(timer.just_finished());
    /// ```
    pub fn finish(&mut self) {
        self.elapsed = self.duration;
        self.just_finished = true;
        
        if !self.repeating {
            self.state = TimerState::Finished;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use approx::assert_relative_eq;

    #[test]
    fn test_timer_creation() {
        let timer = Timer::from_seconds(2.0);
        assert_eq!(timer.duration_seconds(), 2.0);
        assert!(!timer.is_repeating());
        assert!(timer.is_running());
        assert!(!timer.finished());
    }

    #[test]
    fn test_repeating_timer_creation() {
        let timer = Timer::repeating_from_seconds(1.5);
        assert_eq!(timer.duration_seconds(), 1.5);
        assert!(timer.is_repeating());
        assert!(timer.is_running());
    }

    #[test]
    fn test_timer_tick() {
        let mut timer = Timer::from_seconds(1.0);
        
        // 半程
        timer.tick(Duration::from_millis(500));
        assert_eq!(timer.percent(), 0.5);
        assert!(!timer.finished());
        assert!(!timer.just_finished());
        
        // 完成
        timer.tick(Duration::from_millis(500));
        assert!(timer.finished());
        assert!(timer.just_finished());
        assert_eq!(timer.percent(), 1.0);
        
        // 下一帧
        timer.tick(Duration::from_millis(1));
        assert!(timer.finished());
        assert!(!timer.just_finished()); // 不再是 "刚完成"
    }

    #[test]
    fn test_repeating_timer() {
        let mut timer = Timer::repeating_from_seconds(1.0);
        
        // 第一次完成
        timer.tick(Duration::from_secs(1));
        assert!(timer.finished());
        assert!(timer.just_finished());
        
        // 下一帧，应该重置
        timer.tick(Duration::from_millis(1));
        assert!(!timer.finished());
        assert!(!timer.just_finished());
        assert!(timer.elapsed_seconds() < 0.1);
    }

    #[test]
    fn test_timer_overflow() {
        let mut timer = Timer::repeating_from_seconds(1.0);
        
        // 一次性跳过多个周期
        timer.tick(Duration::from_millis(2500)); // 2.5 秒
        
        assert!(timer.just_finished());
        assert_relative_eq!(timer.elapsed_seconds(), 0.5, epsilon = 1e-3);
    }

    #[test]
    fn test_timer_pause_resume() {
        let mut timer = Timer::from_seconds(1.0);
        
        timer.tick(Duration::from_millis(300));
        assert_eq!(timer.percent(), 0.3);
        
        timer.pause();
        assert!(timer.is_paused());
        
        // 暂停状态下不应该推进
        timer.tick(Duration::from_millis(500));
        assert_eq!(timer.percent(), 0.3);
        
        timer.resume();
        assert!(timer.is_running());
        
        timer.tick(Duration::from_millis(700));
        assert!(timer.finished());
    }

    #[test]
    fn test_timer_reset() {
        let mut timer = Timer::from_seconds(1.0);
        timer.tick(Duration::from_millis(800));
        
        assert_eq!(timer.percent(), 0.8);
        
        timer.reset();
        assert_eq!(timer.percent(), 0.0);
        assert!(!timer.finished());
        assert!(timer.is_running());
    }

    #[test]
    fn test_timer_set_duration() {
        let mut timer = Timer::from_seconds(2.0);
        timer.tick(Duration::from_secs(1));
        
        // 延长时间
        timer.set_duration(Duration::from_secs(3));
        assert_eq!(timer.percent(), 1.0 / 3.0);
        
        // 缩短时间到已经过的时间以下
        timer.set_duration(Duration::from_millis(500));
        assert!(timer.finished());
        assert!(timer.just_finished());
    }

    #[test]
    fn test_timer_remaining() {
        let mut timer = Timer::from_seconds(5.0);
        timer.tick(Duration::from_secs(2));
        
        assert_eq!(timer.remaining(), Duration::from_secs(3));
        assert_eq!(timer.remaining_seconds(), 3.0);
    }

    #[test]
    fn test_timer_finish() {
        let mut timer = Timer::from_seconds(10.0);
        timer.finish();
        
        assert!(timer.finished());
        assert!(timer.just_finished());
        assert_eq!(timer.percent(), 1.0);
    }

    #[test]
    fn test_timer_state_transitions() {
        let mut timer = Timer::from_seconds(1.0);
        
        assert_eq!(timer.state(), TimerState::Running);
        
        timer.pause();
        assert_eq!(timer.state(), TimerState::Paused);
        
        timer.resume();
        assert_eq!(timer.state(), TimerState::Running);
        
        timer.tick(Duration::from_secs(1));
        assert_eq!(timer.state(), TimerState::Finished);
    }

    #[test]
    fn test_zero_duration_timer() {
        let mut timer = Timer::new(Duration::ZERO);
        assert_eq!(timer.percent(), 1.0);
        
        timer.tick(Duration::from_millis(1));
        assert!(timer.finished());
        assert!(timer.just_finished());
    }

    #[test]
    fn test_timer_set_repeating() {
        let mut timer = Timer::from_seconds(1.0);
        timer.tick(Duration::from_secs(1));
        
        assert!(timer.finished());
        
        // 改为重复计时器
        timer.set_repeating(true);
        timer.tick(Duration::from_millis(1));
        
        // 应该重置并继续运行
        assert!(!timer.finished());
        assert!(timer.elapsed_seconds() < 0.1);
    }
}
