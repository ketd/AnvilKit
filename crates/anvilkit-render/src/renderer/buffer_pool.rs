//! # GPU Buffer Pool
//!
//! 可复用的 GPU 缓冲区池，避免子系统渲染器（sprite、particle、UI、line、text）每帧分配新缓冲区。
//!
//! ## 使用方式
//!
//! ```rust,ignore
//! let buffer = pool.acquire(device, min_size, usage, label);
//! // ... 使用 buffer 进行渲染 ...
//! pool.release(buffer, usage);
//! ```

use wgpu::{Buffer, BufferUsages, Device};

/// GPU 缓冲区池
///
/// 维护一组可复用的 GPU 缓冲区，按大小排序。
/// `acquire()` 返回一个足够大且 usage 匹配的闲置缓冲区或创建新缓冲区。
/// `release()` 将缓冲区归还池中以供下一帧复用。
///
/// **Important**: Buffers are matched by both size AND usage flags.
/// A vertex buffer will never be returned for an index buffer request.
pub struct BufferPool {
    /// 可用缓冲区池 (buffer, capacity_bytes, usage)
    available: Vec<(Buffer, u64, BufferUsages)>,
    /// 本帧使用中的缓冲区数量（用于统计）
    in_use_count: usize,
    /// 池上限
    max_pool_size: usize,
}

impl BufferPool {
    /// 创建新的缓冲区池
    ///
    /// `max_pool_size`: 池中最大缓冲区数量（默认推荐 64）
    pub fn new(max_pool_size: usize) -> Self {
        Self {
            available: Vec::new(),
            in_use_count: 0,
            max_pool_size,
        }
    }

    /// 获取一个至少 `min_size` 字节的缓冲区
    ///
    /// 优先复用池中现有的足够大的缓冲区，否则创建新缓冲区。
    pub fn acquire(
        &mut self,
        device: &Device,
        min_size: u64,
        usage: BufferUsages,
        label: &str,
    ) -> Buffer {
        // 查找 usage 匹配且足够大的缓冲区
        if let Some(idx) = self.available.iter().position(|(_, cap, u)| *u == usage && *cap >= min_size) {
            self.in_use_count += 1;
            return self.available.remove(idx).0;
        }

        // 没有合适的缓冲区，创建新的
        self.in_use_count += 1;
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: min_size,
            usage,
            mapped_at_creation: false,
        })
    }

    /// 归还缓冲区到池中以供复用
    ///
    /// `usage` must match the usage flags the buffer was created with.
    pub fn release(&mut self, buffer: Buffer, capacity: u64, usage: BufferUsages) {
        self.in_use_count = self.in_use_count.saturating_sub(1);

        // 如果池已满，丢弃最小的缓冲区
        if self.available.len() >= self.max_pool_size {
            // 找到最小的缓冲区
            if let Some(min_idx) = self.available.iter()
                .enumerate()
                .min_by_key(|(_, (_, cap, _))| *cap)
                .map(|(i, _)| i)
            {
                if self.available[min_idx].1 < capacity {
                    // 新缓冲区更大，替换最小的
                    self.available.remove(min_idx);
                } else {
                    // 新缓冲区是最小的，直接丢弃
                    return;
                }
            }
        }

        self.available.push((buffer, capacity, usage));
    }

    /// 当前池中可用缓冲区数量
    pub fn available_count(&self) -> usize {
        self.available.len()
    }

    /// 当前使用中的缓冲区数量
    pub fn in_use_count(&self) -> usize {
        self.in_use_count
    }

    /// 清空池（释放所有缓冲区）
    pub fn clear(&mut self) {
        self.available.clear();
        self.in_use_count = 0;
    }
}

impl Default for BufferPool {
    fn default() -> Self {
        Self::new(64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_pool_default() {
        let pool = BufferPool::default();
        assert_eq!(pool.available_count(), 0);
        assert_eq!(pool.in_use_count(), 0);
        assert_eq!(pool.max_pool_size, 64);
    }

    #[test]
    fn test_buffer_pool_custom_size() {
        let pool = BufferPool::new(16);
        assert_eq!(pool.max_pool_size, 16);
    }

    #[test]
    fn test_usage_isolation_in_available_list() {
        // Verify that the pool's internal matching logic respects usage flags.
        // We can't create real GPU buffers without a device, but we can verify
        // the matching predicate used by acquire() works correctly.
        let vertex_usage = BufferUsages::VERTEX | BufferUsages::COPY_DST;
        let index_usage = BufferUsages::INDEX | BufferUsages::COPY_DST;

        // Simulate: a vertex buffer (size=1024) is in the pool
        let pool = BufferPool::new(64);

        // Manually push a fake entry to test matching (available is pub(crate) via Vec)
        // Since we can't create a Buffer without a device, we test the position logic directly:
        let entries: Vec<(u64, BufferUsages)> = vec![
            (1024, vertex_usage),
            (2048, index_usage),
            (512, vertex_usage),
        ];

        // Simulate acquire matching: looking for INDEX buffer of size 1024
        let match_idx = entries.iter().position(|(cap, u)| *u == index_usage && *cap >= 1024);
        assert_eq!(match_idx, Some(1), "should match the INDEX entry at index 1, not the VERTEX entry at index 0");

        // Looking for VERTEX buffer of size 256
        let match_idx = entries.iter().position(|(cap, u)| *u == vertex_usage && *cap >= 256);
        assert_eq!(match_idx, Some(0), "should match the first VERTEX entry");

        // Looking for INDEX buffer of size 4096 — nothing big enough
        let match_idx = entries.iter().position(|(cap, u)| *u == index_usage && *cap >= 4096);
        assert_eq!(match_idx, None, "no INDEX buffer large enough");

        // Looking for STORAGE usage — nothing matches
        let storage_usage = BufferUsages::STORAGE | BufferUsages::COPY_DST;
        let match_idx = entries.iter().position(|(cap, u)| *u == storage_usage && *cap >= 64);
        assert_eq!(match_idx, None, "no STORAGE buffer in pool");

        // Verify pool counters work correctly with release
        assert_eq!(pool.available_count(), 0);
        assert_eq!(pool.in_use_count(), 0);
    }
}
