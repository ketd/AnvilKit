use crate::block::{BlockType, Face, tile_uv, TILE_UV, TILE_INSET};
use crate::chunk::{ChunkData, CHUNK_SIZE, CHUNK_HEIGHT};
use crate::vertex::BlockVertex;

/// Neighbors: [+X, -X, +Z, -Z] (for cross-chunk face culling)
pub type ChunkNeighbors<'a> = [Option<&'a ChunkData>; 4];

pub struct ChunkMesh {
    pub vertices: Vec<BlockVertex>,
    pub indices: Vec<u32>,
}

/// Standard vertex AO formula: 2 edge neighbors + 1 corner neighbor.
/// Returns 0.0 (fully occluded) to 1.0 (no occlusion).
fn vertex_ao(side1: bool, side2: bool, corner: bool) -> f32 {
    if side1 && side2 {
        return 0.0;
    }
    (3 - (side1 as u8 + side2 as u8 + corner as u8)) as f32 / 3.0
}

/// Check if the block at the given position is an occluder for AO purposes.
#[inline]
fn is_ao_occluder(chunk: &ChunkData, neighbors: &ChunkNeighbors, x: i32, y: i32, z: i32) -> bool {
    get_neighbor_block(chunk, neighbors, x, y, z).is_obstacle()
}

/// Generate mesh for a chunk at world offset (ox, oz) = (cx * CHUNK_SIZE, cz * CHUNK_SIZE).
pub fn mesh_chunk(chunk: &ChunkData, neighbors: &ChunkNeighbors, ox: f32, oz: f32) -> ChunkMesh {
    let mut vertices = Vec::with_capacity(4096);
    let mut indices = Vec::with_capacity(8192);

    for y in 0..CHUNK_HEIGHT {
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let block = chunk.get(x, y, z);
                if block == BlockType::Air {
                    continue;
                }

                let wx = ox + x as f32;
                let wy = y as f32;
                let wz = oz + z as f32;
                let lx = x as i32;
                let ly = y as i32;
                let lz = z as i32;

                if block.is_plant() {
                    emit_plant(&mut vertices, &mut indices, block, wx, wy, wz);
                    continue;
                }

                // Check 6 neighbors — emit face only if neighbor is transparent
                let faces = [
                    (Face::Top, 0, 1, 0),
                    (Face::Bottom, 0, -1, 0),
                    (Face::Right, 1, 0, 0),
                    (Face::Left, -1, 0, 0),
                    (Face::Front, 0, 0, 1),
                    (Face::Back, 0, 0, -1),
                ];

                for &(face, dx, dy, dz) in &faces {
                    let nx = lx + dx;
                    let ny = ly + dy;
                    let nz = lz + dz;

                    let neighbor_block = get_neighbor_block(chunk, neighbors, nx, ny, nz);
                    if !neighbor_block.is_transparent() {
                        continue;
                    }
                    // Don't render internal faces of same transparent block type
                    if block.is_transparent() && neighbor_block == block {
                        continue;
                    }

                    emit_face(&mut vertices, &mut indices, block, face, wx, wy, wz,
                              chunk, neighbors, lx, ly, lz);
                }
            }
        }
    }

    ChunkMesh { vertices, indices }
}

fn get_neighbor_block(
    chunk: &ChunkData,
    neighbors: &ChunkNeighbors,
    x: i32,
    y: i32,
    z: i32,
) -> BlockType {
    if y < 0 || y >= CHUNK_HEIGHT as i32 {
        return BlockType::Air;
    }
    if x >= 0 && x < CHUNK_SIZE as i32 && z >= 0 && z < CHUNK_SIZE as i32 {
        return chunk.get_safe(x, y, z);
    }
    // Cross-chunk: neighbors order [+X, -X, +Z, -Z]
    if x >= CHUNK_SIZE as i32 {
        if let Some(n) = neighbors[0] {
            return n.get_safe(x - CHUNK_SIZE as i32, y, z);
        }
    } else if x < 0 {
        if let Some(n) = neighbors[1] {
            return n.get_safe(x + CHUNK_SIZE as i32, y, z);
        }
    }
    if z >= CHUNK_SIZE as i32 {
        if let Some(n) = neighbors[2] {
            return n.get_safe(x, y, z - CHUNK_SIZE as i32);
        }
    } else if z < 0 {
        if let Some(n) = neighbors[3] {
            return n.get_safe(x, y, z + CHUNK_SIZE as i32);
        }
    }
    BlockType::Air
}

fn emit_face(
    vertices: &mut Vec<BlockVertex>,
    indices: &mut Vec<u32>,
    block: BlockType,
    face: Face,
    x: f32,
    y: f32,
    z: f32,
    chunk: &ChunkData,
    neighbors: &ChunkNeighbors,
    lx: i32,
    ly: i32,
    lz: i32,
) {
    let tile = block.face_tile(face);
    let (du, dv) = tile_uv(tile);
    let u0 = du + TILE_INSET;
    let v0 = dv + TILE_INSET;
    let u1 = du + TILE_UV - TILE_INSET;
    let v1 = dv + TILE_UV - TILE_INSET;
    let normal = face.normal();

    // Compute per-vertex AO.
    // For each face, the 4 vertices each sample 3 neighbors (2 edge + 1 corner).
    // The neighbor positions depend on the face orientation.
    let ao = compute_face_ao(chunk, neighbors, face, lx, ly, lz);

    let base = vertices.len() as u32;

    // Vertex order: p0→p1→p2→p3, indices {0,1,2, 0,2,3}
    // For correct CCW front-face: (p1-p0)×(p2-p0) must equal face normal.
    // UV: v0=top of tile, v1=bottom of tile; u0=left, u1=right.
    let (p0, p1, p2, p3, uv0, uv1, uv2, uv3) = match face {
        Face::Top => ( // +Y: viewed from above, CCW in XZ plane
            [x, y + 1.0, z + 1.0],
            [x + 1.0, y + 1.0, z + 1.0],
            [x + 1.0, y + 1.0, z],
            [x, y + 1.0, z],
            [u0, v1], [u1, v1], [u1, v0], [u0, v0],
        ),
        Face::Bottom => ( // -Y: viewed from below, CCW
            [x, y, z],
            [x + 1.0, y, z],
            [x + 1.0, y, z + 1.0],
            [x, y, z + 1.0],
            [u0, v0], [u1, v0], [u1, v1], [u0, v1],
        ),
        Face::Right => ( // +X: viewed from right, CCW in YZ plane
            [x + 1.0, y, z],
            [x + 1.0, y + 1.0, z],
            [x + 1.0, y + 1.0, z + 1.0],
            [x + 1.0, y, z + 1.0],
            [u1, v1], [u1, v0], [u0, v0], [u0, v1],
        ),
        Face::Left => ( // -X: viewed from left, CCW in YZ plane
            [x, y, z + 1.0],
            [x, y + 1.0, z + 1.0],
            [x, y + 1.0, z],
            [x, y, z],
            [u1, v1], [u1, v0], [u0, v0], [u0, v1],
        ),
        Face::Front => ( // +Z: viewed from front, CCW in XY plane
            [x, y, z + 1.0],
            [x + 1.0, y, z + 1.0],
            [x + 1.0, y + 1.0, z + 1.0],
            [x, y + 1.0, z + 1.0],
            [u0, v1], [u1, v1], [u1, v0], [u0, v0],
        ),
        Face::Back => ( // -Z: viewed from back, CCW in XY plane
            [x + 1.0, y, z],
            [x, y, z],
            [x, y + 1.0, z],
            [x + 1.0, y + 1.0, z],
            [u0, v1], [u1, v1], [u1, v0], [u0, v0],
        ),
    };

    vertices.push(BlockVertex { position: p0, uv: uv0, normal, ao: ao[0] });
    vertices.push(BlockVertex { position: p1, uv: uv1, normal, ao: ao[1] });
    vertices.push(BlockVertex { position: p2, uv: uv2, normal, ao: ao[2] });
    vertices.push(BlockVertex { position: p3, uv: uv3, normal, ao: ao[3] });

    // AO flip optimization: when ao[0]+ao[2] < ao[1]+ao[3], flip the quad diagonal
    // to avoid AO interpolation seams across the diagonal.
    if ao[0] + ao[2] < ao[1] + ao[3] {
        // Flipped: 1-3-2, 1-0-3
        indices.extend_from_slice(&[base + 1, base + 3, base + 2, base + 1, base, base + 3]);
    } else {
        // Normal: 0-2-1, 0-3-2
        indices.extend_from_slice(&[base, base + 2, base + 1, base, base + 3, base + 2]);
    }
}

/// Compute AO values for the 4 vertices of a face.
/// Each vertex checks 2 edge neighbors and 1 corner neighbor relative to the face.
fn compute_face_ao(
    chunk: &ChunkData,
    neighbors: &ChunkNeighbors,
    face: Face,
    x: i32,
    y: i32,
    z: i32,
) -> [f32; 4] {
    // For each face we define the 4 vertices' AO sample offsets.
    // The offsets are relative to the block position in the face's tangent space.
    // Each vertex needs: (side1_offset, side2_offset, corner_offset) relative to block pos,
    // shifted by the face normal direction.
    let o = |dx: i32, dy: i32, dz: i32| -> bool {
        is_ao_occluder(chunk, neighbors, x + dx, y + dy, z + dz)
    };

    match face {
        Face::Top => {
            // Face at y+1. Tangent plane is XZ. Vertices: (-x,-z), (+x,-z), (+x,+z), (-x,+z)
            // but matching our vertex order: (x,z+1), (x+1,z+1), (x+1,z), (x,z) viewed from above
            let a0 = vertex_ao(o(-1, 1, 0), o(0, 1, 1), o(-1, 1, 1));   // p0: x,z+1
            let a1 = vertex_ao(o(1, 1, 0), o(0, 1, 1), o(1, 1, 1));     // p1: x+1,z+1
            let a2 = vertex_ao(o(1, 1, 0), o(0, 1, -1), o(1, 1, -1));   // p2: x+1,z
            let a3 = vertex_ao(o(-1, 1, 0), o(0, 1, -1), o(-1, 1, -1)); // p3: x,z
            [a0, a1, a2, a3]
        }
        Face::Bottom => {
            // Face at y. Vertices: (x,z), (x+1,z), (x+1,z+1), (x,z+1)
            let a0 = vertex_ao(o(-1, -1, 0), o(0, -1, -1), o(-1, -1, -1)); // p0: x,z
            let a1 = vertex_ao(o(1, -1, 0), o(0, -1, -1), o(1, -1, -1));   // p1: x+1,z
            let a2 = vertex_ao(o(1, -1, 0), o(0, -1, 1), o(1, -1, 1));     // p2: x+1,z+1
            let a3 = vertex_ao(o(-1, -1, 0), o(0, -1, 1), o(-1, -1, 1));   // p3: x,z+1
            [a0, a1, a2, a3]
        }
        Face::Right => {
            // Face at x+1. Tangent plane is YZ. Vertices: (y,z), (y+1,z), (y+1,z+1), (y,z+1)
            let a0 = vertex_ao(o(1, -1, 0), o(1, 0, -1), o(1, -1, -1)); // p0: y,z
            let a1 = vertex_ao(o(1, 1, 0), o(1, 0, -1), o(1, 1, -1));   // p1: y+1,z
            let a2 = vertex_ao(o(1, 1, 0), o(1, 0, 1), o(1, 1, 1));     // p2: y+1,z+1
            let a3 = vertex_ao(o(1, -1, 0), o(1, 0, 1), o(1, -1, 1));   // p3: y,z+1
            [a0, a1, a2, a3]
        }
        Face::Left => {
            // Face at x. Vertices: (y,z+1), (y+1,z+1), (y+1,z), (y,z)
            let a0 = vertex_ao(o(-1, -1, 0), o(-1, 0, 1), o(-1, -1, 1));   // p0: y,z+1
            let a1 = vertex_ao(o(-1, 1, 0), o(-1, 0, 1), o(-1, 1, 1));     // p1: y+1,z+1
            let a2 = vertex_ao(o(-1, 1, 0), o(-1, 0, -1), o(-1, 1, -1));   // p2: y+1,z
            let a3 = vertex_ao(o(-1, -1, 0), o(-1, 0, -1), o(-1, -1, -1)); // p3: y,z
            [a0, a1, a2, a3]
        }
        Face::Front => {
            // Face at z+1. Tangent plane is XY. Vertices: (x,y), (x+1,y), (x+1,y+1), (x,y+1)
            let a0 = vertex_ao(o(-1, 0, 1), o(0, -1, 1), o(-1, -1, 1)); // p0: x,y
            let a1 = vertex_ao(o(1, 0, 1), o(0, -1, 1), o(1, -1, 1));   // p1: x+1,y
            let a2 = vertex_ao(o(1, 0, 1), o(0, 1, 1), o(1, 1, 1));     // p2: x+1,y+1
            let a3 = vertex_ao(o(-1, 0, 1), o(0, 1, 1), o(-1, 1, 1));   // p3: x,y+1
            [a0, a1, a2, a3]
        }
        Face::Back => {
            // Face at z. Vertices: (x+1,y), (x,y), (x,y+1), (x+1,y+1)
            let a0 = vertex_ao(o(1, 0, -1), o(0, -1, -1), o(1, -1, -1));   // p0: x+1,y
            let a1 = vertex_ao(o(-1, 0, -1), o(0, -1, -1), o(-1, -1, -1)); // p1: x,y
            let a2 = vertex_ao(o(-1, 0, -1), o(0, 1, -1), o(-1, 1, -1));   // p2: x,y+1
            let a3 = vertex_ao(o(1, 0, -1), o(0, 1, -1), o(1, 1, -1));     // p3: x+1,y+1
            [a0, a1, a2, a3]
        }
    }
}

/// Emit plant as two crossed quads (X-shaped billboard).
fn emit_plant(
    vertices: &mut Vec<BlockVertex>,
    indices: &mut Vec<u32>,
    block: BlockType,
    x: f32,
    y: f32,
    z: f32,
) {
    let tile = block.face_tile(Face::Front);
    let (du, dv) = tile_uv(tile);
    let u0 = du + TILE_INSET;
    let v0 = dv + TILE_INSET;
    let u1 = du + TILE_UV - TILE_INSET;
    let v1 = dv + TILE_UV - TILE_INSET;
    let normal = [0.0, 1.0, 0.0]; // plants use up-facing normal for lighting
    let ao = 1.0;
    let cx = x + 0.5;
    let cz = z + 0.5;
    let d = 0.35; // half-diagonal

    // Quad 1: diagonal from (-d, -, -d) to (+d, -, +d)
    let quad1 = [
        [cx - d, y, cz - d],
        [cx + d, y, cz + d],
        [cx + d, y + 1.0, cz + d],
        [cx - d, y + 1.0, cz - d],
    ];
    // Quad 2: perpendicular diagonal
    let quad2 = [
        [cx - d, y, cz + d],
        [cx + d, y, cz - d],
        [cx + d, y + 1.0, cz - d],
        [cx - d, y + 1.0, cz + d],
    ];
    let uvs = [[u0, v1], [u1, v1], [u1, v0], [u0, v0]];

    for quad in [&quad1, &quad2] {
        let base = vertices.len() as u32;
        for i in 0..4 {
            vertices.push(BlockVertex {
                position: quad[i],
                uv: uvs[i],
                normal,
                ao,
            });
        }
        // Both sides visible (two triangles each direction)
        indices.extend_from_slice(&[
            base, base + 1, base + 2, base, base + 2, base + 3,
            base, base + 2, base + 1, base, base + 3, base + 2,
        ]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world_gen::WorldGenerator;

    #[test]
    fn mesh_has_geometry() {
        let gen = WorldGenerator::new(42);
        let chunk = gen.generate_chunk(0, 0);
        let mesh = mesh_chunk(&chunk, &[None; 4], 0.0, 0.0);
        assert!(mesh.vertices.len() > 100, "Should have many vertices");
        assert!(mesh.indices.len() > 100, "Should have many indices");
        assert_eq!(mesh.indices.len() % 3, 0, "Indices should be multiple of 3");
    }
}
