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

    /// 生成立方体网格数据
    ///
    /// 以原点为中心，边长为 `size` 的立方体。
    /// 每个面有独立法线（24 顶点，36 索引）。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use anvilkit_assets::mesh::MeshData;
    /// let cube = MeshData::generate_box(1.0);
    /// assert_eq!(cube.vertex_count(), 24);
    /// assert_eq!(cube.index_count(), 36);
    /// ```
    pub fn generate_box(size: f32) -> Self {
        let h = size * 0.5;
        let mut positions = Vec::with_capacity(24);
        let mut normals = Vec::with_capacity(24);
        let mut texcoords = Vec::with_capacity(24);
        let mut indices = Vec::with_capacity(36);

        // Face definitions: (normal, [4 corner offsets])
        let faces: [([f32; 3], [[f32; 3]; 4]); 6] = [
            // +Z (front)
            ([0.0, 0.0, 1.0], [[-h, -h, h], [h, -h, h], [h, h, h], [-h, h, h]]),
            // -Z (back)
            ([0.0, 0.0, -1.0], [[h, -h, -h], [-h, -h, -h], [-h, h, -h], [h, h, -h]]),
            // +X (right)
            ([1.0, 0.0, 0.0], [[h, -h, h], [h, -h, -h], [h, h, -h], [h, h, h]]),
            // -X (left)
            ([-1.0, 0.0, 0.0], [[-h, -h, -h], [-h, -h, h], [-h, h, h], [-h, h, -h]]),
            // +Y (top)
            ([0.0, 1.0, 0.0], [[-h, h, h], [h, h, h], [h, h, -h], [-h, h, -h]]),
            // -Y (bottom)
            ([0.0, -1.0, 0.0], [[-h, -h, -h], [h, -h, -h], [h, -h, h], [-h, -h, h]]),
        ];

        let uvs = [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]];

        for (normal, corners) in &faces {
            let base = positions.len() as u32;
            for (j, corner) in corners.iter().enumerate() {
                positions.push(Vec3::new(corner[0], corner[1], corner[2]));
                normals.push(Vec3::new(normal[0], normal[1], normal[2]));
                texcoords.push(Vec2::new(uvs[j][0], uvs[j][1]));
            }
            indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }

        Self {
            positions,
            normals,
            texcoords,
            tangents: vec![[1.0, 0.0, 0.0, 1.0]; 24],
            indices,
        }
    }

    /// 生成平面网格数据
    ///
    /// 位于 XZ 平面，法线朝 +Y，以原点为中心，边长为 `size`。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use anvilkit_assets::mesh::MeshData;
    /// let plane = MeshData::generate_plane(10.0);
    /// assert_eq!(plane.vertex_count(), 4);
    /// assert_eq!(plane.index_count(), 6);
    /// ```
    pub fn generate_plane(size: f32) -> Self {
        let h = size * 0.5;
        Self {
            positions: vec![
                Vec3::new(-h, 0.0, h),
                Vec3::new(h, 0.0, h),
                Vec3::new(h, 0.0, -h),
                Vec3::new(-h, 0.0, -h),
            ],
            normals: vec![Vec3::Y; 4],
            texcoords: vec![
                Vec2::new(0.0, 1.0),
                Vec2::new(1.0, 1.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(0.0, 0.0),
            ],
            tangents: vec![[1.0, 0.0, 0.0, 1.0]; 4],
            indices: vec![0, 1, 2, 0, 2, 3],
        }
    }

    /// 生成球体网格数据
    ///
    /// UV 球体，半径为 `radius`，`segments` 经线数，`rings` 纬线数。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use anvilkit_assets::mesh::MeshData;
    /// let sphere = MeshData::generate_sphere(1.0, 16, 12);
    /// assert!(sphere.vertex_count() > 0);
    /// assert!(sphere.index_count() > 0);
    /// ```
    pub fn generate_sphere(radius: f32, segments: u32, rings: u32) -> Self {
        let mut positions = Vec::new();
        let mut normals = Vec::new();
        let mut texcoords = Vec::new();
        let mut indices = Vec::new();

        for y in 0..=rings {
            let v = y as f32 / rings as f32;
            let phi = v * std::f32::consts::PI;
            for x in 0..=segments {
                let u = x as f32 / segments as f32;
                let theta = u * std::f32::consts::TAU;

                let sin_phi = phi.sin();
                let cos_phi = phi.cos();
                let sin_theta = theta.sin();
                let cos_theta = theta.cos();

                let nx = cos_theta * sin_phi;
                let ny = cos_phi;
                let nz = sin_theta * sin_phi;

                positions.push(Vec3::new(nx * radius, ny * radius, nz * radius));
                normals.push(Vec3::new(nx, ny, nz));
                texcoords.push(Vec2::new(u, v));
            }
        }

        let stride = segments + 1;
        for y in 0..rings {
            for x in 0..segments {
                let a = y * stride + x;
                let b = a + stride;
                indices.extend_from_slice(&[a, b, a + 1, b, b + 1, a + 1]);
            }
        }

        let vert_count = positions.len();
        Self {
            positions,
            normals,
            texcoords,
            tangents: vec![[1.0, 0.0, 0.0, 1.0]; vert_count],
            indices,
        }
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

    #[test]
    fn test_generate_box() {
        let cube = MeshData::generate_box(2.0);
        assert_eq!(cube.vertex_count(), 24);
        assert_eq!(cube.index_count(), 36);
        assert!(cube.validate().is_ok());
    }

    #[test]
    fn test_generate_plane() {
        let plane = MeshData::generate_plane(10.0);
        assert_eq!(plane.vertex_count(), 4);
        assert_eq!(plane.index_count(), 6);
        assert!(plane.validate().is_ok());
        // All normals should be +Y
        for n in &plane.normals {
            assert_eq!(*n, Vec3::Y);
        }
    }

    #[test]
    fn test_generate_sphere() {
        let sphere = MeshData::generate_sphere(1.0, 16, 12);
        assert!(sphere.vertex_count() > 0);
        assert!(sphere.index_count() > 0);
        assert!(sphere.validate().is_ok());
        // Check all positions are on the unit sphere
        for p in &sphere.positions {
            let len = p.length();
            assert!((len - 1.0).abs() < 0.001, "Vertex not on unit sphere: len={}", len);
        }
    }
}
