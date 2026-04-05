use std::collections::{HashMap, HashSet};

use anvilkit_render::prelude::RenderDevice;
use anvilkit_render::renderer::buffer_pool::BufferPool;
use anvilkit_render::renderer::draw::Frustum;

use crate::chunk::{ChunkData, CHUNK_SIZE};
use crate::config;
use crate::lighting::{self, LightMap};
use crate::mesh::{self, ChunkMesh, ChunkNeighbors};
use crate::resources::VoxelWorld;
use crate::world_gen::WorldGenerator;

/// Compute and insert light map for a chunk that was just added to the world.
fn insert_chunk_with_light(world: &mut VoxelWorld, cx: i32, cz: i32, chunk: ChunkData) {
    let mut light = LightMap::new();
    lighting::compute_initial_sky_light(&chunk, &mut light);
    lighting::propagate_sky_light(&chunk, &mut light);
    lighting::compute_block_light(&chunk, &mut light);
    world.chunks.insert((cx, cz), chunk);
    world.light_maps.insert((cx, cz), light);
}

/// Per-chunk GPU buffers.
pub struct ChunkGpuMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
    vb_capacity: u64,
    ib_capacity: u64,
    pub water_vertex_buffer: Option<wgpu::Buffer>,
    pub water_index_buffer: Option<wgpu::Buffer>,
    pub water_index_count: u32,
    water_vb_capacity: u64,
    water_ib_capacity: u64,
    /// Chunk center for frustum culling (world space).
    pub center: glam::Vec3,
}

/// Result of async chunk generation.
pub struct ChunkGenResult {
    pub cx: i32,
    pub cz: i32,
    pub chunk_data: ChunkData,
}

/// Manages chunk loading, unloading, meshing, and GPU buffer lifecycle.
pub struct ChunkManager {
    pub chunk_meshes: HashMap<(i32, i32), ChunkGpuMesh>,
    pub dirty_chunks: HashSet<(i32, i32)>,
    pub pending_chunks: HashSet<(i32, i32)>,
    pub chunk_request_tx: crossbeam_channel::Sender<(i32, i32)>,
    pub chunk_result_rx: crossbeam_channel::Receiver<ChunkGenResult>,
    vb_pool: BufferPool,
    ib_pool: BufferPool,
    pub load_radius: i32,
    pub last_chunk_pos: (i32, i32),
}

impl ChunkManager {
    pub fn new(
        request_tx: crossbeam_channel::Sender<(i32, i32)>,
        result_rx: crossbeam_channel::Receiver<ChunkGenResult>,
    ) -> Self {
        Self {
            chunk_meshes: HashMap::new(),
            dirty_chunks: HashSet::new(),
            pending_chunks: HashSet::new(),
            chunk_request_tx: request_tx,
            chunk_result_rx: result_rx,
            vb_pool: BufferPool::new(128),
            ib_pool: BufferPool::new(128),
            load_radius: config::LOAD_RADIUS,
            last_chunk_pos: (i32::MAX, i32::MAX),
        }
    }

    /// Generate initial chunks synchronously (for first load).
    pub fn generate_initial_chunks(
        world: &mut VoxelWorld,
        world_gen: &WorldGenerator,
        center_cx: i32,
        center_cz: i32,
        radius: i32,
    ) {
        for cx in (center_cx - radius)..=(center_cx + radius) {
            for cz in (center_cz - radius)..=(center_cz + radius) {
                if !world.chunks.contains_key(&(cx, cz)) {
                    let chunk = world_gen.generate_chunk(cx, cz);
                    insert_chunk_with_light(world, cx, cz, chunk);
                }
            }
        }
    }

    /// Send initial chunk requests through the async channel system.
    ///
    /// Instead of generating chunks synchronously, this dispatches all chunk
    /// coordinates within the given radius to the worker thread pool. The
    /// existing `update()` polling will receive results and the game can start
    /// immediately with whatever chunks are ready.
    pub fn request_initial_chunks(&mut self, center_cx: i32, center_cz: i32, radius: i32) {
        for cx in (center_cx - radius)..=(center_cx + radius) {
            for cz in (center_cz - radius)..=(center_cz + radius) {
                let key = (cx, cz);
                if !self.pending_chunks.contains(&key) {
                    let _ = self.chunk_request_tx.send(key);
                    self.pending_chunks.insert(key);
                }
            }
        }
    }

    /// Upload initial chunk meshes to GPU.
    pub fn upload_all(&mut self, world: &VoxelWorld, device: &RenderDevice) {
        let keys: Vec<(i32, i32)> = world
            .chunks
            .keys()
            .copied()
            .filter(|k| !self.chunk_meshes.contains_key(k))
            .collect();
        for (cx, cz) in keys {
            self.mesh_and_upload(world, device, cx, cz);
        }
    }

    /// Update chunks: send async requests, poll results, unload far chunks.
    pub fn update(&mut self, world: &mut VoxelWorld, cam_pos: glam::Vec3) {
        let cx = (cam_pos.x / CHUNK_SIZE as f32).floor() as i32;
        let cz = (cam_pos.z / CHUNK_SIZE as f32).floor() as i32;

        let pos_changed = (cx, cz) != self.last_chunk_pos;
        if pos_changed {
            self.last_chunk_pos = (cx, cz);
        }

        // Send async requests for chunks not yet loaded or pending
        if pos_changed {
            for dcx in -self.load_radius..=self.load_radius {
                for dcz in -self.load_radius..=self.load_radius {
                    let key = (cx + dcx, cz + dcz);
                    if !world.chunks.contains_key(&key) && !self.pending_chunks.contains(&key) {
                        let _ = self.chunk_request_tx.send(key);
                        self.pending_chunks.insert(key);
                    }
                }
            }
        }

        // Poll completed chunks (process up to 4 per frame)
        {
            let mut received = 0;
            while let Ok(result) = self.chunk_result_rx.try_recv() {
                self.pending_chunks.remove(&(result.cx, result.cz));
                insert_chunk_with_light(world, result.cx, result.cz, result.chunk_data);

                // Mark new chunk + four neighbors dirty for meshing with proper neighbor data
                self.dirty_chunks.insert((result.cx, result.cz));
                self.dirty_chunks.insert((result.cx + 1, result.cz));
                self.dirty_chunks.insert((result.cx - 1, result.cz));
                self.dirty_chunks.insert((result.cx, result.cz + 1));
                self.dirty_chunks.insert((result.cx, result.cz - 1));

                received += 1;
                if received >= 4 {
                    break;
                }
            }
        }

        // Unload far chunks
        if pos_changed {
            let r = self.load_radius + 2;
            let far_keys: Vec<(i32, i32)> = self
                .chunk_meshes
                .keys()
                .copied()
                .filter(|&(kx, kz)| (kx - cx).abs() > r || (kz - cz).abs() > r)
                .collect();
            for key in &far_keys {
                self.release_chunk_buffers(key);
                self.pending_chunks.remove(key);
            }
            for key in far_keys {
                world.chunks.remove(&key);
                world.light_maps.remove(&key);
            }
        }
    }

    /// Remesh dirty chunks up to the per-frame budget.
    pub fn remesh_dirty(&mut self, world: &VoxelWorld, device: &RenderDevice) {
        if self.dirty_chunks.is_empty() {
            return;
        }
        let budget = config::REMESH_BUDGET;
        let batch: Vec<(i32, i32)> = self.dirty_chunks.iter().copied().take(budget).collect();
        for &key in &batch {
            self.dirty_chunks.remove(&key);
        }
        for (cx, cz) in batch {
            self.mesh_and_upload(world, device, cx, cz);
        }
    }

    /// Mark a block's chunk and its boundary neighbors as dirty.
    pub fn mark_dirty_with_neighbors(&mut self, bx: i32, bz: i32) {
        let chunk_cx = bx.div_euclid(CHUNK_SIZE as i32);
        let chunk_cz = bz.div_euclid(CHUNK_SIZE as i32);
        self.dirty_chunks.insert((chunk_cx, chunk_cz));
        let lx = bx.rem_euclid(CHUNK_SIZE as i32);
        let lz = bz.rem_euclid(CHUNK_SIZE as i32);
        if lx == 0 { self.dirty_chunks.insert((chunk_cx - 1, chunk_cz)); }
        if lx == CHUNK_SIZE as i32 - 1 { self.dirty_chunks.insert((chunk_cx + 1, chunk_cz)); }
        if lz == 0 { self.dirty_chunks.insert((chunk_cx, chunk_cz - 1)); }
        if lz == CHUNK_SIZE as i32 - 1 { self.dirty_chunks.insert((chunk_cx, chunk_cz + 1)); }
    }

    /// Check if a chunk mesh is visible in the given frustum.
    pub fn is_visible(mesh: &ChunkGpuMesh, frustum: &Frustum) -> bool {
        let half_extents = glam::Vec3::new(16.0, 128.0, 16.0);
        frustum.intersects_aabb(mesh.center, half_extents)
    }

    // --- Private helpers ---

    fn mesh_and_upload(&mut self, world: &VoxelWorld, device: &RenderDevice, cx: i32, cz: i32) {
        let Some(chunk) = world.chunks.get(&(cx, cz)) else { return };
        let neighbors: ChunkNeighbors = [
            world.chunks.get(&(cx + 1, cz)),
            world.chunks.get(&(cx - 1, cz)),
            world.chunks.get(&(cx, cz + 1)),
            world.chunks.get(&(cx, cz - 1)),
        ];
        let light_map = world.light_maps.get(&(cx, cz));
        let ox = (cx * CHUNK_SIZE as i32) as f32;
        let oz = (cz * CHUNK_SIZE as i32) as f32;
        let cm = mesh::mesh_chunk(chunk, &neighbors, light_map, ox, oz);
        self.upload_chunk_mesh(device, cx, cz, cm);
    }

    fn upload_chunk_mesh(&mut self, device: &RenderDevice, cx: i32, cz: i32, cm: ChunkMesh) {
        // Release old buffers back to pool
        self.release_chunk_buffers(&(cx, cz));

        if cm.indices.is_empty() && cm.water_indices.is_empty() {
            return;
        }

        let ox = (cx * CHUNK_SIZE as i32) as f32;
        let oz = (cz * CHUNK_SIZE as i32) as f32;

        let vb_size = (cm.vertices.len() * std::mem::size_of_val(&cm.vertices[0])) as u64;
        let ib_size = (cm.indices.len() * 4) as u64;

        let vb = self.vb_pool.acquire(
            device.device(), vb_size,
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            &format!("Chunk({},{}) VB", cx, cz),
        );
        device.queue().write_buffer(&vb, 0, bytemuck::cast_slice(&cm.vertices));

        let ib = self.ib_pool.acquire(
            device.device(), ib_size,
            wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            &format!("Chunk({},{}) IB", cx, cz),
        );
        device.queue().write_buffer(&ib, 0, bytemuck::cast_slice(&cm.indices));

        let (wvb, wib, wic, wvb_cap, wib_cap) = if !cm.water_indices.is_empty() {
            let wvb_size = (cm.water_vertices.len() * std::mem::size_of_val(&cm.water_vertices[0])) as u64;
            let wib_size = (cm.water_indices.len() * 4) as u64;

            let wvb = self.vb_pool.acquire(
                device.device(), wvb_size,
                wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                &format!("Chunk({},{}) Water VB", cx, cz),
            );
            device.queue().write_buffer(&wvb, 0, bytemuck::cast_slice(&cm.water_vertices));

            let wib = self.ib_pool.acquire(
                device.device(), wib_size,
                wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                &format!("Chunk({},{}) Water IB", cx, cz),
            );
            device.queue().write_buffer(&wib, 0, bytemuck::cast_slice(&cm.water_indices));

            (Some(wvb), Some(wib), cm.water_indices.len() as u32, wvb_size, wib_size)
        } else {
            (None, None, 0, 0, 0)
        };

        let center = glam::Vec3::new(
            ox + CHUNK_SIZE as f32 * 0.5,
            128.0,
            oz + CHUNK_SIZE as f32 * 0.5,
        );

        self.chunk_meshes.insert(
            (cx, cz),
            ChunkGpuMesh {
                vertex_buffer: vb,
                index_buffer: ib,
                index_count: cm.indices.len() as u32,
                vb_capacity: vb_size,
                ib_capacity: ib_size,
                water_vertex_buffer: wvb,
                water_index_buffer: wib,
                water_index_count: wic,
                water_vb_capacity: wvb_cap,
                water_ib_capacity: wib_cap,
                center,
            },
        );
    }

    fn release_chunk_buffers(&mut self, key: &(i32, i32)) {
        let vb_usage = wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST;
        let ib_usage = wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST;
        if let Some(old) = self.chunk_meshes.remove(key) {
            self.vb_pool.release(old.vertex_buffer, old.vb_capacity, vb_usage);
            self.ib_pool.release(old.index_buffer, old.ib_capacity, ib_usage);
            if let Some(wvb) = old.water_vertex_buffer {
                self.vb_pool.release(wvb, old.water_vb_capacity, vb_usage);
            }
            if let Some(wib) = old.water_index_buffer {
                self.ib_pool.release(wib, old.water_ib_capacity, ib_usage);
            }
        }
    }
}
