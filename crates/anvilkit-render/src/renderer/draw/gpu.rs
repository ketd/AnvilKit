//! GPU 缓冲区、渲染目标和实例数据

/// Uniform 批量写入缓冲区
///
/// 将所有 draw commands 的 uniform 数据（model matrix + material params）
/// 打包到单次 `queue.write_buffer()` 调用中，减少 CPU→GPU 传输次数。
///
/// 每个 draw command 占 256 字节（满足 `minUniformBufferOffsetAlignment` 要求）。
pub struct UniformBatchBuffer {
    /// CPU 侧数据缓冲区
    data: Vec<u8>,
    /// 每个 uniform 块的对齐大小（字节）
    alignment: usize,
}

impl UniformBatchBuffer {
    /// 创建批量 uniform 缓冲区
    ///
    /// `alignment` 通常为 256（wgpu 默认 minUniformBufferOffsetAlignment）
    pub fn new(alignment: usize) -> Self {
        Self {
            data: Vec::new(),
            alignment,
        }
    }

    /// 清空缓冲区，准备新一帧
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// 写入一个 draw command 的 uniform 数据
    ///
    /// 自动对齐到 `alignment` 边界。返回在缓冲区中的偏移量。
    pub fn push(&mut self, uniform_bytes: &[u8]) -> u32 {
        let offset = self.data.len();
        self.data.extend_from_slice(uniform_bytes);
        // Pad to alignment
        let remainder = self.data.len() % self.alignment;
        if remainder != 0 {
            self.data.resize(self.data.len() + self.alignment - remainder, 0);
        }
        offset as u32
    }

    /// 获取完整的 CPU 侧数据（用于 queue.write_buffer）
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// 当前缓冲区中的条目数量
    pub fn count(&self) -> usize {
        if self.data.is_empty() { 0 } else { self.data.len() / self.alignment }
    }

    /// 缓冲区总大小（字节）
    pub fn size(&self) -> usize {
        self.data.len()
    }
}

impl Default for UniformBatchBuffer {
    fn default() -> Self {
        Self::new(256)
    }
}

/// 渲染目标
///
/// 指定相机渲染到屏幕或纹理。用于多相机场景（minimap、后视镜等）。
#[derive(Debug, Clone)]
pub enum RenderTarget {
    /// 渲染到屏幕（默认 swap chain）
    Screen,
    /// 渲染到纹理（通过 MeshHandle 引用 RenderAssets 中的 texture）
    Texture {
        /// 目标纹理宽度
        width: u32,
        /// 目标纹理高度
        height: u32,
        /// 纹理标签（调试用）
        label: String,
    },
}

impl Default for RenderTarget {
    fn default() -> Self {
        Self::Screen
    }
}

/// GPU 实例数据（per-instance，128 字节）
///
/// 包含每个实例的变换和材质参数。
/// 用于 GPU instancing 时通过 storage buffer 传递。
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceData {
    /// Object-to-world model matrix (64 bytes).
    pub model: [[f32; 4]; 4],
    /// Inverse-transpose model matrix for normals (64 bytes).
    pub normal_matrix: [[f32; 4]; 4],
}

impl Default for InstanceData {
    fn default() -> Self {
        Self {
            model: glam::Mat4::IDENTITY.to_cols_array_2d(),
            normal_matrix: glam::Mat4::IDENTITY.to_cols_array_2d(),
        }
    }
}
