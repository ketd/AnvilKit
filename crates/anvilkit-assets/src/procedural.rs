//! # 程序化网格生成
//!
//! 提供常用几何体的程序化网格生成函数，返回 [`MeshData`] 用于后续上传到 GPU。
//!
//! ## 支持的几何体
//!
//! - [`generate_sphere`] — UV 球体
//! - [`generate_plane`] — XZ 平面
//! - [`generate_box`] — 立方体/长方体
//!
//! ## 使用示例
//!
//! ```rust
//! use anvilkit_assets::procedural::{generate_sphere, generate_plane, generate_box};
//!
//! let mesh = generate_sphere(1.0, 24, 16);
//! assert_eq!(mesh.vertex_count(), 25 * 17);
//!
//! let mesh = generate_plane(10.0, 10.0);
//! assert_eq!(mesh.vertex_count(), 4);
//! assert_eq!(mesh.index_count(), 6);
//! ```

use glam::{Vec2, Vec3};

use crate::mesh::MeshData;

/// 生成 UV 球体网格
///
/// 使用经纬度参数化生成球体顶点和索引数据。
///
/// # 参数
///
/// - `radius`: 球体半径
/// - `sectors`: 经度分段数（水平切片），建议 ≥ 8
/// - `rings`: 纬度分段数（垂直切片），建议 ≥ 4
///
/// # 返回
///
/// [`MeshData`]，顶点数 = `(sectors + 1) * (rings + 1)`
///
/// # 示例
///
/// ```rust
/// use anvilkit_assets::procedural::generate_sphere;
///
/// let mesh = generate_sphere(1.0, 24, 16);
/// assert_eq!(mesh.vertex_count(), 25 * 17);
/// assert_eq!(mesh.index_count(), 24 * 16 * 6);
/// ```
pub fn generate_sphere(radius: f32, sectors: u32, rings: u32) -> MeshData {
    let sector_count = sectors as usize;
    let ring_count = rings as usize;
    let vert_count = (sector_count + 1) * (ring_count + 1);

    let mut positions = Vec::with_capacity(vert_count);
    let mut normals = Vec::with_capacity(vert_count);
    let mut texcoords = Vec::with_capacity(vert_count);
    let mut tangents = Vec::with_capacity(vert_count);
    let mut indices = Vec::with_capacity(sector_count * ring_count * 6);

    for i in 0..=ring_count {
        // phi: 0 (top) → π (bottom)
        let phi = std::f32::consts::PI * i as f32 / ring_count as f32;
        let sin_phi = phi.sin();
        let cos_phi = phi.cos();

        for j in 0..=sector_count {
            // theta: 0 → 2π
            let theta = 2.0 * std::f32::consts::PI * j as f32 / sector_count as f32;
            let sin_theta = theta.sin();
            let cos_theta = theta.cos();

            let x = sin_phi * cos_theta;
            let y = cos_phi;
            let z = sin_phi * sin_theta;

            positions.push(Vec3::new(radius * x, radius * y, radius * z));
            normals.push(Vec3::new(x, y, z));
            texcoords.push(Vec2::new(
                j as f32 / sector_count as f32,
                i as f32 / ring_count as f32,
            ));

            // Tangent: derivative of position w.r.t. theta, normalized
            let tx = -sin_theta;
            let tz = cos_theta;
            let t_len = (tx * tx + tz * tz).sqrt().max(1e-6);
            tangents.push([tx / t_len, 0.0, tz / t_len, 1.0]);
        }
    }

    // Indices: two triangles per grid cell
    let stride = (sector_count + 1) as u32;
    for i in 0..rings {
        for j in 0..sectors {
            let top_left = i * stride + j;
            let bottom_left = (i + 1) * stride + j;

            indices.push(top_left);
            indices.push(bottom_left);
            indices.push(top_left + 1);

            indices.push(top_left + 1);
            indices.push(bottom_left);
            indices.push(bottom_left + 1);
        }
    }

    MeshData {
        positions,
        normals,
        texcoords,
        tangents,
        indices,
    }
}

/// 生成 XZ 平面网格
///
/// 生成位于 y=0 的水平平面，法线朝上 (0, 1, 0)。
///
/// # 参数
///
/// - `width`: X 轴方向宽度
/// - `depth`: Z 轴方向深度
///
/// # 返回
///
/// [`MeshData`]，4 个顶点, 6 个索引
///
/// # 示例
///
/// ```rust
/// use anvilkit_assets::procedural::generate_plane;
///
/// let mesh = generate_plane(10.0, 10.0);
/// assert_eq!(mesh.vertex_count(), 4);
/// assert_eq!(mesh.index_count(), 6);
/// ```
pub fn generate_plane(width: f32, depth: f32) -> MeshData {
    let hw = width * 0.5;
    let hd = depth * 0.5;

    MeshData {
        positions: vec![
            Vec3::new(-hw, 0.0, -hd),
            Vec3::new(hw, 0.0, -hd),
            Vec3::new(hw, 0.0, hd),
            Vec3::new(-hw, 0.0, hd),
        ],
        normals: vec![Vec3::Y; 4],
        texcoords: vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(0.0, 1.0),
        ],
        tangents: vec![[1.0, 0.0, 0.0, 1.0]; 4],
        indices: vec![0, 1, 2, 0, 2, 3],
    }
}

/// 生成立方体/长方体网格
///
/// 以原点为中心，6 个面各 4 个顶点，共 24 顶点 36 索引。
/// 每个面的法线、切线独立设置，适用于 PBR 法线贴图渲染。
///
/// # 参数
///
/// - `half_extents`: 半尺寸 `[half_x, half_y, half_z]`
///
/// # 返回
///
/// [`MeshData`]，24 个顶点, 36 个索引
///
/// # 示例
///
/// ```rust
/// use anvilkit_assets::procedural::generate_box;
///
/// let mesh = generate_box([0.5, 0.5, 0.5]);
/// assert_eq!(mesh.vertex_count(), 24);
/// assert_eq!(mesh.index_count(), 36);
/// ```
pub fn generate_box(half_extents: [f32; 3]) -> MeshData {
    let [hx, hy, hz] = half_extents;

    // Face data: (normal, tangent, 4 positions)
    let faces: [(Vec3, [f32; 4], [Vec3; 4]); 6] = [
        // +Z face
        (
            Vec3::Z,
            [1.0, 0.0, 0.0, 1.0],
            [
                Vec3::new(-hx, -hy, hz),
                Vec3::new(hx, -hy, hz),
                Vec3::new(hx, hy, hz),
                Vec3::new(-hx, hy, hz),
            ],
        ),
        // -Z face
        (
            Vec3::NEG_Z,
            [-1.0, 0.0, 0.0, 1.0],
            [
                Vec3::new(hx, -hy, -hz),
                Vec3::new(-hx, -hy, -hz),
                Vec3::new(-hx, hy, -hz),
                Vec3::new(hx, hy, -hz),
            ],
        ),
        // +X face
        (
            Vec3::X,
            [0.0, 0.0, 1.0, 1.0],
            [
                Vec3::new(hx, -hy, hz),
                Vec3::new(hx, -hy, -hz),
                Vec3::new(hx, hy, -hz),
                Vec3::new(hx, hy, hz),
            ],
        ),
        // -X face
        (
            Vec3::NEG_X,
            [0.0, 0.0, -1.0, 1.0],
            [
                Vec3::new(-hx, -hy, -hz),
                Vec3::new(-hx, -hy, hz),
                Vec3::new(-hx, hy, hz),
                Vec3::new(-hx, hy, -hz),
            ],
        ),
        // +Y face
        (
            Vec3::Y,
            [1.0, 0.0, 0.0, 1.0],
            [
                Vec3::new(-hx, hy, hz),
                Vec3::new(hx, hy, hz),
                Vec3::new(hx, hy, -hz),
                Vec3::new(-hx, hy, -hz),
            ],
        ),
        // -Y face
        (
            Vec3::NEG_Y,
            [1.0, 0.0, 0.0, 1.0],
            [
                Vec3::new(-hx, -hy, -hz),
                Vec3::new(hx, -hy, -hz),
                Vec3::new(hx, -hy, hz),
                Vec3::new(-hx, -hy, hz),
            ],
        ),
    ];

    let face_uvs: [Vec2; 4] = [
        Vec2::new(0.0, 1.0),
        Vec2::new(1.0, 1.0),
        Vec2::new(1.0, 0.0),
        Vec2::new(0.0, 0.0),
    ];

    let mut positions = Vec::with_capacity(24);
    let mut normals_vec = Vec::with_capacity(24);
    let mut texcoords = Vec::with_capacity(24);
    let mut tangents = Vec::with_capacity(24);
    let mut indices = Vec::with_capacity(36);

    for (face_idx, (normal, tangent, face_positions)) in faces.iter().enumerate() {
        let base = (face_idx * 4) as u32;
        for (v_idx, pos) in face_positions.iter().enumerate() {
            positions.push(*pos);
            normals_vec.push(*normal);
            texcoords.push(face_uvs[v_idx]);
            tangents.push(*tangent);
        }
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    MeshData {
        positions,
        normals: normals_vec,
        texcoords,
        tangents,
        indices,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sphere_vertex_count() {
        let sectors = 24u32;
        let rings = 16u32;
        let mesh = generate_sphere(1.0, sectors, rings);

        let expected_verts = (sectors + 1) as usize * (rings + 1) as usize;
        assert_eq!(mesh.vertex_count(), expected_verts);

        let expected_indices = (sectors * rings * 6) as usize;
        assert_eq!(mesh.index_count(), expected_indices);
    }

    #[test]
    fn test_sphere_normals_match_position_direction() {
        let mesh = generate_sphere(2.0, 12, 8);
        for (pos, normal) in mesh.positions.iter().zip(mesh.normals.iter()) {
            if pos.length() > 1e-4 {
                let expected_normal = pos.normalize();
                let dot = normal.dot(expected_normal);
                assert!(
                    dot > 0.99,
                    "Normal {:?} doesn't match position direction {:?}, dot={}",
                    normal, expected_normal, dot
                );
            }
        }
    }

    #[test]
    fn test_sphere_small() {
        let mesh = generate_sphere(0.5, 4, 2);
        assert_eq!(mesh.vertex_count(), 5 * 3);
        assert_eq!(mesh.index_count(), 4 * 2 * 6);
    }

    #[test]
    fn test_plane_basic() {
        let mesh = generate_plane(10.0, 10.0);
        assert_eq!(mesh.vertex_count(), 4);
        assert_eq!(mesh.index_count(), 6);

        for pos in &mesh.positions {
            assert_eq!(pos.y, 0.0);
        }
        for normal in &mesh.normals {
            assert_eq!(*normal, Vec3::Y);
        }
    }

    #[test]
    fn test_box_basic() {
        let mesh = generate_box([0.5, 0.5, 0.5]);
        assert_eq!(mesh.vertex_count(), 24);
        assert_eq!(mesh.index_count(), 36);
    }

    #[test]
    fn test_box_normals_unit_length() {
        let mesh = generate_box([1.0, 2.0, 3.0]);
        for normal in &mesh.normals {
            let len = normal.length();
            assert!(
                (len - 1.0).abs() < 1e-5,
                "Normal {:?} has length {}",
                normal, len
            );
        }
    }

    #[test]
    fn test_box_asymmetric() {
        let mesh = generate_box([1.0, 0.5, 2.0]);
        assert_eq!(mesh.vertex_count(), 24);
        assert_eq!(mesh.index_count(), 36);

        for pos in &mesh.positions {
            assert!(pos.x.abs() <= 1.0 + 1e-5);
            assert!(pos.y.abs() <= 0.5 + 1e-5);
            assert!(pos.z.abs() <= 2.0 + 1e-5);
        }
    }
}
