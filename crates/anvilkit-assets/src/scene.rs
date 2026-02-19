//! # 场景数据
//!
//! 定义从 glTF 文件提取的完整场景数据（网格 + 材质）。

use crate::mesh::MeshData;
use crate::material::MaterialData;

/// CPU 侧场景数据（单 submesh，向后兼容）
///
/// 包含网格几何数据和对应的材质信息。
///
/// # 示例
///
/// ```rust
/// use anvilkit_assets::scene::SceneData;
/// use anvilkit_assets::mesh::MeshData;
/// use anvilkit_assets::material::MaterialData;
/// use glam::{Vec2, Vec3};
///
/// let scene = SceneData {
///     mesh: MeshData {
///         positions: vec![Vec3::ZERO],
///         normals: vec![Vec3::Z],
///         texcoords: vec![Vec2::ZERO],
///         tangents: vec![[1.0, 0.0, 0.0, 1.0]],
///         indices: vec![0],
///     },
///     material: MaterialData::default(),
/// };
/// assert_eq!(scene.mesh.vertex_count(), 1);
/// ```
#[derive(Debug, Clone)]
pub struct SceneData {
    /// 网格几何数据
    pub mesh: MeshData,
    /// 材质数据
    pub material: MaterialData,
}

/// 单个子网格（mesh primitive + 对应材质）
#[derive(Debug, Clone)]
pub struct Submesh {
    /// 网格几何数据
    pub mesh: MeshData,
    /// 材质数据
    pub material: MaterialData,
}

/// 多子网格场景数据
///
/// 一个 glTF 模型可包含多个 primitive，每个 primitive 有独立的顶点数据和材质。
///
/// # 示例
///
/// ```rust
/// use anvilkit_assets::scene::MultiMeshScene;
///
/// let scene = MultiMeshScene { submeshes: vec![] };
/// assert_eq!(scene.submesh_count(), 0);
/// ```
#[derive(Debug, Clone)]
pub struct MultiMeshScene {
    pub submeshes: Vec<Submesh>,
}

impl MultiMeshScene {
    /// 子网格数量
    pub fn submesh_count(&self) -> usize {
        self.submeshes.len()
    }

    /// 总顶点数
    pub fn total_vertex_count(&self) -> usize {
        self.submeshes.iter().map(|s| s.mesh.vertex_count()).sum()
    }
}
