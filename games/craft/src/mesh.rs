use crate::block::{BlockType, Face, tile_uv, TILE_UV_X, TILE_UV_Y, TILE_INSET};
use crate::chunk::{ChunkData, CHUNK_SIZE, CHUNK_HEIGHT};
use crate::lighting::LightMap;
use crate::vertex::BlockVertex;

/// Neighbors: [+X, -X, +Z, -Z] (for cross-chunk face culling)
pub type ChunkNeighbors<'a> = [Option<&'a ChunkData>; 4];

pub struct ChunkMesh {
    pub vertices: Vec<BlockVertex>,
    pub indices: Vec<u32>,
    pub water_vertices: Vec<BlockVertex>,
    pub water_indices: Vec<u32>,
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

/// Face info stored per cell in the 2D greedy meshing mask.
/// Encodes block type + 4 AO values + light level for greedy merge comparison.
#[derive(Clone, Copy, PartialEq, Eq)]
struct FaceCell {
    block: u8,
    /// AO packed as 4 discrete levels (0..3) per vertex, total 8 bits.
    ao_packed: u8,
    /// Light level (packed sky/block byte) — faces with different light cannot be merged.
    light: u8,
}

impl FaceCell {
    const EMPTY: Self = Self { block: 0, ao_packed: 0, light: 0 };

    fn new(block: BlockType, ao: [f32; 4], light: u8) -> Self {
        // Quantize AO to 2-bit per vertex for merge comparison
        let q = |v: f32| ((v * 3.0).round() as u8).min(3);
        let ao_packed = q(ao[0]) | (q(ao[1]) << 2) | (q(ao[2]) << 4) | (q(ao[3]) << 6);
        Self { block: block as u8, ao_packed, light }
    }

    fn is_empty(self) -> bool {
        self.block == 0
    }

    fn ao_values(self) -> [f32; 4] {
        let unq = |shift: u8| ((self.ao_packed >> shift) & 3) as f32 / 3.0;
        [unq(0), unq(2), unq(4), unq(6)]
    }

    /// Decode light to the f32 encoding the shader expects: sky*16+block.
    fn light_f32(self) -> f32 {
        let sky = (self.light >> 4) as f32;
        let block = (self.light & 0x0F) as f32;
        sky * 16.0 + block
    }
}

/// Generate mesh for a chunk at world offset (ox, oz) = (cx * CHUNK_SIZE, cz * CHUNK_SIZE).
pub fn mesh_chunk(chunk: &ChunkData, neighbors: &ChunkNeighbors, light_map: Option<&LightMap>, ox: f32, oz: f32) -> ChunkMesh {
    let mut vertices = Vec::with_capacity(4096);
    let mut indices = Vec::with_capacity(8192);
    let mut water_vertices = Vec::with_capacity(1024);
    let mut water_indices = Vec::with_capacity(2048);

    // Non-cube blocks: plants (cross billboard) and torches (thin cuboid)
    for y in 0..CHUNK_HEIGHT {
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let block = chunk.get(x, y, z);
                if block.is_plant() {
                    let wx = ox + x as f32;
                    let wy = y as f32;
                    let wz = oz + z as f32;
                    let light = light_map
                        .map(|lm| lm.get_packed(x, y, z))
                        .unwrap_or(0xF0);
                    if block.is_torch_shape() {
                        emit_torch(&mut vertices, &mut indices, block, wx, wy, wz, light);
                    } else {
                        emit_plant(&mut vertices, &mut indices, block, wx, wy, wz, light);
                    }
                }
            }
        }
    }

    // Greedy meshing per face direction
    let face_dirs: [(Face, i32, i32, i32); 6] = [
        (Face::Top,    0,  1,  0),
        (Face::Bottom, 0, -1,  0),
        (Face::Right,  1,  0,  0),
        (Face::Left,  -1,  0,  0),
        (Face::Front,  0,  0,  1),
        (Face::Back,   0,  0, -1),
    ];

    // Pre-allocate mask once; max dimension is CHUNK_SIZE * CHUNK_HEIGHT
    let max_mask_size = CHUNK_SIZE * CHUNK_HEIGHT;
    let mut mask = vec![FaceCell::EMPTY; max_mask_size];

    for &(face, ndx, ndy, ndz) in &face_dirs {
        greedy_face(
            chunk, neighbors, light_map, ox, oz, face, ndx, ndy, ndz,
            &mut vertices, &mut indices,
            &mut water_vertices, &mut water_indices,
            &mut mask,
        );
    }

    ChunkMesh { vertices, indices, water_vertices, water_indices }
}

/// Greedy meshing for one face direction across the entire chunk.
fn greedy_face(
    chunk: &ChunkData,
    neighbors: &ChunkNeighbors,
    light_map: Option<&LightMap>,
    ox: f32,
    oz: f32,
    face: Face,
    ndx: i32,
    ndy: i32,
    ndz: i32,
    vertices: &mut Vec<BlockVertex>,
    indices: &mut Vec<u32>,
    water_vertices: &mut Vec<BlockVertex>,
    water_indices: &mut Vec<u32>,
    mask: &mut Vec<FaceCell>,
) {
    // For each face direction, we iterate slices perpendicular to the normal.
    // The "depth" axis is the normal direction; u,v are the two tangent axes.
    //
    // Face::Top/Bottom → depth=Y, u=X, v=Z → slice dims: CHUNK_SIZE × CHUNK_SIZE, depth=CHUNK_HEIGHT
    // Face::Right/Left → depth=X, u=Z, v=Y → slice dims: CHUNK_SIZE × CHUNK_HEIGHT, depth=CHUNK_SIZE
    // Face::Front/Back → depth=Z, u=X, v=Y → slice dims: CHUNK_SIZE × CHUNK_HEIGHT, depth=CHUNK_SIZE

    let (depth_max, u_max, v_max) = match face {
        Face::Top | Face::Bottom => (CHUNK_HEIGHT, CHUNK_SIZE, CHUNK_SIZE),
        Face::Right | Face::Left => (CHUNK_SIZE, CHUNK_SIZE, CHUNK_HEIGHT),
        Face::Front | Face::Back => (CHUNK_SIZE, CHUNK_SIZE, CHUNK_HEIGHT),
    };

    // Map (depth, u, v) → (x, y, z)
    let to_xyz = |d: usize, u: usize, v: usize| -> (i32, i32, i32) {
        match face {
            Face::Top | Face::Bottom => (u as i32, d as i32, v as i32),
            Face::Right | Face::Left => (d as i32, v as i32, u as i32),
            Face::Front | Face::Back => (u as i32, v as i32, d as i32),
        }
    };

    // Resize mask to fit this face's slice dimensions; reuses the caller's allocation
    mask.resize(u_max * v_max, FaceCell::EMPTY);

    for d in 0..depth_max {
        // Clear the mask for this slice
        mask[..u_max * v_max].fill(FaceCell::EMPTY);

        // Build the 2D mask for this slice
        for v in 0..v_max {
            for u in 0..u_max {
                let (x, y, z) = to_xyz(d, u, v);
                let block = chunk.get_safe(x, y, z);

                if block == BlockType::Air || block.is_plant() {
                    mask[u + v * u_max] = FaceCell::EMPTY;
                    continue;
                }

                // Check neighbor in the face normal direction
                let nx = x + ndx;
                let ny = y + ndy;
                let nz = z + ndz;
                let neighbor = get_neighbor_block(chunk, neighbors, nx, ny, nz);

                if !neighbor.is_transparent() {
                    mask[u + v * u_max] = FaceCell::EMPTY;
                    continue;
                }
                // Don't render internal faces of same transparent type
                if block.is_transparent() && neighbor == block {
                    mask[u + v * u_max] = FaceCell::EMPTY;
                    continue;
                }
                // Don't generate faces toward torch-shaped neighbors —
                // they occupy negligible space and exposing adjacent faces
                // looks like a full block outline around a tiny item.
                if neighbor.is_torch_shape() {
                    mask[u + v * u_max] = FaceCell::EMPTY;
                    continue;
                }

                // Sample light from the neighbor block (the air/transparent space the face looks into)
                let face_light = light_map
                    .map(|lm| lm.get_packed_safe(nx, ny, nz))
                    .unwrap_or(0xF0);

                if block.is_water() {
                    // Water: no AO
                    mask[u + v * u_max] = FaceCell::new(block, [1.0; 4], face_light);
                } else {
                    let ao = compute_face_ao(chunk, neighbors, face, x, y, z);
                    mask[u + v * u_max] = FaceCell::new(block, ao, face_light);
                }
            }
        }

        // Greedy merge the mask into rectangles
        let mut v = 0;
        while v < v_max {
            let mut u = 0;
            while u < u_max {
                let cell = mask[u + v * u_max];
                if cell.is_empty() {
                    u += 1;
                    continue;
                }

                // Find width (extend along u)
                let mut w = 1;
                while u + w < u_max && mask[(u + w) + v * u_max] == cell {
                    w += 1;
                }

                // Find height (extend along v)
                let mut h = 1;
                'outer: while v + h < v_max {
                    for du in 0..w {
                        if mask[(u + du) + (v + h) * u_max] != cell {
                            break 'outer;
                        }
                    }
                    h += 1;
                }

                // Clear the merged region
                for dv in 0..h {
                    for du in 0..w {
                        mask[(u + du) + (v + dv) * u_max] = FaceCell::EMPTY;
                    }
                }

                // Emit the merged quad
                let (x0, y0, z0) = to_xyz(d, u, v);
                let block = BlockType::from_u8(cell.block);
                let ao = cell.ao_values();

                let wx = ox + x0 as f32;
                let wy = y0 as f32;
                let wz = oz + z0 as f32;

                let light_val = cell.light_f32();
                if block.is_water() {
                    emit_greedy_quad(water_vertices, water_indices, block, face, wx, wy, wz, w, h, [1.0; 4], light_val);
                } else {
                    emit_greedy_quad(vertices, indices, block, face, wx, wy, wz, w, h, ao, light_val);
                }

                u += w;
            }
            v += 1;
        }
    }
}

/// Emit a greedy-merged quad spanning w×h blocks.
///
/// UV encoding: `uv = [tile_index, -1.0]` for all vertices.
/// The sentinel `uv.y = -1.0` tells the shader to compute atlas UV from
/// world position (tiled per-block), avoiding texture bleeding across
/// atlas tiles when quads span multiple blocks.
fn emit_greedy_quad(
    vertices: &mut Vec<BlockVertex>,
    indices: &mut Vec<u32>,
    block: BlockType,
    face: Face,
    x: f32,
    y: f32,
    z: f32,
    w: usize,
    h: usize,
    ao: [f32; 4],
    light: f32,
) {
    let tile = block.face_tile(face);
    let normal = face.normal();
    let wf = w as f32;
    let hf = h as f32;

    // Encode tile index; shader computes atlas UV from world_pos
    let uv = [tile as f32, -1.0];

    let base = vertices.len() as u32;

    let (p0, p1, p2, p3) = match face {
        Face::Top => (
            [x,      y + 1.0, z + hf],
            [x + wf, y + 1.0, z + hf],
            [x + wf, y + 1.0, z],
            [x,      y + 1.0, z],
        ),
        Face::Bottom => (
            [x,      y, z],
            [x + wf, y, z],
            [x + wf, y, z + hf],
            [x,      y, z + hf],
        ),
        Face::Right => (
            [x + 1.0, y,      z],
            [x + 1.0, y + hf, z],
            [x + 1.0, y + hf, z + wf],
            [x + 1.0, y,      z + wf],
        ),
        Face::Left => (
            [x, y,      z + wf],
            [x, y + hf, z + wf],
            [x, y + hf, z],
            [x, y,      z],
        ),
        Face::Front => (
            [x,      y,      z + 1.0],
            [x + wf, y,      z + 1.0],
            [x + wf, y + hf, z + 1.0],
            [x,      y + hf, z + 1.0],
        ),
        Face::Back => (
            [x + wf, y,      z],
            [x,      y,      z],
            [x,      y + hf, z],
            [x + wf, y + hf, z],
        ),
    };

    vertices.push(BlockVertex { position: p0, uv, normal, ao: ao[0], light });
    vertices.push(BlockVertex { position: p1, uv, normal, ao: ao[1], light });
    vertices.push(BlockVertex { position: p2, uv, normal, ao: ao[2], light });
    vertices.push(BlockVertex { position: p3, uv, normal, ao: ao[3], light });

    // AO flip optimization
    if ao[0] + ao[2] < ao[1] + ao[3] {
        indices.extend_from_slice(&[base + 1, base + 3, base + 2, base + 1, base, base + 3]);
    } else {
        indices.extend_from_slice(&[base, base + 2, base + 1, base, base + 3, base + 2]);
    }
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
    // Clamp coordinates into the correct neighbor chunk.
    // Handle X and Z independently (not else-if) so diagonal corners work.
    let mut lx = x;
    let mut lz = z;
    let mut source: Option<&ChunkData> = None;

    let in_x = lx >= 0 && lx < CHUNK_SIZE as i32;
    let in_z = lz >= 0 && lz < CHUNK_SIZE as i32;

    if in_x && in_z {
        return chunk.get_safe(lx, y, lz);
    }

    // Cross-chunk: neighbors order [+X, -X, +Z, -Z]
    // For diagonal corners (e.g. x<0 && z<0) we don't have a diagonal neighbor,
    // so fall back to Air — but first handle single-axis cases correctly.
    if !in_x && in_z {
        // Only X out of bounds
        if lx >= CHUNK_SIZE as i32 {
            source = neighbors[0];
            lx -= CHUNK_SIZE as i32;
        } else {
            source = neighbors[1];
            lx += CHUNK_SIZE as i32;
        }
    } else if in_x && !in_z {
        // Only Z out of bounds
        if lz >= CHUNK_SIZE as i32 {
            source = neighbors[2];
            lz -= CHUNK_SIZE as i32;
        } else {
            source = neighbors[3];
            lz += CHUNK_SIZE as i32;
        }
    }
    // Diagonal case (!in_x && !in_z): no diagonal neighbor available, return Air

    match source {
        Some(n) if lx >= 0 && lx < CHUNK_SIZE as i32 && lz >= 0 && lz < CHUNK_SIZE as i32 => {
            n.get_safe(lx, y, lz)
        }
        _ => BlockType::Air,
    }
}

/// Compute AO values for the 4 vertices of a face.
fn compute_face_ao(
    chunk: &ChunkData,
    neighbors: &ChunkNeighbors,
    face: Face,
    x: i32,
    y: i32,
    z: i32,
) -> [f32; 4] {
    let o = |dx: i32, dy: i32, dz: i32| -> bool {
        is_ao_occluder(chunk, neighbors, x + dx, y + dy, z + dz)
    };

    match face {
        Face::Top => {
            let a0 = vertex_ao(o(-1, 1, 0), o(0, 1, 1), o(-1, 1, 1));
            let a1 = vertex_ao(o(1, 1, 0), o(0, 1, 1), o(1, 1, 1));
            let a2 = vertex_ao(o(1, 1, 0), o(0, 1, -1), o(1, 1, -1));
            let a3 = vertex_ao(o(-1, 1, 0), o(0, 1, -1), o(-1, 1, -1));
            [a0, a1, a2, a3]
        }
        Face::Bottom => {
            let a0 = vertex_ao(o(-1, -1, 0), o(0, -1, -1), o(-1, -1, -1));
            let a1 = vertex_ao(o(1, -1, 0), o(0, -1, -1), o(1, -1, -1));
            let a2 = vertex_ao(o(1, -1, 0), o(0, -1, 1), o(1, -1, 1));
            let a3 = vertex_ao(o(-1, -1, 0), o(0, -1, 1), o(-1, -1, 1));
            [a0, a1, a2, a3]
        }
        Face::Right => {
            let a0 = vertex_ao(o(1, -1, 0), o(1, 0, -1), o(1, -1, -1));
            let a1 = vertex_ao(o(1, 1, 0), o(1, 0, -1), o(1, 1, -1));
            let a2 = vertex_ao(o(1, 1, 0), o(1, 0, 1), o(1, 1, 1));
            let a3 = vertex_ao(o(1, -1, 0), o(1, 0, 1), o(1, -1, 1));
            [a0, a1, a2, a3]
        }
        Face::Left => {
            let a0 = vertex_ao(o(-1, -1, 0), o(-1, 0, 1), o(-1, -1, 1));
            let a1 = vertex_ao(o(-1, 1, 0), o(-1, 0, 1), o(-1, 1, 1));
            let a2 = vertex_ao(o(-1, 1, 0), o(-1, 0, -1), o(-1, 1, -1));
            let a3 = vertex_ao(o(-1, -1, 0), o(-1, 0, -1), o(-1, -1, -1));
            [a0, a1, a2, a3]
        }
        Face::Front => {
            let a0 = vertex_ao(o(-1, 0, 1), o(0, -1, 1), o(-1, -1, 1));
            let a1 = vertex_ao(o(1, 0, 1), o(0, -1, 1), o(1, -1, 1));
            let a2 = vertex_ao(o(1, 0, 1), o(0, 1, 1), o(1, 1, 1));
            let a3 = vertex_ao(o(-1, 0, 1), o(0, 1, 1), o(-1, 1, 1));
            [a0, a1, a2, a3]
        }
        Face::Back => {
            let a0 = vertex_ao(o(1, 0, -1), o(0, -1, -1), o(1, -1, -1));
            let a1 = vertex_ao(o(-1, 0, -1), o(0, -1, -1), o(-1, -1, -1));
            let a2 = vertex_ao(o(-1, 0, -1), o(0, 1, -1), o(-1, 1, -1));
            let a3 = vertex_ao(o(1, 0, -1), o(0, 1, -1), o(1, 1, -1));
            [a0, a1, a2, a3]
        }
    }
}

/// Emit a torch-shaped block as a thin 3D cuboid (2×10×2 in 1/16ths of a block).
/// Uses 4 side faces + 1 top face with direct atlas UV coordinates.
fn emit_torch(
    vertices: &mut Vec<BlockVertex>,
    indices: &mut Vec<u32>,
    block: BlockType,
    x: f32,
    y: f32,
    z: f32,
    light_packed: u8,
) {
    let tile = block.face_tile(Face::Front);
    let (du, dv) = tile_uv(tile);

    // Decode light
    let sky = (light_packed >> 4) as f32;
    let block_l = (light_packed & 0x0F) as f32;
    let light = sky * 16.0 + block_l;
    let ao = 1.0;

    // Torch dimensions in block units (1 block = 16 pixels)
    let hw = 2.0 / 16.0;       // half-width: 2px = 0.125
    let h = 10.0 / 16.0;       // height: 10px = 0.625
    let cx = x + 0.5;
    let cz = z + 0.5;

    let x0 = cx - hw;
    let x1 = cx + hw;
    let y0 = y;
    let y1 = y + h;
    let z0 = cz - hw;
    let z1 = cz + hw;

    // UV: map the full torch texture tile to each face.
    // The narrow cuboid width means nearest-neighbor sampling picks the correct center columns.
    let side_u0 = du + TILE_INSET;
    let side_u1 = du + TILE_UV_X - TILE_INSET;
    let side_v0 = dv + TILE_INSET;                    // top of tile
    let side_v1 = dv + TILE_UV_Y - TILE_INSET;        // bottom of tile

    // Top UV: sample the flame area from the center of the tile
    let top_u0 = du + TILE_UV_X * (6.0 / 16.0);
    let top_u1 = du + TILE_UV_X * (10.0 / 16.0);
    let top_v0 = dv + TILE_UV_Y * (2.0 / 16.0);
    let top_v1 = dv + TILE_UV_Y * (6.0 / 16.0);

    // Helper: emit one quad (4 vertices, 6 indices)
    let mut push_face = |p0: [f32; 3], p1: [f32; 3], p2: [f32; 3], p3: [f32; 3],
                          uv0: [f32; 2], uv1: [f32; 2], uv2: [f32; 2], uv3: [f32; 2],
                          n: [f32; 3]| {
        let base = vertices.len() as u32;
        vertices.push(BlockVertex { position: p0, uv: uv0, normal: n, ao, light });
        vertices.push(BlockVertex { position: p1, uv: uv1, normal: n, ao, light });
        vertices.push(BlockVertex { position: p2, uv: uv2, normal: n, ao, light });
        vertices.push(BlockVertex { position: p3, uv: uv3, normal: n, ao, light });
        indices.extend_from_slice(&[base, base+1, base+2, base, base+2, base+3]);
        // Back face for double-sided rendering
        indices.extend_from_slice(&[base, base+2, base+1, base, base+3, base+2]);
    };

    // Front face (+Z)
    push_face(
        [x0, y0, z1], [x1, y0, z1], [x1, y1, z1], [x0, y1, z1],
        [side_u0, side_v1], [side_u1, side_v1], [side_u1, side_v0], [side_u0, side_v0],
        [0.0, 0.0, 1.0],
    );
    // Back face (-Z)
    push_face(
        [x1, y0, z0], [x0, y0, z0], [x0, y1, z0], [x1, y1, z0],
        [side_u0, side_v1], [side_u1, side_v1], [side_u1, side_v0], [side_u0, side_v0],
        [0.0, 0.0, -1.0],
    );
    // Right face (+X)
    push_face(
        [x1, y0, z1], [x1, y0, z0], [x1, y1, z0], [x1, y1, z1],
        [side_u0, side_v1], [side_u1, side_v1], [side_u1, side_v0], [side_u0, side_v0],
        [1.0, 0.0, 0.0],
    );
    // Left face (-X)
    push_face(
        [x0, y0, z0], [x0, y0, z1], [x0, y1, z1], [x0, y1, z0],
        [side_u0, side_v1], [side_u1, side_v1], [side_u1, side_v0], [side_u0, side_v0],
        [-1.0, 0.0, 0.0],
    );
    // Top face (+Y)
    push_face(
        [x0, y1, z0], [x0, y1, z1], [x1, y1, z1], [x1, y1, z0],
        [top_u0, top_v0], [top_u0, top_v1], [top_u1, top_v1], [top_u1, top_v0],
        [0.0, 1.0, 0.0],
    );
}

/// Emit plant as two crossed quads (X-shaped billboard).
fn emit_plant(
    vertices: &mut Vec<BlockVertex>,
    indices: &mut Vec<u32>,
    block: BlockType,
    x: f32,
    y: f32,
    z: f32,
    light_packed: u8,
) {
    let tile = block.face_tile(Face::Front);
    let (du, dv) = tile_uv(tile);
    let u0 = du + TILE_INSET;
    let v0 = dv + TILE_INSET;
    let u1 = du + TILE_UV_X - TILE_INSET;
    let v1 = dv + TILE_UV_Y - TILE_INSET;
    let normal = [0.0, 1.0, 0.0];
    let ao = 1.0;
    let cx = x + 0.5;
    let cz = z + 0.5;

    let d = 0.35;
    let h = 1.0;

    // Decode light to f32 encoding
    let sky = (light_packed >> 4) as f32;
    let block_l = (light_packed & 0x0F) as f32;
    let light = sky * 16.0 + block_l;

    let quad1 = [
        [cx - d, y, cz - d],
        [cx + d, y, cz + d],
        [cx + d, y + h, cz + d],
        [cx - d, y + h, cz - d],
    ];
    let quad2 = [
        [cx - d, y, cz + d],
        [cx + d, y, cz - d],
        [cx + d, y + h, cz - d],
        [cx - d, y + h, cz + d],
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
                light,
            });
        }
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
    fn torch_emits_cuboid_not_cube() {
        let mut chunk = ChunkData::new();
        // Place a stone floor and a torch on top
        chunk.set(5, 10, 5, BlockType::Stone);
        chunk.set(5, 11, 5, BlockType::Torch);

        let mesh = mesh_chunk(&chunk, &[None; 4], None, 0.0, 0.0);

        // Torch is 5 faces × 4 verts = 20 verts (from emit_torch)
        // Stone has exposed faces (top is toward torch which is_torch_shape → culled)
        // So stone should have 5 faces max (all except top toward torch)

        // Check: torch vertices should have positions within the thin cuboid
        // The cuboid is 4/16 wide centered at (5.5, ?, 5.5)
        let torch_verts: Vec<_> = mesh.vertices.iter().filter(|v| {
            // Torch vertices have y between 11.0 and 11.625
            v.position[1] >= 11.0 && v.position[1] <= 11.7
            // And UVs in the direct-UV range (not tile-index encoding)
            && v.uv[1] >= 0.0
        }).collect();

        assert!(torch_verts.len() >= 20,
            "Torch should produce at least 20 vertices (5 faces × 4), got {}",
            torch_verts.len());

        // Verify torch is thin: all X positions should be within [5.375, 5.625]
        for v in &torch_verts {
            let px = v.position[0];
            assert!(px >= 5.3 && px <= 5.7,
                "Torch vertex X={} should be near center (5.375..5.625)", px);
        }

        // Verify the greedy mesher does NOT generate a top face on stone
        // pointing toward the torch (torch_shape neighbor → culled)
        let stone_top_toward_torch: Vec<_> = mesh.vertices.iter().filter(|v| {
            v.position[1] >= 10.99 && v.position[1] <= 11.01
            && v.uv[1] < 0.0  // tile-index encoding = cube face
            && v.normal[1] > 0.5  // +Y normal = top face
        }).collect();
        assert_eq!(stone_top_toward_torch.len(), 0,
            "Stone top face toward torch should be culled, got {} verts",
            stone_top_toward_torch.len());
    }

    #[test]
    fn mesh_has_geometry() {
        let gen = WorldGenerator::new(42);
        let chunk = gen.generate_chunk(0, 0);
        let mesh = mesh_chunk(&chunk, &[None; 4], None, 0.0, 0.0);
        assert!(mesh.vertices.len() > 100, "Should have many opaque vertices");
        assert!(mesh.indices.len() > 100, "Should have many opaque indices");
        assert_eq!(mesh.indices.len() % 3, 0, "Opaque indices should be multiple of 3");
        assert_eq!(mesh.water_indices.len() % 3, 0, "Water indices should be multiple of 3");
    }

    #[test]
    fn greedy_reduces_vertices() {
        // A flat plane of identical blocks should merge into very few quads
        let mut chunk = ChunkData::new();
        // Fill a flat layer of stone at y=10
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                chunk.set(x, 10, z, BlockType::Stone);
            }
        }
        let mesh = mesh_chunk(&chunk, &[None; 4], None, 0.0, 0.0);
        // Without greedy: 32*32 = 1024 top faces × 4 verts = 4096 verts (just for top)
        // With greedy: should be dramatically fewer
        assert!(mesh.vertices.len() < 500,
            "Greedy meshing should produce far fewer vertices for uniform plane, got {}",
            mesh.vertices.len());
    }
}
