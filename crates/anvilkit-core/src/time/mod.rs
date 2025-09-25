//! # 时间管理系统
//! 
//! AnvilKit 的时间管理系统提供了游戏开发中必需的时间跟踪和计时功能。
//! 
//! ## 模块组织
//! 
//! - [`time`]: 核心时间资源，跟踪帧时间和应用运行时间
//! - [`timer`]: 计时器工具，用于延时和周期性事件
//! - [`stopwatch`]: 秒表工具，用于性能测量和调试
//! - [`frame_counter`]: 帧计数器，用于 FPS 计算和性能监控
//! 
//! ## 设计原则
//! 
//! 1. **高精度**: 使用 `std::time::Instant` 提供微秒级精度
//! 2. **零成本抽象**: 编译时优化，运行时开销最小
//! 3. **易于使用**: 提供直观的 API 和常用的便利方法
//! 4. **线程安全**: 所有类型都实现了 `Send` 和 `Sync`
//! 
//! ## 使用示例
//! 
//! ```rust
//! use anvilkit_core::time::{Time, Timer};
//! use std::time::Duration;
//!
//! // 创建时间管理器
//! let mut time = Time::new();
//!
//! // 创建 1 秒计时器
//! let mut timer = Timer::from_seconds(1.0);
//!
//! // 模拟游戏循环
//! for _ in 0..5 {
//!     time.update();
//!     timer.tick(time.delta());
//!
//!     if timer.just_finished() {
//!         println!("Timer finished!");
//!         timer.reset();
//!     }
//!
//!     // 模拟一些工作
//!     std::thread::sleep(Duration::from_millis(10));
//! }
//! ```

pub mod time;
pub mod timer;

// 重新导出主要类型
pub use time::Time;
pub use timer::Timer;

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_time_module_integration() {
        let mut time = Time::new();
        let mut timer = Timer::from_seconds(0.1);

        // 模拟几帧更新
        for _ in 0..5 {
            std::thread::sleep(Duration::from_millis(20));
            time.update();
            timer.tick(time.delta());
        }

        // 验证时间系统正常工作
        assert!(time.elapsed_seconds() > 0.0);
        assert!(timer.elapsed_seconds() > 0.0);
    }
}
