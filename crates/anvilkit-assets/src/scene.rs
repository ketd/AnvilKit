//! # 场景数据
//!
//! 定义从 glTF 文件提取的完整场景数据（网格 + 材质）。

use crate::mesh::MeshData;
use crate::material::MaterialData;

/// CPU 侧场景数据
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
