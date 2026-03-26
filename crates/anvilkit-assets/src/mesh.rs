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
    /// 验证所有顶点属性数组长度一致
    ///
    /// # 返回
    ///
    /// 如果长度不一致返回错误描述
    pub fn validate(&self) -> Result<(), String> {
        let n = self.positions.len();
        if self.normals.len() != n {
            return Err(format!(
                "normals.len()={} != positions.len()={}", self.normals.len(), n
            ));
        }
        if self.texcoords.len() != n {
            return Err(format!(
                "texcoords.len()={} != positions.len()={}", self.texcoords.len(), n
            ));
        }
        if self.tangents.len() != n {
            return Err(format!(
                "tangents.len()={} != positions.len()={}", self.tangents.len(), n
            ));
        }
        Ok(())
    }

    /// 顶点数量
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

    /// 转换为交错 PBR 顶点格式
    ///
    /// 返回 `Vec<InterleavedPbrVertex>`，每个元素 48 字节，
    /// 与 `anvilkit-render` 的 `PbrVertex` 内存布局一致。
    /// 缺失的 tangent 默认为 `[1, 0, 0, 1]`。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use anvilkit_assets::mesh::MeshData;
    /// use glam::{Vec2, Vec3};
    ///
    /// let mesh = MeshData {
    ///     positions: vec![Vec3::new(1.0, 2.0, 3.0)],
    ///     normals: vec![Vec3::Y],
    ///     texcoords: vec![Vec2::new(0.5, 0.5)],
    ///     tangents: vec![[1.0, 0.0, 0.0, 1.0]],
    ///     indices: vec![0],
    /// };
    /// let verts = mesh.to_pbr_vertices();
    /// assert_eq!(verts.len(), 1);
    /// assert_eq!(verts[0].position, [1.0, 2.0, 3.0]);
    /// ```
    pub fn to_pbr_vertices(&self) -> Vec<InterleavedPbrVertex> {
        (0..self.vertex_count())
            .map(|i| InterleavedPbrVertex {
                position: self.positions[i].into(),
                normal: self.normals[i].into(),
                texcoord: self.texcoords[i].into(),
                tangent: if i < self.tangents.len() {
                    self.tangents[i]
                } else {
                    [1.0, 0.0, 0.0, 1.0]
                },
            })
            .collect()
    }
}

/// 交错 PBR 顶点数据
///
/// 内存布局与 `anvilkit-render` 的 `PbrVertex` 完全一致（48 字节 stride）。
/// 用于将 CPU 侧 `MeshData` 转换为 GPU 上传格式。
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct InterleavedPbrVertex {
    /// 顶点位置 (object space)
    pub position: [f32; 3],
    /// 顶点法线 (unit vector)
    pub normal: [f32; 3],
    /// 纹理坐标 (UV0)
    pub texcoord: [f32; 2],
    /// 切线向量 (xyz=tangent, w=bitangent sign)
    pub tangent: [f32; 4],
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

    #[test]
    fn test_to_pbr_vertices() {
        let mesh = MeshData {
            positions: vec![Vec3::new(1.0, 2.0, 3.0), Vec3::new(4.0, 5.0, 6.0)],
            normals: vec![Vec3::Y, Vec3::Z],
            texcoords: vec![Vec2::new(0.5, 0.5), Vec2::new(1.0, 0.0)],
            tangents: vec![[1.0, 0.0, 0.0, 1.0], [0.0, 1.0, 0.0, -1.0]],
            indices: vec![0, 1],
        };
        let verts = mesh.to_pbr_vertices();
        assert_eq!(verts.len(), 2);
        assert_eq!(verts[0].position, [1.0, 2.0, 3.0]);
        assert_eq!(verts[0].normal, [0.0, 1.0, 0.0]);
        assert_eq!(verts[0].texcoord, [0.5, 0.5]);
        assert_eq!(verts[1].tangent, [0.0, 1.0, 0.0, -1.0]);
    }

    #[test]
    fn test_to_pbr_vertices_missing_tangents() {
        let mesh = MeshData {
            positions: vec![Vec3::ZERO],
            normals: vec![Vec3::Z],
            texcoords: vec![Vec2::ZERO],
            tangents: vec![], // 缺失 tangents
            indices: vec![0],
        };
        let verts = mesh.to_pbr_vertices();
        assert_eq!(verts.len(), 1);
        assert_eq!(verts[0].tangent, [1.0, 0.0, 0.0, 1.0]); // 默认值
    }
}
