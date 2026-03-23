//! # Frame Performance Profiler
//!
//! Provides CPU-side frame timing and per-section profiling for the game loop.
//!
//! Feature-gated behind the `"debug"` feature. When not enabled, all methods are
//! no-ops with zero overhead.
//!
//! ## Usage
//!
//! ```rust
//! use anvilkit_ecs::profiler::FrameProfiler;
//!
//! let mut profiler = FrameProfiler::new();
//! profiler.begin_frame();
//! // ... game logic ...
//! profiler.begin_section("physics");
//! // ... physics ...
//! profiler.end_section("physics");
//! profiler.end_frame();
//!
//! println!("FPS: {:.1}", profiler.fps());
//! println!("Avg frame: {:.2} ms", profiler.avg_frame_time_ms());
//! ```

#[cfg(feature = "debug")]
mod inner {
    use std::collections::{HashMap, VecDeque};
    use std::time::Instant;
    use bevy_ecs::prelude::Resource;

    /// Default number of frames to keep in history.
    const DEFAULT_MAX_HISTORY: usize = 300;

    /// Frame performance profiler resource.
    ///
    /// Tracks per-frame CPU timings, optional GPU timings, and named section
    /// timings. Keeps a rolling history window (default 300 frames).
    #[derive(Resource)]
    pub struct FrameProfiler {
        /// Rolling history of frame times in seconds.
        pub frame_times: VecDeque<f64>,
        /// Rolling history of GPU times in seconds (filled externally).
        pub gpu_times: VecDeque<f64>,
        /// Per-named-section timing histories (seconds).
        pub section_times: HashMap<String, VecDeque<f64>>,
        /// Total number of frames recorded.
        pub frame_count: u64,
        /// Maximum number of frames to keep in history.
        pub max_history: usize,

        /// Start instant of the current frame, if `begin_frame` was called.
        pub(crate) current_frame_start: Option<Instant>,
        /// Start instants of currently-open sections.
        pub(crate) section_starts: HashMap<String, Instant>,
    }

    impl FrameProfiler {
        /// Create a new profiler with default settings.
        pub fn new() -> Self {
            Self {
                frame_times: VecDeque::with_capacity(DEFAULT_MAX_HISTORY),
                gpu_times: VecDeque::with_capacity(DEFAULT_MAX_HISTORY),
                section_times: HashMap::new(),
                frame_count: 0,
                max_history: DEFAULT_MAX_HISTORY,
                current_frame_start: None,
                section_starts: HashMap::new(),
            }
        }

        /// Record the start of a new frame.
        pub fn begin_frame(&mut self) {
            self.current_frame_start = Some(Instant::now());
        }

        /// Record the end of the current frame. Computes frame time and pushes
        /// it to the history ring buffer.
        pub fn end_frame(&mut self) {
            if let Some(start) = self.current_frame_start.take() {
                let elapsed = start.elapsed().as_secs_f64();
                if self.frame_times.len() >= self.max_history {
                    self.frame_times.pop_front();
                }
                self.frame_times.push_back(elapsed);
                self.frame_count += 1;
            }
        }

        /// Begin a named timing section within the current frame.
        pub fn begin_section(&mut self, name: &str) {
            self.section_starts.insert(name.to_string(), Instant::now());
        }

        /// End a named timing section, computing elapsed time and storing it.
        pub fn end_section(&mut self, name: &str) {
            if let Some(start) = self.section_starts.remove(name) {
                let elapsed = start.elapsed().as_secs_f64();
                let history = self
                    .section_times
                    .entry(name.to_string())
                    .or_insert_with(|| VecDeque::with_capacity(self.max_history));
                if history.len() >= self.max_history {
                    history.pop_front();
                }
                history.push_back(elapsed);
            }
        }

        /// Average frames per second over the history window.
        pub fn fps(&self) -> f64 {
            if self.frame_times.is_empty() {
                return 0.0;
            }
            let avg = self.frame_times.iter().copied().sum::<f64>() / self.frame_times.len() as f64;
            if avg > 0.0 {
                1.0 / avg
            } else {
                0.0
            }
        }

        /// Average frame time in milliseconds over the history window.
        pub fn avg_frame_time_ms(&self) -> f64 {
            if self.frame_times.is_empty() {
                return 0.0;
            }
            let avg = self.frame_times.iter().copied().sum::<f64>() / self.frame_times.len() as f64;
            avg * 1000.0
        }

        /// Average time in milliseconds for a named section, or `None` if no
        /// data has been recorded for that section.
        pub fn avg_section_time_ms(&self, name: &str) -> Option<f64> {
            self.section_times.get(name).map(|times| {
                if times.is_empty() {
                    0.0
                } else {
                    let avg = times.iter().copied().sum::<f64>() / times.len() as f64;
                    avg * 1000.0
                }
            })
        }

        /// Minimum frame time in milliseconds over the history window.
        pub fn min_frame_time_ms(&self) -> f64 {
            self.frame_times
                .iter()
                .copied()
                .fold(f64::INFINITY, f64::min)
                * 1000.0
        }

        /// Maximum frame time in milliseconds over the history window.
        pub fn max_frame_time_ms(&self) -> f64 {
            self.frame_times
                .iter()
                .copied()
                .fold(0.0_f64, f64::max)
                * 1000.0
        }

        /// Percentile frame time in milliseconds. For example, `p = 0.99`
        /// returns the 99th percentile frame time.
        pub fn percentile_frame_time_ms(&self, p: f64) -> f64 {
            if self.frame_times.is_empty() {
                return 0.0;
            }
            let mut sorted: Vec<f64> = self.frame_times.iter().copied().collect();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

            let index = ((sorted.len() as f64 * p).ceil() as usize).saturating_sub(1);
            let index = index.min(sorted.len() - 1);
            sorted[index] * 1000.0
        }
    }

    impl Default for FrameProfiler {
        fn default() -> Self {
            Self::new()
        }
    }
}

#[cfg(not(feature = "debug"))]
mod inner {
    use bevy_ecs::prelude::Resource;

    /// No-op frame profiler stub (debug feature disabled).
    #[derive(Resource, Default)]
    pub struct FrameProfiler;

    impl FrameProfiler {
        /// Create a new (no-op) profiler.
        #[inline(always)]
        pub fn new() -> Self {
            Self
        }
        /// No-op.
        #[inline(always)]
        pub fn begin_frame(&mut self) {}
        /// No-op.
        #[inline(always)]
        pub fn end_frame(&mut self) {}
        /// No-op.
        #[inline(always)]
        pub fn begin_section(&mut self, _name: &str) {}
        /// No-op.
        #[inline(always)]
        pub fn end_section(&mut self, _name: &str) {}
        /// Always returns 0.
        #[inline(always)]
        pub fn fps(&self) -> f64 {
            0.0
        }
        /// Always returns 0.
        #[inline(always)]
        pub fn avg_frame_time_ms(&self) -> f64 {
            0.0
        }
        /// Always returns `None`.
        #[inline(always)]
        pub fn avg_section_time_ms(&self, _name: &str) -> Option<f64> {
            None
        }
        /// Always returns 0.
        #[inline(always)]
        pub fn min_frame_time_ms(&self) -> f64 {
            0.0
        }
        /// Always returns 0.
        #[inline(always)]
        pub fn max_frame_time_ms(&self) -> f64 {
            0.0
        }
        /// Always returns 0.
        #[inline(always)]
        pub fn percentile_frame_time_ms(&self, _p: f64) -> f64 {
            0.0
        }
    }
}

pub use inner::FrameProfiler;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profiler_fps() {
        let mut profiler = FrameProfiler::new();

        // Record several frames
        for _ in 0..10 {
            profiler.begin_frame();
            // Simulate a tiny bit of work
            std::hint::black_box(0u64);
            profiler.end_frame();
        }

        #[cfg(feature = "debug")]
        {
            assert!(profiler.fps() > 0.0, "FPS should be positive");
            assert_eq!(profiler.frame_count, 10);
            assert_eq!(profiler.frame_times.len(), 10);
            assert!(profiler.avg_frame_time_ms() >= 0.0);
            assert!(profiler.min_frame_time_ms() >= 0.0);
            assert!(profiler.max_frame_time_ms() >= profiler.min_frame_time_ms());
            assert!(profiler.percentile_frame_time_ms(0.99) >= 0.0);
        }

        #[cfg(not(feature = "debug"))]
        {
            assert_eq!(profiler.fps(), 0.0);
        }
    }

    #[test]
    fn test_profiler_sections() {
        let mut profiler = FrameProfiler::new();

        for _ in 0..5 {
            profiler.begin_frame();
            profiler.begin_section("physics");
            std::hint::black_box(0u64);
            profiler.end_section("physics");
            profiler.end_frame();
        }

        #[cfg(feature = "debug")]
        {
            let avg = profiler.avg_section_time_ms("physics");
            assert!(avg.is_some(), "Section 'physics' should have data");
            assert!(avg.unwrap() >= 0.0, "Section avg should be non-negative");

            assert!(
                profiler.avg_section_time_ms("nonexistent").is_none(),
                "Unknown section should return None"
            );
        }

        #[cfg(not(feature = "debug"))]
        {
            assert!(profiler.avg_section_time_ms("physics").is_none());
        }
    }
}
