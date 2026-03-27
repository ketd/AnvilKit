//! # Shared renderer utilities
//!
//! Common types and helpers used by multiple renderers to reduce duplication.

use bytemuck::{Pod, Zeroable};

/// A cached GPU vertex/instance buffer that grows as needed but never shrinks.
///
/// Used to avoid re-allocating GPU buffers every frame when the required size
/// is less than or equal to the current capacity.
pub struct CachedBuffer {
    inner: Option<(wgpu::Buffer, u64)>,
    label: &'static str,
    usage: wgpu::BufferUsages,
}

impl CachedBuffer {
    /// Create a new cached buffer with the given label and usage flags.
    pub fn new(label: &'static str, usage: wgpu::BufferUsages) -> Self {
        Self {
            inner: None,
            label,
            usage,
        }
    }

    /// Create a cached vertex buffer (VERTEX | COPY_DST).
    pub fn vertex(label: &'static str) -> Self {
        Self::new(label, wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST)
    }

    /// Ensure the buffer is at least `needed` bytes, creating a new one if necessary.
    /// Returns a reference to the underlying wgpu::Buffer.
    pub fn ensure_and_write(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        data: &[u8],
    ) -> &wgpu::Buffer {
        let needed = data.len() as u64;
        let reuse = self.inner.as_ref().map_or(false, |(_, cap)| *cap >= needed);
        if !reuse {
            self.inner = Some((
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some(self.label),
                    size: needed,
                    usage: self.usage,
                    mapped_at_creation: false,
                }),
                needed,
            ));
        }
        let buf = &self.inner.as_ref().expect("buffer must be initialized above").0;
        queue.write_buffer(buf, 0, data);
        buf
    }
}

/// A single 4x4 matrix uniform (64 bytes).
///
/// Used for orthographic projections (2D renderers) and view-projection matrices
/// (3D renderers). All renderers need the same GPU layout: a single `mat4x4<f32>`.
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct MatrixUniform {
    /// The 4x4 matrix in column-major order.
    pub matrix: [[f32; 4]; 4],
}

impl MatrixUniform {
    /// Create from a glam::Mat4.
    pub fn from_mat4(m: &glam::Mat4) -> Self {
        Self {
            matrix: m.to_cols_array_2d(),
        }
    }

    /// Identity matrix uniform.
    pub fn identity() -> Self {
        Self::from_mat4(&glam::Mat4::IDENTITY)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matrix_uniform_size() {
        assert_eq!(std::mem::size_of::<MatrixUniform>(), 64);
    }

    #[test]
    fn test_matrix_uniform_identity() {
        let u = MatrixUniform::identity();
        assert_eq!(u.matrix[0][0], 1.0);
        assert_eq!(u.matrix[1][1], 1.0);
        assert_eq!(u.matrix[2][2], 1.0);
        assert_eq!(u.matrix[3][3], 1.0);
    }

    #[test]
    fn test_cached_buffer_new() {
        let buf = CachedBuffer::vertex("test");
        assert!(buf.inner.is_none());
    }
}
