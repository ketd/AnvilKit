//! # CPU 侧网格数据
//!
//! 定义从 glTF 文件提取的网格数据结构。

use glam::{Vec2, Vec3};

/// CPU 侧网格数据
///
/// 包含从 glTF 文件提取的顶点属性和索引数据。
/// 所有属性数组长度一致（`positions.len() == normals.len() == texcoords.len() == tangents.len()`）。
///
/// # 示例
///
/// ```rust
/// use anvilkit_assets::mesh::MeshData;
/// use glam::{Vec2, Vec3};
///
/// let mesh = MeshData {
///     positions: vec![Vec3::ZERO, Vec3::X, Vec3::Y],
///     normals: vec![Vec3::Z, Vec3::Z, Vec3::Z],
///     texcoords: vec![Vec2::ZERO, Vec2::X, Vec2::Y],
///     tangents: vec![[1.0, 0.0, 0.0, 1.0]; 3],
///     indices: vec![0, 1, 2],
/// };
/// assert_eq!(mesh.vertex_count(), 3);
/// assert_eq!(mesh.index_count(), 3);
/// ```
#[derive(Debug, Clone)]
pub struct MeshData {
    /// 顶点位置（物体空间）
    pub positions: Vec<Vec3>,
    /// 顶点法线（单位向量）
    pub normals: Vec<Vec3>,
    /// 纹理坐标（UV 通道 0，缺失时为 Vec2::ZERO）
    pub texcoords: Vec<Vec2>,
    /// 切线向量（xyz=tangent 方向, w=bitangent sign），缺失时为 [1,0,0,1]
    pub tangents: Vec<[f32; 4]>,
    /// 三角形索引 (u32)
    pub indices: Vec<u32>,
}

impl MeshData {
    /// 顶点数量
    ///
    /// # 示例
    ///
    /// ```rust
    /// use anvilkit_assets::mesh::MeshData;
    /// use glam::{Vec2, Vec3};
    ///
    /// let mesh = MeshData {
    ///     positions: vec![Vec3::ZERO; 100],
    ///     normals: vec![Vec3::Z; 100],
    ///     texcoords: vec![Vec2::ZERO; 100],
    ///     tangents: vec![[1.0, 0.0, 0.0, 1.0]; 100],
    ///     indices: vec![0; 300],
    /// };
    /// assert_eq!(mesh.vertex_count(), 100);
    /// ```
    pub fn vertex_count(&self) -> usize {
        self.positions.len()
    }

    /// 索引数量
    ///
    /// # 示例
    ///
    /// ```rust
    /// use anvilkit_assets::mesh::MeshData;
    /// use glam::{Vec2, Vec3};
    ///
    /// let mesh = MeshData {
    ///     positions: vec![Vec3::ZERO; 3],
    ///     normals: vec![Vec3::Z; 3],
    ///     texcoords: vec![Vec2::ZERO; 3],
    ///     tangents: vec![[1.0, 0.0, 0.0, 1.0]; 3],
    ///     indices: vec![0, 1, 2, 2, 1, 0],
    /// };
    /// assert_eq!(mesh.index_count(), 6);
    /// ```
    pub fn index_count(&self) -> usize {
        self.indices.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mesh_data_counts() {
        let mesh = MeshData {
            positions: vec![Vec3::ZERO; 24],
            normals: vec![Vec3::Z; 24],
            texcoords: vec![Vec2::ZERO; 24],
            tangents: vec![[1.0, 0.0, 0.0, 1.0]; 24],
            indices: vec![0; 36],
        };
        assert_eq!(mesh.vertex_count(), 24);
        assert_eq!(mesh.index_count(), 36);
    }

    #[test]
    fn test_mesh_data_empty() {
        let mesh = MeshData {
            positions: vec![],
            normals: vec![],
            texcoords: vec![],
            tangents: vec![],
            indices: vec![],
        };
        assert_eq!(mesh.vertex_count(), 0);
        assert_eq!(mesh.index_count(), 0);
    }
}
